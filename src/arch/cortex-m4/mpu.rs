use core::intrinsics;

/// Constructor field is private to limit who can create a new MPU
pub struct MPU(());

impl MPU {

    pub const unsafe fn new() -> MPU {
        MPU(())
    }

    pub fn enable_mpu(&mut self) {
        unsafe {
            let ctrl = 0xE000ED94 as *mut usize;
            intrinsics::volatile_store(ctrl, 0b101);
        }
    }

    pub fn set_mpu(&mut self, region_num: usize,
                          start_addr: usize, len: usize,
                          execute: bool, ap: usize) {
        unsafe {
            let rbar = 0xE000ED9C as *mut usize;
            let rasr = 0xE000EDA0 as *mut usize;

            intrinsics::volatile_store(rbar, region_num | 1 << 4 | start_addr);
            let xn = if execute { 0 } else { 1 };
            intrinsics::volatile_store(rasr, 1 | len << 1 | ap << 24 | xn << 28);
        }
    }

}

