//! In-process evaluation example using testcontainers with flagd
//!
//! This example demonstrates in-process flag evaluation by periodically
//! evaluating a boolean flag. Edit `examples/flags/basic-flags.json` while
//! running to see live flag updates.
//!
//! Run with: cargo run --example in_process --all-features
//!
//! Then edit basic-flags.json and change "defaultVariant": "false" to "true"
//! to see the flag value change in real-time.

mod common;

use common::start_flagd_sync;
use open_feature::EvaluationContext;
use open_feature::provider::FeatureProvider;
use open_feature_flagd::{FlagdOptions, FlagdProvider, ResolverType};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let flags_path = format!("{}/examples/flags", manifest_dir);

    println!("Starting flagd container...");
    let (_container, sync_port) = start_flagd_sync(&flags_path, "basic-flags.json").await?;
    println!("flagd sync service available on port {}", sync_port);

    // Configure the flagd provider for in-process evaluation
    let provider = FlagdProvider::new(FlagdOptions {
        host: "localhost".to_string(),
        port: sync_port,
        resolver_type: ResolverType::InProcess,
        ..Default::default()
    })
    .await
    .expect("Failed to create provider");

    let ctx = EvaluationContext::default();

    println!("\nEvaluating 'basic-boolean' flag every 2 seconds...");
    println!("Edit examples/flags/basic-flags.json to change the flag value.");
    println!("Press Ctrl+C to stop.\n");

    loop {
        let result = provider
            .resolve_bool_value("basic-boolean", &ctx)
            .await
            .expect("Failed to resolve flag");

        println!("basic-boolean = {}", result.value);

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
