use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use open_feature::provider::{FeatureProvider, ProviderMetadata, ResolutionDetails};
use open_feature::{
    EvaluationContext, EvaluationContextFieldValue, EvaluationError, EvaluationErrorCode,
    EvaluationReason, EvaluationResult, FlagMetadata, StructValue, Value,
};
use reqwest::header::RETRY_AFTER;
use reqwest::{Certificate, Client, StatusCode, redirect};
use std::any;
use tokio::sync::Mutex;
use tracing::{debug, error, instrument};
use url::Url;

use crate::{OfrepError, OfrepOptions};

#[derive(Debug)]
pub struct Resolver {
    base_url: String,
    metadata: ProviderMetadata,
    client: Client,
    /// The time until which this provider's OFREP server asked it to wait.
    retry_after: Mutex<DateTime<Utc>>,
}

impl Resolver {
    pub fn new(options: &OfrepOptions) -> Result<Self, OfrepError> {
        Ok(Self {
            base_url: options.base_url.clone(),
            metadata: ProviderMetadata::new("ofrep"),
            client: Self::build_client(options)?,
            retry_after: Mutex::new(Utc::now()),
        })
    }

    fn build_client(options: &OfrepOptions) -> Result<Client, OfrepError> {
        // A redirect would send the evaluation context to another origin, so it is reported as an
        // unexpected status instead of being followed.
        let mut client_builder = Client::builder()
            .default_headers(options.headers.clone())
            .connect_timeout(options.connect_timeout)
            .redirect(redirect::Policy::none());

        let url = Url::parse(&options.base_url).map_err(|e| {
            OfrepError::Config(format!("Invalid base url: '{}' ({})", options.base_url, e))
        })?;

        if url.scheme() == "https"
            && let Some(path) = options.cert_path.as_deref()
        {
            let cert_pem = std::fs::read(path).map_err(|e| {
                OfrepError::Config(format!("Failed to read certificate file '{}': {}", path, e))
            })?;
            let ca_cert = Certificate::from_pem(&cert_pem).map_err(|e| {
                OfrepError::Config(format!(
                    "Failed to parse certificate file '{}': {}",
                    path, e
                ))
            })?;
            client_builder = client_builder.add_root_certificate(ca_cert);
        }

        client_builder
            .build()
            .map_err(|e| OfrepError::Config(format!("Failed to build HTTP client: {}", e)))
    }

    /// The time a Retry-After header names, as delay seconds or an HTTP date. A delay too large
    /// to represent waits until the latest representable time.
    fn parse_retry_after(retry_after: &str) -> DateTime<Utc> {
        let now = Utc::now();
        let retry_after = retry_after.trim();

        if let Ok(seconds) = retry_after.parse::<i64>() {
            return Duration::try_seconds(seconds.max(0))
                .and_then(|delay| now.checked_add_signed(delay))
                .unwrap_or(DateTime::<Utc>::MAX_UTC);
        }

        if let Ok(date) = DateTime::parse_from_rfc2822(retry_after) {
            return date.with_timezone(&Utc);
        }

        if let Ok(date) = retry_after.parse::<DateTime<Utc>>() {
            return date;
        }

        debug!("Failed to parse Retry-After header : {}", retry_after);
        now
    }

    async fn is_rate_limit_exceeded(&self) -> bool {
        Utc::now() < *self.retry_after.lock().await
    }

    #[instrument(skip(self, evaluation_context), fields(flag_key = %flag_key))]
    async fn resolve_value<T: std::fmt::Debug>(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
        convertor: fn(serde_json::Value) -> Option<T>,
    ) -> EvaluationResult<ResolutionDetails<T>> {
        if self.is_rate_limit_exceeded().await {
            return Err(EvaluationError {
                code: EvaluationErrorCode::General("Rate limit exceeded".to_string()),
                message: Some(
                    "Rate limit exceeded. Please wait before making another request.".to_string(),
                ),
            });
        }

        debug!("Resolving {} flag", std::any::type_name::<T>());
        let payload = serde_json::json!({
            "context": context_to_json(evaluation_context)
        });

        let response = self
            .client
            .post(format!(
                "{}/ofrep/v1/evaluate/flags/{}",
                self.base_url, flag_key
            ))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to parse response {} value", any::type_name::<T>());
                EvaluationError {
                    code: EvaluationErrorCode::General(format!(
                        "Failed to resolve {} value ",
                        std::any::type_name::<T>()
                    )),
                    message: Some(e.to_string()),
                }
            })?;

        debug!(status = response.status().as_u16(), "Received response");

        match response.status() {
            StatusCode::BAD_REQUEST => {
                let body = error_body(response).await;
                let details = body["errorDetails"].as_str().map(String::from);
                let code = match body["errorCode"].as_str() {
                    Some("PARSE_ERROR") => EvaluationErrorCode::ParseError,
                    Some("TARGETING_KEY_MISSING") => EvaluationErrorCode::TargetingKeyMissing,
                    Some("INVALID_CONTEXT") | None => EvaluationErrorCode::InvalidContext,
                    Some(_) => EvaluationErrorCode::General("Bad evaluation request".to_string()),
                };
                let message = details.unwrap_or_else(|| match code {
                    EvaluationErrorCode::InvalidContext => "Invalid context".to_string(),
                    _ => "Bad evaluation request".to_string(),
                });
                return Err(EvaluationError {
                    code,
                    message: Some(message),
                });
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                return Err(EvaluationError {
                    code: EvaluationErrorCode::General(
                        "authentication/authorization error".to_string(),
                    ),
                    message: Some("authentication/authorization error".to_string()),
                });
            }
            StatusCode::NOT_FOUND => {
                let body = error_body(response).await;
                return Err(EvaluationError {
                    code: EvaluationErrorCode::FlagNotFound,
                    message: Some(
                        body["errorDetails"]
                            .as_str()
                            .map(String::from)
                            .unwrap_or_else(|| format!("Flag: {flag_key} not found")),
                    ),
                });
            }
            StatusCode::TOO_MANY_REQUESTS => {
                let header_retry_after: Option<&str> = response
                    .headers()
                    .get(RETRY_AFTER)
                    .and_then(|value| value.to_str().ok());

                let mut retry_after = self.retry_after.lock().await;
                *retry_after = match header_retry_after {
                    Some(header_retry_after) => Resolver::parse_retry_after(header_retry_after),
                    None => {
                        debug!("Couldn't parse the retry-after header.");
                        Utc::now()
                    }
                };

                return Err(EvaluationError {
                    code: EvaluationErrorCode::General("Rate limit exceeded".to_string()),
                    message: Some(format!("Rate limit exceeded. Retry after {}", *retry_after)),
                });
            }
            StatusCode::OK => {
                let result = response.json::<serde_json::Value>().await.map_err(|e| {
                    error!(error = %e, "Failed to parse {} response", any::type_name::<T>());
                    EvaluationError {
                        code: EvaluationErrorCode::ParseError,
                        message: Some(e.to_string()),
                    }
                })?;
                let Some(raw_value) = result.as_object().map(|body| body.get("value")) else {
                    error!("OFREP response is not a JSON object");
                    return Err(EvaluationError {
                        code: EvaluationErrorCode::ParseError,
                        message: Some("The OFREP response is not a JSON object".to_string()),
                    });
                };
                // A success without a value defers to the code default. Providers never see the
                // code default, so it reaches the caller as an error, like any other fallback.
                let Some(raw_value) = raw_value else {
                    debug!(reason = ?result["reason"], "Flag defers to the code default");
                    return Err(EvaluationError {
                        code: EvaluationErrorCode::General("Code default".to_string()),
                        message: Some(format!(
                            "The OFREP server deferred flag {flag_key} to the code default"
                        )),
                    });
                };
                let value = convertor(raw_value.clone()).ok_or_else(|| {
                    error!("Invalid {} value in response", any::type_name::<T>());
                    EvaluationError {
                        code: EvaluationErrorCode::TypeMismatch,
                        message: Some(format!(
                            "Resolved value {raw_value} is not of type {}",
                            std::any::type_name::<T>()
                        )),
                    }
                })?;

                debug!(value = ?value, variant = ?result["variant"], "Flag evaluated");
                Ok(ResolutionDetails {
                    value,
                    variant: result["variant"].as_str().map(String::from),
                    reason: result["reason"].as_str().map(reason_from_str),
                    flag_metadata: metadata_from_json(&result["metadata"]),
                })
            }
            status => {
                let body = error_body(response).await;
                Err(EvaluationError {
                    code: EvaluationErrorCode::General(format!(
                        "Unexpected response status {status}"
                    )),
                    message: Some(
                        body["errorDetails"]
                            .as_str()
                            .map(String::from)
                            .unwrap_or_else(|| {
                                format!(
                                    "The OFREP server answered {status} instead of a flag evaluation"
                                )
                            }),
                    ),
                })
            }
        }
    }
}

/// Limits for reading the optional body of an error response, so a slow or oversized body cannot
/// delay the fallback.
const ERROR_BODY_MAX_BYTES: usize = 64 * 1024;
const ERROR_BODY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

/// The JSON body of an OFREP error response, or `null` when it has none, is not JSON, or exceeds
/// the size or time limit.
async fn error_body(mut response: reqwest::Response) -> serde_json::Value {
    let read = async {
        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await.ok()? {
            if body.len() + chunk.len() > ERROR_BODY_MAX_BYTES {
                return None;
            }
            body.extend_from_slice(&chunk);
        }
        serde_json::from_slice(&body).ok()
    };
    tokio::time::timeout(ERROR_BODY_TIMEOUT, read)
        .await
        .ok()
        .flatten()
        .unwrap_or_default()
}

/// The evaluation reason an OFREP response names.
fn reason_from_str(reason: &str) -> EvaluationReason {
    match reason {
        "STATIC" => EvaluationReason::Static,
        "DEFAULT" => EvaluationReason::Default,
        "TARGETING_MATCH" => EvaluationReason::TargetingMatch,
        "SPLIT" => EvaluationReason::Split,
        "CACHED" => EvaluationReason::Cached,
        "DISABLED" => EvaluationReason::Disabled,
        "UNKNOWN" => EvaluationReason::Unknown,
        "ERROR" => EvaluationReason::Error,
        other => EvaluationReason::Other(other.to_string()),
    }
}

/// The flag metadata of an OFREP response: its boolean, number and string fields.
fn metadata_from_json(metadata: &serde_json::Value) -> Option<FlagMetadata> {
    let metadata = metadata.as_object()?;
    let mut flag_metadata = FlagMetadata::default();
    for (key, value) in metadata {
        match value {
            serde_json::Value::Bool(value) => flag_metadata.add_value(key, *value),
            serde_json::Value::Number(number) => match number.as_i64() {
                Some(value) => flag_metadata.add_value(key, value),
                None => {
                    if let Some(value) = number.as_f64() {
                        flag_metadata.add_value(key, value);
                    }
                }
            },
            serde_json::Value::String(value) => flag_metadata.add_value(key, value.as_str()),
            _ => {}
        }
    }
    Some(flag_metadata)
}

#[async_trait]
impl FeatureProvider for Resolver {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn resolve_bool_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<bool>> {
        self.resolve_value(flag_key, evaluation_context, |value| value.as_bool())
            .await
    }

    async fn resolve_string_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<String>> {
        self.resolve_value(flag_key, evaluation_context, |value| {
            value.as_str().map(|s| s.to_string())
        })
        .await
    }

    async fn resolve_float_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<f64>> {
        self.resolve_value(flag_key, evaluation_context, |value| value.as_f64())
            .await
    }

    async fn resolve_int_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<i64>> {
        self.resolve_value(flag_key, evaluation_context, |value| value.as_i64())
            .await
    }

    async fn resolve_struct_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<StructValue>> {
        self.resolve_value(flag_key, evaluation_context, |value| {
            value.into_feature_value().as_struct().cloned()
        })
        .await
    }
}

fn context_to_json(context: &EvaluationContext) -> serde_json::Value {
    let mut fields = serde_json::Map::new();

    if let Some(targeting_key) = &context.targeting_key {
        fields.insert(
            "targetingKey".to_string(),
            serde_json::Value::String(targeting_key.clone()),
        );
    }

    for (key, value) in &context.custom_fields {
        let json_value = match value {
            EvaluationContextFieldValue::String(s) => serde_json::Value::String(s.clone()),
            EvaluationContextFieldValue::Bool(b) => serde_json::Value::Bool(*b),
            EvaluationContextFieldValue::Int(i) => serde_json::Value::Number((*i).into()),
            EvaluationContextFieldValue::Float(f) => {
                if let Some(n) = serde_json::Number::from_f64(*f) {
                    serde_json::Value::Number(n)
                } else {
                    serde_json::Value::Null
                }
            }
            EvaluationContextFieldValue::DateTime(dt) => serde_json::Value::String(dt.to_string()),
            EvaluationContextFieldValue::Struct(s) => serde_json::Value::String(format!("{s:?}")),
        };
        fields.insert(key.clone(), json_value);
    }

    serde_json::Value::Object(fields)
}

/// Trait for converting JSON values into OpenFeature values
trait IntoFeatureValue {
    /// Converts a JSON value into an OpenFeature value
    fn into_feature_value(self) -> Value;
}

impl IntoFeatureValue for serde_json::Value {
    fn into_feature_value(self) -> Value {
        match self {
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Int(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Float(f)
                } else {
                    Value::Int(0)
                }
            }
            serde_json::Value::String(s) => Value::String(s),
            serde_json::Value::Array(arr) => {
                Value::Array(arr.into_iter().map(|v| v.into_feature_value()).collect())
            }
            serde_json::Value::Object(obj) => {
                let mut struct_value = StructValue::default();
                for (k, v) in obj {
                    struct_value.add_field(k, v.into_feature_value());
                }
                Value::Struct(struct_value)
            }
            serde_json::Value::Null => Value::String("".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use open_feature::FlagMetadataValue;
    use serde_json::json;
    use test_log::test;
    use tokio::time::{Duration, sleep};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup_mock_server() -> (MockServer, Resolver) {
        let mock_server = MockServer::start().await;
        let options = OfrepOptions {
            base_url: mock_server.uri(),
            ..Default::default()
        };
        let resolver = Resolver::new(&options).unwrap();
        (mock_server, resolver)
    }

    fn test_cert_path() -> String {
        format!(
            "{}/../flagd/flagd-testbed/ssl/custom-root-cert.crt",
            env!("CARGO_MANIFEST_DIR")
        )
    }

    #[test]
    fn test_resolver_new_with_https_custom_ca() {
        let options = OfrepOptions {
            base_url: "https://localhost:8016".to_string(),
            cert_path: Some(test_cert_path()),
            ..Default::default()
        };

        let resolver = Resolver::new(&options);

        assert!(resolver.is_ok());
    }

    #[test]
    fn test_resolver_new_with_https_missing_cert_fails() {
        let options = OfrepOptions {
            base_url: "https://localhost:8016".to_string(),
            cert_path: Some("/nonexistent/path/to/cert.pem".to_string()),
            ..Default::default()
        };

        let resolver = Resolver::new(&options);

        assert_eq!(
            resolver.unwrap_err(),
            OfrepError::Config(
                "Failed to read certificate file '/nonexistent/path/to/cert.pem': No such file or directory (os error 2)"
                    .to_string()
            )
        );
    }

    #[test(tokio::test)]
    #[serial_test::serial]
    async fn test_resolve_bool_value() {
        let (mock_server, resolver) = setup_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "value": true,
                "variant": "on",
                "reason": "STATIC"
            })))
            .mount(&mock_server)
            .await;

        let context = EvaluationContext::default().with_targeting_key("test-user");
        let result = resolver
            .resolve_bool_value("test-flag", &context)
            .await
            .unwrap();

        assert_eq!(result.value, true);
        assert_eq!(result.variant, Some("on".to_string()));
        assert_eq!(result.reason, Some(open_feature::EvaluationReason::Static));
    }

    #[test(tokio::test)]
    #[serial_test::serial]
    async fn test_resolve_string_value() {
        let (mock_server, resolver) = setup_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "value": "test-value",
                "variant": "key1",
                "reason": "STATIC"
            })))
            .mount(&mock_server)
            .await;

        let context = EvaluationContext::default().with_targeting_key("test-user");
        let result = resolver
            .resolve_string_value("test-flag", &context)
            .await
            .unwrap();

        assert_eq!(result.value, "test-value");
        assert_eq!(result.variant, Some("key1".to_string()));
        assert_eq!(result.reason, Some(open_feature::EvaluationReason::Static));
    }

    #[test(tokio::test)]
    #[serial_test::serial]
    async fn test_resolve_float_value() {
        let (mock_server, resolver) = setup_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "value": 1.23,
                "variant": "one",
                "reason": "STATIC"
            })))
            .mount(&mock_server)
            .await;

        let context = EvaluationContext::default().with_targeting_key("test-user");
        let result = resolver
            .resolve_float_value("test-flag", &context)
            .await
            .unwrap();

        assert_eq!(result.value, 1.23);
        assert_eq!(result.variant, Some("one".to_string()));
        assert_eq!(result.reason, Some(open_feature::EvaluationReason::Static));
    }

    #[test(tokio::test)]
    #[serial_test::serial]
    async fn test_resolve_int_value() {
        let (mock_server, resolver) = setup_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "value": 42,
                "variant": "one",
                "reason": "STATIC"
            })))
            .mount(&mock_server)
            .await;

        let context = EvaluationContext::default().with_targeting_key("test-user");
        let result = resolver
            .resolve_int_value("test-flag", &context)
            .await
            .unwrap();

        assert_eq!(result.value, 42);
        assert_eq!(result.variant, Some("one".to_string()));
        assert_eq!(result.reason, Some(open_feature::EvaluationReason::Static));
    }

    #[test(tokio::test)]
    #[serial_test::serial]
    async fn test_resolve_struct_value() {
        let (mock_server, resolver) = setup_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "value": {
                    "key": "val",
                    "number": 42,
                    "boolean": true,
                    "nested": {
                        "inner": "value"
                    }
                },
                "variant": "object1",
                "reason": "STATIC"
            })))
            .mount(&mock_server)
            .await;

        let context = EvaluationContext::default().with_targeting_key("test-user");
        let result = resolver
            .resolve_struct_value("test-flag", &context)
            .await
            .unwrap();

        let value = &result.value;
        assert_eq!(value.fields.get("key").unwrap().as_str().unwrap(), "val");
        assert_eq!(value.fields.get("number").unwrap().as_i64().unwrap(), 42);
        assert_eq!(
            value.fields.get("boolean").unwrap().as_bool().unwrap(),
            true
        );

        let nested = value.fields.get("nested").unwrap().as_struct().unwrap();
        assert_eq!(
            nested.fields.get("inner").unwrap().as_str().unwrap(),
            "value"
        );

        assert_eq!(result.variant, Some("object1".to_string()));
        assert_eq!(result.reason, Some(open_feature::EvaluationReason::Static));
    }

    #[test(tokio::test)]
    #[serial_test::serial]
    async fn test_error_400() {
        let (mock_server, resolver) = setup_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(ResponseTemplate::new(400).set_body_json(json!({})))
            .mount(&mock_server)
            .await;

        let context = EvaluationContext::default();
        let result_bool = resolver.resolve_bool_value("test-flag", &context).await;
        let result_int = resolver.resolve_int_value("test-flag", &context).await;
        let result_float = resolver.resolve_float_value("test-flag", &context).await;
        let result_string = resolver.resolve_string_value("test-flag", &context).await;
        let result_struct = resolver.resolve_struct_value("test-flag", &context).await;

        assert!(result_bool.is_err());
        assert!(result_int.is_err());
        assert!(result_float.is_err());
        assert!(result_string.is_err());
        assert!(result_struct.is_err());

        assert_eq!(
            result_bool.unwrap_err().code,
            EvaluationErrorCode::InvalidContext
        );
        assert_eq!(
            result_int.unwrap_err().code,
            EvaluationErrorCode::InvalidContext
        );
        assert_eq!(
            result_float.unwrap_err().code,
            EvaluationErrorCode::InvalidContext
        );
        assert_eq!(
            result_string.unwrap_err().code,
            EvaluationErrorCode::InvalidContext
        );
        assert_eq!(
            result_struct.unwrap_err().code,
            EvaluationErrorCode::InvalidContext
        );
    }

    #[test(tokio::test)]
    #[serial_test::serial]
    async fn test_error_401() {
        let (mock_server, resolver) = setup_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(ResponseTemplate::new(401).set_body_json(json!({})))
            .mount(&mock_server)
            .await;

        let context = EvaluationContext::default();

        let result_bool = resolver.resolve_bool_value("test-flag", &context).await;
        let result_int = resolver.resolve_int_value("test-flag", &context).await;
        let result_float = resolver.resolve_float_value("test-flag", &context).await;
        let result_string = resolver.resolve_string_value("test-flag", &context).await;
        let result_struct = resolver.resolve_struct_value("test-flag", &context).await;

        assert!(result_bool.is_err());
        assert!(result_int.is_err());
        assert!(result_float.is_err());
        assert!(result_string.is_err());
        assert!(result_struct.is_err());

        assert_eq!(
            result_bool.unwrap_err().code,
            EvaluationErrorCode::General("authentication/authorization error".to_string())
        );
        assert_eq!(
            result_int.unwrap_err().code,
            EvaluationErrorCode::General("authentication/authorization error".to_string())
        );
        assert_eq!(
            result_float.unwrap_err().code,
            EvaluationErrorCode::General("authentication/authorization error".to_string())
        );
        assert_eq!(
            result_string.unwrap_err().code,
            EvaluationErrorCode::General("authentication/authorization error".to_string())
        );
        assert_eq!(
            result_struct.unwrap_err().code,
            EvaluationErrorCode::General("authentication/authorization error".to_string())
        );
    }

    #[test(tokio::test)]
    #[serial_test::serial]
    async fn test_error_403() {
        let (mock_server, resolver) = setup_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(ResponseTemplate::new(403).set_body_json(json!({})))
            .mount(&mock_server)
            .await;

        let context = EvaluationContext::default();

        let result_bool = resolver.resolve_bool_value("test-flag", &context).await;
        let result_int = resolver.resolve_int_value("test-flag", &context).await;
        let result_float = resolver.resolve_float_value("test-flag", &context).await;
        let result_string = resolver.resolve_string_value("test-flag", &context).await;
        let result_struct = resolver.resolve_struct_value("test-flag", &context).await;

        assert!(result_bool.is_err());
        assert!(result_int.is_err());
        assert!(result_float.is_err());
        assert!(result_string.is_err());
        assert!(result_struct.is_err());

        assert_eq!(
            result_bool.unwrap_err().code,
            EvaluationErrorCode::General("authentication/authorization error".to_string())
        );
        assert_eq!(
            result_int.unwrap_err().code,
            EvaluationErrorCode::General("authentication/authorization error".to_string())
        );
        assert_eq!(
            result_float.unwrap_err().code,
            EvaluationErrorCode::General("authentication/authorization error".to_string())
        );
        assert_eq!(
            result_string.unwrap_err().code,
            EvaluationErrorCode::General("authentication/authorization error".to_string())
        );
        assert_eq!(
            result_struct.unwrap_err().code,
            EvaluationErrorCode::General("authentication/authorization error".to_string())
        );
    }

    #[test(tokio::test)]
    #[serial_test::serial]
    async fn test_error_404() {
        let (mock_server, resolver) = setup_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({})))
            .mount(&mock_server)
            .await;

        let context = EvaluationContext::default();

        let result_bool = resolver.resolve_bool_value("test-flag", &context).await;
        let result_int = resolver.resolve_int_value("test-flag", &context).await;
        let result_float = resolver.resolve_float_value("test-flag", &context).await;
        let result_string = resolver.resolve_string_value("test-flag", &context).await;
        let result_struct = resolver.resolve_struct_value("test-flag", &context).await;

        assert!(result_bool.is_err());
        assert!(result_int.is_err());
        assert!(result_float.is_err());
        assert!(result_string.is_err());
        assert!(result_struct.is_err());

        let result_bool_error = result_bool.unwrap_err();
        assert_eq!(result_bool_error.code, EvaluationErrorCode::FlagNotFound);
        assert_eq!(
            result_bool_error.message.unwrap(),
            "Flag: test-flag not found"
        );

        let result_int_error = result_int.unwrap_err();
        assert_eq!(result_int_error.code, EvaluationErrorCode::FlagNotFound);
        assert_eq!(
            result_int_error.message.unwrap(),
            "Flag: test-flag not found"
        );

        let result_float_error = result_float.unwrap_err();
        assert_eq!(result_float_error.code, EvaluationErrorCode::FlagNotFound);
        assert_eq!(
            result_float_error.message.unwrap(),
            "Flag: test-flag not found"
        );

        let result_string_error = result_string.unwrap_err();
        assert_eq!(result_string_error.code, EvaluationErrorCode::FlagNotFound);
        assert_eq!(
            result_string_error.message.unwrap(),
            "Flag: test-flag not found"
        );

        let result_struct_error = result_struct.unwrap_err();
        assert_eq!(result_struct_error.code, EvaluationErrorCode::FlagNotFound);
        assert_eq!(
            result_struct_error.message.unwrap(),
            "Flag: test-flag not found"
        );
    }

    #[test(tokio::test)]
    #[serial_test::serial]
    async fn test_error_429() {
        let (mock_server, resolver) = setup_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("Retry-After", "3")
                    .set_body_json(json!({})),
            )
            .mount(&mock_server)
            .await;

        let context = EvaluationContext::default();

        let result_bool = resolver.resolve_bool_value("test-flag", &context).await;
        let result_bool_2 = resolver.resolve_bool_value("test-flag", &context).await;

        assert!(result_bool.is_err());
        let result_bool_error = result_bool.unwrap_err();
        assert_eq!(
            result_bool_error.code,
            EvaluationErrorCode::General("Rate limit exceeded".to_string())
        );
        assert!(
            result_bool_error
                .message
                .unwrap()
                .starts_with("Rate limit exceeded. Retry after")
        );

        assert!(result_bool_2.is_err());
        let result_bool_error_2 = result_bool_2.unwrap_err();
        assert_eq!(
            result_bool_error_2.code,
            EvaluationErrorCode::General("Rate limit exceeded".to_string())
        );
        assert_eq!(
            result_bool_error_2.message.unwrap(),
            "Rate limit exceeded. Please wait before making another request."
        );

        sleep(Duration::from_secs(3)).await;

        let result_bool_3 = resolver.resolve_bool_value("test-flag", &context).await;
        assert!(result_bool_3.is_err());

        let result_bool_error_3 = result_bool_3.unwrap_err();
        assert_eq!(
            result_bool_error_3.code,
            EvaluationErrorCode::General("Rate limit exceeded".to_string())
        );
        assert!(
            result_bool_error_3
                .message
                .unwrap()
                .starts_with("Rate limit exceeded. Retry after")
        );
    }

    #[test(tokio::test)]
    async fn test_reason_and_metadata_come_from_the_response() {
        let (mock_server, resolver) = setup_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "value": true,
                "variant": "on",
                "reason": "TARGETING_MATCH",
                "metadata": {"revision": 7, "ratio": 0.5, "owner": "team", "stable": true}
            })))
            .mount(&mock_server)
            .await;

        let context = EvaluationContext::default();
        let result = resolver
            .resolve_bool_value("test-flag", &context)
            .await
            .unwrap();

        assert_eq!(result.reason, Some(EvaluationReason::TargetingMatch));
        let metadata = result.flag_metadata.unwrap().values;
        assert_eq!(metadata.get("revision"), Some(&FlagMetadataValue::Int(7)));
        assert_eq!(metadata.get("ratio"), Some(&FlagMetadataValue::Float(0.5)));
        assert_eq!(
            metadata.get("owner"),
            Some(&FlagMetadataValue::String("team".to_string()))
        );
        assert_eq!(metadata.get("stable"), Some(&FlagMetadataValue::Bool(true)));
    }

    #[test(tokio::test)]
    async fn test_server_error_with_a_valid_body_is_an_error() {
        let (mock_server, resolver) = setup_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(ResponseTemplate::new(500).set_body_json(json!({
                "value": true,
                "reason": "STATIC"
            })))
            .mount(&mock_server)
            .await;

        let context = EvaluationContext::default();
        let error = resolver
            .resolve_bool_value("test-flag", &context)
            .await
            .unwrap_err();

        assert_eq!(
            error.code,
            EvaluationErrorCode::General(
                "Unexpected response status 500 Internal Server Error".to_string()
            )
        );
    }

    #[test(tokio::test)]
    async fn test_redirect_to_another_origin_is_not_followed() {
        let (mock_server, resolver) = setup_mock_server().await;
        let other_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(ResponseTemplate::new(307).insert_header(
                "Location",
                format!("{}/ofrep/v1/evaluate/flags/test-flag", other_server.uri()),
            ))
            .mount(&mock_server)
            .await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"value": true})))
            .expect(0)
            .mount(&other_server)
            .await;

        let context = EvaluationContext::default().with_targeting_key("test-user");
        let error = resolver
            .resolve_bool_value("test-flag", &context)
            .await
            .unwrap_err();

        assert_eq!(
            error.code,
            EvaluationErrorCode::General(
                "Unexpected response status 307 Temporary Redirect".to_string()
            )
        );
        other_server.verify().await;
    }

    #[test(tokio::test)]
    async fn test_huge_retry_after_is_a_rate_limit_not_a_panic() {
        let (mock_server, resolver) = setup_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(
                ResponseTemplate::new(429).insert_header("Retry-After", i64::MAX.to_string()),
            )
            .mount(&mock_server)
            .await;

        let context = EvaluationContext::default();
        let first = resolver.resolve_bool_value("test-flag", &context).await;
        let second = resolver.resolve_bool_value("test-flag", &context).await;

        assert_eq!(
            first.unwrap_err().code,
            EvaluationErrorCode::General("Rate limit exceeded".to_string())
        );
        assert_eq!(
            second.unwrap_err().message.unwrap(),
            "Rate limit exceeded. Please wait before making another request."
        );
    }

    #[test(tokio::test)]
    async fn test_rate_limit_of_one_provider_leaves_another_alone() {
        let (limited_server, limited) = setup_mock_server().await;
        let (healthy_server, healthy) = setup_mock_server().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "30"))
            .mount(&limited_server)
            .await;
        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "value": true,
                "reason": "STATIC"
            })))
            .expect(1)
            .mount(&healthy_server)
            .await;

        let context = EvaluationContext::default();
        assert!(
            limited
                .resolve_bool_value("test-flag", &context)
                .await
                .is_err()
        );
        let result = healthy
            .resolve_bool_value("test-flag", &context)
            .await
            .unwrap();

        assert!(result.value);
        healthy_server.verify().await;
    }

    #[test]
    fn test_retry_after_accepts_an_http_date() {
        let retry_after = Resolver::parse_retry_after("Wed, 21 Oct 2099 07:28:00 GMT");
        assert_eq!(
            retry_after,
            DateTime::parse_from_rfc3339("2099-10-21T07:28:00Z").unwrap()
        );
    }

    async fn resolve_bool_with(
        response: ResponseTemplate,
    ) -> EvaluationResult<ResolutionDetails<bool>> {
        let (mock_server, resolver) = setup_mock_server().await;
        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        resolver
            .resolve_bool_value("test-flag", &EvaluationContext::default())
            .await
    }

    #[test(tokio::test)]
    async fn test_error_400_maps_the_error_code_of_the_body() {
        let cases = [
            ("PARSE_ERROR", EvaluationErrorCode::ParseError),
            (
                "TARGETING_KEY_MISSING",
                EvaluationErrorCode::TargetingKeyMissing,
            ),
            ("INVALID_CONTEXT", EvaluationErrorCode::InvalidContext),
            (
                "GENERAL",
                EvaluationErrorCode::General("Bad evaluation request".to_string()),
            ),
        ];

        for (error_code, expected) in cases {
            let error = resolve_bool_with(ResponseTemplate::new(400).set_body_json(json!({
                "key": "test-flag",
                "errorCode": error_code,
                "errorDetails": "details from the server"
            })))
            .await
            .unwrap_err();

            assert_eq!(error.code, expected, "errorCode {error_code}");
            assert_eq!(error.message.as_deref(), Some("details from the server"));
        }
    }

    #[test(tokio::test)]
    async fn test_error_404_and_500_report_the_error_details_of_the_body() {
        let not_found = resolve_bool_with(ResponseTemplate::new(404).set_body_json(json!({
            "key": "test-flag",
            "errorCode": "FLAG_NOT_FOUND",
            "errorDetails": "Flag 'test-flag' was not found"
        })))
        .await
        .unwrap_err();
        let server_error = resolve_bool_with(
            ResponseTemplate::new(500)
                .set_body_json(json!({"errorDetails": "An internal server error occurred"})),
        )
        .await
        .unwrap_err();

        assert_eq!(not_found.code, EvaluationErrorCode::FlagNotFound);
        assert_eq!(
            not_found.message.as_deref(),
            Some("Flag 'test-flag' was not found")
        );
        assert_eq!(
            server_error.message.as_deref(),
            Some("An internal server error occurred")
        );
    }

    #[test(tokio::test)]
    async fn test_success_without_a_value_defers_to_the_code_default() {
        let error = resolve_bool_with(ResponseTemplate::new(200).set_body_json(json!({
            "key": "test-flag",
            "reason": "DEFAULT",
            "variant": "control"
        })))
        .await
        .unwrap_err();

        assert_eq!(
            error.code,
            EvaluationErrorCode::General("Code default".to_string())
        );
    }

    #[test(tokio::test)]
    async fn test_value_of_another_type_is_a_type_mismatch() {
        let error = resolve_bool_with(ResponseTemplate::new(200).set_body_json(json!({
            "key": "test-flag",
            "value": "yes",
            "reason": "STATIC"
        })))
        .await
        .unwrap_err();

        assert_eq!(error.code, EvaluationErrorCode::TypeMismatch);
    }

    #[test(tokio::test)]
    async fn test_null_value_is_a_type_mismatch_not_a_code_default() {
        let error = resolve_bool_with(
            ResponseTemplate::new(200).set_body_json(json!({"key": "test-flag", "value": null})),
        )
        .await
        .unwrap_err();

        assert_eq!(error.code, EvaluationErrorCode::TypeMismatch);
    }

    #[test(tokio::test)]
    async fn test_success_body_that_is_not_an_object_is_a_parse_error() {
        let error = resolve_bool_with(ResponseTemplate::new(200).set_body_json(json!([true])))
            .await
            .unwrap_err();

        assert_eq!(error.code, EvaluationErrorCode::ParseError);
    }

    #[test(tokio::test)]
    async fn test_error_400_fallbacks() {
        let unknown_code = resolve_bool_with(
            ResponseTemplate::new(400).set_body_json(json!({"errorCode": "SOMETHING_NEW"})),
        )
        .await
        .unwrap_err();
        let code_without_details = resolve_bool_with(
            ResponseTemplate::new(400).set_body_json(json!({"errorCode": "PARSE_ERROR"})),
        )
        .await
        .unwrap_err();
        let not_json = resolve_bool_with(ResponseTemplate::new(400).set_body_string("oops"))
            .await
            .unwrap_err();
        let oversized = resolve_bool_with(ResponseTemplate::new(400).set_body_json(json!({
            "errorCode": "PARSE_ERROR",
            "errorDetails": "x".repeat(ERROR_BODY_MAX_BYTES)
        })))
        .await
        .unwrap_err();

        assert_eq!(
            unknown_code.code,
            EvaluationErrorCode::General("Bad evaluation request".to_string())
        );
        assert_eq!(
            unknown_code.message.as_deref(),
            Some("Bad evaluation request")
        );
        assert_eq!(code_without_details.code, EvaluationErrorCode::ParseError);
        assert_eq!(
            code_without_details.message.as_deref(),
            Some("Bad evaluation request")
        );
        for error in [not_json, oversized] {
            assert_eq!(error.code, EvaluationErrorCode::InvalidContext);
            assert_eq!(error.message.as_deref(), Some("Invalid context"));
        }
    }

    #[test(tokio::test)]
    async fn test_stalled_error_body_does_not_delay_the_fallback() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        // Sends the status line and headers, then never finishes the announced body.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0u8; 4096];
            let _ = socket.read(&mut request).await;
            socket
                .write_all(
                    b"HTTP/1.1 400 Bad Request\r\ncontent-type: application/json\r\n\
                      content-length: 1000\r\n\r\n{\"errorCode\":",
                )
                .await
                .unwrap();
            sleep(Duration::from_secs(30)).await;
        });
        let resolver = Resolver::new(&OfrepOptions {
            base_url: format!("http://{address}"),
            ..Default::default()
        })
        .unwrap();

        let started = std::time::Instant::now();
        let error = resolver
            .resolve_bool_value("test-flag", &EvaluationContext::default())
            .await
            .unwrap_err();

        assert_eq!(error.code, EvaluationErrorCode::InvalidContext);
        assert!(started.elapsed() < Duration::from_secs(5));
    }
}
