use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let src = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("layout.ld");
    let dst = Path::new(&env::var("OUT_DIR").unwrap()).join("../../layout.ld");
    fs::copy(src, dst).unwrap();
}
