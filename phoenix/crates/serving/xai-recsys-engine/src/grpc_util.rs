// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use crate::proto_parser::{BodyKind, Func, parse};
use bytes::{Bytes, BytesMut};
use futures::future::{BoxFuture, join_all};
use futures::{StreamExt, stream};
use http_body::{Body, Frame};
use http_body_util::{BodyExt, Full, StreamBody};
use lazy_static::lazy_static;
use std::convert::Infallible;
use std::{
    future::poll_fn,
    mem,
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};
use tokio::{runtime, task, time::timeout};
use tonic::{
    Code, Status, body,
    codegen::{Service, http},
    metadata,
    transport::Channel,
};

pub fn concat_bodies<I>(bodies: I) -> impl Body<Data = Bytes, Error = Infallible> + Send
where
    I: IntoIterator<Item = Full<Bytes>>,
    I::IntoIter: Send,
{
    let combined_stream = stream::iter(
        bodies
            .into_iter()
            .map(|body| body.into_data_stream().map(|res| res.map(Frame::data))),
    )
    .flatten();
    StreamBody::new(combined_stream)
}

pub fn get_bytes_mut() -> BytesMut {
    BytesMut::zeroed(5)
}

pub fn freeze(mut buf: BytesMut) -> Full<Bytes> {
    let len = buf.len() as u32 - 5;
    buf[1..5].copy_from_slice(&len.to_be_bytes());
    Full::new(buf.freeze())
}

pub fn add_ok_trailer<T: Body<Data = Bytes, Error = Infallible> + Send + 'static>(
    body: T,
) -> body::Body {
    body::Body::new(body.with_trailers(async {
        let mut trailers = http::header::HeaderMap::new();
        Status::ok("").add_header(&mut trailers).unwrap();
        Some(Ok(trailers))
    }))
}

pub fn make_response(payload: Result<body::Body, Status>) -> http::Response<body::Body> {
    let mut response = match payload {
        Ok(body) => http::Response::new(body),
        Err(status) => {
            let mut response = http::Response::new(body::Body::default());
            status.add_header(response.headers_mut()).unwrap();
            response
        }
    };
    response
        .headers_mut()
        .insert(http::header::CONTENT_TYPE, metadata::GRPC_CONTENT_TYPE);
    response
}

type GrpcCallFuture = BoxFuture<'static, Result<body::Body, Status>>;

pub trait GrpcCall {
    fn grpc_ready<'a>(&'a mut self) -> BoxFuture<'a, Result<(), Status>>;
    fn grpc_call(&mut self, uri: &str, body: body::Body) -> GrpcCallFuture;
}

impl GrpcCall for Channel {
    fn grpc_ready<'a>(&'a mut self) -> BoxFuture<'a, Result<(), Status>> {
        Box::pin(async {
            poll_fn(|cx| self.poll_ready(cx))
                .await
                .map_err(|_| Status::unknown("service was not ready"))
        })
    }

    fn grpc_call(&mut self, uri: &str, body: body::Body) -> GrpcCallFuture {
        let request = http::Request::builder()
            .uri(uri)
            .method(http::Method::POST)
            .version(http::Version::HTTP_2)
            .header(http::header::TE, "trailers")
            .header(http::header::CONTENT_TYPE, metadata::GRPC_CONTENT_TYPE)
            .body(body)
            .unwrap();
        let call = self.call(request);
        Box::pin(async move {
            let response = call.await.map_err(|e| Status::from_error(Box::new(e)))?;
            if response.status() != http::StatusCode::OK {
                return Err(Status::unknown(format!(
                    "http status code {}",
                    response.status().as_u16()
                )));
            }
            if let Some(status) = Status::from_header_map(response.headers())
                && status.code() != Code::Ok
            {
                return Err(status);
            }
            Ok(response.into_body())
        })
    }
}

lazy_static! {
    static ref GRPC_CALL_TIMEOUT: Duration = {
        std::env::var("COPY_PORT_GRPC_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(300))
    };
}

pub async fn ready_call_parse<'a, T: Body<Data = Bytes, Error = Infallible> + Send + 'static>(
    uri: &str,
    proto: Vec<(u64, Func<'a>)>,
    body: T,
    channel: &mut Channel,
    context: &str,
) -> Result<(), Status> {
    let fut = async {
        channel.grpc_ready().await?;
        let body = channel.grpc_call(uri, body::Body::new(body)).await?;
        parse(proto, body, BodyKind::Response).await
    };

    match timeout(*GRPC_CALL_TIMEOUT, fut).await {
        Ok(result) => result,
        Err(_) => Err(Status::deadline_exceeded(format!(
            "copy_port gRPC call to {} timed out after {:?}{}",
            uri,
            *GRPC_CALL_TIMEOUT,
            if context.is_empty() {
                String::new()
            } else {
                format!(" ({context})")
            }
        ))),
    }
}

pub fn block_on<T: Send + 'static>(
    runtime: &runtime::Runtime,
    futures: Vec<BoxFuture<'static, T>>,
    timeout: Option<Duration>,
) -> Option<Vec<T>> {
    let num_futures = futures.len();
    if num_futures == 0 {
        return Some(Vec::new());
    }
    let mu_cv = Arc::new((Mutex::new(Vec::new()), Condvar::new()));
    let mu_cv2 = mu_cv.clone();
    runtime.spawn(async move {
        let result = join_all(futures.into_iter().map(task::spawn))
            .await
            .into_iter()
            .collect::<Result<_, _>>()
            .unwrap();
        let (mu, cv) = &*mu_cv;
        *mu.lock().unwrap() = result;
        cv.notify_one();
    });

    let (mu, cv) = &*mu_cv2;
    let mut guard = mu.lock().unwrap();
    let deadline = timeout.map(|d| std::time::Instant::now() + d);
    while guard.is_empty() {
        if let Some(deadline) = deadline {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                log::error!(
                    "copy_port block_on timed out after {:?} waiting for {} futures",
                    timeout.unwrap(),
                    num_futures
                );
                return None;
            }
            let (new_guard, timeout_result) = cv.wait_timeout(guard, remaining).unwrap();
            guard = new_guard;
            if timeout_result.timed_out() && guard.is_empty() {
                log::error!(
                    "copy_port block_on timed out after {:?} waiting for {} futures",
                    timeout.unwrap(),
                    num_futures
                );
                return None;
            }
        } else {
            guard = cv.wait(guard).unwrap();
        }
    }
    Some(mem::take(&mut guard))
}

pub(crate) const TRANSFER_FAILED_SENTINEL: usize = usize::MAX;

const MAX_PACING_SLEEP: Duration = Duration::from_secs(60);

fn pacing_sleep(
    total_bytes: u64,
    rate_limit_bytes_per_sec: u64,
    elapsed: Duration,
) -> (Duration, bool) {
    let expected =
        Duration::try_from_secs_f64(total_bytes as f64 / rate_limit_bytes_per_sec as f64)
            .unwrap_or(Duration::MAX);
    let sleep = expected.saturating_sub(elapsed);
    if sleep > MAX_PACING_SLEEP {
        (MAX_PACING_SLEEP, true)
    } else {
        (sleep, false)
    }
}

pub fn block_on_rate_limited(
    runtime: &runtime::Runtime,
    futures: Vec<BoxFuture<'static, (usize, u32)>>,
    rate_limit_bytes_per_sec: u64,
    max_concurrent: usize,
) -> Option<Vec<(usize, u32)>> {
    let max_concurrent = max_concurrent.max(1);
    let start = std::time::Instant::now();
    let mut total_bytes: u64 = 0;
    let mut results = Vec::with_capacity(futures.len());
    let mut iter = futures.into_iter();
    loop {
        let batch: Vec<_> = iter.by_ref().take(max_concurrent).collect();
        if batch.is_empty() {
            break;
        }
        let batch_size = batch.len();
        let batch_results = block_on(runtime, batch, None)?;
        let failed = batch_results
            .iter()
            .filter(|r| r.0 == TRANSFER_FAILED_SENTINEL)
            .count();
        if failed > 0 {
            log::error!(
                "rate_limiter: {failed}/{batch_size} transfers in batch failed; \
                 skipping remaining downloads so the caller can fail and retry"
            );
            results.extend(batch_results);
            return Some(results);
        }
        let mut batch_bytes: u64 = 0;
        for result in batch_results {
            batch_bytes = batch_bytes.saturating_add(result.0 as u64);
            results.push(result);
        }
        total_bytes = total_bytes.saturating_add(batch_bytes);
        let elapsed = start.elapsed();
        let (sleep, clamped) = pacing_sleep(total_bytes, rate_limit_bytes_per_sec, elapsed);
        if clamped {
            log::warn!(
                "rate_limiter: computed pacing sleep for {total_bytes} bytes exceeds the \
                 {MAX_PACING_SLEEP:?} cap; byte accounting is likely corrupt, capping"
            );
        }
        if !sleep.is_zero() {
            log::info!(
                "rate_limiter: batch of {} downloaded {:.2} GiB ({:.2} GiB total) in {:.2}s, \
                 sleeping {:.2}s to stay under {:.2} GiB/s",
                batch_size,
                batch_bytes as f64 / (1u64 << 30) as f64,
                total_bytes as f64 / (1u64 << 30) as f64,
                elapsed.as_secs_f64(),
                sleep.as_secs_f64(),
                rate_limit_bytes_per_sec as f64 / (1u64 << 30) as f64,
            );
            std::thread::sleep(sleep);
        }
    }
    Some(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn boxed(results: Vec<(usize, u32)>) -> Vec<BoxFuture<'static, (usize, u32)>> {
        results
            .into_iter()
            .map(|r| Box::pin(async move { r }) as BoxFuture<'static, (usize, u32)>)
            .collect()
    }

    fn test_runtime() -> runtime::Runtime {
        runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap()
    }

    #[test]
    fn rate_limited_returns_early_on_failed_transfer() {
        let rt = test_runtime();
        let futures = boxed(vec![(100, 1), (TRANSFER_FAILED_SENTINEL, 0), (200, 2)]);
        let start = std::time::Instant::now();
        let results = block_on_rate_limited(&rt, futures, 1, 2).unwrap();
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "must not pace a failed batch"
        );
        assert_eq!(results, vec![(100, 1), (TRANSFER_FAILED_SENTINEL, 0)]);
    }

    #[test]
    fn rate_limited_returns_all_results_on_success() {
        let rt = test_runtime();
        let futures = boxed(vec![(1024, 1), (2048, 2), (4096, 3)]);
        let results = block_on_rate_limited(&rt, futures, 1 << 30, 2).unwrap();
        assert_eq!(results, vec![(1024, 1), (2048, 2), (4096, 3)]);
    }

    #[test]
    fn pacing_sleep_clamps_corrupt_accounting() {
        let (sleep, clamped) = pacing_sleep(u64::MAX, 2 << 30, Duration::from_secs(10));
        assert!(clamped);
        assert_eq!(sleep, MAX_PACING_SLEEP);
    }

    #[test]
    fn pacing_sleep_normal_case_unclamped() {
        let (sleep, clamped) = pacing_sleep(4 << 30, 2 << 30, Duration::from_secs(1));
        assert!(!clamped);
        assert_eq!(sleep, Duration::from_secs(1));
        let (sleep, clamped) = pacing_sleep(1 << 30, 2 << 30, Duration::from_secs(5));
        assert!(!clamped);
        assert_eq!(sleep, Duration::ZERO);
    }
}
