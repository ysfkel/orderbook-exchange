fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        // .out_dir("src/protos")
        .compile_protos(&["proto/markets.proto"], &["proto"])?;

    Ok(())
}
