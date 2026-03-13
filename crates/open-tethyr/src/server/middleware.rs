//! Server Middleware

use axum::http::{Request, Response, HeaderValue};
use axum::body::Body;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use uuid::Uuid;
use std::future::Future;
use std::pin::Pin;

/// Correlation ID layer
#[derive(Clone)]
pub struct CorrelationIdLayer;

impl<S> Layer<S> for CorrelationIdLayer {
    type Service = CorrelationIdService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        CorrelationIdService { inner }
    }
}

#[derive(Clone)]
pub struct CorrelationIdService<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for CorrelationIdService<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let correlation_id = Uuid::new_v4().to_string();
        let mut svc = self.inner.clone();
        Box::pin(async move {
            let mut response = svc.call(req).await?;
            response.headers_mut().insert(
                "X-Correlation-Id",
                HeaderValue::from_str(&correlation_id).unwrap_or_else(|_| HeaderValue::from_static("unknown")),
            );
            Ok(response)
        })
    }
}
