use anyhow::{Context, Result};
use std::str::FromStr;
use tonic::transport::{Endpoint, Uri};
use tracing::debug;

pub struct UpstreamConfig {
    endpoint: Endpoint,
    authority: Uri,
}

impl UpstreamConfig {
    pub fn new(target: String, is_in_process: bool) -> Result<Self> {
        debug!("Creating upstream config for target: {}", target);

        if target.starts_with("http://") {
            debug!("Target is already an HTTP endpoint");
            let uri = Uri::from_str(&target)?;
            let endpoint = Endpoint::from_shared(target)?;
            return Ok(Self {
                endpoint,
                authority: uri
                    .authority()
                    .map(|a| a.as_str())
                    .unwrap_or_default()
                    .parse()?,
            });
        }

        let (endpoint_str, authority) = if target.starts_with("envoy://") {
            let uri = Uri::from_str(&target).context("Failed to parse target URI")?;
            let authority = uri.path().trim_start_matches('/');

            if authority.is_empty() {
                return Err(anyhow::anyhow!("Service name (authority) cannot be empty"));
            }

            let host = uri.host().unwrap_or("localhost");
            let port = uri.port_u16().unwrap_or(9211); // Use Envoy port directly

            (format!("http://{}:{}", host, port), authority.to_string())
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

        let endpoint = Endpoint::from_shared(endpoint_str)?;
        let authority_uri =
            Uri::from_str(authority.as_str()).context("Failed to parse authority")?;

        Ok(Self {
            endpoint,
            authority: authority_uri,
        })
    }

    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    pub fn authority(&self) -> &Uri {
        &self.authority
    }
}
