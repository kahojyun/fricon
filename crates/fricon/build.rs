use std::{error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=../../migrations");
    let mut protos = vec![];
    for p in fs::read_dir("../../proto/fricon/v1")? {
        protos.push(p?.path());
    }
    tonic_build::configure()
        .bytes(["."])
        .compile_protos(&protos, &["../../proto"])?;
    Ok(())
}
