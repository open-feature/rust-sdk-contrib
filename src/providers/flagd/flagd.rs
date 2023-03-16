use rust_sdk::providers::traits::FeatureProvider;

include!("proto/rust/mod.rs");
pub struct Provider {
  
}

impl FeatureProvider for Provider {
    fn new() -> Self {
        Provider {
            //service: service::ChannelService::new(),
        }
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

#[cfg(test)]
mod tests {
    use rust_sdk::{ClientMetadata, traits::Client};

    use crate::flagd::{Provider};

    #[test]
    fn test_provider() {

        let provider = Provider{
         //   service: service::ChannelService::new(),
        };

        let client = rust_sdk::OpenFeatureClient::<Provider>::new(
            "test".to_string(),
            provider,
        );

        assert!(client.meta_data().name == "test");

    }
}