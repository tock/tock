extern crate regex;

use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::iter;
use std::path::Path;
use regex::Regex;

// This is the name of the file that will get generated with the static attributes in them.
pub static KERNEL_ATTRIBUTES_FILE: &'static str = "kernel_attribute_git.rs";

// This is the name of the file where the board-specific linker addresses are stored.
// This is used to determine where applications will be placed in flash.
pub static CHIP_LAYOUT_FILE: &'static str = "chip_layout.ld";

/// Takes an attribute name and value and writes valid Rust to create a kernel
/// attribute
pub fn write_attribute<W: Write>(dest: &mut W, name: &str, value: &str) {
    let _ = write!(dest,
                   "
#[link_section=\".kernel_attribute.{}\"]
#[no_mangle]
pub static KERNEL_ATTRIBUTE_{}: [u8; 64] = [
    ",
                   name,
                   name.to_ascii_uppercase());

    // Write up to 8 bytes of name ; zero-pad up to 8 bytes
    for byte in name.bytes().chain(iter::repeat(0)).take(8) {
        let _ = write!(dest, "{:#x}, ", byte);
    }

    // attribute length
    let _ = write!(dest, "{:#x}, ", value.len());

    // Write up to 55 bytes of value ; zero-pad up to 55 bytes
    for byte in value.bytes().chain(iter::repeat(0)).take(55) {
        let _ = write!(dest, "{:#x}, ", byte);
    }

    // And finish the array
    let _ = write!(dest, " ]; ");
}

pub fn get_file() -> File {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(KERNEL_ATTRIBUTES_FILE);
    let f = File::create(&dest_path).unwrap();
    f
}

pub fn kernel_attribute_git<W: Write>(dest: &mut W) {
    let attr = env::var("TOCK_KERNEL_VERSION").unwrap_or("notgit".to_string());
    write_attribute(dest, "git", &attr);
}

pub fn kernel_attribute_appaddr<W: Write>(dest: &mut W) {
    let src_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let chip_layout_path = Path::new(&src_dir).join(CHIP_LAYOUT_FILE);
    match File::open(&chip_layout_path) {
        Ok(mut f) => {
            let mut contents = String::new();
            f.read_to_string(&mut contents).expect("Unable to read the file");

            // Search the mini linker file for PROG_ORIGIN as use the address of that
            // variable.
            let re = Regex::new(r"PROG_ORIGIN[\s=]*([0-9x]+);").unwrap();
            let caps = re.captures(contents.as_str()).unwrap();
            write_attribute(dest, "appaddr", caps.get(1).unwrap().as_str());
        }
        Err(_) => {}
    }
}

pub fn write_standard_attributes_to_build_file() {
    let mut writer = get_file();
    kernel_attribute_git(&mut writer);
    kernel_attribute_appaddr(&mut writer);
}
