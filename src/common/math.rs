// wrappers for unsafe core::intrinsics math functions
//  core::intrinsics functions can be found at 
//      https://doc.rust-lang.org/core/intrinsics/
//  add additional wrappers as needed
use core::intrinsics as int;

extern {
    fn __errno() -> &mut i32;
}

pub fn powf32(base: f32, exponent: f32) -> f32 {
    unsafe { int::powf32(base, exponent) }
}

pub fn powif32(base: f32, exponent: i32) -> f32 {
    unsafe { int::powif32(base, exponent) }
}

pub fn sqrtf32(num: f32) -> f32 {
    unsafe { int::sqrtf32(num) }
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

