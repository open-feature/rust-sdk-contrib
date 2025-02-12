use std::borrow::Cow;
use std::fs::OpenOptions;
use std::io::Write;
use tempfile::NamedTempFile;
use testcontainers::core::logs::LogSource;
use testcontainers::core::wait::LogWaitStrategy;
use testcontainers::core::{ContainerPort, Image, Mount, WaitFor};

use std::sync::Arc;

pub const FLAGD_CONFIG: &str = r#"{
    "$schema": "https://flagd.dev/schema/v0/flags.json",
    "flags": {
        "bool-flag": {
            "state": "ENABLED",
            "variants": {
                "on": true,
                "off": false
            },
            "defaultVariant": "on"
        },
        "string-flag": {
            "state": "ENABLED",
            "variants": {
                "greeting": "hello",
                "farewell": "goodbye"
            },
            "defaultVariant": "greeting"
        },
        "int-flag": {
            "state": "ENABLED",
            "variants": {
                "low": 42,
                "high": 100
            },
            "defaultVariant": "low"
        },
        "float-flag": {
            "state": "ENABLED",
            "variants": {
                "pi": 3.14,
                "e": 2.718
            },
            "defaultVariant": "pi"
        },
        "struct-flag": {
            "state": "ENABLED",
            "variants": {
                "object": {
                    "key": "value",
                    "number": 42
                }
            },
            "defaultVariant": "object"
        }
    }
}"#;

pub const ENVOY_CONFIG: &str = r#"
static_resources:
  listeners:
  - name: listener_0
    address:
      socket_address:
        address: 0.0.0.0
        port_value: 9211
    filter_chains:
    - filters:
      - name: envoy.filters.network.http_connection_manager
        typed_config:
          "@type": type.googleapis.com/envoy.extensions.filters.network.http_connection_manager.v3.HttpConnectionManager
          stat_prefix: ingress_http
          http2_protocol_options: {}
          route_config:
            name: local_route
            virtual_hosts:
            - name: local_service
              domains: ["*"]
              routes:
              - match:
                  prefix: "/"
                route:
                  cluster: flagd_service
          http_filters:
          - name: envoy.filters.http.router
            typed_config:
              "@type": type.googleapis.com/envoy.extensions.filters.http.router.v3.Router
  clusters:
  - name: flagd_service
    connect_timeout: 1s
    type: STRICT_DNS
    lb_policy: ROUND_ROBIN
    http2_protocol_options: {}
    load_assignment:
      cluster_name: flagd_service
      endpoints:
      - lb_endpoints:
        - endpoint:
            address:
              socket_address:
                address: flagd_host
                port_value: 8015
"#;

pub const FLAGD_PORT: u16 = 8013;
pub const FLAGD_SYNC_PORT: u16 = 8015;
pub const FLAGD_OFREP_PORT: u16 = 8016;

#[derive(Debug, Clone)]
pub struct ConfigFile {
    #[allow(dead_code)]
    content: String,
    temp_file: Arc<NamedTempFile>,
}

impl ConfigFile {
    pub fn new(content: String) -> Self {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write(content.as_bytes()).unwrap();

        Self {
            content,
            temp_file: Arc::new(temp_file),
        }
    }

    pub fn update(&self, new_content: String) {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.temp_file.path())
            .unwrap();

        file.write_all(new_content.as_bytes()).unwrap();
        file.sync_all().unwrap();
    }

    pub fn path(&self) -> String {
        self.temp_file.path().to_str().unwrap().to_string()
    }
}

#[derive(Debug)]
pub struct Flagd {
    config_file: Arc<ConfigFile>,
    exposed_ports: Vec<ContainerPort>,
    mount: Mount,
    cmd: Vec<String>,
}

impl Clone for Flagd {
    fn clone(&self) -> Self {
        Self {
            config_file: Arc::clone(&self.config_file),
            exposed_ports: self.exposed_ports.clone(),
            mount: self.mount.clone(),
            cmd: self.cmd.clone(),
        }
    }
}

impl Flagd {
    #![allow(dead_code)]
    pub fn new() -> Self {
        let config_file = Arc::new(ConfigFile::new(FLAGD_CONFIG.to_string()));
        let mount: Mount = Mount::bind_mount(config_file.path(), "/etc/flagd/config.json".to_string());

        Self {
            config_file,
            exposed_ports: vec![
                ContainerPort::Tcp(FLAGD_PORT),
                ContainerPort::Tcp(FLAGD_SYNC_PORT),
                ContainerPort::Tcp(FLAGD_OFREP_PORT),
            ],
            mount,
            cmd: vec![
                "start".to_string(),
                "--port".to_string(),
                "8013".to_string(),
                "--uri".to_string(),
                "file:/etc/flagd/config.json".to_string(),
            ],
        }
    }

    pub fn with_config(mut self, config: impl Into<String>) -> Self {
        self.config_file = Arc::new(ConfigFile::new(config.into()));
        self.mount = Mount::bind_mount(
            self.config_file.path(),
            "/etc/flagd/config.json".to_string(),
        );
        self
    }

    pub fn with_sources(mut self, sources_config: String) -> Self {
        self.cmd = vec![
            "start".to_string(),
            "--port".to_string(),
            "8013".to_string(),
            format!("--sources={}", sources_config),
        ];

        self
    }

    #[allow(dead_code)]
    pub fn update_config(&self, new_config: String) {
        self.config_file.update(new_config);
    }
}

impl Image for Flagd {
    fn name(&self) -> &str {
        "ghcr.io/open-feature/flagd"
    }

    fn tag(&self) -> &str {
        "latest"
    }

    fn cmd(&self) -> impl IntoIterator<Item = impl Into<Cow<'_, str>>> {
        self.cmd.clone()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![
            WaitFor::Log(LogWaitStrategy::new(
                LogSource::StdErr,
                "Flag IResolver listening at [::]:8013",
            )),
            WaitFor::Log(LogWaitStrategy::new(
                LogSource::StdErr,
                "ofrep service listening at 8016",
            )),
            WaitFor::millis(100),
        ]
    }

    fn expose_ports(&self) -> &[ContainerPort] {
        &self.exposed_ports
    }

    fn mounts(&self) -> impl IntoIterator<Item = &Mount> {
        std::iter::once(&self.mount)
    }
}

#[derive(Debug)]
pub struct Envoy {
    config_file: Arc<ConfigFile>,
    exposed_ports: Vec<ContainerPort>,
    mount: Mount,
    cmd: Vec<String>,
}

impl Clone for Envoy {
    fn clone(&self) -> Self {
        Self {
            config_file: Arc::clone(&self.config_file),
            exposed_ports: self.exposed_ports.clone(),
            mount: self.mount.clone(),
            cmd: self.cmd.clone(),
        }
    }
}

impl Envoy {
    pub fn new() -> Self {
        let config_file = Arc::new(ConfigFile::new(ENVOY_CONFIG.to_string()));
        let mount = Mount::bind_mount(config_file.path(), "/etc/envoy/envoy.yaml".to_string());

        Self {
            config_file,
            exposed_ports: vec![ContainerPort::Tcp(9211)],
            mount,
            cmd: vec![
                "-c".to_string(),
                "/etc/envoy/envoy.yaml".to_string(),
            ],
        }
    }

    pub fn with_config(mut self, config: impl Into<String>) -> Self {
        self.config_file = Arc::new(ConfigFile::new(config.into()));
        self.mount = Mount::bind_mount(
            self.config_file.path(),
            "/etc/envoy/envoy.yaml".to_string(),
        );
        self
    }
}

impl Image for Envoy {
    fn name(&self) -> &str {
        "envoyproxy/envoy"
    }

    fn tag(&self) -> &str {
        "v1.28-latest"
    }

    fn cmd(&self) -> impl IntoIterator<Item = impl Into<Cow<'_, str>>> {
        self.cmd.clone()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![
            WaitFor::Log(LogWaitStrategy::new(
                LogSource::StdErr,
                "all dependencies initialized. starting workers"
            )),
            WaitFor::millis(100),
        ]
    }

    fn expose_ports(&self) -> &[ContainerPort] {
        &self.exposed_ports
    }

    fn mounts(&self) -> impl IntoIterator<Item = &Mount> {
        std::iter::once(&self.mount)
    }
}
