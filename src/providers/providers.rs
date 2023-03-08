use rust_sdk::providers::traits::FeatureProvider;



pub struct FlagDProvider {}

impl FeatureProvider for FlagDProvider {
    fn new() -> Self {
        FlagDProvider {}
    }

    fn meta_data(&self) -> rust_sdk::providers::types::ProviderMetadata {
        rust_sdk::providers::types::ProviderMetadata {
            name: "flagd".to_string(),
        }
    }

    fn resolution<T>(
        &self,
        flag: String,
        default_value: T,
        eval_ctx: rust_sdk::evaluation::FlattenedContext,
    ) -> anyhow::Result<rust_sdk::providers::types::ResolutionDetails<T>>
    where
        T: Clone {
        todo!()
    }
}