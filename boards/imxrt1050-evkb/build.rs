fn main() {
    println!("cargo:rerun-if-changed=layout.ld");
    println!("cargo:rerun-if-changed=chip_layout.ld");
    println!("cargo:rerun-if-changed=../kernel_layout.ld");

    // rebuild if `asm.s` changed
    println!("cargo:rerun-if-changed=hdr.s"); // <- NEW!
}
