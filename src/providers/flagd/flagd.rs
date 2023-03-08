use rust_sdk::providers::traits::FeatureProvider;


pub struct Provider {}

impl FeatureProvider for Provider {
    fn new() -> Self {
        Provider {}
    }

    fn meta_data(&self) -> rust_sdk::providers::types::ProviderMetadata {
        rust_sdk::providers::types::ProviderMetadata {
            name: "flagd".to_string(),
        }
    }

    fn resolution<T>(
        &self,
        _flag: String,
        _default_value: T,
        _eval_ctx: rust_sdk::evaluation::FlattenedContext,
    ) -> anyhow::Result<rust_sdk::providers::types::ResolutionDetails<T>>
    where
        T: Clone {
        todo!()
    }
}