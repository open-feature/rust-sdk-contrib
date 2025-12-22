//! HTTP client instrumentation for OpenTelemetry (REST/OFREP resolver)
//!
//! Provides utilities to instrument reqwest HTTP calls with tracing spans
//! and context propagation.

use tracing::Span;

use super::propagation::inject_context_to_reqwest_headers;
use super::span::{make_http_client_span, record_http_status};

/// Create an instrumented span for an HTTP request and inject trace context
///
/// Returns a span that should be entered during the request execution.
/// Call `record_http_response` after the request completes.
#[must_use]
pub fn instrument_http_request(
    method: &str,
    url: &str,
    host: &str,
    headers: &mut reqwest::header::HeaderMap,
) -> Span {
    let span = make_http_client_span(method, url, host);

    // Inject trace context into headers using the span's context
    {
        let _enter = span.enter();
        inject_context_to_reqwest_headers(headers);
    }

    span
}

/// Record HTTP response status on the span
pub fn record_http_response(span: &Span, status_code: u16) {
    record_http_status(span, status_code);
}

/// Record HTTP error on the span
pub fn record_http_error(span: &Span, error: &str) {
    span.record("otel.status_code", "ERROR");
    span.record("error.type", error);
}
