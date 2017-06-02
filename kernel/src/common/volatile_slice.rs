use core::ptr;

/// A wrapper around a mutable slice that forces accesses
/// to use volatile reads and writes
#[derive(Copy, Clone)]
pub struct VolatileSlice<T>{
    ptr: *mut T,
    len: usize,
}

impl<T: Copy> VolatileSlice<T> {
    pub fn new(buf: &'static [T]) -> VolatileSlice<T> {
        VolatileSlice{
            ptr: buf.as_ptr() as *mut T,
            len: buf.len(),
        }
    }

    pub fn new_mut(buf: &'static mut [T]) -> VolatileSlice<T> {
        VolatileSlice{
            ptr: buf.as_mut_ptr(),
            len: buf.len(),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_mut_ptr(&self) -> *mut T {
        self.ptr
    }

    pub fn prefix_copy_from_slice(&self, src: &[T]) {
        if self.len < src.len() {
            panic!("too small to copy from slice");
        }
        for (i, p) in src.iter().enumerate() {
            unsafe {
                ptr::write_volatile(self.ptr.offset(i as isize), *p);
            }
        }
    }
}

pub fn copy_from_volatile_slice<T: Copy>(dst: &mut [T], src: VolatileSlice<T>) {
    for (i, p) in dst.iter_mut().enumerate() {
        unsafe {
            *p = ptr::read_volatile(src.ptr.offset(i as isize));
        }
    }
}
