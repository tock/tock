use core::cell::UnsafeCell;

pub struct CoreLocal<T>(UnsafeCell<T>);

impl<T> CoreLocal<T> {
    pub const unsafe fn new_single_core(val: T) -> Self {
	CoreLocal(UnsafeCell::new(val))
    }
}

impl<T> CoreLocal<T> {
    pub fn with<F, R>(&self, f: F) -> R where F: FnOnce(&T) -> R {
	f(unsafe { &*self.0.get() })
    }
}

unsafe impl<T> Sync for CoreLocal<T> {}
