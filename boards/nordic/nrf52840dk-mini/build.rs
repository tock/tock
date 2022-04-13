fn main() {
    println!("cargo:rerun-if-changed=layout.ld");
    println!("cargo:rerun-if-changed=../../kernel_layout.ld");
    println!("cargo:rerun-if-changed=../nrf52840_chip_layout.ld");
}
