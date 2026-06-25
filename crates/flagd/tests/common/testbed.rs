use std::borrow::Cow;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use open_feature_flagd::ResolverType;
use testcontainers::core::logs::LogSource;
use testcontainers::core::wait::LogWaitStrategy;
use testcontainers::core::{ContainerPort, Image, Mount, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, ImageExt};

use crate::common::{ConfigFile, ENVOY_CONFIG, ENVOY_PORT, Envoy};

const RPC_PORT: u16 = 8013;
const SYNC_PORT: u16 = 8015;
const OFREP_PORT: u16 = 8016;
const TESTBED_CONTEXT_VALUE: &str = r#"{"injectedmetadata":"set"}"#;
const TARGET_URI_NETWORK: &str = "flagd-testbed-target-uri";

static CONTAINER_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub(crate) struct RunningTestbed {
    _flagd: ContainerAsync<TestbedFlagd>,
    _envoy: Option<ContainerAsync<Envoy>>,
}

pub(crate) struct ProviderEndpoint {
    pub(crate) port: u16,
    pub(crate) target_uri: Option<String>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum SyncMetadata {
    Enabled,
    Disabled,
}

impl SyncMetadata {
    fn is_disabled(self) -> bool {
        matches!(self, Self::Disabled)
    }
}

#[derive(Debug, Clone)]
struct TestbedFlagd {
    _flags_file: Arc<ConfigFile>,
    _config_file: Arc<ConfigFile>,
    mounts: Vec<Mount>,
    cmd: Vec<String>,
    exposed_ports: Vec<ContainerPort>,
}

impl TestbedFlagd {
    fn new(sync_metadata: SyncMetadata) -> Self {
        let flags_file = Arc::new(ConfigFile::new(testbed_flags()));
        let config_file = Arc::new(ConfigFile::new(testbed_flagd_config(sync_metadata)));
        let mounts = vec![
            Mount::bind_mount(flags_file.path(), "/etc/flagd/flags.json".to_string()),
            Mount::bind_mount(config_file.path(), "/etc/flagd/config.json".to_string()),
        ];

        Self {
            _flags_file: flags_file,
            _config_file: config_file,
            mounts,
            cmd: vec![
                "start".to_string(),
                "--config".to_string(),
                "/etc/flagd/config.json".to_string(),
            ],
            exposed_ports: vec![
                ContainerPort::Tcp(RPC_PORT),
                ContainerPort::Tcp(SYNC_PORT),
                ContainerPort::Tcp(OFREP_PORT),
            ],
        }
    }
}

impl Image for TestbedFlagd {
    fn name(&self) -> &str {
        "ghcr.io/open-feature/flagd"
    }

    fn tag(&self) -> &str {
        "v0.16.0"
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
        self.mounts.iter()
    }
}

pub(crate) async fn start_testbed(
    requested_target_uri: Option<&str>,
    resolver_type: &ResolverType,
    sync_metadata: SyncMetadata,
) -> (RunningTestbed, ProviderEndpoint) {
    let flagd_name = requested_target_uri.map(|_| next_container_name("flagd_testbed_target_uri"));
    let flagd = start_flagd(sync_metadata, flagd_name.as_deref()).await;

    let (envoy, endpoint) = if let Some(target_uri) = requested_target_uri {
        let flagd_port = resolver_container_port(resolver_type);
        let (envoy, envoy_port) =
            start_envoy_for_target_uri(target_uri, flagd_port, flagd_name.as_deref().unwrap())
                .await;

        (
            Some(envoy),
            ProviderEndpoint {
                port: envoy_port,
                target_uri: Some(target_uri.replace("<port>", &envoy_port.to_string())),
            },
        )
    } else {
        (
            None,
            ProviderEndpoint {
                port: direct_provider_port(&flagd, resolver_type).await,
                target_uri: None,
            },
        )
    };

    (
        RunningTestbed {
            _flagd: flagd,
            _envoy: envoy,
        },
        endpoint,
    )
}

async fn start_flagd(
    sync_metadata: SyncMetadata,
    container_name: Option<&str>,
) -> ContainerAsync<TestbedFlagd> {
    let image = TestbedFlagd::new(sync_metadata);

    if let Some(container_name) = container_name {
        image
            .with_network(TARGET_URI_NETWORK.to_string())
            .with_container_name(container_name)
            .start()
            .await
            .expect("failed to start flagd")
    } else {
        image.start().await.expect("failed to start flagd")
    }
}

async fn direct_provider_port(
    flagd: &ContainerAsync<TestbedFlagd>,
    resolver_type: &ResolverType,
) -> u16 {
    flagd
        .get_host_port_ipv4(resolver_container_port(resolver_type))
        .await
        .unwrap()
}

fn resolver_container_port(resolver_type: &ResolverType) -> u16 {
    match resolver_type {
        ResolverType::Rpc => RPC_PORT,
        ResolverType::InProcess => SYNC_PORT,
        ResolverType::Rest => panic!("REST is not used by this runner"),
        ResolverType::File => panic!("File is not used by this runner"),
    }
}

async fn start_envoy_for_target_uri(
    target_uri: &str,
    flagd_port: u16,
    flagd_name: &str,
) -> (ContainerAsync<Envoy>, u16) {
    let authority = target_uri
        .rsplit_once('/')
        .map(|(_, authority)| authority)
        .expect("targetUri must contain an authority path");
    let envoy = Envoy::new()
        .with_config(
            ENVOY_CONFIG
                .replace("b-features-api.service", authority)
                .replace("address: flagd", &format!("address: {flagd_name}"))
                .replace("port_value: 8015", &format!("port_value: {flagd_port}")),
        )
        .with_network(TARGET_URI_NETWORK.to_string())
        .with_container_name(next_container_name("envoy_testbed_target_uri"))
        .start()
        .await
        .expect("failed to start envoy");
    let envoy_port = envoy.get_host_port_ipv4(ENVOY_PORT).await.unwrap();
    (envoy, envoy_port)
}

fn testbed_flagd_config(sync_metadata: SyncMetadata) -> String {
    let mut config = serde_json::json!({
        "sources": [
            {
                "uri": "/etc/flagd/flags.json",
                "provider": "file"
            }
        ],
        "context-value": serde_json::from_str::<serde_json::Value>(TESTBED_CONTEXT_VALUE).unwrap()
    });

    if sync_metadata.is_disabled() {
        config["disable-sync-metadata"] = serde_json::Value::Bool(true);
    }

    serde_json::to_string(&config).unwrap()
}

fn testbed_flags() -> String {
    let mut testing_flags: serde_json::Value =
        serde_json::from_str(include_str!("../../flagd-testbed/flags/testing-flags.json")).unwrap();
    let metadata_flags: serde_json::Value = serde_json::from_str(include_str!(
        "../../flagd-testbed/flags/metadata-flags.json"
    ))
    .unwrap();

    let testing_flags_map = testing_flags["flags"].as_object_mut().unwrap();
    testing_flags_map.extend(metadata_flags["flags"].as_object().unwrap().clone());

    serde_json::to_string(&testing_flags).unwrap()
}

fn next_container_name(prefix: &str) -> String {
    let id = CONTAINER_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}_{id}")
}
