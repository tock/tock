use cc::Build;

fn main() {
    println!("cargo:rerun-if-changed=layout.ld");
    println!("cargo:rerun-if-changed=chip_layout.ld");
    println!("cargo:rerun-if-changed=../kernel_layout.ld");

    // println!("AIIIIIICICICIICICIICCIICIICICIICICICICIICICIIICICI");
    // println!("AIIIIIICICICIICICIICCIICIICICIICICICICIICICIIICICI");
    // println!("AIIIIIICICICIICICIICCIICIICICIICICICICIICICIIICICI");
    // println!("AIIIIIICICICIICICIICCIICIICICIICICICICIICICIIICICI");
    
    // assemble the `asm.s` file
    Build::new().file("hdr.s").compile("asm"); // <- NEW!

    // rebuild if `asm.s` changed
    println!("cargo:rerun-if-changed=hdr.s"); // <- NEW!
}
