#[cfg(target_pointer_width = "32")]
use crate::volatile_cell::VolatileCell;

#[cfg(target_pointer_width = "32")]
#[repr(C)]
pub struct ConstPtr32<T> {
    ptr: VolatileCell<*const T>,
}

#[cfg(not(target_pointer_width = "32"))]
#[repr(C)]
pub struct ConstPtr32<T> {
    _dummy: u32,
    _phantom: core::marker::PhantomData<T>,
}

#[cfg(target_pointer_width = "32")]
impl<T> ConstPtr32<T> {
    pub fn new(ptr: *const T) -> Self {
        ConstPtr32 {
            ptr: VolatileCell::new(ptr),
        }
    }

    #[inline]
    pub fn get(&self) -> *const T {
        self.ptr.get()
    }

    #[inline]
    pub fn set(&mut self, ptr: *const T) {
        self.ptr.set(ptr);
    }
}

#[cfg(not(target_pointer_width = "32"))]
impl<T> ConstPtr32<T> {
    pub fn new(_ptr: *const T) -> Self {
        unimplemented!()
    }

    pub fn get(&self) -> *const T {
        unimplemented!()
    }

    pub fn set(&mut self, _ptr: *const T) {
        unimplemented!()
    }
}
