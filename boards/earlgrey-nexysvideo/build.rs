use std::env;

fn main() {
    println!("cargo:rerun-if-changed=layout.ld");
    println!("cargo:rerun-if-changed=../kernel_layout.ld");

    // CARGO_CFG_TEST is not passed to build.rs
    let test = env::var("CARGO_FEATURE_SEMIHOST_ASM");
    let out_dir = env::var("OUT_DIR").unwrap();

    if test.is_ok() {
        println!("cargo:rustc-link-search={}/../../../", out_dir);
        println!("cargo:rerun-if-changed=libriscv32imc-unknown-none-elf.a");
        println!(
            "cargo:rustc-link-lib=static={}",
            "riscv32imc-unknown-none-elf"
        );
    }
}
