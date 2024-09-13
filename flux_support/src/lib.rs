mod flux_register_interface;
mod flux_ptr;
mod flux_range;
mod extern_specs;
mod math;
pub use flux_register_interface::*;
pub use flux_ptr::*;
pub use flux_range::*;
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
