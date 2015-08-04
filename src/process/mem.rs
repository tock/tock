use core::prelude::*;
use core::mem;
use core::ops::{Deref,DerefMut};
use core::ptr::Unique;
use core::raw::Slice;
use process::Process;

pub struct AppPtr<T> {
    ptr: Unique<T>,
    process: *mut ()
}

impl<T> AppPtr<T> {
    pub unsafe fn new(ptr: *mut T, process: *mut ()) -> AppPtr<T> {
        AppPtr {
            ptr: Unique::new(ptr),
            process: process
        }
    }
}

impl<T> Deref for AppPtr<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            self.ptr.get()
        }
    }
}

impl<T> DerefMut for AppPtr<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            self.ptr.get_mut()
        }
    }
}

impl<T> Drop for AppPtr<T> {
    fn drop(&mut self) {
        unsafe {
            let process : &mut Process = mem::transmute(self.process);
            process.free(self.ptr.get_mut());
        }
    }
}

pub struct AppSlice<T> {
    ptr: AppPtr<T>,
    len: usize
}

impl<T> AppSlice<T> {
    pub unsafe fn new(ptr: *mut T, len: usize, process_ptr: *mut ())
            -> AppSlice<T> {
        AppSlice {
            ptr: AppPtr::new(ptr, process_ptr),
            len: len
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl<T> AsRef<[T]> for AppSlice<T> {
    fn as_ref(&self) -> &[T] {
        unsafe {
            mem::transmute(Slice{
                data: self.ptr.ptr.get(),
                len: self.len
            })
        }
    }
}

impl<T> AsMut<[T]> for AppSlice<T> {
    fn as_mut(&mut self) -> &mut [T] {
        unsafe {
            mem::transmute(Slice{
                data: self.ptr.ptr.get(),
                len: self.len
            })
        }
    }
}

