//! Simple capsule that checks if a permission is set to true or false

pub struct Permissions {}

impl Permissions {
    pub fn new() -> Permissions {
        Permissions {}
    }

    // checks to see if the corresponding permission bit is flipped to 1 or 0
    pub fn check(&self, permissions: &[u8], driver_num: usize) -> bool {
        if let Some(bit) = crate::driver::get_permission_bit(driver_num) {
            if bit / 8 >= permissions.len() {
                false // driver not found
            } else {
                permissions[bit / 8] & (1 << bit % 8) == 1 << bit % 8
            }
        } else {
            false // driver not found
        }
    }
}
