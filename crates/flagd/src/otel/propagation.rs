//! OpenTelemetry context propagation utilities
//!
//! Provides context injection/extraction for HTTP headers to propagate
//! trace context across service boundaries.

use opentelemetry::Context;
use opentelemetry::propagation::{Extractor, Injector};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Extract the OpenTelemetry context from the current tracing span
#[must_use]
pub fn context_from_current_span() -> Context {
    Span::current().context()
}

/// Extract the OpenTelemetry context from a specific tracing span
#[must_use]
pub fn context_from_span(span: &Span) -> Context {
    span.context()
}

/// Inject OpenTelemetry context into HTTP headers for propagation
pub fn inject_context<I: Injector>(context: &Context, injector: &mut I) {
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(context, injector);
    });
}

/// Extract OpenTelemetry context from HTTP headers
#[must_use]
pub fn extract_context<E: Extractor>(extractor: &E) -> Context {
    opentelemetry::global::get_text_map_propagator(|propagator| propagator.extract(extractor))
}

/// HTTP header injector for tonic/http requests
pub struct HeaderInjector<'a>(pub &'a mut http::HeaderMap);

impl Injector for HeaderInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(header_name) = http::header::HeaderName::try_from(key)
            && let Ok(header_value) = http::header::HeaderValue::from_str(&value)
        {
            self.0.insert(header_name, header_value);
        }
    }
}

/// HTTP header extractor for incoming requests
pub struct HeaderExtractor<'a>(pub &'a http::HeaderMap);

impl Extractor for HeaderExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(http::HeaderName::as_str).collect()
    }
}

/// Inject context into HTTP headers from the current span
pub fn inject_context_to_headers(headers: &mut http::HeaderMap) {
    let context = context_from_current_span();
    let mut injector = HeaderInjector(headers);
    inject_context(&context, &mut injector);
}

/// Inject context into HTTP headers from a specific span
pub fn inject_span_context_to_headers(span: &Span, headers: &mut http::HeaderMap) {
    let context = context_from_span(span);
    let mut injector = HeaderInjector(headers);
    inject_context(&context, &mut injector);
}

#[cfg(feature = "rest")]
/// Inject context into reqwest headers
pub fn inject_context_to_reqwest_headers(headers: &mut reqwest::header::HeaderMap) {
    use opentelemetry::propagation::Injector;

    struct ReqwestHeaderInjector<'a>(&'a mut reqwest::header::HeaderMap);

    impl Injector for ReqwestHeaderInjector<'_> {
        fn set(&mut self, key: &str, value: String) {
            if let Ok(header_name) = reqwest::header::HeaderName::try_from(key)
                && let Ok(header_value) = reqwest::header::HeaderValue::from_str(&value)
            {
                self.0.insert(header_name, header_value);
            }
        }
    }

    let context = context_from_current_span();
    let mut injector = ReqwestHeaderInjector(headers);
    inject_context(&context, &mut injector);
}
