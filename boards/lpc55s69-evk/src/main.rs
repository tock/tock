#![no_std]
#![no_main]
//#![deny(missing_docs)]

mod io;

#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x2000] = [0; 0x2000];

#[allow(unused)]
use lpc55s6x::BASE_VECTORS;

#[no_mangle]
pub unsafe fn main() {
    loop {
        cortexm33::support::nop();
    }
}
