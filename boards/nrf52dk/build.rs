extern crate tock_kernel_attributes;

fn main() {
    println!("cargo:rerun-if-changed=layout.ld");
    println!("cargo:rerun-if-changed=chip_layout.ld");
    println!("cargo:rerun-if-changed=../kernel_layout.ld");

    tock_kernel_attributes::write_standard_attributes_to_build_file();
}
