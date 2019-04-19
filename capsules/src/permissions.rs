//! Simple capsule that checks if a permission is set to true or false

pub struct Permissions {}

impl Permissions {
    pub fn new() -> Permissions {
        Permissions {}
    }

    // checks to see if the corresponding permission bit is flipped to 1 or 0
    pub fn check(&self, permissions: &[u32], driver_num: usize) -> bool {
        if permissions.iter().find(|&&n| n == driver_num as u32) == None {
            false // driver not found
        } else {
            true
        }
    }
}
