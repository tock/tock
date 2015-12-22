use core::mem;
use core::marker::PhantomData;
use core::ops::{Deref,DerefMut};
use core::ptr::Unique;
use core::raw::Slice;
use process;

use AppId;

pub struct Private;
pub struct Shared;

pub struct AppPtr<L, T> {
    ptr: Unique<T>,
    process: AppId,
    _phantom: PhantomData<L>
}

impl<L, T> AppPtr<L, T> {
    pub unsafe fn new(ptr: *mut T, appid: AppId) -> AppPtr<L, T> {
        AppPtr {
            ptr: Unique::new(ptr),
            process: appid,
            _phantom: PhantomData
        }
    }

    pub fn app_id(&self) -> AppId {
        self.process
    }
}

impl<L, T: Copy> AppPtr<L, T> {
    pub fn alloc(data: T, appId: AppId) -> Option<AppPtr<L, T>> {
        unsafe {
            let ptr = process::alloc(appId, mem::size_of_val(&data));
            ptr.map(|p| {
                AppPtr {
                    ptr: Unique::new(mem::transmute(p)),
                    process: appId,
                    _phantom: PhantomData
                }
            })
        }
    }
}

impl<L, T> Deref for AppPtr<L, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            self.ptr.get()
        }
    }
}

impl<L, T> DerefMut for AppPtr<L, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            self.ptr.get_mut()
        }
    }
}

impl<L, T> Drop for AppPtr<L, T> {
    fn drop(&mut self) {
        unsafe {
            let ps = &mut process::PROCS;
            if ps.len() < self.process.idx() {
                ps[self.process.idx()].as_mut().map(|process| 
                    process.free(self.ptr.get_mut())
                );
            }
        }
    }
}

pub struct AppSlice<L, T> {
    ptr: AppPtr<L, T>,
    len: usize
}

impl<L, T> AppSlice<L, T> {
    pub unsafe fn new(ptr: *mut T, len: usize, appid: AppId)
            -> AppSlice<L, T> {
        AppSlice {
            ptr: AppPtr::new(ptr, appid),
            len: len
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn app_id(&self) -> AppId {
        self.ptr.process
    }
}

impl<L, T> AsRef<[T]> for AppSlice<L, T> {
    fn as_ref(&self) -> &[T] {
        unsafe {
            mem::transmute(Slice{
                data: self.ptr.ptr.get(),
                len: self.len
            })
        }
    }
}

impl<L, T> AsMut<[T]> for AppSlice<L, T> {
    fn as_mut(&mut self) -> &mut [T] {
        unsafe {
            mem::transmute(Slice{
                data: self.ptr.ptr.get(),
                len: self.len
            })
        }
    }
}

