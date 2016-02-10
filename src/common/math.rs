// wrappers for unsafe core::intrinsics math functions
//  core::intrinsics functions can be found at 
//      https://doc.rust-lang.org/core/intrinsics/
//  add additional wrappers as needed

extern {
    static mut errno: i32;
}

use core::intrinsics::powf32;
pub fn pow_f32(base: f32, exponent: f32) -> f32 {
    unsafe { powf32(base, exponent) }
}

use core::intrinsics::powif32;
pub fn powi_f32(base: f32, exponent: i32) -> f32 {
    unsafe { powif32(base, exponent) }
}

use core::intrinsics::sqrtf32;
pub fn sqrt(num: f32) -> f32 {
    unsafe { sqrtf32(num) }
}

// return errno value and zero it out
pub fn get_errno() -> i32 {
    unsafe {
        let ret = errno;
        errno = 0;
        ret
    }
}

