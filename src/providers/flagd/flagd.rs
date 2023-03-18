use rust_sdk::providers::traits::FeatureProvider;

pub mod proto;

pub struct Provider {
  }

impl FeatureProvider for Provider {

    fn new() -> Self {
        Provider {
          
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
    use rust_sdk::{ClientMetadata, traits::Client, providers::traits::FeatureProvider};

    use crate::flagd::{Provider, proto::{self, rust::schema::v1::{ResolveStringRequest, ResolveAllRequest, ResolveBooleanRequest}}};

    
    #[tokio::test]
    async fn test_provider() {

        let svc =
         proto::rust::schema::v1::service_client::ServiceClient::<tonic::transport::Channel>::connect("http://0.0.0.0:8013");

        let client = rust_sdk::OpenFeatureClient::<Provider>::new(
            "test".to_string(),
            Provider::new(),
        );

        let mut client = svc.await.unwrap();

        let result = client.resolve_boolean(ResolveBooleanRequest {
            flag_key: "myFlag".to_string(),
            context: None,
        }).await.unwrap();

       print!("{:?}", result)
     
    }
}