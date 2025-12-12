fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let mut protos = Vec::new();

    // Check for features via CARGO_FEATURE_* environment variables
    if std::env::var("CARGO_FEATURE_RPC").is_ok() {
        protos.push("schemas/protobuf/flagd/evaluation/v1/evaluation.proto");
    }

    if std::env::var("CARGO_FEATURE_IN_PROCESS").is_ok() {
        protos.push("schemas/protobuf/flagd/sync/v1/sync.proto");
    }

    if !protos.is_empty() {
        tonic_prost_build::configure()
            .build_server(true)
            .out_dir(&out_dir)
            .compile_protos(&protos, &["schemas/protobuf/"])
            .unwrap();
    }
}
