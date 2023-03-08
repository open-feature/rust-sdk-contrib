
#[path ="providers/providers.rs"]
pub mod providers;

#[cfg(test)]
mod tests {

    // Upstream Rust SDK
    use rust_sdk::OpenFeatureClient;
    use rust_sdk::providers::NoopProvider;
    use rust_sdk::providers::traits::FeatureProvider;
    use rust_sdk::traits::Client;
    // Local providers
    use crate::providers;
  
    
    #[test]
    fn test_noop_provider() {
        let client = OpenFeatureClient::<NoopProvider>::new(
            "test".to_string(),
            NoopProvider::new(),
        );
        let result = client.value::<i64>("flag-key-here".to_string(),
            0, client.evaluation_context() );
        assert_eq!(result.unwrap(), 0);
    }
    #[test]
    fn test_flagd_provider() {
        let client = OpenFeatureClient::<providers::FlagDProvider>::new(
            "test".to_string(),
            providers::FlagDProvider::new(),
        );
        let result = client.value::<i64>("flag-key-here".to_string(),
            0, client.evaluation_context() );
        assert_eq!(result.unwrap(), 0);
    }
}
