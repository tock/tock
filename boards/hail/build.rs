fn main() {
    println!("cargo:rerun-if-changed=layout.ld");
    println!("cargo:rerun-if-changed=chip_layout.ld");
    println!("cargo:rerun-if-changed=../kernel_layout.ld");
    println!(
        "cargo:rustc-link-arg-bins=-L{}",
        std::env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_string())
    );
}
