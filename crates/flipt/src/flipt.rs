use async_trait::async_trait;
use flipt::evaluation::models::{EvaluationRequest, VariantEvaluationResponse};
use open_feature::{
    EvaluationContext, EvaluationError, EvaluationErrorCode, EvaluationResult, StructValue, Value,
    provider::{FeatureProvider, ProviderMetadata, ResolutionDetails},
};
use url::Url;

use crate::utils::{parse_json, translate_context, translate_error};

// reexports
pub use flipt::{ClientTokenAuthentication, JWTAuthentication, NoneAuthentication};

const DEFAULT_ENTITY_ID: &str = "";
const METADATA: &str = "flipt";

pub struct Config<A>
where
    A: flipt::AuthenticationStrategy,
{
    /// The URL of the Flipt server
    pub url: String,
    /// The authentication strategy to use
    pub authentication_strategy: A,
    /// Timeout in seconds
    pub timeout: u64,
}

/// A feature provider that uses Flipt as a backend
pub struct FliptProvider {
    metadata: ProviderMetadata,
    client: flipt::api::FliptClient,
    namespace: String,
}

impl FliptProvider {
    /// Create a new Flipt provider
    pub fn new<A: flipt::AuthenticationStrategy>(
        namespace: String,
        config: Config<A>,
    ) -> Result<Self, String> {
        let url = match Url::parse(&config.url) {
            Ok(url) => url,
            Err(e) => return Err(e.to_string()),
        };

        let flipt_config = flipt::ConfigBuilder::<A>::default()
            .with_endpoint(url.clone())
            .with_auth_strategy(config.authentication_strategy)
            .with_timeout(std::time::Duration::from_secs(config.timeout))
            .build();
        let client = match flipt::api::FliptClient::new(flipt_config) {
            Ok(fpconfig) => fpconfig,
            Err(e) => return Err(e.to_string()),
        };

        Ok(Self {
            metadata: ProviderMetadata::new(METADATA),
            client,
            namespace,
        })
    }
}

#[async_trait]
impl FeatureProvider for FliptProvider {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn resolve_bool_value(
        &self,
        flag_key: &str,
        ctx: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<bool>> {
        self.client
            .evaluation
            .boolean(&EvaluationRequest {
                namespace_key: self.namespace.clone(),
                flag_key: flag_key.into(),
                entity_id: ctx
                    .targeting_key
                    .clone()
                    .unwrap_or(DEFAULT_ENTITY_ID.to_owned()),
                context: translate_context(ctx),
                reference: None,
            })
            .await
            .map_err(translate_error)
            .map(|v| ResolutionDetails::new(v.enabled))
    }

    async fn resolve_int_value(
        &self,
        flag_key: &str,
        ctx: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<i64>> {
        let res = variant_helper(self, flag_key, ctx).await?;
        // parse a variant key as i64
        res.variant_key
            .parse::<i64>()
            .map_err(|e| EvaluationError {
                code: EvaluationErrorCode::General("Parse error".to_owned()),
                message: Some(format!(
                    "Expected a number in range of i64, but found `{}` ({:?})",
                    res.variant_attachment, e
                )),
            })
            .map(ResolutionDetails::new)
    }

    async fn resolve_float_value(
        &self,
        flag_key: &str,
        ctx: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<f64>> {
        let res = variant_helper(self, flag_key, ctx).await?;
        // parse a variant key as f64
        res.variant_key
            .parse::<f64>()
            .map_err(|e| EvaluationError {
                code: EvaluationErrorCode::General("Parse error".to_owned()),
                message: Some(format!(
                    "Expected a number in range of f64, but found `{}` ({:?})",
                    res.variant_attachment, e
                )),
            })
            .map(ResolutionDetails::new)
    }

    async fn resolve_string_value(
        &self,
        flag_key: &str,
        ctx: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<String>> {
        let res = variant_helper(self, flag_key, ctx).await?;
        // parse a variant key as i64
        Ok(ResolutionDetails::new(res.variant_key))
    }

    async fn resolve_struct_value(
        &self,
        flag_key: &str,
        ctx: &EvaluationContext,
    ) -> Result<ResolutionDetails<StructValue>, EvaluationError> {
        let res = variant_helper(self, flag_key, ctx).await?;
        // parse a variant attachment as a struct value
        let v = parse_json(&res.variant_attachment)?;
        if let Value::Struct(sv) = v {
            Ok(ResolutionDetails::new(sv))
        } else {
            Err(EvaluationError {
                code: EvaluationErrorCode::General("Parse error".to_owned()),
                message: Some(format!(
                    "Expected a struct value, but found `{}`",
                    res.variant_attachment
                )),
            })
        }
    }
}

async fn variant_helper(
    provider: &FliptProvider,
    flag_key: &str,
    ctx: &EvaluationContext,
) -> Result<VariantEvaluationResponse, EvaluationError> {
    provider
        .client
        .evaluation
        .variant(&EvaluationRequest {
            namespace_key: provider.namespace.clone(),
            flag_key: flag_key.into(),
            entity_id: ctx
                .targeting_key
                .clone()
                .unwrap_or(DEFAULT_ENTITY_ID.to_owned()),
            context: translate_context(ctx),
            reference: None,
        })
        .await
        .map_err(translate_error)
}
