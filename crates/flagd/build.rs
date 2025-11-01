fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    tonic_prost_build::configure()
        .build_server(true)
        .out_dir(&out_dir)
        .compile_protos(
            &[
                "schemas/protobuf/flagd/evaluation/v1/evaluation.proto",
                "schemas/protobuf/flagd/sync/v1/sync.proto",
            ],
            &["schemas/protobuf/"],
        )
        .unwrap();
}
