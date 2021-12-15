/// Build script that generate types and interfaces from protobuf schema.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .compile(&["proto/node_rpc.proto"], &["proto"])
        .unwrap();
    Ok(())
}