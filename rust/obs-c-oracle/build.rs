use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let libobs = manifest.join("../../libobs").canonicalize().unwrap();

    cc::Build::new()
        .file("oracle/bitstream.c")
        .include(&libobs)
        .std("c11")
        .compile("obs_c_oracle");

    println!("cargo:rerun-if-changed=oracle");
    println!(
        "cargo:rerun-if-changed={}",
        libobs.join("util/bitstream.c").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        libobs.join("util/bitstream.h").display()
    );
}
