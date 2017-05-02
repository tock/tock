
use std::ascii::AsciiExt;
use std::env;
use std::fs::File;
use std::io::Write;
use std::iter;
use std::path::Path;


/* I _wanted_ to implement this as macros that would generate code to include
   in the current compilation unit, but that proved untractable with the
   current Rust macro system (or my comprehension of it). Vexing.

/// Takes an attribute name and value and converts it to a Rust byte
/// array (a [u8; 64])
macro_rules! attribute_to_array {
    ($attr_name:ident, $attr_val:expr) => {
        #[link_section=".kernel_attributes"]
        #[no_mangle]
        static $attr_name: [u8; 64] = [1; 64];
    }
}

*/

pub static KERNEL_ATTRIBUTES_FILE: &'static str = "kernel_attribute_git.rs";


/// Takes an attribute name and value and writes valid Rust to create a kernel
/// attribute
pub fn write_attribute<W: Write>(mut dest: W, name: &str, value: &str) {
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

pub fn kernel_attribute_git<W: Write>(dest: W) {
    //let attr: &str = env::var("TOCK_KERNEL_VERSION").ok().map_or("notgit", |env| { &env });
    let attr = env::var("TOCK_KERNEL_VERSION").unwrap_or("notgit".to_string());
    write_attribute(dest, "git", &attr);
}

pub fn write_standard_attributes_to_build_file() {
    let writer = get_file();
    kernel_attribute_git(writer);
}

#[cfg(test)]
mod tests {
    use super::*;

    //#[test]
    //fn attribute_to_array_macro() {
    //    attribute_to_array!(git, env::var("TOCK_KERNEL_VERSION").unwrap());
    //}

    #[test]
    fn attribute_to_writer() {
        let vec = Vec::new();
        kernel_attribute_git(vec);
    }
}
