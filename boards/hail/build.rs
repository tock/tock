#[path = "../common_build.rs"]
mod common_build;

fn main() {
    println!("cargo:rerun-if-changed=layout.ld");
    println!("cargo:rerun-if-changed=chip_layout.ld");
    println!("cargo:rerun-if-changed=../kernel_layout.ld");

    common_build::tock_default_linker_args();
}
