use anyhow::{Context, Result};
use tracing::debug;
use std::str::FromStr;
use tonic::transport::{Endpoint, Uri};

pub struct EnvoyNameResolver;

impl EnvoyNameResolver {
    pub fn new(target: String, is_in_process: bool) -> Result<(Endpoint, Uri)> {
        debug!("Starting name resolution for target: {}", target);
        
        if target.starts_with("http://") {
            debug!("Target is already an HTTP endpoint");
            let uri = Uri::from_str(&target)?;
            let endpoint = Endpoint::from_shared(target)?;
            return Ok((endpoint, uri));
        }
        
        let (endpoint_str, authority) = if target.starts_with("envoy://") {
            let uri = Uri::from_str(&target).context("Failed to parse target URI")?;
            
            let authority = uri.path().trim_start_matches('/').to_string();
            debug!("Extracted authority from path: {}", authority);
            
            if authority.is_empty() {
                return Err(anyhow::anyhow!("Service name (authority) cannot be empty"));
            }
    
            let host = uri.host().unwrap_or("localhost");
            let port = uri.port_u16().unwrap_or(if is_in_process { 8015 } else { 8013 });
            debug!("Using host:port {}:{}", host, port);
            
            (format!("http://{}:{}", host, port), authority)
        } else {
            let parts: Vec<&str> = target.split(':').collect();
            let host = parts.first().unwrap_or(&"localhost").to_string();
            let port = parts
                .get(1)
                .and_then(|p| p.parse().ok())
                .unwrap_or(if is_in_process { 8015 } else { 8013 });
            
            debug!("Using standard resolution with {}:{}", host, port);
            (format!("http://{}:{}", host, port), host)
        };
    
        debug!("Creating endpoint with URI: {}", endpoint_str);
        let endpoint = Endpoint::from_shared(endpoint_str)?;
        
        debug!("Setting authority: {}", authority);
        let origin_uri = Uri::from_str(&format!("http://{}", authority))?;
    
        Ok((endpoint, origin_uri))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envoy_resolution() {
        let (endpoint, uri) = EnvoyNameResolver::new(
            "envoy://localhost:9211/foo.service".to_string(), 
            false
        ).unwrap();

        assert_eq!(endpoint.uri().to_string(), "http://localhost:9211/");
        assert_eq!(uri.to_string(), "http://foo.service/");
    }

    #[test]
    fn test_standard_resolution() {
        let (endpoint, uri) = EnvoyNameResolver::new(
            "localhost:8013".to_string(),
            false
        ).unwrap();

        assert_eq!(endpoint.uri().to_string(), "http://localhost:8013/");
        assert_eq!(uri.to_string(), "http://localhost/");
    }

    #[test]
    fn test_default_ports() {
        let (rpc_endpoint, _) = EnvoyNameResolver::new(
            "localhost".to_string(),
            false
        ).unwrap();
        assert_eq!(rpc_endpoint.uri().to_string(), "http://localhost:8013/");

        let (in_process_endpoint, _) = EnvoyNameResolver::new(
            "localhost".to_string(),
            true
        ).unwrap();
        assert_eq!(in_process_endpoint.uri().to_string(), "http://localhost:8015/");
    }

    #[test]
    fn test_empty_service_name() {
        let result = EnvoyNameResolver::new(
            "envoy://localhost:9211/".to_string(),
            false
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_uri() {
        let result = EnvoyNameResolver::new(
            "envoy://".to_string(),
            false
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_port() {
        let (endpoint, uri) = EnvoyNameResolver::new(
            "envoy://localhost:9999/test.service".to_string(),
            false
        ).unwrap();

        assert_eq!(endpoint.uri().to_string(), "http://localhost:9999/");
        assert_eq!(uri.to_string(), "http://test.service/");
    }
}