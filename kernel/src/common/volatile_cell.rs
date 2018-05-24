//! Implementation of a type for accessing MCU registers.

use core::ptr;

/// `VolatileCell` provides a wrapper around unsafe volatile pointer reads
/// and writes. This is particularly useful for accessing microcontroller
/// registers.
// Source: https://github.com/hackndev/zinc/tree/master/volatile_cell
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VolatileCell<T> {
    value: T,
}

impl<T> VolatileCell<T> {
    pub const fn new(value: T) -> Self {
        VolatileCell { value: value }
    }

    #[inline]
    pub fn get(&self) -> T {
        unsafe { ptr::read_volatile(&self.value) }
    }

    #[inline]
    pub fn set(&self, value: T) {
        unsafe { ptr::write_volatile(&self.value as *const T as *mut T, value) }
    }
}

impl<T: Default> Default for VolatileCell<T> {
    fn default() -> Self {
        VolatileCell::new(Default::default())
    }
}
