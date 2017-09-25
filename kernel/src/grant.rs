//! Data structure to store a list of userspace applications.

use callback::AppId;
use core::marker::PhantomData;
use core::mem::size_of;
use core::ops::{Deref, DerefMut};
use core::ptr::{read_volatile, write_volatile, Unique};
use debug;
use process::{self, Error};

pub static mut CONTAINER_COUNTER: usize = 0;

pub struct Grant<T: Default> {
    grant_num: usize,
    ptr: PhantomData<T>,
}

pub struct AppliedGrant<T> {
    appid: usize,
    grant: *mut T,
    _phantom: PhantomData<T>,
}

pub unsafe fn kernel_grant_for<T>(app_id: usize) -> *mut T {
    match app_id {
        debug::APPID_IDX => debug::get_grant(),
        _ => panic!("lookup for invalid kernel grant {}", app_id),
    }
}

impl<T> AppliedGrant<T> {
    pub fn enter<F, R>(self, fun: F) -> R
        where F: FnOnce(&mut Owned<T>, &mut Allocator) -> R,
              R: Copy
    {
        let mut allocator = Allocator {
            app: unsafe { Some(process::PROCS[self.appid].as_mut().unwrap()) },
            app_id: self.appid,
        };
        let mut root = unsafe { Owned::new(self.grant, self.appid) };
        fun(&mut root, &mut allocator)
    }
}

pub struct Allocator<'a> {
    app: Option<&'a mut process::Process<'a>>,
    app_id: usize,
}

pub struct Owned<T: ?Sized> {
    data: Unique<T>,
    app_id: usize,
}

impl<T: ?Sized> Owned<T> {
    pub unsafe fn new(data: *mut T, app_id: usize) -> Owned<T> {
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
            if AppId::is_kernel_idx(app_id) {
                /* kernel free is nop */
;
            } else {
                match process::PROCS[app_id] {
                    None => {}
                    Some(ref mut app) => {
                        app.free(data);
                    }
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

impl<'a> Allocator<'a> {
    pub fn alloc<T>(&mut self, data: T) -> Result<Owned<T>, Error> {
        unsafe {
            let app_id = self.app_id;
            match self.app.as_mut() {
                Some(app) => {
                    app.alloc(size_of::<T>()).map_or(Err(Error::OutOfMemory), |arr| {
                        let mut owned = Owned::new(arr.as_mut_ptr() as *mut T, app_id);
                        *owned = data;
                        Ok(owned)
                    })
                }
                None => {
                    if !AppId::is_kernel_idx(app_id) {
                        panic!("No app for allocator for {}", app_id);
                    }
                    panic!("Request to allocate in kernel grant");
                }
            }
        }
    }
}

pub struct Borrowed<'a, T: 'a + ?Sized> {
    data: &'a mut T,
    app_id: usize,
}

impl<'a, T: 'a + ?Sized> Borrowed<'a, T> {
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

impl<'a, T: 'a + ?Sized> Deref for Borrowed<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T: 'a + ?Sized> DerefMut for Borrowed<'a, T> {
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
            if AppId::is_kernel(appid) {
                let cntr = kernel_grant_for::<T>(app_id);
                Some(AppliedGrant {
                    appid: app_id,
                    grant: cntr,
                    _phantom: PhantomData,
                })
            } else {
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
    }

    pub fn enter<F, R>(&self, appid: AppId, fun: F) -> Result<R, Error>
        where F: FnOnce(&mut Borrowed<T>, &mut Allocator) -> R,
              R: Copy
    {
        unsafe {
            let app_id = appid.idx();
            if AppId::is_kernel(appid) {
                let root_ptr = kernel_grant_for::<T>(app_id);
                let mut root = Borrowed::new(&mut *root_ptr, app_id);
                let mut allocator = Allocator {
                    app: None,
                    app_id: app_id,
                };
                let res = fun(&mut root, &mut allocator);
                Ok(res)
            } else {
                match process::PROCS[app_id] {
                    Some(ref mut app) => {
                        app.grant_for_or_alloc::<T>(self.grant_num)
                            .map_or(Err(Error::OutOfMemory), move |root_ptr| {
                                let mut root = Borrowed::new(&mut *root_ptr, app_id);
                                let mut allocator = Allocator {
                                    app: Some(app),
                                    app_id: app_id,
                                };
                                let res = fun(&mut root, &mut allocator);
                                Ok(res)
                            })
                    }
                    None => Err(Error::NoSuchApp),
                }
            }
        }
    }

    pub fn each<F>(&self, fun: F)
        where F: Fn(&mut Owned<T>)
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

impl<'a, T: Default> Iterator for Iter<'a, T> {
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
