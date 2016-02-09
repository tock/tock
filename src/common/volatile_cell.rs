// Source: https://github.com/hackndev/zinc/tree/master/volatile_cell
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VolatileCell<T> {
    value: T,
}

#[allow(dead_code)]
impl<T> VolatileCell<T> {
    #[inline]
    pub fn get(&self) -> T {
        unsafe {
            ::core::intrinsics::volatile_load(&self.value)
        }
    }

    #[inline]
    pub fn set(&self, value: T) {
        unsafe {
            ::core::intrinsics::volatile_store(&self.value as *const T as *mut T, value)
        }
    }
}
