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

    pub unsafe fn borrow_mut<'a: 'b,'b>(&'a self) -> &'b mut T {
        &mut *self.value.get()
    }
}

impl<T> ::core::ops::Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            &*self.value.get()
        }
    }
}

impl<T> ::core::ops::DerefMut for Shared<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            &mut *self.value.get()
        }
    }
}

