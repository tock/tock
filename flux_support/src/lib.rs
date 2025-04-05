#![no_std]

mod extern_specs;
mod flux_arr;
mod flux_pair;
mod flux_ptr;
mod flux_range;
mod flux_register_interface;
mod math;
use core::panic;
pub use flux_arr::*;
pub use flux_pair::*;
pub use flux_ptr::*;
pub use flux_range::*;
pub use flux_register_interface::*;
pub use math::*;

#[allow(dead_code)]
#[flux_rs::sig(fn(x: bool[true]))]
pub const fn assert(_x: bool) {}

#[flux_rs::sig(fn(b:bool) ensures b)]
pub const fn assume(b: bool) {
    if !b {
        panic!("assume fails")
    }
}
