//! A wrapper around an immutable slice of `Copy` elements
//! that provides access via a mutable slice

use core;

pub struct CopySlice<'a, T: 'a + Copy>{
    ptr: *mut T,
    len: usize,
    _phantom: core::marker::PhantomData<&'a T>,
}

impl<'a, T: Copy> CopySlice<'a, T> {
    pub fn new(buf: &'a [T]) -> CopySlice<'a, T> {
        CopySlice{
            ptr: buf.as_ptr() as *mut T,
            len: buf.len(),
            _phantom: core::marker::PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_mut(&self) -> &'a mut [T] {
        unsafe {
            core::slice::from_raw_parts_mut(self.ptr, self.len)
        }
    }

    pub fn as_slice(&self) -> &'a [T] {
        unsafe {
            core::slice::from_raw_parts(self.ptr, self.len)
        }
    }
}

pub fn static_mut_bytes_8() -> &'static mut [u8] {
    unsafe {
        static mut STORAGE: [u8; 8] = [0xee; 8];
        &mut STORAGE
    }
}

pub fn static_mut_bytes_100() -> &'static mut [u8] {
    unsafe {
        static mut STORAGE: [u8; 100] = [0xee; 100];
        &mut STORAGE
    }
}
