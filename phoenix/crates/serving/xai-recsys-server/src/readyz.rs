// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};

use bytes::Bytes;
use http::{Method, Request, Response, StatusCode};
use http_body_util::Full;

#[derive(Clone)]
pub(crate) struct ReadyzProbe {
    pub(crate) is_accepting: Arc<AtomicBool>,
    pub(crate) is_terminating: Arc<AtomicBool>,
}

impl ReadyzProbe {
    fn is_ready(&self) -> bool {
        self.is_accepting.load(Ordering::Acquire) && !self.is_terminating.load(Ordering::Acquire)
    }
}

#[derive(Clone)]
pub(crate) struct ReadyzLayer {
    probe: Option<ReadyzProbe>,
}

impl ReadyzLayer {
    pub(crate) fn new(probe: Option<ReadyzProbe>) -> Self {
        Self { probe }
    }
}

impl<S> tower::Layer<S> for ReadyzLayer {
    type Service = ReadyzService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ReadyzService {
            inner,
            probe: self.probe.clone(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct ReadyzService<S> {
    inner: S,
    probe: Option<ReadyzProbe>,
}

impl<S> tower::Service<Request<tonic::body::Body>> for ReadyzService<S>
where
    S: tower::Service<Request<tonic::body::Body>, Response = Response<tonic::body::Body>>,
{
    type Response = Response<tonic::body::Body>;
    type Error = S::Error;
    type Future =
        futures::future::Either<std::future::Ready<Result<Self::Response, Self::Error>>, S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<tonic::body::Body>) -> Self::Future {
        if let Some(probe) = &self.probe
            && req.method() == Method::GET
            && req.uri().path() == "/readyz"
        {
            let (status, body) = if probe.is_ready() {
                (StatusCode::OK, "Ready")
            } else {
                (StatusCode::SERVICE_UNAVAILABLE, "Not Ready")
            };
            let response = Response::builder()
                .status(status)
                .header(http::header::CONTENT_TYPE, "text/plain")
                .body(tonic::body::Body::new(Full::new(Bytes::from_static(
                    body.as_bytes(),
                ))))
                .expect("static readyz response is valid");
            return futures::future::Either::Left(std::future::ready(Ok(response)));
        }
        futures::future::Either::Right(self.inner.call(req))
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use http_body_util::BodyExt;
    use tower::{Layer, Service, ServiceExt};

    use super::*;

    fn probe(accepting: bool, terminating: bool) -> ReadyzProbe {
        ReadyzProbe {
            is_accepting: Arc::new(AtomicBool::new(accepting)),
            is_terminating: Arc::new(AtomicBool::new(terminating)),
        }
    }

    fn inner() -> impl Service<
        Request<tonic::body::Body>,
        Response = Response<tonic::body::Body>,
        Error = Infallible,
    > + Clone {
        tower::service_fn(|_req: Request<tonic::body::Body>| async {
            Ok(Response::new(tonic::body::Body::new(Full::new(
                Bytes::from_static(b"grpc-inner"),
            ))))
        })
    }

    fn request(method: Method, path: &str) -> Request<tonic::body::Body> {
        Request::builder()
            .method(method)
            .uri(path)
            .body(tonic::body::Body::default())
            .unwrap()
    }

    async fn call(
        svc: &mut ReadyzService<
            impl Service<
                Request<tonic::body::Body>,
                Response = Response<tonic::body::Body>,
                Error = Infallible,
            > + Clone,
        >,
        method: Method,
        path: &str,
    ) -> (StatusCode, String) {
        let resp = svc
            .ready()
            .await
            .unwrap()
            .call(request(method, path))
            .await
            .unwrap();
        let status = resp.status();
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8_lossy(&body).into_owned())
    }

    #[tokio::test]
    async fn ready_returns_200() {
        let mut svc = ReadyzLayer::new(Some(probe(true, false))).layer(inner());
        let (status, body) = call(&mut svc, Method::GET, "/readyz").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "Ready");
    }

    #[tokio::test]
    async fn draining_returns_503() {
        let mut svc = ReadyzLayer::new(Some(probe(false, false))).layer(inner());
        let (status, body) = call(&mut svc, Method::GET, "/readyz").await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body, "Not Ready");
    }

    #[tokio::test]
    async fn terminating_returns_503() {
        let mut svc = ReadyzLayer::new(Some(probe(true, true))).layer(inner());
        let (status, _) = call(&mut svc, Method::GET, "/readyz").await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn readiness_flip_is_observed_live() {
        let p = probe(true, false);
        let mut svc = ReadyzLayer::new(Some(p.clone())).layer(inner());
        assert_eq!(
            call(&mut svc, Method::GET, "/readyz").await.0,
            StatusCode::OK
        );
        p.is_accepting.store(false, Ordering::Release);
        assert_eq!(
            call(&mut svc, Method::GET, "/readyz").await.0,
            StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[tokio::test]
    async fn grpc_requests_pass_through() {
        let mut svc = ReadyzLayer::new(Some(probe(false, false))).layer(inner());
        let (status, body) = call(
            &mut svc,
            Method::POST,
            "/xai.recsys.Predictor/PredictNextActions",
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "grpc-inner");
    }

    #[tokio::test]
    async fn post_readyz_passes_through() {
        let mut svc = ReadyzLayer::new(Some(probe(false, false))).layer(inner());
        let (_, body) = call(&mut svc, Method::POST, "/readyz").await;
        assert_eq!(body, "grpc-inner");
    }

    #[tokio::test]
    async fn disabled_is_pure_passthrough() {
        let mut svc = ReadyzLayer::new(None).layer(inner());
        let (_, body) = call(&mut svc, Method::GET, "/readyz").await;
        assert_eq!(body, "grpc-inner");
    }
}
