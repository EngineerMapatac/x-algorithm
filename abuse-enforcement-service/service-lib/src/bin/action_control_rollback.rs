use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use clap::Parser;
use xai_abuse_enforcement_service::action_control_rollback::{plan, switch_lkey, verify, REASON};
use xai_abuse_enforcement_types::action_control::{ActionControlListResponse, PodView};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    service_url: String,
    #[arg(long = "pod-url", required = true)]
    pod_urls: Vec<String>,
    #[arg(long)]
    updated_by: String,
    #[arg(long, default_value = "ACTION_CONTROL_TOKEN")]
    token_env: String,
    #[arg(long)]
    apply: bool,
    #[arg(long, default_value_t = 60)]
    verify_timeout_secs: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let token = std::env::var(&args.token_env)
        .with_context(|| format!("set {} to the service's bearer key", args.token_env))?;
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let list: ActionControlListResponse = http
        .get(format!("{}/api/action_controls", args.service_url))
        .bearer_auth(&token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    if list.source != "store" {
        bail!(
            "the record list came from a pod's copy ({}), not the store; retry when the store answers",
            list.error.unwrap_or_default()
        );
    }
    let planned = plan(&list, &args.updated_by);
    println!("{} off switch(es) to turn on ({REASON}):", planned.len());
    for s in &planned {
        println!("  {}", switch_lkey(s));
    }
    if !args.apply {
        println!("plan only; rerun with --apply to write the switches");
        return Ok(());
    }
    for s in &planned {
        http.put(format!("{}/api/off_switches", args.service_url))
            .bearer_auth(&token)
            .json(s)
            .send()
            .await?
            .error_for_status()
            .with_context(|| format!("PUT /api/off_switches {}", switch_lkey(s)))?;
    }
    let deadline = Instant::now() + Duration::from_secs(args.verify_timeout_secs);
    loop {
        let mut pods = Vec::with_capacity(args.pod_urls.len());
        let mut unreachable = Vec::new();
        for url in &args.pod_urls {
            match http
                .get(format!("{url}/api/action_controls/pod"))
                .bearer_auth(&token)
                .send()
                .await
                .and_then(|r| r.error_for_status())
            {
                Ok(r) => pods.push(r.json::<PodView>().await?),
                Err(e) => unreachable.push(format!("{url}: {e}")),
            }
        }
        let missing = verify(&planned, &pods);
        if missing.is_empty() && unreachable.is_empty() {
            println!(
                "verified on {} pod(s): every page-managed scope is switched off. Safe to roll back or set ignore_store.",
                pods.len()
            );
            return Ok(());
        }
        if Instant::now() > deadline {
            for m in missing.iter().chain(&unreachable) {
                eprintln!("  {m}");
            }
            bail!("not verified: do NOT roll back yet");
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
