//! # RPC Flag Resolver
//!
//! Evaluates feature flags using gRPC protocol with the flagd service.
//!
//! ## Features
//!
//! * High-performance gRPC-based flag evaluation
//! * Bi-directional streaming support
//! * Event-driven configuration updates
//! * Type-safe evaluation
//! * Structured error handling
//! * Comprehensive logging
//!
//! ## Supported Types
//!
//! * Boolean flags
//! * String flags
//! * Integer flags
//! * Float flags
//! * Structured flags
//!
//! ## Example
//!
//! ```rust,no_run
//! use open_feature_flagd::resolver::rpc::RpcResolver;
//! use open_feature_flagd::FlagdOptions;
//! use open_feature::provider::FeatureProvider;
//! use open_feature::EvaluationContext;
//!
//! #[tokio::main]
//! async fn main() {
//!     let options = FlagdOptions {
//!         host: "localhost".to_string(),
//!         port: 8013,
//!         deadline_ms: 500,
//!         ..Default::default()
//!     };
//!     let resolver = RpcResolver::new(&options).await.unwrap();
//!     let context = EvaluationContext::default();
//!
//!     let result = resolver.resolve_bool_value("my-flag", &context).await.unwrap();
//!     println!("Flag value: {}", result.value);
//! }
//! ```

#[allow(unused_imports)]
use crate::flagd::evaluation::v1::EventStreamRequest;
use crate::flagd::evaluation::v1::{
    ResolveBooleanRequest, ResolveBooleanResponse, ResolveFloatRequest, ResolveFloatResponse,
    ResolveIntRequest, ResolveIntResponse, ResolveObjectRequest, ResolveObjectResponse,
    ResolveStringRequest, ResolveStringResponse, service_client::ServiceClient,
};
use crate::{FlagdOptions, convert_context, convert_proto_struct_to_struct_value};
use async_trait::async_trait;
use hyper_util::rt::TokioIo;
use open_feature::provider::{FeatureProvider, ProviderMetadata, ResolutionDetails};
use open_feature::{
    EvaluationContext, EvaluationError, EvaluationErrorCode, EvaluationReason, FlagMetadata,
    FlagMetadataValue, StructValue,
};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::net::UnixStream;
use tokio::time::sleep;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;
use tracing::{debug, error, instrument, warn};

use super::common::upstream::UpstreamConfig;

type ClientType = ServiceClient<Channel>;

fn convert_proto_metadata(metadata: prost_types::Struct) -> FlagMetadata {
    let mut values = HashMap::new();
    for (k, v) in metadata.fields {
        let metadata_value = match v.kind.unwrap() {
            prost_types::value::Kind::BoolValue(b) => FlagMetadataValue::Bool(b),
            prost_types::value::Kind::NumberValue(n) => FlagMetadataValue::Float(n),
            prost_types::value::Kind::StringValue(s) => FlagMetadataValue::String(s),
            _ => FlagMetadataValue::String("unsupported".to_string()),
        };
        values.insert(k, metadata_value);
    }
    FlagMetadata { values }
}

/// Maps gRPC status codes to OpenFeature error codes
///
/// This ensures consistent error handling across different resolver types
/// and proper conformance with the OpenFeature specification.
fn map_grpc_status_to_error_code(status: &tonic::Status) -> EvaluationErrorCode {
    use tonic::Code;
    match status.code() {
        Code::NotFound => EvaluationErrorCode::FlagNotFound,
        Code::InvalidArgument => EvaluationErrorCode::InvalidContext,
        Code::Unauthenticated | Code::PermissionDenied => {
            EvaluationErrorCode::General("authentication/authorization error".to_string())
        }
        Code::FailedPrecondition => EvaluationErrorCode::TypeMismatch,
        Code::DeadlineExceeded | Code::Cancelled => {
            EvaluationErrorCode::General("request timeout or cancelled".to_string())
        }
        Code::Unavailable => EvaluationErrorCode::General("service unavailable".to_string()),
        _ => EvaluationErrorCode::General(format!("{:?}", status.code())),
    }
}

pub struct RpcResolver {
    client: ClientType,
    metadata: OnceLock<ProviderMetadata>,
}

impl RpcResolver {
    #[instrument(skip(options))]
    pub async fn new(
        options: &FlagdOptions,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        debug!("initializing RPC resolver connection to {}", options.host);

        let mut retry_delay = Duration::from_millis(options.retry_backoff_ms as u64);
        let mut attempts = 0;

        loop {
            match RpcResolver::establish_connection(options).await {
                Ok(client) => {
                    debug!("Successfully established RPC connection");
                    return Ok(Self {
                        client,
                        metadata: OnceLock::new(),
                    });
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= options.retry_grace_period {
                        error!("Failed to establish connection after {} attempts", attempts);
                        return Err(e);
                    }

                    warn!(
                        "Connection attempt {} failed, retrying in {}ms: {}",
                        attempts,
                        retry_delay.as_millis(),
                        e
                    );

                    sleep(retry_delay).await;
                    retry_delay = Duration::from_millis((retry_delay.as_millis() * 2) as u64)
                        .min(Duration::from_millis(options.retry_backoff_max_ms as u64));
                }
            }
        }
    }

    async fn establish_connection(
        options: &FlagdOptions,
    ) -> Result<ClientType, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(socket_path) = &options.socket_path {
            debug!("Attempting Unix socket connection to: {}", socket_path);
            let socket_path = socket_path.clone();
            let channel = Endpoint::try_from("http://[::]:50051")?
                .connect_with_connector(service_fn(move |_: Uri| {
                    let path = socket_path.clone();
                    async move {
                        let stream = UnixStream::connect(path).await?;
                        Ok::<_, std::io::Error>(TokioIo::new(stream))
                    }
                }))
                .await?;

            return Ok(ServiceClient::new(channel));
        }

        let target = options
            .target_uri
            .clone()
            .unwrap_or_else(|| format!("{}:{}", options.host, options.port));
        let upstream_config =
            UpstreamConfig::new(target, false, options.tls, options.cert_path.as_deref())?;
        let mut endpoint = upstream_config.endpoint().clone();

        // Extend support for envoy names resolution
        if let Some(uri) = &options.target_uri
            && uri.starts_with("envoy://")
        {
            // Expected format: envoy://<host:port>/<desired_authority>
            let without_prefix = uri.trim_start_matches("envoy://");
            let segments: Vec<&str> = without_prefix.split('/').collect();
            if segments.len() >= 2 {
                let authority_str = segments[1];
                // Create a full URI from the authority for endpoint.origin()
                let authority_uri =
                    std::str::FromStr::from_str(&format!("http://{}", authority_str))?;
                endpoint = endpoint.origin(authority_uri);
            }
        }

        let channel = endpoint
            .timeout(Duration::from_millis(options.deadline_ms as u64))
            .connect()
            .await?;

        Ok(ServiceClient::new(channel))
    }
}

#[async_trait]
impl FeatureProvider for RpcResolver {
    fn metadata(&self) -> &ProviderMetadata {
        self.metadata.get_or_init(|| ProviderMetadata::new("flagd"))
    }

    #[instrument(skip(self, context))]
    async fn resolve_bool_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<bool>, EvaluationError> {
        debug!(flag_key, "resolving boolean flag");
        let request = ResolveBooleanRequest {
            flag_key: flag_key.to_string(),
            context: convert_context(context),
        };

        match self.client.clone().resolve_boolean(request).await {
            Ok(response) => {
                let inner: ResolveBooleanResponse = response.into_inner();
                debug!(flag_key, value = inner.value, reason = %inner.reason, "boolean flag resolved");
                Ok(ResolutionDetails {
                    value: inner.value,
                    variant: Some(inner.variant),
                    reason: Some(EvaluationReason::Other(inner.reason)),
                    flag_metadata: inner.metadata.map(convert_proto_metadata),
                })
            }
            Err(status) => {
                error!(flag_key, error = %status, "failed to resolve boolean flag");
                Err(EvaluationError {
                    code: map_grpc_status_to_error_code(&status),
                    message: Some(status.message().to_string()),
                })
            }
        }
    }

    #[instrument(skip(self, context))]
    async fn resolve_string_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<String>, EvaluationError> {
        debug!(flag_key, "resolving string flag");
        let request = ResolveStringRequest {
            flag_key: flag_key.to_string(),
            context: convert_context(context),
        };

        match self.client.clone().resolve_string(request).await {
            Ok(response) => {
                let inner: ResolveStringResponse = response.into_inner();
                debug!(flag_key, value = %inner.value, reason = %inner.reason, "string flag resolved");
                Ok(ResolutionDetails {
                    value: inner.value,
                    variant: Some(inner.variant),
                    reason: Some(EvaluationReason::Other(inner.reason)),
                    flag_metadata: inner.metadata.map(convert_proto_metadata),
                })
            }
            Err(status) => {
                error!(flag_key, error = %status, "failed to resolve string flag");
                Err(EvaluationError {
                    code: map_grpc_status_to_error_code(&status),
                    message: Some(status.message().to_string()),
                })
            }
        }
    }

    #[instrument(skip(self, context))]
    async fn resolve_float_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<f64>, EvaluationError> {
        debug!(flag_key, "resolving float flag");
        let request = ResolveFloatRequest {
            flag_key: flag_key.to_string(),
            context: convert_context(context),
        };

        match self.client.clone().resolve_float(request).await {
            Ok(response) => {
                let inner: ResolveFloatResponse = response.into_inner();
                debug!(flag_key, value = inner.value, reason = %inner.reason, "float flag resolved");
                Ok(ResolutionDetails {
                    value: inner.value,
                    variant: Some(inner.variant),
                    reason: Some(EvaluationReason::Other(inner.reason)),
                    flag_metadata: inner.metadata.map(convert_proto_metadata),
                })
            }
            Err(status) => {
                error!(flag_key, error = %status, "failed to resolve float flag");
                Err(EvaluationError {
                    code: map_grpc_status_to_error_code(&status),
                    message: Some(status.message().to_string()),
                })
            }
        }
    }

    #[instrument(skip(self, context))]
    async fn resolve_int_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<i64>, EvaluationError> {
        debug!(flag_key, "resolving integer flag");
        let request = ResolveIntRequest {
            flag_key: flag_key.to_string(),
            context: convert_context(context),
        };

        match self.client.clone().resolve_int(request).await {
            Ok(response) => {
                let inner: ResolveIntResponse = response.into_inner();
                debug!(flag_key, value = inner.value, reason = %inner.reason, "integer flag resolved");
                Ok(ResolutionDetails {
                    value: inner.value,
                    variant: Some(inner.variant),
                    reason: Some(EvaluationReason::Other(inner.reason)),
                    flag_metadata: inner.metadata.map(convert_proto_metadata),
                })
            }
            Err(status) => {
                error!(flag_key, error = %status, "failed to resolve integer flag");
                Err(EvaluationError {
                    code: map_grpc_status_to_error_code(&status),
                    message: Some(status.message().to_string()),
                })
            }
        }
    }

    #[instrument(skip(self, context))]
    async fn resolve_struct_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<StructValue>, EvaluationError> {
        debug!(flag_key, "resolving struct flag");
        let request = ResolveObjectRequest {
            flag_key: flag_key.to_string(),
            context: convert_context(context),
        };

        match self.client.clone().resolve_object(request).await {
            Ok(response) => {
                let inner: ResolveObjectResponse = response.into_inner();
                debug!(flag_key, reason = %inner.reason, "struct flag resolved");
                Ok(ResolutionDetails {
                    value: convert_proto_struct_to_struct_value(inner.value.unwrap_or_default()),
                    variant: Some(inner.variant),
                    reason: Some(EvaluationReason::Other(inner.reason)),
                    flag_metadata: inner.metadata.map(convert_proto_metadata),
                })
            }
            Err(status) => {
                error!(flag_key, error = %status, "failed to resolve struct flag");
                Err(EvaluationError {
                    code: map_grpc_status_to_error_code(&status),
                    message: Some(status.message().to_string()),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flagd::evaluation::v1::{
        EventStreamResponse, ResolveAllRequest, ResolveAllResponse,
        service_server::{Service, ServiceServer},
    };
    use futures_core::Stream;
    use serial_test::serial;
    use std::{collections::BTreeMap, pin::Pin};
    use tempfile::TempDir;
    use test_log::test;
    use tokio::net::UnixListener;
    use tokio::sync::oneshot;
    use tokio::{net::TcpListener, time::Instant};
    use tokio_stream::wrappers::UnixListenerStream;
    use tonic::{Request, Response, Status, transport::Server};

    pub struct MockFlagService;

    #[tonic::async_trait]
    impl Service for MockFlagService {
        async fn resolve_boolean(
            &self,
            _request: Request<ResolveBooleanRequest>,
        ) -> Result<Response<ResolveBooleanResponse>, Status> {
            Ok(Response::new(ResolveBooleanResponse {
                value: true,
                reason: "test".to_string(),
                variant: "test".to_string(),
                metadata: Some(create_test_metadata()),
            }))
        }

        async fn resolve_string(
            &self,
            _request: Request<ResolveStringRequest>,
        ) -> Result<Response<ResolveStringResponse>, Status> {
            Ok(Response::new(ResolveStringResponse {
                value: "test".to_string(),
                reason: "test".to_string(),
                variant: "test".to_string(),
                metadata: Some(create_test_metadata()),
            }))
        }

        async fn resolve_float(
            &self,
            _request: Request<ResolveFloatRequest>,
        ) -> Result<Response<ResolveFloatResponse>, Status> {
            Ok(Response::new(ResolveFloatResponse {
                value: 1.0,
                reason: "test".to_string(),
                variant: "test".to_string(),
                metadata: Some(create_test_metadata()),
            }))
        }

        async fn resolve_int(
            &self,
            _request: Request<ResolveIntRequest>,
        ) -> Result<Response<ResolveIntResponse>, Status> {
            Ok(Response::new(ResolveIntResponse {
                value: 42,
                reason: "test".to_string(),
                variant: "test".to_string(),
                metadata: Some(create_test_metadata()),
            }))
        }

        async fn resolve_object(
            &self,
            _request: Request<ResolveObjectRequest>,
        ) -> Result<Response<ResolveObjectResponse>, Status> {
            let mut fields = BTreeMap::new();
            fields.insert(
                "key".to_string(),
                prost_types::Value {
                    kind: Some(prost_types::value::Kind::StringValue("value".to_string())),
                },
            );

            Ok(Response::new(ResolveObjectResponse {
                value: Some(prost_types::Struct { fields }),
                reason: "test".to_string(),
                variant: "test".to_string(),
                metadata: Some(create_test_metadata()),
            }))
        }

        async fn resolve_all(
            &self,
            _request: Request<ResolveAllRequest>,
        ) -> Result<Response<ResolveAllResponse>, Status> {
            Ok(Response::new(ResolveAllResponse {
                flags: Default::default(),
                metadata: Some(create_test_metadata()),
            }))
        }

        type EventStreamStream =
            Pin<Box<dyn Stream<Item = Result<EventStreamResponse, Status>> + Send + 'static>>;

        async fn event_stream(
            &self,
            _request: Request<EventStreamRequest>,
        ) -> Result<Response<Self::EventStreamStream>, Status> {
            let output = tokio_stream::empty();
            Ok(Response::new(Box::pin(output)))
        }
    }

    fn create_test_metadata() -> prost_types::Struct {
        let mut fields = BTreeMap::new();
        fields.insert(
            "bool_key".to_string(),
            prost_types::Value {
                kind: Some(prost_types::value::Kind::BoolValue(true)),
            },
        );
        fields.insert(
            "number_key".to_string(),
            prost_types::Value {
                kind: Some(prost_types::value::Kind::NumberValue(42.0)),
            },
        );
        fields.insert(
            "string_key".to_string(),
            prost_types::Value {
                kind: Some(prost_types::value::Kind::StringValue("test".to_string())),
            },
        );
        prost_types::Struct { fields }
    }

    struct TestServer {
        target: String,
        _shutdown: oneshot::Sender<()>,
    }

    impl TestServer {
        async fn new() -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            let (tx, rx) = oneshot::channel();

            let server = tonic::transport::Server::builder()
                .add_service(ServiceServer::new(MockFlagService))
                .serve(addr);

            tokio::spawn(async move {
                tokio::select! {
                    _ = server => {},
                    _ = rx => {},
                }
            });

            Self {
                target: format!("{}:{}", addr.ip(), addr.port()),
                _shutdown: tx,
            }
        }
    }

    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn test_dns_resolution() {
        let server = TestServer::new().await;
        // Add delay to ensure server is ready
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let options = FlagdOptions {
            host: server.target.clone(),
            port: 8013,
            target_uri: None,
            deadline_ms: 500,
            ..Default::default()
        };
        let resolver = RpcResolver::new(&options).await.unwrap();
        let context = EvaluationContext::default().with_targeting_key("test-user");

        let result = resolver
            .resolve_bool_value("test-flag", &context)
            .await
            .unwrap();
        assert_eq!(result.value, true);
    }

    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn test_envoy_resolution() {
        let server = TestServer::new().await;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let options = FlagdOptions {
            host: server.target.clone(),
            port: 8013,
            target_uri: Some(format!("envoy://{}/flagd-service", server.target)),
            deadline_ms: 500,
            ..Default::default()
        };

        let resolver = RpcResolver::new(&options).await.unwrap();
        let context = EvaluationContext::default().with_targeting_key("test-user");

        let result = resolver
            .resolve_bool_value("test-flag", &context)
            .await
            .unwrap();
        assert_eq!(result.value, true);
    }

    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn test_value_resolution() {
        let server = TestServer::new().await;
        let options = FlagdOptions {
            host: server.target.clone(),
            port: 8013,
            target_uri: None,
            deadline_ms: 500,
            ..Default::default()
        };
        let resolver = RpcResolver::new(&options).await.unwrap();
        let context = EvaluationContext::default().with_targeting_key("test-user");

        // Test all value types
        assert_eq!(
            resolver
                .resolve_bool_value("test-flag", &context)
                .await
                .unwrap()
                .value,
            true
        );
        assert_eq!(
            resolver
                .resolve_string_value("test-flag", &context)
                .await
                .unwrap()
                .value,
            "test"
        );
        assert_eq!(
            resolver
                .resolve_float_value("test-flag", &context)
                .await
                .unwrap()
                .value,
            1.0
        );
        assert_eq!(
            resolver
                .resolve_int_value("test-flag", &context)
                .await
                .unwrap()
                .value,
            42
        );

        let struct_result = resolver
            .resolve_struct_value("test-flag", &context)
            .await
            .unwrap();
        assert!(!struct_result.value.fields.is_empty());
    }

    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn test_metadata() {
        let metadata = create_test_metadata();
        let flag_metadata = convert_proto_metadata(metadata);

        assert!(matches!(
            flag_metadata.values.get("bool_key"),
            Some(FlagMetadataValue::Bool(true))
        ));
        assert!(matches!(
            flag_metadata.values.get("number_key"),
            Some(FlagMetadataValue::Float(42.0))
        ));
        assert!(matches!(
            flag_metadata.values.get("string_key"),
            Some(FlagMetadataValue::String(s)) if s == "test"
        ));
    }

    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn test_standard_connection() {
        let server = TestServer::new().await;
        let parts: Vec<&str> = server.target.split(':').collect();
        let options = FlagdOptions {
            host: parts[0].to_string(),
            port: parts[1].parse().unwrap(),
            target_uri: None,
            deadline_ms: 500,
            ..Default::default()
        };

        let resolver = RpcResolver::new(&options).await.unwrap();
        let context = EvaluationContext::default().with_targeting_key("test-user");

        let result = resolver
            .resolve_bool_value("test-flag", &context)
            .await
            .unwrap();
        assert_eq!(result.value, true);
    }

    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn test_envoy_connection() {
        let server = TestServer::new().await;
        let parts: Vec<&str> = server.target.split(':').collect();
        let options = FlagdOptions {
            host: parts[0].to_string(),
            port: parts[1].parse().unwrap(),
            target_uri: Some(format!("envoy://{}/flagd-service", server.target)),
            deadline_ms: 500,
            ..Default::default()
        };

        let resolver = RpcResolver::new(&options).await.unwrap();
        let context = EvaluationContext::default().with_targeting_key("test-user");

        let result = resolver
            .resolve_bool_value("test-flag", &context)
            .await
            .unwrap();
        assert_eq!(result.value, true);
    }

    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    #[serial]
    async fn test_retry_mechanism() {
        // Bind to a port but don't accept connections - this causes immediate connection failures
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        // Drop the listener immediately to ensure the port rejects connections
        drop(listener);

        let options = FlagdOptions {
            host: addr.ip().to_string(),
            port: addr.port(),
            retry_backoff_ms: 100,
            retry_backoff_max_ms: 400,
            retry_grace_period: 3,
            deadline_ms: 100, // Short timeout for fast failures
            ..Default::default()
        };

        let start = Instant::now();
        let result = RpcResolver::new(&options).await;
        let duration = start.elapsed();

        assert!(result.is_err());
        // Should take at least 300ms (100ms + 200ms delays)
        assert!(duration.as_millis() >= 300);
        // Allow some buffer for system overhead and processing time
        assert!(duration.as_millis() < 600);
    }

    #[test(tokio::test)]
    async fn test_successful_retry() {
        let server = TestServer::new().await;
        let options = FlagdOptions {
            host: server.target.clone(),
            port: 8013,
            retry_backoff_ms: 100,
            retry_backoff_max_ms: 400,
            retry_grace_period: 3,
            ..Default::default()
        };

        let resolver = RpcResolver::new(&options).await.unwrap();
        let context = EvaluationContext::default();

        let result = resolver
            .resolve_bool_value("test-flag", &context)
            .await
            .unwrap();
        assert_eq!(result.value, true);
    }

    #[test(tokio::test)]
    async fn test_rpc_unix_socket_connection() {
        let tmp_dir = TempDir::new().unwrap();
        let socket_path = tmp_dir.path().join("test.sock");
        let socket_path_str = socket_path.to_str().unwrap().to_string();

        // Start mock gRPC server with proper shutdown handling
        let server_handle = tokio::spawn(async move {
            let uds = UnixListener::bind(&socket_path).unwrap();
            Server::builder()
                .add_service(ServiceServer::new(MockFlagService))
                .serve_with_incoming(UnixListenerStream::new(uds))
                .await
                .unwrap();
        });

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        let options = FlagdOptions {
            socket_path: Some(socket_path_str),
            retry_backoff_ms: 100,
            retry_backoff_max_ms: 400,
            retry_grace_period: 3,
            ..Default::default()
        };

        let resolver = RpcResolver::new(&options).await;
        assert!(resolver.is_ok());

        // Clean shutdown
        server_handle.abort();
    }

    #[test]
    fn test_grpc_error_code_mapping() {
        use tonic::Code;

        // Test NOT_FOUND -> FlagNotFound
        let status = tonic::Status::new(Code::NotFound, "Flag not found");
        let error_code = map_grpc_status_to_error_code(&status);
        assert!(matches!(error_code, EvaluationErrorCode::FlagNotFound));

        // Test INVALID_ARGUMENT -> InvalidContext
        let status = tonic::Status::new(Code::InvalidArgument, "Invalid context");
        let error_code = map_grpc_status_to_error_code(&status);
        assert!(matches!(error_code, EvaluationErrorCode::InvalidContext));

        // Test UNAUTHENTICATED -> General
        let status = tonic::Status::new(Code::Unauthenticated, "Not authenticated");
        let error_code = map_grpc_status_to_error_code(&status);
        assert!(matches!(error_code, EvaluationErrorCode::General(_)));

        // Test PERMISSION_DENIED -> General
        let status = tonic::Status::new(Code::PermissionDenied, "Access denied");
        let error_code = map_grpc_status_to_error_code(&status);
        assert!(matches!(error_code, EvaluationErrorCode::General(_)));

        // Test FAILED_PRECONDITION -> TypeMismatch
        let status = tonic::Status::new(Code::FailedPrecondition, "Type mismatch");
        let error_code = map_grpc_status_to_error_code(&status);
        assert!(matches!(error_code, EvaluationErrorCode::TypeMismatch));

        // Test DEADLINE_EXCEEDED -> General
        let status = tonic::Status::new(Code::DeadlineExceeded, "Timeout");
        let error_code = map_grpc_status_to_error_code(&status);
        assert!(matches!(error_code, EvaluationErrorCode::General(_)));

        // Test UNAVAILABLE -> General
        let status = tonic::Status::new(Code::Unavailable, "Service unavailable");
        let error_code = map_grpc_status_to_error_code(&status);
        assert!(matches!(error_code, EvaluationErrorCode::General(_)));
    }
}
