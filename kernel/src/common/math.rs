//! Helper functions for common mathematical operations.

use core::convert::{From, Into};
use core::f32;

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

// f32 log10 function adapted from [micromath](https://github.com/NeoBirth/micromath)
const EXPONENT_MASK: u32 = 0b01111111_10000000_00000000_00000000;
const EXPONENT_BIAS: u32 = 127;

pub fn abs(n: f32) -> f32 {
    f32::from_bits(n.to_bits() & 0x7FFF_FFFF)
}

fn extract_exponent_bits(x: f32) -> u32 {
    (x.to_bits() & EXPONENT_MASK).overflowing_shr(23).0
}

fn extract_exponent_value(x: f32) -> i32 {
    (extract_exponent_bits(x) as i32) - EXPONENT_BIAS as i32
}

fn ln_1to2_series_approximation(x: f32) -> f32 {
    // idea from https://stackoverflow.com/a/44232045/
    // modified to not be restricted to int range and only values of x above 1.0.
    // and got rid of most of the slow conversions,
    // should work for all positive values of x.

    //x may essentially be 1.0 but, as clippy notes, these kinds of
    //floating point comparisons can fail when the bit pattern is not the sames
    if abs(x - 1.0_f32) < f32::EPSILON {
        return 0.0_f32;
    }
    let x_less_than_1: bool = x < 1.0;
    // Note: we could use the fast inverse approximation here found in super::inv::inv_approx, but
    // the precision of such an approximation is assumed not good enough.
    let x_working: f32 = if x_less_than_1 { 1.0 / x } else { x };
    //according to the SO post ln(x) = ln((2^n)*y)= ln(2^n) + ln(y) = ln(2) * n + ln(y)
    //get exponent value
    let base2_exponent: u32 = extract_exponent_value(x_working) as u32;
    let divisor: f32 = f32::from_bits(x_working.to_bits() & EXPONENT_MASK);
    //supposedly normalizing between 1.0 and 2.0
    let x_working: f32 = x_working / divisor;
    //approximate polynomial generated from maple in the post using Remez Algorithm:
    //https://en.wikipedia.org/wiki/Remez_algorithm
    let ln_1to2_polynomial: f32 = -1.741_793_9_f32
        + (2.821_202_6_f32
            + (-1.469_956_8_f32 + (0.447_179_55_f32 - 0.056_570_851_f32 * x_working) * x_working)
                * x_working)
            * x_working;
    // ln(2) * n + ln(y)
    let result: f32 = (base2_exponent as f32) * f32::consts::LN_2 + ln_1to2_polynomial;
    if x_less_than_1 {
        -result
    } else {
        result
    }
}

pub fn log10(x: f32) -> f32 {
    //using change of base log10(x) = ln(x)/ln(10)
    let ln10_recip = f32::consts::LOG10_E;
    let fract_base_ln = ln10_recip;
    let value_ln = ln_1to2_series_approximation(x);
    value_ln * fract_base_ln
}

//-----------------------------------------------------------
