use std::{error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=migrations");
    println!("cargo::rerun-if-changed=proto");
    let mut protos = vec![];
    for p in fs::read_dir("proto/fricon/v1")? {
        protos.push(p?.path());
    }
    tonic_prost_build::configure()
        .bytes(".")
        .compile_protos(&protos, &[PathBuf::from("proto")])?;
    Ok(())
}
