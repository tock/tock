use callback::AppId;
use core::intrinsics::{volatile_load, volatile_store};
use core::marker::PhantomData;
use core::mem::size_of;
use core::ops::{Deref, DerefMut};
use core::ptr::Unique;
use core::raw::Repr;
use process::{self, Error};

pub struct Container<T: Default> {
    container_num: usize,
    ptr: PhantomData<T>
}

pub static mut CONTAINER_COUNTER : usize = 0;

pub struct Allocator<'a> {
    app: &'a mut process::Process<'a>,
    app_id: usize
}

pub struct Owned<T: ?Sized> {
    data: Unique<T>,
    app_id: usize
}

impl<T: ?Sized> Owned<T> {
    pub unsafe fn new(data: *mut T, app_id: usize) -> Owned<T> {
        Owned { data: Unique::new(data), app_id: app_id }
    }
}

impl<T: ?Sized> Drop for Owned<T> {
    fn drop(&mut self) {
        unsafe {
            let app_id = self.app_id;
            let data = self.data.get_mut() as *mut T as *mut u8;
            match process::PROCS[app_id] {
                None => {},
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
        unsafe { self.data.get() }
    }
}

impl<T: ?Sized> DerefMut for Owned<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.data.get_mut() }
    }
}

impl<'a> Allocator<'a> {
    pub fn alloc<T>(&mut self, data: T) -> Result<Owned<T>, Error> {
        unsafe {
            let app_id = self.app_id;
            self.app.alloc(size_of::<T>()).map_or(Err(Error::OutOfMemory),
                |arr| {
                    let mut owned = Owned::new(arr.repr().data as *mut T, app_id);
                    *owned = data;
                    Ok(owned)
            })
        }
    }
}

impl<T: Default> Container<T> {
    pub unsafe fn create() -> Container<T> {
        let ctr = volatile_load(&CONTAINER_COUNTER);
        volatile_store(&mut CONTAINER_COUNTER, ctr + 1);
        Container {
            container_num: ctr,
            ptr: PhantomData
        }
    }

    pub fn enter<F, R>(&self, appid: AppId, fun: F) -> Result<R, Error>
        where F: FnOnce(&mut Owned<T>, &mut Allocator) -> R, R: Copy {
        unsafe {
            let app_id = appid.idx();
            match process::PROCS[app_id] {
                Some(ref mut app) => {
                    app.container_for_or_alloc::<T>(self.container_num).map_or(
                        Err(Error::OutOfMemory), move |root_ptr| {
                            let mut root = Owned::new(root_ptr, app_id);
                            let mut allocator = Allocator {
                                app: app,
                                app_id: app_id
                            };
                            let res = fun(&mut root, &mut allocator);
                            Ok(res)
                    })
                },
                None => Err(Error::NoSuchApp)
            }
        }
    }

    pub fn each<F>(&self, fun: F) where F: Fn(&mut Owned<T>) {
        unsafe {
            let itr = process::PROCS.iter_mut().filter_map(|p| p.as_mut());
            for (app_id, app) in itr.enumerate() {
                let ctr_ptr = app.container_for::<T>(self.container_num);
                if !ctr_ptr.is_null() {
                    let root_ptr = *ctr_ptr;
                    let mut root = Owned::new(root_ptr, app_id);
                    fun(&mut root);
                }
            }
        }
    }
}

