//! Data structure to store a list of userspace applications.

use callback::AppId;
use core::marker::PhantomData;
use core::mem::size_of;
use core::ops::{Deref, DerefMut};
use core::ptr::{read_volatile, write_volatile, Unique};
use process::{self, Error};

crate static mut CONTAINER_COUNTER: usize = 0;

pub struct Grant<T: Default> {
    grant_num: usize,
    ptr: PhantomData<T>,
}

pub struct AppliedGrant<T> {
    appid: usize,
    grant: *mut T,
    _phantom: PhantomData<T>,
}

impl<T> AppliedGrant<T> {
    pub fn enter<F, R>(self, fun: F) -> R
    where
        F: FnOnce(&mut Owned<T>, &mut Allocator) -> R,
        R: Copy,
    {
        let proc = unsafe {
            process::PROCS[self.appid]
                .as_mut()
                .expect("Request to allocate in nonexistent app")
        };
        let mut allocator = Allocator {
            app: proc,
            app_id: self.appid,
        };
        let mut root = unsafe { Owned::new(self.grant, self.appid) };
        fun(&mut root, &mut allocator)
    }
}

pub struct Allocator<'a> {
    app: &'a mut process::Process<'a>,
    app_id: usize,
}

pub struct Owned<T: ?Sized> {
    data: Unique<T>,
    app_id: usize,
}

impl<T: ?Sized> Owned<T> {
    unsafe fn new(data: *mut T, app_id: usize) -> Owned<T> {
        Owned {
            data: Unique::new_unchecked(data),
            app_id: app_id,
        }
    }

    pub fn appid(&self) -> AppId {
        AppId::new(self.app_id)
    }
}

impl<T: ?Sized> Drop for Owned<T> {
    fn drop(&mut self) {
        unsafe {
            let app_id = self.app_id;
            let data = self.data.as_ptr() as *mut u8;
            match process::PROCS[app_id] {
                None => {}
                Some(ref mut app) => {
                    app.free(data);
                }
            }
        }
    }
}

impl<T: ?Sized> Deref for Owned<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.data.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for Owned<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.data.as_mut() }
    }
}

impl Allocator<'a> {
    pub fn alloc<T>(&mut self, data: T) -> Result<Owned<T>, Error> {
        unsafe {
            let app_id = self.app_id;
            self.app
                .alloc(size_of::<T>())
                .map_or(Err(Error::OutOfMemory), |arr| {
                    let mut owned = Owned::new(arr.as_mut_ptr() as *mut T, app_id);
                    *owned = data;
                    Ok(owned)
                })
        }
    }
}

pub struct Borrowed<'a, T: 'a + ?Sized> {
    data: &'a mut T,
    app_id: usize,
}

impl<T: 'a + ?Sized> Borrowed<'a, T> {
    pub fn new(data: &'a mut T, app_id: usize) -> Borrowed<T> {
        Borrowed {
            data: data,
            app_id: app_id,
        }
    }

    pub fn appid(&self) -> AppId {
        AppId::new(self.app_id)
    }
}

impl<T: 'a + ?Sized> Deref for Borrowed<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<T: 'a + ?Sized> DerefMut for Borrowed<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<T: Default> Grant<T> {
    pub unsafe fn create() -> Grant<T> {
        let ctr = read_volatile(&CONTAINER_COUNTER);
        write_volatile(&mut CONTAINER_COUNTER, ctr + 1);
        Grant {
            grant_num: ctr,
            ptr: PhantomData,
        }
    }

    pub fn grant(&self, appid: AppId) -> Option<AppliedGrant<T>> {
        unsafe {
            let app_id = appid.idx();
            match process::PROCS[app_id] {
                Some(ref mut app) => {
                    let cntr = app.grant_for::<T>(self.grant_num);
                    if cntr.is_null() {
                        None
                    } else {
                        Some(AppliedGrant {
                            appid: app_id,
                            grant: cntr,
                            _phantom: PhantomData,
                        })
                    }
                }
                None => None,
            }
        }
    }

    pub fn enter<F, R>(&self, appid: AppId, fun: F) -> Result<R, Error>
    where
        F: FnOnce(&mut Borrowed<T>, &mut Allocator) -> R,
        R: Copy,
    {
        unsafe {
            let app_id = appid.idx();
            match process::PROCS[app_id] {
                Some(ref mut app) => app.grant_for_or_alloc::<T>(self.grant_num).map_or(
                    Err(Error::OutOfMemory),
                    move |root_ptr| {
                        let mut root = Borrowed::new(&mut *root_ptr, app_id);
                        let mut allocator = Allocator {
                            app: app,
                            app_id: app_id,
                        };
                        let res = fun(&mut root, &mut allocator);
                        Ok(res)
                    },
                ),
                None => Err(Error::NoSuchApp),
            }
        }
    }

    pub fn each<F>(&self, fun: F)
    where
        F: Fn(&mut Owned<T>),
    {
        unsafe {
            let itr = process::PROCS.iter_mut().filter_map(|p| p.as_mut());
            for (app_id, app) in itr.enumerate() {
                let root_ptr = app.grant_for::<T>(self.grant_num);
                if !root_ptr.is_null() {
                    let mut root = Owned::new(root_ptr, app_id);
                    fun(&mut root);
                }
            }
        }
    }

    pub fn iter(&self) -> Iter<T> {
        unsafe {
            Iter {
                grant: self,
                index: 0,
                len: process::PROCS.len(),
            }
        }
    }
}

pub struct Iter<'a, T: 'a + Default> {
    grant: &'a Grant<T>,
    index: usize,
    len: usize,
}

impl<T: Default> Iterator for Iter<'a, T> {
    type Item = AppliedGrant<T>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.len {
            let idx = self.index;
            self.index += 1;
            let res = self.grant.grant(AppId::new(idx));
            if res.is_some() {
                return res;
            }
        }
        None
    }
}
