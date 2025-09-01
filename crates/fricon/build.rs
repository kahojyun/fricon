fn main() {
    println!("cargo::rerun-if-changed=migrations");
    println!("cargo::rerun-if-changed=proto");
    let protos = [
        "proto/fricon/v1alpha/dataset.proto",
        "proto/fricon/v1alpha/fricon.proto",
    ];
    tonic_prost_build::configure()
        .bytes(".")
        .compile_protos(&protos, &["proto"])
        .expect("failed to compile protobuf files");
}
