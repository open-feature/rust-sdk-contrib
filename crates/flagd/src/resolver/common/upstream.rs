use crate::error::FlagdError;
use std::str::FromStr;
use tonic::transport::{Endpoint, Uri};
use tracing::debug;

pub struct UpstreamConfig {
    endpoint: Endpoint,
    authority: Option<String>, // Only set for custom name resolution (envoy://)
}

impl UpstreamConfig {
    pub fn new(target: String, is_in_process: bool) -> Result<Self, FlagdError> {
        debug!("Creating upstream config for target: {}", target);

        if target.starts_with("http://") {
            debug!("Target is already an HTTP endpoint");
            let endpoint = Endpoint::from_shared(target)
                .map_err(|e| FlagdError::Config(format!("Invalid endpoint: {}", e)))?;
            return Ok(Self {
                endpoint,
                authority: None, // Standard HTTP doesn't need custom authority
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

            (format!("http://{}:{}", host, port), Some(authority.to_string()))
        } else {
            let parts: Vec<&str> = target.split(':').collect();
            let host = parts.first().unwrap_or(&"localhost").to_string();
            let port = parts
                .get(1)
                .and_then(|p| p.parse().ok())
                .unwrap_or(if is_in_process { 8015 } else { 8013 });

            debug!("Using standard resolution with {}:{}", host, port);
            (format!("http://{}:{}", host, port), None) // Standard resolution doesn't need custom authority
        };

        let endpoint = Endpoint::from_shared(endpoint_str)
            .map_err(|e| FlagdError::Config(format!("Invalid endpoint: {}", e)))?;

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
