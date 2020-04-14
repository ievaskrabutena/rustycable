fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .format(true)
        .out_dir("./src")
        .compile(&["proto/anycable.proto"], &["proto"])?;

    Ok(())
}
