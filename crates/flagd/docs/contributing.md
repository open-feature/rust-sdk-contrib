# Contributing
## Development
After cloning the repository, you first need to update git odules:
```bash
pushd rust-sdk-contrib/crates/flagd
# Update and pull git submodules
git submodule update --init
```
Afterwards, you need to install `protoc`:
- For MacOS: `brew install protobuf`
- For Fedora: `dnf install protobuf protobuf-devel`

Once steps mentioned above are done, `cargo build` will build crate.

## Testing
To run tests across a flagd server, `testcontainers-rs` crate been used to spin up containers. `Docker` is needed to be alled to run E2E tests.
> At the time of writing, `podman` was tested and did not work.

If it is not possible to access docker, unit tests can be run :
```bash
cargo test --lib
```

open-feature-flagd uses `test-log` to have tracing logs table. To have full visibility on test logs, you can use:

```bash
RUST_LOG_SPAN_EVENTS=full RUST_LOG=debug cargo test -- capture
```