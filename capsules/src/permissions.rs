//! Simple capsule that checks if a permission is set to true or false

pub struct Permissions {}

impl Permissions {
    pub fn new() -> Permissions {
        Permissions {}
    }

    // checks to see if the corresponding permission is allowed
    pub fn check(&self, permissions: &[u32], driver_num: usize) -> bool {
        match permissions.binary_search(&(driver_num as u32)) {
            Ok(_)  => true,
            Err(_) => false,
        }
    }
}
