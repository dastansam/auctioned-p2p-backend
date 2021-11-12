/// Build script that generate types and interfaces from protobuf schema.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/node_rpc.proto")?;
    Ok(())
}