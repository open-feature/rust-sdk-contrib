//! Common utilities for flagd examples

use std::time::Duration;
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{ContainerPort, Mount, WaitFor, logs::LogSource, wait::LogWaitStrategy},
    runners::AsyncRunner,
};

pub const FLAGD_SYNC_PORT: u16 = 8015;

/// Start a flagd container configured for in-process sync (port 8015)
pub async fn start_flagd_sync(
    flags_path: &str,
    flags_file: &str,
) -> Result<(ContainerAsync<GenericImage>, u16), Box<dyn std::error::Error>> {
    // Use fsnotify provider for faster file change detection
    let sources_config = format!(
        r#"[{{"uri":"/flags/{}","provider":"fsnotify"}}]"#,
        flags_file
    );

    let container = GenericImage::new("ghcr.io/open-feature/flagd", "latest")
        .with_exposed_port(ContainerPort::Tcp(FLAGD_SYNC_PORT))
        .with_wait_for(WaitFor::Log(LogWaitStrategy::new(
            LogSource::StdErr,
            "Flag IResolver listening at",
        )))
        .with_mount(Mount::bind_mount(flags_path.to_string(), "/flags"))
        .with_cmd(["start", "--sources", &sources_config])
        .start()
        .await?;

    let sync_port = container
        .get_host_port_ipv4(ContainerPort::Tcp(FLAGD_SYNC_PORT))
        .await?;

    // Give flagd a moment to fully initialize
    tokio::time::sleep(Duration::from_millis(500)).await;

    Ok((container, sync_port))
}
