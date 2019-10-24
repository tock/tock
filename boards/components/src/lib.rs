#![no_std]
#![feature(in_band_lifetimes)]

#[macro_use]
pub mod isl29035;
pub mod rng;
#[macro_use]
pub mod crc;
#[macro_use]
pub mod alarm;
pub mod console;
pub mod nrf51822;
pub mod process_console;

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
