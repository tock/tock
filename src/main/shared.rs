use core::prelude::*;
use core::cell::UnsafeCell;

// Should T be `Sync`?
pub struct Shared<T> {
    value: UnsafeCell<T>
}

impl<T> Shared<T> {
    pub fn new(value: T) -> Shared<T> {
        Shared {
            value: UnsafeCell::new(value)
        }
    }

    pub fn borrow_mut(&self) -> &mut T {
        unsafe { &mut *self.value.get() }
    }
}

