use core::cell::UnsafeCell;

pub struct Shared<T> {
    value: UnsafeCell<T>
}

impl<T> Shared<T> {
    pub fn new(value: T) -> Shared<T> {
        Shared {
            value: UnsafeCell::new(value)
        }
    }

    pub fn borrow_mut<'a: 'b,'b>(&'a self) -> &'b mut T {
        unsafe {
            &mut *self.value.get()
        }
    }
}

