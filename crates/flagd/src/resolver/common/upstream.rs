use crate::error::FlagdError;
use std::str::FromStr;
use tonic::transport::ClientTlsConfig;
use tonic::transport::{Endpoint, Uri};
use tracing::debug;

pub struct UpstreamConfig {
    endpoint: Endpoint,
    authority: Option<String>, // Only set for custom name resolution (envoy://)
}

impl UpstreamConfig {
    pub fn new(target: String, is_in_process: bool, tls: bool) -> Result<Self, FlagdError> {
        debug!(
            "Creating upstream config for target: {}, tls: {}",
            target, tls
        );

        let scheme = if tls { "https" } else { "http" };

        if target.starts_with("http://") || target.starts_with("https://") {
            debug!("Target is already an HTTP(S) endpoint");
            let mut endpoint = Endpoint::from_shared(target.clone())
                .map_err(|e| FlagdError::Config(format!("Invalid endpoint: {}", e)))?;

            // Apply TLS config for https URLs
            if target.starts_with("https://") {
                endpoint = endpoint
                    .tls_config(ClientTlsConfig::new().with_enabled_roots())
                    .map_err(|e| FlagdError::Config(format!("TLS config error: {}", e)))?;
            }

            return Ok(Self {
                endpoint,
                authority: None, // Standard HTTP(S) doesn't need custom authority
            });
        }

        let (endpoint_str, authority) = if target.starts_with("envoy://") {
            let uri = Uri::from_str(&target)
                .map_err(|e| FlagdError::Config(format!("Failed to parse target URI: {}", e)))?;
            let authority = uri.path().trim_start_matches('/');

            if authority.is_empty() {
                return Err(FlagdError::Config(
                    "Service name (authority) cannot be empty".to_string(),
                ));
            }

            let host = uri.host().unwrap_or("localhost");
            let port = uri.port_u16().unwrap_or(9211); // Use Envoy port directly

            (
                format!("{}://{}:{}", scheme, host, port),
                Some(authority.to_string()),
            )
        } else {
            let parts: Vec<&str> = target.split(':').collect();
            let host = parts.first().unwrap_or(&"localhost").to_string();
            let port = parts
                .get(1)
                .and_then(|p| p.parse().ok())
                .unwrap_or(if is_in_process { 8015 } else { 8013 });

            debug!("Using standard resolution with {}:{}", host, port);
            (format!("{}://{}:{}", scheme, host, port), None)
        };

        let mut endpoint = Endpoint::from_shared(endpoint_str)
            .map_err(|e| FlagdError::Config(format!("Invalid endpoint: {}", e)))?;

        // Apply TLS config when tls is enabled
        if tls {
            endpoint = endpoint
                .tls_config(ClientTlsConfig::new().with_enabled_roots())
                .map_err(|e| FlagdError::Config(format!("TLS config error: {}", e)))?;
        }

        Ok(Self {
            endpoint,
            authority,
        })
    }

    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    pub fn authority(&self) -> Option<String> {
        self.authority.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_disabled_uses_http_scheme() {
        let config = UpstreamConfig::new("localhost:8013".to_string(), false, false).unwrap();
        assert!(config.endpoint().uri().to_string().starts_with("http://"));
        assert_eq!(
            config.endpoint().uri().to_string(),
            "http://localhost:8013/"
        );
    }

    #[test]
    fn test_tls_enabled_uses_https_scheme() {
        let config = UpstreamConfig::new("localhost:8013".to_string(), false, true).unwrap();
        assert!(config.endpoint().uri().to_string().starts_with("https://"));
        assert_eq!(
            config.endpoint().uri().to_string(),
            "https://localhost:8013/"
        );
    }

    #[test]
    fn test_in_process_default_port_with_tls() {
        let config = UpstreamConfig::new("localhost".to_string(), true, true).unwrap();
        assert_eq!(
            config.endpoint().uri().to_string(),
            "https://localhost:8015/"
        );
    }

    #[test]
    fn test_rpc_default_port_with_tls() {
        let config = UpstreamConfig::new("localhost".to_string(), false, true).unwrap();
        assert_eq!(
            config.endpoint().uri().to_string(),
            "https://localhost:8013/"
        );
    }

    #[test]
    fn test_explicit_http_url_preserved() {
        let config =
            UpstreamConfig::new("http://example.com:9000".to_string(), false, true).unwrap();
        assert_eq!(
            config.endpoint().uri().to_string(),
            "http://example.com:9000/"
        );
    }

    #[test]
    fn test_explicit_https_url_preserved() {
        let config =
            UpstreamConfig::new("https://example.com:9000".to_string(), false, false).unwrap();
        assert_eq!(
            config.endpoint().uri().to_string(),
            "https://example.com:9000/"
        );
    }

    #[test]
    fn test_envoy_target_with_tls() {
        let config =
            UpstreamConfig::new("envoy://localhost:9211/my-service".to_string(), false, true)
                .unwrap();
        assert!(config.endpoint().uri().to_string().starts_with("https://"));
        assert_eq!(config.authority(), Some("my-service".to_string()));
    }

    #[test]
    fn test_envoy_target_without_tls() {
        let config = UpstreamConfig::new(
            "envoy://localhost:9211/my-service".to_string(),
            false,
            false,
        )
        .unwrap();
        assert!(config.endpoint().uri().to_string().starts_with("http://"));
        assert_eq!(config.authority(), Some("my-service".to_string()));
    }
}
