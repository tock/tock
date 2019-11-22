#![no_std]
#![feature(in_band_lifetimes)]

pub mod alarm;
pub mod console;
pub mod crc;
pub mod debug_writer;
pub mod isl29035;
pub mod nrf51822;
pub mod process_console;
pub mod rng;
pub mod si7021;
pub mod spi;

/// Same as `static_init!()` but without actually creating the static buffer.
/// The static buffer must be passed in.
#[macro_export]
macro_rules! static_init_half {
    ($B:expr, $T:ty, $e:expr) => {{
        use core::{mem, ptr};
        let tmp: &'static mut $T = mem::transmute($B);
        ptr::write(tmp as *mut $T, $e);
        tmp
    };};
}
