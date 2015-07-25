use core::prelude::*;
use core::ptr::{Unique,copy_nonoverlapping};
use core::mem::transmute;
use core::mem;
use core::nonzero::NonZero;
use process;
use process::Process;
use common::Queue;

pub struct Callback {
    // We want more expressive types for this. For now, the kernel is expected
    // to unsafely cast these to `Process` and `fn()`, respectively. Even these
    // types, however, leak some information about the calling application that
    // we probably shouldn't leak.
    process_ptr: *mut (),
    fn_ptr: NonZero<*mut ()>
}

impl Callback {
    pub unsafe fn new(process: &mut Process<'static>, fn_ptr: *mut ()) -> Callback {
        Callback {
            process_ptr: process as *mut Process<'static> as *mut (),
            fn_ptr: NonZero::new(fn_ptr)
        }
    }

    pub fn schedule(&mut self, r0: usize, r1: usize, r2: usize) {
        unsafe {
            let process : &mut Process = transmute(self.process_ptr);
            process.callbacks.enqueue(process::Callback{
                r0: r0,
                r1: r1,
                r2: r2,
                pc: *self.fn_ptr as usize
            });
        }
    }

    pub fn allocate<T>(&mut self, val: T) -> Option<AppPtr<T>> {
        unsafe {
            let process : &mut Process = transmute(self.process_ptr);
            let size = mem::size_of_val(&val);
            process.alloc(size).map(|buf| {
                let dest = &mut buf[0] as *mut u8 as *mut T;
                copy_nonoverlapping(&val, dest, 1);
                AppPtr {
                    ptr: Unique::new(dest),
                    process: self.process_ptr
                }
            })
        }
    }
}

pub struct AppPtr<T> {
    ptr: Unique<T>,
    process: *mut ()
}

impl<T> ::core::ops::Deref for AppPtr<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            self.ptr.get()
        }
    }
}

impl<T> ::core::ops::DerefMut for AppPtr<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            self.ptr.get_mut()
        }
    }
}

impl<T> Drop for AppPtr<T> {
    fn drop(&mut self) {
        unsafe {
            let process : &mut Process = transmute(self.process);
            process.free(self.ptr.get_mut());
        }
    }
}

