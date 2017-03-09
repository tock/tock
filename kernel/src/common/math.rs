use core::intrinsics as int;

// wrappers for unsafe core::intrinsics math functions
//  core::intrinsics functions can be found at
//      https://doc.rust-lang.org/core/intrinsics/
//  add additional wrappers as needed

pub fn powf32(base: f32, exponent: f32) -> f32 {
    unsafe { int::powf32(base, exponent) }
}

pub fn powif32(base: f32, exponent: i32) -> f32 {
    unsafe { int::powif32(base, exponent) }
}

pub fn sqrtf32(num: f32) -> f32 {
    unsafe { int::sqrtf32(num) }
}


// errno from stdlib for use in Rust

extern "C" {
    fn __errno() -> &mut i32;
}

// return errno value and zero it out
pub fn get_errno() -> i32 {
    unsafe {
        let errnoaddr = __errno();
        let ret = *errnoaddr;
        *errnoaddr = 0;
        ret
    }
}


// other math functions that are generally useful

// get closest power of two greater than the given number
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

// get log base 2 of a number
// Note: this is the floor of the result. Also, an input of 0 results in an
// output of 0
pub fn log_base_two(num: u32) -> u32 {
    if num == 0 {
        0
    } else {
        31 - num.leading_zeros()
    }
}
