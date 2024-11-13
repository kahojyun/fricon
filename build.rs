use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=migrations");
    tonic_build::configure()
        .build_client(false)
        .compile_protos(&["proto/fricon/v1/fricon.proto"], &["proto"])?;
    Ok(())
}
