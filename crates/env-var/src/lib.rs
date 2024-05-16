use async_trait::async_trait;
use open_feature::{
    provider::{FeatureProvider, ProviderMetadata, ResolutionDetails},
    EvaluationContext, EvaluationError, EvaluationErrorCode, EvaluationReason, EvaluationResult,
    StructValue,
};
/// Environment Variables Provider Metadata
const METADATA: &str = "Environment Variables Provider";

/// Environment Variables Provider
///
/// This provider resolves feature flags from environment variables.
/// The provider supports the following types:
/// - Int
/// - Float
/// - String
/// - Bool
/// - Struct (not supported)
///
/// The provider will return [`EvaluationResult::Err(EvaluationError)`] if the flag is not found or if the value is not of the expected type.
#[derive(Debug)]
pub struct EnvVarProvider {
    metadata: ProviderMetadata,
}

/// Default implementation for the Environment Variables Provider
impl Default for EnvVarProvider {
    fn default() -> Self {
        Self {
            metadata: ProviderMetadata::new(METADATA),
        }
    }
}

/// Implementation of the FeatureProvider trait for the Environment Variables Provider
#[async_trait]
impl FeatureProvider for EnvVarProvider {
    /// Returns the provider metadata
    /// # Example
    /// ```rust
    /// #[tokio::test]
    /// async fn test_metadata() {
    ///    let provider = EnvVarProvider::default();
    ///   assert_eq!(provider.metadata().name, "Environment Variables Provider");
    /// }
    /// ```
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    /// A logical true or false, as represented idiomatically in the implementation languages.
    ///
    /// # Example
    /// ```rust
    /// #[tokio::test]
    /// async fn test_resolve_string_value() {
    ///     let provider = EnvVarProvider::default();
    ///     let flag_key = "TEST_ENV_VAR";
    ///     let value = "false";
    ///     std::env::set_var(flag_key, value);
    ///
    ///     let res = provider
    ///         .resolve_string_value(flag_key, &EvaluationContext::default())
    ///         .await;
    ///     assert!(res.is_ok());
    ///     assert_eq!(res.unwrap().value, value);
    /// }
    /// ```
    async fn resolve_bool_value(
        &self,
        flag_key: &str,
        _evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<bool>> {
        return evaluate_environment_variable(flag_key, _evaluation_context);
    }

    /// The 64-bit signed integer type.
    /// # Example
    /// ```rust
    /// #[tokio::test]
    /// async fn test_resolve_int_value() {
    ///     let flag_key = "TEST_INT_ENV_VAR";
    ///     let flag_value = std::i64::MAX.to_string();
    ///     let provider = EnvVarProvider::default();
    ///     std::env::set_var(flag_key, &flag_value);
    ///     let result = provider.resolve_int_value(flag_key, &EvaluationContext::default()).await;
    ///     assert!(result.is_ok());
    ///     assert_eq!(result.unwrap().value, flag_value.parse::<i64>().unwrap());
    /// }
    /// ```
    async fn resolve_int_value(
        &self,
        flag_key: &str,
        _evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<i64>> {
        return evaluate_environment_variable(flag_key, _evaluation_context);
    }

    /// A 64-bit floating point type
    ///
    /// # Example
    /// ```rust
    /// #[tokio::test]
    /// async fn test_resolve_float_value() {
    ///     let flag_key = "TEST_FLOAT_ENV_VAR";
    ///     let flag_value = std::f64::consts::PI.to_string();
    ///     let provider = EnvVarProvider::default();
    ///
    ///     std::env::set_var(flag_key, &flag_value);
    ///
    ///     let result = provider
    ///         .resolve_float_value(flag_key, &EvaluationContext::default())
    ///         .await;
    ///     assert!(result.is_ok());
    ///     assert_eq!(result.unwrap().value, flag_value.parse::<f64>().unwrap());
    /// }
    /// ```
    async fn resolve_float_value(
        &self,
        flag_key: &str,
        _evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<f64>> {
        return evaluate_environment_variable(flag_key, _evaluation_context);
    }

    /// A UTF-8 encoded string.
    /// # Example
    /// ```rust
    /// #[tokio::test]
    /// async fn test_resolve_string_value() {
    ///     let provider = EnvVarProvider::default();
    ///     let flag_key = "TEST_ENV_VAR";
    ///     let value = "flag_value";
    ///     std::env::set_var(flag_key, value);
    ///
    ///     let res = provider
    ///         .resolve_string_value(flag_key, &EvaluationContext::default())
    ///         .await;
    ///     assert!(res.is_ok());
    ///     assert_eq!(res.unwrap().value, value);
    /// }
    /// ```
    async fn resolve_string_value(
        &self,
        flag_key: &str,
        _evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<String>> {
        return evaluate_environment_variable(flag_key, _evaluation_context);
    }

    /// Structured data, presented however is idiomatic in the implementation language, such as JSON or YAML.
    async fn resolve_struct_value(
        &self,
        _flag_key: &str,
        _evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<StructValue>> {
        return error(EvaluationErrorCode::General(
            "Structs are not supported".to_string(),
        ));
    }
}

/// Helper function to evaluate the environment variable
/// # Example
/// ```rust
/// #[tokio::test]
/// async fn test_evaluate_environment_variable() {
///    let provider = EnvVarProvider::default();
///    let flag_key = "TEST_ENV_VAR_NOT_FOUND";
/// let res = evaluate_environment_variable(flag_key, &EvaluationContext::default());
/// assert!(res.is_err());
/// assert_eq!(res.unwrap_err().code, EvaluationErrorCode::FlagNotFound);
/// }
/// ```
fn evaluate_environment_variable<T: std::str::FromStr>(
    flag_key: &str,
    _evaluation_context: &EvaluationContext,
) -> EvaluationResult<ResolutionDetails<T>> {
    match std::env::var(flag_key) {
        Ok(value) => match value.parse::<T>() {
            Ok(parsed_value) => {
                return EvaluationResult::Ok(
                    ResolutionDetails::builder()
                        .value(parsed_value)
                        .reason(EvaluationReason::Static)
                        .build(),
                )
            }
            Err(_) => {
                return error(EvaluationErrorCode::TypeMismatch);
            }
        },
        Err(_) => {
            return error(EvaluationErrorCode::FlagNotFound);
        }
    };
}
/// Error helper function to return an [`EvaluationResult`] with an [`EvaluationError`]
/// # Example
/// ```rust
/// #[tokio::test]
/// async fn test_error() {
///     let provider = EnvVarProvider::default();
///     let flag_key = "TEST_ENV_VAR_NOT_FOUND";
///     let res = provider.resolve_string_value(flag_key, &EvaluationContext::default()).await;
///     assert!(res.is_err());
///     assert_eq!(res.unwrap_err().code, EvaluationErrorCode::FlagNotFound);
/// }
/// ```
fn error<T>(evaluation_error_code: EvaluationErrorCode) -> EvaluationResult<T> {
    Err(EvaluationError::builder()
        .message("Error evaluating environment variable")
        .code(evaluation_error_code)
        .build())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_metadata() {
        let provider = EnvVarProvider::default();
        assert_eq!(provider.metadata().name, "Environment Variables Provider");
    }

    #[tokio::test]
    async fn resolve_err_values() {
        let provider = EnvVarProvider::default();
        let context = EvaluationContext::default();

        assert!(provider.resolve_bool_value("", &context).await.is_err());
        assert!(provider.resolve_int_value("", &context).await.is_err());
        assert!(provider.resolve_float_value("", &context).await.is_err());
        assert!(provider.resolve_string_value("", &context).await.is_err());
        assert!(provider.resolve_struct_value("", &context).await.is_err());
    }
}
