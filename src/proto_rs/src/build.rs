extern crate prost_build;

fn main() {
    println!("cargo:rerun-if-changed=../../src/proto/schema.proto");
    prost_build::Config::new()
        .out_dir("src/")
        .compile_protos(&["../../src/proto/schema.proto"], &["../../src/"])
        .expect("Failed to compile protos");
}
