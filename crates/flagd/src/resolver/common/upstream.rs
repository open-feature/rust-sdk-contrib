use crate::error::FlagdError;
use std::str::FromStr;
use tonic::transport::{Certificate, ClientTlsConfig};
use tonic::transport::{Endpoint, Uri};
use tracing::debug;

#[derive(Debug)]
pub struct UpstreamConfig {
    endpoint: Endpoint,
    authority: Option<String>, // Only set for custom name resolution (envoy://)
}

impl UpstreamConfig {
    /// Creates a new upstream configuration for connecting to flagd.
    ///
    /// # Arguments
    /// * `target` - The target address (host:port, URL, or envoy:// URI)
    /// * `is_in_process` - Whether this is for in-process resolver (affects default port)
    /// * `tls` - Whether to use TLS for the connection
    /// * `cert_path` - Optional path to a PEM-encoded CA certificate for custom/self-signed certs
    ///
    /// # TLS Behavior
    /// - If `cert_path` is provided, the certificate is loaded and used as the trusted CA
    /// - If `cert_path` is None and TLS is enabled, system/webpki roots are used
    /// - Self-signed certificates require providing the CA cert via `cert_path`
    pub fn new(
        target: String,
        is_in_process: bool,
        tls: bool,
        cert_path: Option<&str>,
    ) -> Result<Self, FlagdError> {
        debug!(
            "Creating upstream config for target: {}, tls: {}, cert_path: {:?}",
            target, tls, cert_path
        );

        let scheme = if tls { "https" } else { "http" };

        if target.starts_with("http://") || target.starts_with("https://") {
            debug!("Target is already an HTTP(S) endpoint");
            let mut endpoint = Endpoint::from_shared(target.clone())
                .map_err(|e| FlagdError::Config(format!("Invalid endpoint: {}", e)))?;

            // Apply TLS config for https URLs
            if target.starts_with("https://") {
                let tls_config = Self::build_tls_config(cert_path)?;
                endpoint = endpoint
                    .tls_config(tls_config)
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
            let tls_config = Self::build_tls_config(cert_path)?;
            endpoint = endpoint
                .tls_config(tls_config)
                .map_err(|e| FlagdError::Config(format!("TLS config error: {}", e)))?;
        }

        Ok(Self {
            endpoint,
            authority,
        })
    }

    /// Builds a TLS configuration, optionally loading a custom CA certificate.
    ///
    /// # Arguments
    /// * `cert_path` - Optional path to a PEM-encoded CA certificate file
    ///
    /// # Returns
    /// A configured `ClientTlsConfig` with either custom CA or system roots
    fn build_tls_config(cert_path: Option<&str>) -> Result<ClientTlsConfig, FlagdError> {
        let mut tls_config = ClientTlsConfig::new();

        if let Some(path) = cert_path {
            debug!("Loading custom CA certificate from: {}", path);
            let cert_pem = std::fs::read(path).map_err(|e| {
                FlagdError::Config(format!("Failed to read certificate file '{}': {}", path, e))
            })?;
            let ca_cert = Certificate::from_pem(cert_pem);
            tls_config = tls_config.ca_certificate(ca_cert);
        } else {
            tls_config = tls_config.with_enabled_roots();
        }

        Ok(tls_config)
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
        let config = UpstreamConfig::new("localhost:8013".to_string(), false, false, None).unwrap();
        assert!(config.endpoint().uri().to_string().starts_with("http://"));
        assert_eq!(
            config.endpoint().uri().to_string(),
            "http://localhost:8013/"
        );
    }

    #[test]
    fn test_tls_enabled_uses_https_scheme() {
        let config = UpstreamConfig::new("localhost:8013".to_string(), false, true, None).unwrap();
        assert!(config.endpoint().uri().to_string().starts_with("https://"));
        assert_eq!(
            config.endpoint().uri().to_string(),
            "https://localhost:8013/"
        );
    }

    #[test]
    fn test_in_process_default_port_with_tls() {
        let config = UpstreamConfig::new("localhost".to_string(), true, true, None).unwrap();
        assert_eq!(
            config.endpoint().uri().to_string(),
            "https://localhost:8015/"
        );
    }

    #[test]
    fn test_rpc_default_port_with_tls() {
        let config = UpstreamConfig::new("localhost".to_string(), false, true, None).unwrap();
        assert_eq!(
            config.endpoint().uri().to_string(),
            "https://localhost:8013/"
        );
    }

    #[test]
    fn test_explicit_http_url_preserved() {
        let config =
            UpstreamConfig::new("http://example.com:9000".to_string(), false, true, None).unwrap();
        assert_eq!(
            config.endpoint().uri().to_string(),
            "http://example.com:9000/"
        );
    }

    #[test]
    fn test_explicit_https_url_preserved() {
        let config =
            UpstreamConfig::new("https://example.com:9000".to_string(), false, false, None)
                .unwrap();
        assert_eq!(
            config.endpoint().uri().to_string(),
            "https://example.com:9000/"
        );
    }

    #[test]
    fn test_envoy_target_with_tls() {
        let config = UpstreamConfig::new(
            "envoy://localhost:9211/my-service".to_string(),
            false,
            true,
            None,
        )
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
            None,
        )
        .unwrap();
        assert!(config.endpoint().uri().to_string().starts_with("http://"));
        assert_eq!(config.authority(), Some("my-service".to_string()));
    }

    #[test]
    fn test_cert_path_file_not_found() {
        let result = UpstreamConfig::new(
            "localhost:8013".to_string(),
            false,
            true,
            Some("/nonexistent/path/to/cert.pem"),
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to read certificate file"));
    }

    #[test]
    fn test_tls_with_no_cert_path_uses_system_roots() {
        // This test verifies that TLS works without a custom cert (uses system roots)
        let config = UpstreamConfig::new("localhost:8013".to_string(), false, true, None).unwrap();
        assert!(config.endpoint().uri().to_string().starts_with("https://"));
    }
}
