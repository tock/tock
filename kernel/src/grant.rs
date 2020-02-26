//! Data structure to store a list of userspace applications.

use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ops::{Deref, DerefMut};
use core::ptr::{write, write_volatile, Unique};

use crate::callback::AppId;
use crate::process::Error;
use crate::sched::Kernel;

/// Region of process memory reserved for the kernel.
pub struct Grant<'ker, T: Default> {
    crate kernel: &'ker Kernel<'ker>,
    grant_num: usize,
    ptr: PhantomData<T>,
}

pub struct AppliedGrant<'ker, T> {
    appid: AppId<'ker>,
    grant: *mut T,
    _phantom: PhantomData<T>,
}

impl<'ker, T> AppliedGrant<'ker, T> {
    pub fn enter<F, R>(self, fun: F) -> R
    where
        F: FnOnce(&mut Owned<'ker, T>, &mut Allocator<'ker>) -> R,
        R: Copy,
    {
        let mut allocator = Allocator { appid: self.appid };
        let mut root = unsafe { Owned::new(self.grant, self.appid) };
        fun(&mut root, &mut allocator)
    }
}

pub struct Allocator<'ker> {
    appid: AppId<'ker>,
}

pub struct Owned<'ker, T: ?Sized> {
    data: Unique<T>,
    appid: AppId<'ker>,
}

impl<'ker, T: ?Sized> Owned<'ker, T> {
    unsafe fn new(data: *mut T, appid: AppId<'ker>) -> Owned<'ker, T> {
        Owned {
            data: Unique::new_unchecked(data),
            appid: appid,
        }
    }

    pub fn appid(&self) -> AppId<'ker> {
        self.appid
    }
}

impl<T: ?Sized> Drop for Owned<'_, T> {
    fn drop(&mut self) {
        unsafe {
            let data = self.data.as_ptr() as *mut u8;
            self.appid
                .kernel
                .process_map_or((), self.appid.idx(), |process| {
                    process.free(data);
                });
        }
    }
}

impl<T: ?Sized> Deref for Owned<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.data.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for Owned<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.data.as_mut() }
    }
}

impl<'ker> Allocator<'ker> {
    pub fn alloc<T>(&mut self, data: T) -> Result<Owned<'ker, T>, Error> {
        unsafe {
            self.appid
                .kernel
                .process_map_or(Err(Error::NoSuchApp), self.appid.idx(), |process| {
                    process.alloc(size_of::<T>(), align_of::<T>()).map_or(
                        Err(Error::OutOfMemory),
                        |arr| {
                            let ptr = arr.as_mut_ptr() as *mut T;
                            // We use `ptr::write` to avoid `Drop`ping the uninitialized memory in
                            // case `T` implements the `Drop` trait.
                            write(ptr, data);
                            Ok(Owned::new(ptr, self.appid))
                        },
                    )
                })
        }
    }
}

pub struct Borrowed<'a, 'ker, T: 'a + ?Sized> {
    data: &'a mut T,
    appid: AppId<'ker>,
}

impl<T: 'a + ?Sized> Borrowed<'a, 'ker, T> {
    pub fn new(data: &'a mut T, appid: AppId<'ker>) -> Borrowed<'a, 'ker, T> {
        Borrowed {
            data: data,
            appid: appid,
        }
    }

    pub fn appid(&self) -> AppId<'ker> {
        self.appid
    }
}

impl<T: 'a + ?Sized> Deref for Borrowed<'a, '_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<T: 'a + ?Sized> DerefMut for Borrowed<'a, '_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<'ker, T: Default> Grant<'ker, T> {
    crate fn new(kernel: &'ker Kernel<'ker>, grant_index: usize) -> Grant<'ker, T> {
        Grant {
            kernel: kernel,
            grant_num: grant_index,
            ptr: PhantomData,
        }
    }

    pub fn grant(&self, appid: AppId<'ker>) -> Option<AppliedGrant<'ker, T>> {
        unsafe {
            appid.kernel.process_map_or(None, appid.idx(), |process| {
                let cntr = *(process.grant_ptr(self.grant_num) as *mut *mut T);
                if cntr.is_null() {
                    None
                } else {
                    Some(AppliedGrant {
                        appid: appid,
                        grant: cntr,
                        _phantom: PhantomData,
                    })
                }
            })
        }
    }

    pub fn enter<F, R>(&self, appid: AppId<'ker>, fun: F) -> Result<R, Error>
    where
        F: FnOnce(&mut Borrowed<'_, 'ker, T>, &mut Allocator<'ker>) -> R,
        R: Copy,
    {
        unsafe {
            appid
                .kernel
                .process_map_or(Err(Error::NoSuchApp), appid.idx(), |process| {
                    // Here is an example of how the grants are laid out in a
                    // process's memory:
                    //
                    // Mem. Addr.
                    // 0x0040000  ┌────────────────────
                    //            │   GrantPointer0 [0x003FFC8]
                    //            │   GrantPointer1 [0x003FFC0]
                    //            │   ...
                    //            │   GrantPointerN [0x0000000 (NULL)]
                    // 0x003FFE0  ├────────────────────
                    //            │   GrantRegion0
                    // 0x003FFC8  ├────────────────────
                    //            │   GrantRegion1
                    // 0x003FFC0  ├────────────────────
                    //            │
                    //            │   --unallocated--
                    //            │
                    //            └────────────────────
                    //
                    // An array of pointers (one per possible grant region)
                    // point to where the actual grant memory is allocated
                    // inside of the process. The grant memory is not allocated
                    // until the actual grant region is actually used.
                    //
                    // This function provides the app access to the specific
                    // grant memory, and allocates the grant region in the
                    // process memory if needed.
                    //
                    // Get a pointer to where the grant pointer is stored in the
                    // process memory.
                    let ctr_ptr = process.grant_ptr(self.grant_num) as *mut *mut T;
                    // If the pointer at that location is NULL then the grant
                    // memory needs to be allocated.
                    let new_grant = if (*ctr_ptr).is_null() {
                        process
                            .alloc(size_of::<T>(), align_of::<T>())
                            .map(|root_arr| {
                                let root_ptr = root_arr.as_mut_ptr() as *mut T;
                                // Initialize the grant contents using ptr::write, to
                                // ensure that we don't try to drop the contents of
                                // uninitialized memory when T implements Drop.
                                write(root_ptr, Default::default());
                                // Record the location in the grant pointer.
                                write_volatile(ctr_ptr, root_ptr);
                                root_ptr
                            })
                    } else {
                        Some(*ctr_ptr)
                    };

                    // If the grant region already exists or there was enough
                    // memory to allocate it, call the passed in closure with
                    // the borrowed grant region.
                    new_grant.map_or(Err(Error::OutOfMemory), move |root_ptr| {
                        let root_ptr = root_ptr as *mut T;
                        let mut root = Borrowed::new(&mut *root_ptr, appid);
                        let mut allocator = Allocator { appid: appid };
                        let res = fun(&mut root, &mut allocator);
                        Ok(res)
                    })
                })
        }
    }

    pub fn each<F>(&self, fun: F)
    where
        F: Fn(&mut Owned<'ker, T>),
    {
        self.kernel.process_each(|process| unsafe {
            let root_ptr = *(process.grant_ptr(self.grant_num) as *mut *mut T);
            if !root_ptr.is_null() {
                let mut root = Owned::new(root_ptr, process.appid());
                fun(&mut root);
            }
        });
    }

    pub fn iter(&'a self) -> Iter<'a, 'ker, T> {
        Iter {
            grant: self,
            index: 0,
            len: self.kernel.number_of_process_slots(),
        }
    }
}

pub struct Iter<'a, 'ker, T: 'a + Default> {
    grant: &'a Grant<'ker, T>,
    index: usize,
    len: usize,
}

impl<T: Default> Iterator for Iter<'a, 'ker, T> {
    type Item = AppliedGrant<'ker, T>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.len {
            let idx = self.index;
            self.index += 1;
            let res = self.grant.grant(AppId::new(self.grant.kernel, idx));
            if res.is_some() {
                return res;
            }
        }
        None
    }
}
