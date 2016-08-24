extern crate gcc;

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    gcc::Config::new()
        .compiler(&Path::new("arm-none-eabi-gcc"))
        .flag("-mcpu=cortex-m0")
        .flag("-mthumb")
        .file("src/crt1.c")
        .compile("libcrt1.a");

    let src = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("layout.ld");
    let dst = Path::new(&env::var("OUT_DIR").unwrap()).join("../../layout.ld");
    fs::copy(src, dst).unwrap();
}
