//! Span creation utilities for OpenTelemetry instrumentation

use opentelemetry::trace::SpanKind;
use tracing::Span;

/// Crate version for telemetry attributes
pub const PROVIDER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Semantic convention attributes for feature flag evaluation
pub mod attributes {
    pub const FEATURE_FLAG_KEY: &str = "feature_flag.key";
    pub const FEATURE_FLAG_VARIANT: &str = "feature_flag.variant";
    pub const FEATURE_FLAG_PROVIDER_NAME: &str = "feature_flag.provider_name";
    pub const FEATURE_FLAG_PROVIDER_VERSION: &str = "feature_flag.provider_version";
    pub const RPC_SYSTEM: &str = "rpc.system";
    pub const RPC_SERVICE: &str = "rpc.service";
    pub const RPC_METHOD: &str = "rpc.method";
    pub const RPC_GRPC_STATUS_CODE: &str = "rpc.grpc.status_code";
    pub const SERVER_ADDRESS: &str = "server.address";
    pub const SERVER_PORT: &str = "server.port";
    pub const OTEL_STATUS_CODE: &str = "otel.status_code";
    pub const ERROR_TYPE: &str = "error.type";
}

/// Create a span for flag evaluation operations
#[must_use]
pub fn make_flag_evaluation_span(flag_key: &str, resolver_type: &str) -> Span {
    tracing::trace_span!(
        "feature_flag.evaluate",
        otel.name = format!("evaluate {}", flag_key),
        otel.kind = ?SpanKind::Client,
        { attributes::FEATURE_FLAG_KEY } = flag_key,
        { attributes::FEATURE_FLAG_PROVIDER_NAME } = "flagd",
        { attributes::FEATURE_FLAG_PROVIDER_VERSION } = PROVIDER_VERSION,
        resolver_type = resolver_type,
        { attributes::FEATURE_FLAG_VARIANT } = tracing::field::Empty,
        { attributes::OTEL_STATUS_CODE } = tracing::field::Empty,
        { attributes::ERROR_TYPE } = tracing::field::Empty,
    )
}

/// Create a span for gRPC client calls
#[must_use]
pub fn make_grpc_client_span(service: &str, method: &str, host: &str, port: u16) -> Span {
    let span_name = format!("{}/{}", service, method);
    tracing::trace_span!(
        "grpc.client",
        otel.name = %span_name,
        otel.kind = ?SpanKind::Client,
        { attributes::RPC_SYSTEM } = "grpc",
        { attributes::RPC_SERVICE } = %service,
        { attributes::RPC_METHOD } = %method,
        { attributes::SERVER_ADDRESS } = %host,
        { attributes::SERVER_PORT } = port,
        { attributes::RPC_GRPC_STATUS_CODE } = tracing::field::Empty,
        { attributes::OTEL_STATUS_CODE } = tracing::field::Empty,
        { attributes::ERROR_TYPE } = tracing::field::Empty,
    )
}

/// Create a span for HTTP client calls (REST/OFREP)
#[must_use]
pub fn make_http_client_span(method: &str, url: &str, host: &str) -> Span {
    let span_name = format!("{} {}", method, url);
    tracing::trace_span!(
        "http.client",
        otel.name = %span_name,
        otel.kind = ?SpanKind::Client,
        http.request.method = %method,
        url.full = %url,
        server.address = %host,
        http.response.status_code = tracing::field::Empty,
        { attributes::OTEL_STATUS_CODE } = tracing::field::Empty,
        { attributes::ERROR_TYPE } = tracing::field::Empty,
    )
}

/// Record successful evaluation result on a span
pub fn record_evaluation_success(span: &Span, variant: &str) {
    span.record(attributes::FEATURE_FLAG_VARIANT, variant);
    span.record(attributes::OTEL_STATUS_CODE, "OK");
}

/// Record successful evaluation without a variant (e.g., cached results)
pub fn record_evaluation_success_no_variant(span: &Span) {
    span.record(attributes::OTEL_STATUS_CODE, "OK");
}

/// Record evaluation error on a span
pub fn record_evaluation_error(span: &Span, error: &str) {
    span.record(attributes::OTEL_STATUS_CODE, "ERROR");
    span.record(attributes::ERROR_TYPE, error);
}

/// Record gRPC status on a span
pub fn record_grpc_status(span: &Span, status_code: i32) {
    span.record(attributes::RPC_GRPC_STATUS_CODE, status_code);
    if status_code == 0 {
        span.record(attributes::OTEL_STATUS_CODE, "OK");
    } else {
        span.record(attributes::OTEL_STATUS_CODE, "ERROR");
    }
}

/// Record HTTP response status on a span
pub fn record_http_status(span: &Span, status_code: u16) {
    span.record("http.response.status_code", status_code);
    if (200..300).contains(&status_code) {
        span.record(attributes::OTEL_STATUS_CODE, "OK");
    } else {
        span.record(attributes::OTEL_STATUS_CODE, "ERROR");
    }
}
