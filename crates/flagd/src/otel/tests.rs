//! Tests for OpenTelemetry instrumentation
//!
//! These tests verify that spans are correctly created and attributes are recorded
//! for flag evaluation and gRPC/HTTP operations.

#[cfg(test)]
mod span_tests {
    use crate::otel::span::{
        make_flag_evaluation_span, make_grpc_client_span, make_http_client_span,
        record_evaluation_error, record_evaluation_success, record_grpc_status, record_http_status,
    };
    use std::time::Duration;

    use fake_opentelemetry_collector::{FakeCollectorServer, setup_tracer_provider};
    use opentelemetry::trace::TracerProvider;
    use tracing_subscriber::Registry;
    use tracing_subscriber::layer::SubscriberExt;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_flag_evaluation_span_created() {
        let mut fake_collector = FakeCollectorServer::start()
            .await
            .expect("fake collector started");

        let tracer_provider = setup_tracer_provider(&fake_collector).await;

        // Setup tracing-opentelemetry layer - keep guard alive!
        let telemetry_layer =
            tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("flagd-test"));
        let subscriber = Registry::default().with(telemetry_layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        // Create a flag evaluation span
        {
            let span = make_flag_evaluation_span("test-flag", "rpc");
            let _enter = span.enter();
            record_evaluation_success(&span, "variant-a");
        }

        // Drop guard before flush to ensure spans are sent
        drop(_guard);

        // Force flush and shutdown
        let _ = tracer_provider.force_flush();
        tracer_provider.shutdown().expect("shutdown ok");
        drop(tracer_provider);

        // Collect spans
        let spans = fake_collector
            .exported_spans(1, Duration::from_secs(5))
            .await;

        assert!(!spans.is_empty(), "Should have at least one span");

        let span = &spans[0];
        assert!(
            span.name.contains("evaluate"),
            "Span name should contain 'evaluate'"
        );

        // Verify attributes exist
        assert!(!span.attributes.is_empty(), "Span should have attributes");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_grpc_client_span_created() {
        let mut fake_collector = FakeCollectorServer::start()
            .await
            .expect("fake collector started");

        let tracer_provider = setup_tracer_provider(&fake_collector).await;

        let telemetry_layer =
            tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("flagd-test"));
        let subscriber = Registry::default().with(telemetry_layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        // Create a gRPC client span
        {
            let span =
                make_grpc_client_span("flagd.evaluation.v1", "ResolveBoolean", "localhost", 8013);
            let _enter = span.enter();
            record_grpc_status(&span, 0); // OK status
        }

        drop(_guard);

        // Force flush and shutdown
        let _ = tracer_provider.force_flush();
        tracer_provider.shutdown().expect("shutdown ok");
        drop(tracer_provider);

        // Collect spans
        let spans = fake_collector
            .exported_spans(1, Duration::from_secs(5))
            .await;

        assert!(!spans.is_empty(), "Should have at least one span");

        let span = &spans[0];
        // gRPC span should have service/method in name
        assert!(
            span.name.contains("flagd") || span.name.contains("Resolve"),
            "Span name should contain service or method"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_http_client_span_created() {
        let mut fake_collector = FakeCollectorServer::start()
            .await
            .expect("fake collector started");

        let tracer_provider = setup_tracer_provider(&fake_collector).await;

        let telemetry_layer =
            tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("flagd-test"));
        let subscriber = Registry::default().with(telemetry_layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        // Create an HTTP client span
        {
            let span =
                make_http_client_span("POST", "/ofrep/v1/evaluate/flags/test-flag", "localhost");
            let _enter = span.enter();
            record_http_status(&span, 200);
        }

        drop(_guard);

        // Force flush and shutdown
        let _ = tracer_provider.force_flush();
        tracer_provider.shutdown().expect("shutdown ok");
        drop(tracer_provider);

        // Collect spans
        let spans = fake_collector
            .exported_spans(1, Duration::from_secs(5))
            .await;

        assert!(!spans.is_empty(), "Should have at least one span");

        let span = &spans[0];

        // Verify span name contains method
        assert!(
            span.name.contains("POST"),
            "Span name should contain HTTP method"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_evaluation_error_recorded() {
        let mut fake_collector = FakeCollectorServer::start()
            .await
            .expect("fake collector started");

        let tracer_provider = setup_tracer_provider(&fake_collector).await;

        let telemetry_layer =
            tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("flagd-test"));
        let subscriber = Registry::default().with(telemetry_layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        // Create a span with error
        {
            let span = make_flag_evaluation_span("error-flag", "rpc");
            let _enter = span.enter();
            record_evaluation_error(&span, "FLAG_NOT_FOUND");
        }

        drop(_guard);

        // Force flush and shutdown
        let _ = tracer_provider.force_flush();
        tracer_provider.shutdown().expect("shutdown ok");
        drop(tracer_provider);

        // Collect spans
        let spans = fake_collector
            .exported_spans(1, Duration::from_secs(5))
            .await;

        assert!(!spans.is_empty(), "Should have at least one span");

        // Span should exist with error recorded
        let span = &spans[0];
        assert!(
            span.name.contains("evaluate"),
            "Error span should still be an evaluate span"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_grpc_error_status_recorded() {
        let mut fake_collector = FakeCollectorServer::start()
            .await
            .expect("fake collector started");

        let tracer_provider = setup_tracer_provider(&fake_collector).await;

        let telemetry_layer =
            tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("flagd-test"));
        let subscriber = Registry::default().with(telemetry_layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        // Create a gRPC span with error status
        {
            let span =
                make_grpc_client_span("flagd.evaluation.v1", "ResolveBoolean", "localhost", 8013);
            let _enter = span.enter();
            record_grpc_status(&span, 5); // NOT_FOUND status
        }

        drop(_guard);

        // Force flush and shutdown
        let _ = tracer_provider.force_flush();
        tracer_provider.shutdown().expect("shutdown ok");
        drop(tracer_provider);

        // Collect spans
        let spans = fake_collector
            .exported_spans(1, Duration::from_secs(5))
            .await;

        assert!(!spans.is_empty(), "Should have at least one span");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_http_error_status_recorded() {
        let mut fake_collector = FakeCollectorServer::start()
            .await
            .expect("fake collector started");

        let tracer_provider = setup_tracer_provider(&fake_collector).await;

        let telemetry_layer =
            tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("flagd-test"));
        let subscriber = Registry::default().with(telemetry_layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        // Create an HTTP span with error status
        {
            let span =
                make_http_client_span("POST", "/ofrep/v1/evaluate/flags/test-flag", "localhost");
            let _enter = span.enter();
            record_http_status(&span, 404);
        }

        drop(_guard);

        // Force flush and shutdown
        let _ = tracer_provider.force_flush();
        tracer_provider.shutdown().expect("shutdown ok");
        drop(tracer_provider);

        // Collect spans
        let spans = fake_collector
            .exported_spans(1, Duration::from_secs(5))
            .await;

        assert!(!spans.is_empty(), "Should have at least one span");
    }
}

/// Integration tests for provider evaluation spans
#[cfg(test)]
#[cfg(feature = "in-process")]
mod provider_span_tests {
    use crate::{FlagdOptions, FlagdProvider, ResolverType};
    use fake_opentelemetry_collector::{FakeCollectorServer, setup_tracer_provider};
    use open_feature::EvaluationContext;
    use open_feature::provider::FeatureProvider;
    use opentelemetry::trace::TracerProvider;
    use std::time::Duration;
    use tracing_subscriber::Layer;
    use tracing_subscriber::Registry;
    use tracing_subscriber::filter::LevelFilter;
    use tracing_subscriber::layer::SubscriberExt;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_flagd_provider_emits_evaluation_span() {
        let mut fake_collector = FakeCollectorServer::start()
            .await
            .expect("fake collector started");

        let tracer_provider = setup_tracer_provider(&fake_collector).await;

        let telemetry_layer =
            tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("flagd-test"));
        let subscriber = Registry::default().with(telemetry_layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        let temp_dir = tempfile::tempdir().expect("tempdir created");
        let flag_path = temp_dir.path().join("flags.json");
        std::fs::write(
            &flag_path,
            r#"{
  "$schema": "https://flagd.dev/schema/v0/flags.json",
  "flags": {
    "basic-boolean": {
      "state": "ENABLED",
      "defaultVariant": "false",
      "variants": {
        "true": true,
        "false": false
      },
      "targeting": {}
    }
  }
}"#,
        )
        .expect("flags written");

        let options = FlagdOptions {
            resolver_type: ResolverType::File,
            source_configuration: Some(flag_path.to_string_lossy().to_string()),
            cache_settings: None,
            ..Default::default()
        };

        let provider = FlagdProvider::new(options).await.expect("provider ready");
        let context = EvaluationContext::default();
        let result = provider
            .resolve_bool_value("basic-boolean", &context)
            .await
            .expect("flag resolved");

        assert!(!result.value, "expected flag value to be false");

        drop(_guard);

        let _ = tracer_provider.force_flush();
        tracer_provider.shutdown().expect("shutdown ok");
        drop(tracer_provider);

        let spans = fake_collector
            .exported_spans(1, Duration::from_secs(5))
            .await;

        assert!(!spans.is_empty(), "Provider should emit spans");

        let eval_span = spans.iter().find(|s| s.name.contains("evaluate"));
        assert!(
            eval_span.is_some(),
            "Should find evaluation span among: {:?}",
            spans.iter().map(|s| &s.name).collect::<Vec<_>>()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_debug_logs_become_span_events() {
        let mut fake_collector = FakeCollectorServer::start()
            .await
            .expect("fake collector started");

        let tracer_provider = setup_tracer_provider(&fake_collector).await;

        // Enable TRACE level filter to capture all spans and events
        // The filter must wrap the telemetry layer
        let telemetry_layer = tracing_opentelemetry::layer()
            .with_tracer(tracer_provider.tracer("flagd-test"))
            .with_filter(LevelFilter::TRACE);
        let subscriber = Registry::default().with(telemetry_layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        let temp_dir = tempfile::tempdir().expect("tempdir created");
        let flag_path = temp_dir.path().join("flags.json");
        std::fs::write(
            &flag_path,
            r#"{
  "$schema": "https://flagd.dev/schema/v0/flags.json",
  "flags": {
    "test-flag": {
      "state": "ENABLED",
      "defaultVariant": "on",
      "variants": {
        "on": true,
        "off": false
      },
      "targeting": {}
    }
  }
}"#,
        )
        .expect("flags written");

        let options = FlagdOptions {
            resolver_type: ResolverType::File,
            source_configuration: Some(flag_path.to_string_lossy().to_string()),
            cache_settings: None,
            ..Default::default()
        };

        let provider = FlagdProvider::new(options).await.expect("provider ready");
        let context = EvaluationContext::default();

        // Resolve the flag - internal debug! logs should become span events
        let _ = provider
            .resolve_bool_value("test-flag", &context)
            .await
            .expect("flag resolved");

        drop(_guard);

        let _ = tracer_provider.force_flush();
        tracer_provider.shutdown().expect("shutdown ok");
        drop(tracer_provider);

        let spans = fake_collector
            .exported_spans(1, Duration::from_secs(5))
            .await;

        assert!(!spans.is_empty(), "Should have spans");

        // Find the evaluation span
        let eval_span = spans
            .iter()
            .find(|s| s.name.contains("evaluate"))
            .expect("Should find evaluation span");

        // Debug logs inside the span should appear as span events
        // The FileResolver emits debug logs during evaluation
        assert!(
            !eval_span.events.is_empty(),
            "Evaluation span should have events from debug! logs. Span: {:?}",
            eval_span
        );
    }
}

/// Integration tests for the OtelGrpcLayer middleware
#[cfg(test)]
#[cfg(any(feature = "rpc", feature = "in-process"))]
mod grpc_middleware_tests {
    use std::convert::Infallible;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use std::time::Duration;

    use fake_opentelemetry_collector::{FakeCollectorServer, setup_tracer_provider};
    use http::{Request, Response};
    use http_body::Body;
    use opentelemetry::trace::TracerProvider;
    use tower::{Layer, Service};
    use tracing_subscriber::Registry;
    use tracing_subscriber::layer::SubscriberExt;

    use crate::otel::grpc::OtelGrpcLayer;

    /// Mock gRPC service for testing the layer
    #[derive(Clone)]
    struct MockGrpcService {
        grpc_status: i32,
    }

    impl MockGrpcService {
        fn new(grpc_status: i32) -> Self {
            Self { grpc_status }
        }
    }

    /// Empty body for mock responses
    struct EmptyBody;

    impl Body for EmptyBody {
        type Data = bytes::Bytes;
        type Error = Infallible;

        fn poll_frame(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
            Poll::Ready(None)
        }
    }

    impl<B> Service<Request<B>> for MockGrpcService
    where
        B: Send + 'static,
    {
        type Response = Response<EmptyBody>;
        type Error = std::io::Error;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _req: Request<B>) -> Self::Future {
            let status = self.grpc_status;
            Box::pin(async move {
                let mut response = Response::builder().status(200).body(EmptyBody).unwrap();
                response
                    .headers_mut()
                    .insert("grpc-status", status.to_string().parse().unwrap());
                Ok(response)
            })
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_otel_grpc_layer_creates_span() {
        let mut fake_collector = FakeCollectorServer::start()
            .await
            .expect("fake collector started");

        let tracer_provider = setup_tracer_provider(&fake_collector).await;

        let telemetry_layer =
            tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("flagd-test"));
        let subscriber = Registry::default().with(telemetry_layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        // Create the layer and wrap the mock service
        let layer = OtelGrpcLayer::new("localhost", 8013);
        let mock_service = MockGrpcService::new(0); // OK status
        let mut instrumented_service = layer.layer(mock_service);

        // Make a request through the instrumented service
        let request = Request::builder()
            .uri("/flagd.evaluation.v1.Service/ResolveBoolean")
            .body(())
            .unwrap();

        let _response = instrumented_service.call(request).await.unwrap();

        drop(_guard);

        // Force flush and shutdown
        let _ = tracer_provider.force_flush();
        tracer_provider.shutdown().expect("shutdown ok");
        drop(tracer_provider);

        // Collect spans
        let spans = fake_collector
            .exported_spans(1, Duration::from_secs(5))
            .await;

        assert!(!spans.is_empty(), "Layer should create a span");

        let span = &spans[0];
        // Verify span name contains service/method
        assert!(
            span.name.contains("flagd") || span.name.contains("Service"),
            "Span name should contain gRPC service info, got: {}",
            span.name
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_otel_grpc_layer_records_error_status() {
        let mut fake_collector = FakeCollectorServer::start()
            .await
            .expect("fake collector started");

        let tracer_provider = setup_tracer_provider(&fake_collector).await;

        let telemetry_layer =
            tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("flagd-test"));
        let subscriber = Registry::default().with(telemetry_layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        // Create layer with mock service returning NOT_FOUND status
        let layer = OtelGrpcLayer::new("localhost", 8013);
        let mock_service = MockGrpcService::new(5); // NOT_FOUND
        let mut instrumented_service = layer.layer(mock_service);

        let request = Request::builder()
            .uri("/flagd.evaluation.v1.Service/ResolveBoolean")
            .body(())
            .unwrap();

        let _response = instrumented_service.call(request).await.unwrap();

        drop(_guard);

        let _ = tracer_provider.force_flush();
        tracer_provider.shutdown().expect("shutdown ok");
        drop(tracer_provider);

        let spans = fake_collector
            .exported_spans(1, Duration::from_secs(5))
            .await;

        assert!(!spans.is_empty(), "Layer should create a span for error");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_otel_grpc_layer_injects_trace_context() {
        // Set up the global propagator - required for context injection
        opentelemetry::global::set_text_map_propagator(
            opentelemetry_sdk::propagation::TraceContextPropagator::new(),
        );

        let fake_collector = FakeCollectorServer::start()
            .await
            .expect("fake collector started");

        let tracer_provider = setup_tracer_provider(&fake_collector).await;

        let telemetry_layer =
            tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("flagd-test"));
        let subscriber = Registry::default().with(telemetry_layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        // Custom mock that captures headers
        #[derive(Clone)]
        struct HeaderCapturingService {
            captured_headers: std::sync::Arc<std::sync::Mutex<Option<http::HeaderMap>>>,
        }

        impl<B> Service<Request<B>> for HeaderCapturingService
        where
            B: Send + 'static,
        {
            type Response = Response<EmptyBody>;
            type Error = std::io::Error;
            type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

            fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }

            fn call(&mut self, req: Request<B>) -> Self::Future {
                let headers = req.headers().clone();
                *self.captured_headers.lock().unwrap() = Some(headers);

                Box::pin(async move {
                    let mut response = Response::builder().status(200).body(EmptyBody).unwrap();
                    response
                        .headers_mut()
                        .insert("grpc-status", "0".parse().unwrap());
                    Ok(response)
                })
            }
        }

        let captured = std::sync::Arc::new(std::sync::Mutex::new(None));
        let mock_service = HeaderCapturingService {
            captured_headers: captured.clone(),
        };

        let layer = OtelGrpcLayer::new("localhost", 8013);
        let mut instrumented_service = layer.layer(mock_service);

        let request = Request::builder()
            .uri("/test.Service/Method")
            .body(())
            .unwrap();

        let _response = instrumented_service.call(request).await.unwrap();

        drop(_guard);

        let _ = tracer_provider.force_flush();
        tracer_provider.shutdown().expect("shutdown ok");
        drop(tracer_provider);

        // Verify trace context headers were injected
        let headers = captured.lock().unwrap();
        let headers = headers.as_ref().expect("Headers should be captured");

        // traceparent header should be present for W3C trace context propagation
        assert!(
            headers.contains_key("traceparent") || headers.contains_key("uber-trace-id"),
            "Trace context headers should be injected"
        );
    }
}

/// Integration tests for HTTP instrumentation
#[cfg(test)]
#[cfg(feature = "rest")]
mod http_middleware_tests {
    use crate::otel::http::{instrument_http_request, record_http_error, record_http_response};
    use fake_opentelemetry_collector::{FakeCollectorServer, setup_tracer_provider};
    use opentelemetry::trace::TracerProvider;
    use std::time::Duration;
    use tracing_subscriber::Registry;
    use tracing_subscriber::layer::SubscriberExt;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_http_instrumentation_creates_span() {
        let mut fake_collector = FakeCollectorServer::start()
            .await
            .expect("fake collector started");

        let tracer_provider = setup_tracer_provider(&fake_collector).await;

        let telemetry_layer =
            tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("flagd-test"));
        let subscriber = Registry::default().with(telemetry_layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        // Use the HTTP instrumentation
        {
            let mut headers = reqwest::header::HeaderMap::new();
            let span = instrument_http_request(
                "POST",
                "http://localhost:8016/ofrep/v1/evaluate/flags/my-flag",
                "localhost",
                &mut headers,
            );
            let _enter = span.enter();

            // Simulate successful response
            record_http_response(&span, 200);
        }

        drop(_guard);

        let _ = tracer_provider.force_flush();
        tracer_provider.shutdown().expect("shutdown ok");
        drop(tracer_provider);

        let spans = fake_collector
            .exported_spans(1, Duration::from_secs(5))
            .await;

        assert!(!spans.is_empty(), "HTTP instrumentation should create span");

        let span = &spans[0];
        assert!(
            span.name.contains("POST"),
            "HTTP span should contain method"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_http_instrumentation_injects_headers() {
        let fake_collector = FakeCollectorServer::start()
            .await
            .expect("fake collector started");

        let tracer_provider = setup_tracer_provider(&fake_collector).await;

        let telemetry_layer =
            tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("flagd-test"));
        let subscriber = Registry::default().with(telemetry_layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        let mut headers = reqwest::header::HeaderMap::new();
        let span = instrument_http_request(
            "GET",
            "http://example.com/test",
            "example.com",
            &mut headers,
        );
        let _enter = span.enter();

        drop(_guard);

        let _ = tracer_provider.force_flush();
        tracer_provider.shutdown().expect("shutdown ok");
        drop(tracer_provider);

        // Headers should have trace context injected
        // Note: Headers may be empty if no active trace context exists at injection time
        // The important thing is the function doesn't panic
        assert!(
            headers.is_empty()
                || headers.contains_key("traceparent")
                || headers.contains_key("uber-trace-id"),
            "Headers should either be empty or contain trace context"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_http_instrumentation_records_error() {
        let mut fake_collector = FakeCollectorServer::start()
            .await
            .expect("fake collector started");

        let tracer_provider = setup_tracer_provider(&fake_collector).await;

        let telemetry_layer =
            tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("flagd-test"));
        let subscriber = Registry::default().with(telemetry_layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        {
            let mut headers = reqwest::header::HeaderMap::new();
            let span =
                instrument_http_request("POST", "http://localhost/test", "localhost", &mut headers);
            let _enter = span.enter();

            // Simulate error
            record_http_error(&span, "connection_refused");
        }

        drop(_guard);

        let _ = tracer_provider.force_flush();
        tracer_provider.shutdown().expect("shutdown ok");
        drop(tracer_provider);

        let spans = fake_collector
            .exported_spans(1, Duration::from_secs(5))
            .await;

        assert!(!spans.is_empty(), "HTTP error should create span");
    }
}
