#![no_std]
#![no_main]
#![feature(used, panic_implementation)]
//! CCFG - Customer Configuration
//!
//! For details see p. 710 in the cc2650 technical reference manual.
//!
//! Currently setup to use the default settings.

#[used]
#[link_section = ".init"]
pub static CCFG_CONF: [u32; 22] = [
    0x01800000,
    0xFF820010,
    0x0058FFFD,
    0xF3FFFF3A,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0x00FFFFFF,
    0xFFFFFFFF,
    0xFFFFFF00,
    0xFFC5C5C5,
    0xFFC5C5C5,
    0x00000000,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
];

#[panic_implementation]
#[no_mangle]
pub unsafe extern "C" fn panic_fmt(_pi: &core::panic::PanicInfo) -> ! {
    loop {}
}
