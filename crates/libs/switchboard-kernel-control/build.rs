fn main() {
    if let Err(err) = fallible_build() {
        eprintln!("Error during protobuf compilation: {}", err);
        std::process::exit(1);
    }
}

fn fallible_build() -> Result<(), Box<dyn std::error::Error>> {
    let build_client = std::env::var("CARGO_FEATURE_CLIENT").is_ok();
    let build_server = std::env::var("CARGO_FEATURE_SERVER").is_ok();
    tonic_prost_build::configure()
        .build_server(build_server)
        .build_client(build_client)
        .compile_protos(&["protocol/kernel.proto"], &["protocol"])?;
    tonic_prost_build::compile_protos("protocol/kernel.proto")?;
    Ok(())
}
