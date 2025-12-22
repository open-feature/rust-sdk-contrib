//! gRPC client instrumentation layer for OpenTelemetry
//!
//! Provides a Tower layer that wraps gRPC client calls with tracing spans
//! and context propagation for distributed tracing.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use http::{Request, Response, Uri};
use pin_project_lite::pin_project;
use tonic::client::GrpcService;
use tower::{Layer, Service};
use tracing::Span;

use super::propagation::{HeaderInjector, context_from_span, inject_context};
use super::span::{make_grpc_client_span, record_grpc_status};

/// Tower layer that adds OpenTelemetry instrumentation to gRPC clients.
///
/// This layer:
/// - Creates a tracing span for each gRPC call
/// - Propagates OpenTelemetry context via HTTP headers
/// - Records gRPC status codes on span completion
#[derive(Default, Debug, Clone)]
pub struct OtelGrpcLayer {
    host: String,
    port: u16,
}

impl OtelGrpcLayer {
    /// Create a new OtelGrpcLayer with the target host and port
    #[must_use]
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }
}

impl<S> Layer<S> for OtelGrpcLayer {
    type Service = OtelGrpcService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        OtelGrpcService {
            inner,
            host: self.host.clone(),
            port: self.port,
        }
    }
}

/// Instrumented gRPC service wrapper
#[derive(Debug, Clone)]
pub struct OtelGrpcService<S> {
    inner: S,
    host: String,
    port: u16,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for OtelGrpcService<S>
where
    S: GrpcService<ReqBody, ResponseBody = ResBody> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: std::error::Error + 'static,
    ReqBody: Send + 'static,
    ResBody: http_body::Body,
{
    type Response = Response<ResBody>;
    type Error = S::Error;
    type Future = OtelResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let (service, method) = extract_service_method(req.uri());
        let span = make_grpc_client_span(&service, &method, &self.host, self.port);

        // Inject trace context into headers for propagation
        let context = context_from_span(&span);
        inject_context(&context, &mut HeaderInjector(req.headers_mut()));

        let future = {
            let _enter = span.enter();
            self.inner.call(req)
        };

        OtelResponseFuture {
            inner: future,
            span,
        }
    }
}

/// Extract gRPC service and method from URI path
fn extract_service_method(uri: &Uri) -> (String, String) {
    let path = uri.path();
    let mut parts = path.split('/').filter(|x| !x.is_empty());
    let service = parts.next().unwrap_or("unknown").to_string();
    let method = parts.next().unwrap_or("unknown").to_string();
    (service, method)
}

pin_project! {
    /// Response future that records span completion
    pub struct OtelResponseFuture<F> {
        #[pin]
        inner: F,
        span: Span,
    }
}

impl<F, ResBody, E> Future for OtelResponseFuture<F>
where
    F: Future<Output = Result<Response<ResBody>, E>>,
    E: std::error::Error + 'static,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = this.span.enter();

        match this.inner.poll(cx) {
            Poll::Ready(result) => {
                match &result {
                    Ok(response) => {
                        // Try to extract grpc-status from headers
                        let status = response
                            .headers()
                            .get("grpc-status")
                            .and_then(|v| v.to_str().ok())
                            .and_then(|v| v.parse::<i32>().ok())
                            .unwrap_or(0);
                        record_grpc_status(this.span, status);
                    }
                    Err(e) => {
                        this.span.record("otel.status_code", "ERROR");
                        this.span.record("error.type", e.to_string().as_str());
                    }
                }
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
