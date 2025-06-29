use async_trait::async_trait;
use open_feature::provider::{FeatureProvider, ProviderMetadata, ResolutionDetails};
use open_feature::{
    EvaluationContext, EvaluationContextFieldValue, EvaluationError, EvaluationErrorCode,
    EvaluationResult, StructValue, Value,
};
use reqwest::Client;
use reqwest::StatusCode;
use serde_json;
use tracing::{debug, error, instrument};

use crate::OfrepOptions;

#[derive(Debug)]
pub struct Resolver {
    base_url: String,
    metadata: ProviderMetadata,
    client: Client,
}

impl Resolver {
    pub fn new(options: &OfrepOptions) -> Self {
        Self {
            base_url: options.base_url.clone(),
            metadata: ProviderMetadata::new("ofrep"),
            client: Client::builder()
                .default_headers(options.headers.clone())
                .connect_timeout(options.connect_timeout)
                .build()
                .expect("Failed to build HTTP client"),
        }
    }
}

#[async_trait]
impl FeatureProvider for Resolver {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    #[instrument(skip(self, evaluation_context), fields(flag_key = %flag_key))]
    async fn resolve_bool_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<bool>> {
        debug!("Resolving boolean flag");

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
                error!(error = %e, "Failed to resolve boolean value");
                EvaluationError {
                    code: EvaluationErrorCode::General(
                        "Failed to resolve boolean value".to_string(),
                    ),
                    message: Some(e.to_string()),
                }
            })?;

        debug!(status = response.status().as_u16(), "Received response");

        match response.status() {
            StatusCode::BAD_REQUEST => {
                return Err(EvaluationError {
                    code: EvaluationErrorCode::InvalidContext,
                    message: Some("Invalid context".to_string()),
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
                return Err(EvaluationError {
                    code: EvaluationErrorCode::FlagNotFound,
                    message: Some(format!("Flag: {flag_key} not found")),
                });
            }
            _ => {
                let result = response.json::<serde_json::Value>().await.map_err(|e| {
                    error!(error = %e, "Failed to parse boolean response");
                    EvaluationError {
                        code: EvaluationErrorCode::ParseError,
                        message: Some(e.to_string()),
                    }
                })?;

                let value = result["value"].as_bool().ok_or_else(|| {
                    error!("Invalid boolean value in response");
                    EvaluationError {
                        code: EvaluationErrorCode::ParseError,
                        message: Some("Invalid boolean value".to_string()),
                    }
                })?;

                debug!(value = value, variant = ?result["variant"], "Flag evaluated");
                Ok(ResolutionDetails {
                    value,
                    variant: result["variant"].as_str().map(String::from),
                    reason: Some(open_feature::EvaluationReason::Static),
                    flag_metadata: Default::default(),
                })
            }
        }
    }

    async fn resolve_string_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<String>> {
        debug!("Resolving string flag");

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
                error!(error = %e, "Failed to resolve string value");
                EvaluationError {
                    code: EvaluationErrorCode::General(
                        "Failed to resolve string value".to_string(),
                    ),
                    message: Some(e.to_string()),
                }
            })?;

        debug!(status = response.status().as_u16(), "Received response");

        match response.status() {
            StatusCode::BAD_REQUEST => {
                return Err(EvaluationError {
                    code: EvaluationErrorCode::InvalidContext,
                    message: Some("Invalid context".to_string()),
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
                return Err(EvaluationError {
                    code: EvaluationErrorCode::FlagNotFound,
                    message: Some(format!("Flag: {flag_key} not found")),
                });
            }
            _ => {
                let result = response.json::<serde_json::Value>().await.map_err(|e| {
                    error!(error = %e, "Failed to parse string response");
                    EvaluationError {
                        code: EvaluationErrorCode::ParseError,
                        message: Some(e.to_string()),
                    }
                })?;

                let value = result["value"]
                    .as_str()
                    .ok_or_else(|| {
                        error!("Invalid string value in response");
                        EvaluationError {
                            code: EvaluationErrorCode::ParseError,
                            message: Some("Invalid string value".to_string()),
                        }
                    })?
                    .to_string();

                debug!(value = %value, variant = ?result["variant"], "Flag evaluated");
                Ok(ResolutionDetails {
                    value,
                    variant: result["variant"].as_str().map(String::from),
                    reason: Some(open_feature::EvaluationReason::Static),
                    flag_metadata: Default::default(),
                })
            }
        }
    }

    async fn resolve_float_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<f64>> {
        debug!("Resolving float flag");

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
                error!(error = %e, "Failed to resolve float value");
                EvaluationError {
                    code: EvaluationErrorCode::General("Failed to resolve float value".to_string()),
                    message: Some(e.to_string()),
                }
            })?;

        debug!(status = response.status().as_u16(), "Received response");

        match response.status() {
            StatusCode::BAD_REQUEST => {
                return Err(EvaluationError {
                    code: EvaluationErrorCode::InvalidContext,
                    message: Some("Invalid context".to_string()),
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
                return Err(EvaluationError {
                    code: EvaluationErrorCode::FlagNotFound,
                    message: Some(format!("Flag: {flag_key} not found")),
                });
            }
            _ => {
                let result = response.json::<serde_json::Value>().await.map_err(|e| {
                    error!(error = %e, "Failed to parse float response");
                    EvaluationError {
                        code: EvaluationErrorCode::ParseError,
                        message: Some(e.to_string()),
                    }
                })?;

                let value = result["value"].as_f64().ok_or_else(|| {
                    error!("Invalid float value in response");
                    EvaluationError {
                        code: EvaluationErrorCode::ParseError,
                        message: Some("Invalid float value".to_string()),
                    }
                })?;

                debug!(value = value, variant = ?result["variant"], "Flag evaluated");
                Ok(ResolutionDetails {
                    value,
                    variant: result["variant"].as_str().map(String::from),
                    reason: Some(open_feature::EvaluationReason::Static),
                    flag_metadata: Default::default(),
                })
            }
        }
    }

    async fn resolve_int_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<i64>> {
        debug!("Resolving integer flag");

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
                error!(error = %e, "Failed to resolve integer value");
                EvaluationError {
                    code: EvaluationErrorCode::General(
                        "Failed to resolve integer value".to_string(),
                    ),
                    message: Some(e.to_string()),
                }
            })?;

        debug!(status = response.status().as_u16(), "Received response");

        match response.status() {
            StatusCode::BAD_REQUEST => {
                return Err(EvaluationError {
                    code: EvaluationErrorCode::InvalidContext,
                    message: Some("Invalid context".to_string()),
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
                return Err(EvaluationError {
                    code: EvaluationErrorCode::FlagNotFound,
                    message: Some(format!("Flag: {flag_key} not found")),
                });
            }
            _ => {
                let result = response.json::<serde_json::Value>().await.map_err(|e| {
                    error!(error = %e, "Failed to parse integer response");
                    EvaluationError {
                        code: EvaluationErrorCode::ParseError,
                        message: Some(e.to_string()),
                    }
                })?;

                let value = result["value"].as_i64().ok_or_else(|| {
                    error!("Invalid integer value in response");
                    EvaluationError {
                        code: EvaluationErrorCode::ParseError,
                        message: Some("Invalid integer value".to_string()),
                    }
                })?;

                debug!(value = value, variant = ?result["variant"], "Flag evaluated");
                Ok(ResolutionDetails {
                    value,
                    variant: result["variant"].as_str().map(String::from),
                    reason: Some(open_feature::EvaluationReason::Static),
                    flag_metadata: Default::default(),
                })
            }
        }
    }

    async fn resolve_struct_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<StructValue>> {
        debug!("Resolving struct flag");

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
                error!(error = %e, "Failed to resolve struct value");
                EvaluationError {
                    code: EvaluationErrorCode::General(
                        "Failed to resolve struct value".to_string(),
                    ),
                    message: Some(e.to_string()),
                }
            })?;

        debug!(status = response.status().as_u16(), "Received response");

        match response.status() {
            StatusCode::BAD_REQUEST => {
                return Err(EvaluationError {
                    code: EvaluationErrorCode::InvalidContext,
                    message: Some("Invalid context".to_string()),
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
                return Err(EvaluationError {
                    code: EvaluationErrorCode::FlagNotFound,
                    message: Some(format!("Flag: {flag_key} not found")),
                });
            }
            _ => {
                let result = response.json::<serde_json::Value>().await.map_err(|e| {
                    error!(error = %e, "Failed to parse struct response");
                    EvaluationError {
                        code: EvaluationErrorCode::ParseError,
                        message: Some(e.to_string()),
                    }
                })?;

                let value = result["value"]
                    .clone()
                    .into_feature_value()
                    .as_struct()
                    .ok_or_else(|| {
                        error!("Invalid struct value in response");
                        EvaluationError {
                            code: EvaluationErrorCode::ParseError,
                            message: Some("Invalid struct value".to_string()),
                        }
                    })?
                    .clone();

                debug!(variant = ?result["variant"], "Flag evaluated");
                Ok(ResolutionDetails {
                    value,
                    variant: result["variant"].as_str().map(String::from),
                    reason: Some(open_feature::EvaluationReason::Static),
                    flag_metadata: Default::default(),
                })
            }
        }
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
            EvaluationContextFieldValue::Struct(s) => serde_json::Value::String(format!("{:?}", s)),
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
    use serde_json::json;
    use test_log::test;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup_mock_server() -> (MockServer, Resolver) {
        let mock_server = MockServer::start().await;
        let options = OfrepOptions {
            base_url: mock_server.uri(),
            ..Default::default()
        };
        let resolver = Resolver::new(&options);
        (mock_server, resolver)
    }

    #[test(tokio::test)]
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
}
