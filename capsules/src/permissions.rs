//! Simple capsule that checks if a permission is set to true or false

#![forbid(unsafe_code)]

pub struct Permissions<> {
}

impl<> Permissions<> {
    pub fn new() -> Permissions<> {
        Permissions {
        }
    }

    pub fn start(&self) {
    }

    pub fn check (&self, permissions: u64, driver_num: usize) -> bool {
        let bit = crate::driver::get_permission_bit(driver_num);
        permissions & (1 << bit) == (1 << bit)
    }
}
