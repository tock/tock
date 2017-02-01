extern crate gcc;

use std::path::Path;

fn main() {
    gcc::Config::new()
        .compiler(&Path::new("arm-none-eabi-gcc"))
        .flag("-mcpu=cortex-m0")
        .flag("-mthumb")
        .file("src/crt1.c")
        .compile("libcrt1.a");
}
