//! Simple capsule that checks if a permission is set to true or false

pub struct Permissions {}

impl Permissions {
    pub fn new() -> Permissions {
        Permissions {}
    }

    pub fn check(&self, permissions: &[u8], driver_num: usize) -> bool {
        if let Some(bit) = crate::driver::get_permission_bit(driver_num) {
            permissions[bit / 8] & (1 << bit % 8) == 1 << bit % 8
        } else {
            // driver not found
            false
        }
    }
}
