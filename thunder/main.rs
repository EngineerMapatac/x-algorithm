use anyhow::{Context, Result};
use axum::Router;
use clap::Parser;
use log::{info, warn};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tonic::service::Routes;
use xai_http_server::{CancellationToken, GrpcConfig, HttpServer};

use xai_x_rpc::grpc_client::TlsMode;

use thunder::{
    args, kafka_utils, metrics::INIT_DURATION_SECONDS, posts::post_store::PostStore,
    strato_client::StratoClient, thunder_service::ThunderServiceImpl,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let args = args::Args::parse();
    record_s2s_client_cert_resolution();

    let post_store = Arc::new(PostStore::new(
        args.post_retention_seconds,
        args.request_timeout_ms,
    ));
    info!(
        "Initialized PostStore for in-memory post storage (retention: {} seconds / {:.1} days, request_timeout: {}ms)",
        args.post_retention_seconds,
        args.post_retention_seconds as f64 / 86400.0,
        args.request_timeout_ms
    );

    let strato_client = Arc::new(StratoClient::new());
    info!("Initialized StratoClient");

    let thunder_service = ThunderServiceImpl::new(
        Arc::clone(&post_store),
        Arc::clone(&strato_client),
        args.qps_limit,
    );
    info!(
        "Initialized InNetworkPostsService with qps_limit={}",
        args.qps_limit
    );
    let routes = Routes::new(thunder_service.server());

    let mut grpc_config = GrpcConfig::new(args.grpc_port, routes);

    match TlsMode::server_mtls_from_env().context("failed to read GRPC_MTLS_* env")? {
        Some(tls_mode) => {
            let tls_config = tls_mode
                .resolve_server()
                .context("failed to build ServerTlsConfig from GRPC_MTLS_* certs")?;
            grpc_config = grpc_config.with_tls_config(tls_config);
            info!("gRPC server S2S mTLS ENABLED (GRPC_MTLS_* env present)");
        }
        None => {
            info!("gRPC server mTLS disabled (plaintext h2c)");
        }
    }

    let http_server = HttpServer::builder(
        args.http_port,
        Router::new(),
        CancellationToken::new(),
        Duration::from_secs(10),
    )
    .with_grpc(grpc_config)
    .build()
    .await
    .context("Failed to create HTTP server")?;

    info!("HTTP server on port: {}", args.http_port);
    info!("gRPC server on port: {}", args.grpc_port);
    info!(
        "Metrics server on: http://0.0.0.0:{}/metrics",
        args.http_port
    );

    if args.enable_profiling {
        xai_profiling::spawn_server(3000, CancellationToken::new()).await;
    }

    let xai_user = std::env::var("XAI_USER").unwrap();

    let start = Instant::now();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<i64>(args.kafka_num_threads);
    kafka_utils::start_kafka(&args, post_store.clone(), &xai_user, tx).await?;

    if args.is_serving {
        for _ in 0..args.kafka_num_threads {
            rx.recv().await;
        }
        info!("Kafka catchup took {:?}", start.elapsed());

        post_store.finalize_init().await?;

        let init_duration = start.elapsed();
        INIT_DURATION_SECONDS.set(init_duration.as_secs_f64());
        info!("Total init took {:?}", init_duration);

        Arc::clone(&post_store).start_stats_logger();
        info!("Started PostStore stats logger",);

        Arc::clone(&post_store).start_auto_trim(2);
        info!(
            "Started PostStore auto-trim task (interval: 2 minutes, retention: {:.1} days)",
            args.post_retention_seconds as f64 / 86400.0
        );
    }

    http_server.set_readiness(true);
    info!("HTTP/gRPC server is ready");

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        if http_server.is_terminated() {
            break;
        }
    }
    info!("Server terminated");

    Ok(())
}

fn record_s2s_client_cert_resolution() {
    let mut builder = xai_s2s::S2sConfig::builder();
    if let (Ok(crt), Ok(key), Ok(chain)) = (
        std::env::var("S2S_CRT_PATH"),
        std::env::var("S2S_KEY_PATH"),
        std::env::var("S2S_CHAIN_PATH"),
    ) {
        builder = builder.full_chain_file(crt).key_file(key).chain_file(chain);
    }
    match builder.build().resolve_cert_paths(xai_s2s::Role::Client) {
        Ok(paths) => info!(
            "S2S client cert resolved via xai-s2s: {}",
            paths.cert.display()
        ),
        Err(e) => warn!("S2S client cert resolution failed (metrics only): {e:#}"),
    }
}
