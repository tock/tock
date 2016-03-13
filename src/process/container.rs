use callback::AppId;
use core::intrinsics::{volatile_load, volatile_store};
use core::marker::PhantomData;
use core::mem::size_of;
use core::raw::Repr;
use mem::{AppPtr, Private};
use process;

pub struct Container<T: Default> {
    container_num: usize,
    ptr: PhantomData<T>
}

pub enum Error {
    NoSuchApp,
    OutOfMemory
}

static mut CONTAINER_COUNTER : usize = 0;

pub struct Allocator<'a> {
    app: &'a mut process::Process<'a>,
    app_id: AppId
}

impl<'a> Allocator<'a> {
    pub fn alloc<T>(&mut self, data: T) -> Result<AppPtr<Private, T>, Error> {
        unsafe {
            let appid = self.app_id;
            self.app.alloc(size_of::<T>()).map_or(Err(Error::OutOfMemory),
                |arr| {
                    Ok(AppPtr::new(arr.repr().data as *mut T, appid))
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
        where F: Fn(&mut AppPtr<Private, T>, &mut Allocator) -> R, R: Copy {
        unsafe {
            match process::PROCS[appid.idx()] {
                Some(ref mut app) => {
                    app.container_for(self.container_num).or_else(|| {
                        app.alloc(size_of::<T>()).map(|root_arr| {
                            root_arr.repr().data as *mut _
                        })
                    }).map_or(Err(Error::OutOfMemory), move |root_ptr| {
                        let mut root = AppPtr::new(root_ptr as *mut _, appid);
                        let mut allocator = Allocator { app: app, app_id: appid };
                        let res = fun(&mut root, &mut allocator);
                        Ok(res)
                    })
                },
                None => Err(Error::NoSuchApp)
            }
        }
    }
}

