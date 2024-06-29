fn main() {
    println!("cargo::rerun-if-changed=migrations");
    tonic_build::compile_protos("proto/fricon.proto").expect("Failed to compile proto file");
}
