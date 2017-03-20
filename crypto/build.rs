extern crate gcc;

use std::env;
use std::path::Path;

fn main() {
    gcc::Config::new()
        .compiler(&Path::new("arm-none-eabi-gcc"))
        .flag("-mcpu=cortex-m4")
        .flag("-mthumb")
        .file("src/mul.s")
        .file("src/sqr.s")
        .file("src/cortex_m0_reduce25519.s")
        .file("src/cortex_m0_mpy121666.s")
        .compile("libcrt1.a");
}
