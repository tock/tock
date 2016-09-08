// Source: https://github.com/hackndev/zinc/tree/master/volatile_cell
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VolatileCell<T> {
    value: T,
}

#[allow(dead_code)]
impl<T> VolatileCell<T> {
    pub const fn new(value: T) -> Self {
        VolatileCell { value: value }
    }

    #[inline]
    pub fn get(&self) -> T {
        unsafe { ::core::ptr::read_volatile(&self.value) }
    }

    #[inline]
    pub fn set(&self, value: T) {
        unsafe { ::core::ptr::write_volatile(&self.value as *const T as *mut T, value) }
    }
}
