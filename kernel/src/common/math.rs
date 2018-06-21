//! Helper functions for common mathematical operations.

use core::convert::{From, Into};
use core::intrinsics as int;

// wrappers for unsafe core::intrinsics math functions
//  core::intrinsics functions can be found at
//      https://doc.rust-lang.org/core/intrinsics/
//  add additional wrappers as needed

/// Provide `sqrtf32` with the unsafe hidden.
pub fn sqrtf32(num: f32) -> f32 {
    unsafe { int::sqrtf32(num) }
}

// errno from stdlib for use in Rust

extern "C" {
    fn __errno() -> &'static mut i32;
}

/// Return errno value and zero it out.
pub fn get_errno() -> i32 {
    unsafe {
        let errnoaddr = __errno();
        let ret = *errnoaddr;
        *errnoaddr = 0;
        ret
    }
}

/// Get closest power of two greater than the given number.
pub fn closest_power_of_two(mut num: u32) -> u32 {
    num -= 1;
    num |= num >> 1;
    num |= num >> 2;
    num |= num >> 4;
    num |= num >> 8;
    num |= num >> 16;
    num += 1;
    num
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct PowerOfTwo(u32);

/// Represents an integral power-of-two as an exponent
impl PowerOfTwo {
    /// Returns the base-2 exponent as a numeric type
    pub fn exp<R>(self) -> R
    where
        R: From<u32>,
    {
        From::from(self.0)
    }

    /// Converts a number two the nearest `PowerOfTwo` less-than-or-equal to it.
    pub fn floor<F: Into<u32>>(f: F) -> PowerOfTwo {
        PowerOfTwo(log_base_two(f.into()))
    }

    /// Converts a number two the nearest `PowerOfTwo` greater-than-or-equal to
    /// it.
    pub fn ceiling<F: Into<u32>>(f: F) -> PowerOfTwo {
        PowerOfTwo(log_base_two(closest_power_of_two(f.into())))
    }

    /// Creates a new `PowerOfTwo` representing the number zero.
    pub fn zero() -> PowerOfTwo {
        PowerOfTwo(0)
    }

    /// Converts a `PowerOfTwo` to a number.
    pub fn as_num<F: From<u32>>(self) -> F {
        (1 << self.0).into()
    }
}

/// Get log base 2 of a number.
/// Note: this is the floor of the result. Also, an input of 0 results in an
/// output of 0
pub fn log_base_two(num: u32) -> u32 {
    if num == 0 {
        0
    } else {
        31 - num.leading_zeros()
    }
}

/// Log base 2 of 64 bit unsigned integers.
pub fn log_base_two_u64(num: u64) -> u32 {
    if num == 0 {
        0
    } else {
        63 - num.leading_zeros()
    }
}
