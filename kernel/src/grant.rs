//! Data structure to store a list of userspace applications.

use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ops::{Deref, DerefMut};
use core::ptr::{write, NonNull};

use crate::callback::AppId;
use crate::process::{Error, ProcessType};
use crate::sched::Kernel;

/// Region of process memory reserved for the kernel.
pub struct Grant<T: Default> {
    crate kernel: &'static Kernel,
    grant_num: usize,
    ptr: PhantomData<T>,
}

pub struct AppliedGrant<T> {
    appid: AppId,
    grant: NonNull<T>,
    _phantom: PhantomData<T>,
}

impl<T> AppliedGrant<T> {
    pub fn enter<F, R>(self, fun: F) -> R
    where
        F: FnOnce(&mut Owned<T>, &mut Allocator) -> R,
        R: Copy,
    {
        let mut allocator = Allocator { appid: self.appid };
        let mut root = Owned::new(self.grant, self.appid);
        fun(&mut root, &mut allocator)
    }
}

pub struct Allocator {
    appid: AppId,
}

pub struct Owned<T: ?Sized> {
    data: NonNull<T>,
    appid: AppId,
}

impl<T: ?Sized> Owned<T> {
    fn new(data: NonNull<T>, appid: AppId) -> Owned<T> {
        Owned {
            data: data,
            appid: appid,
        }
    }

    pub fn appid(&self) -> AppId {
        self.appid
    }
}

impl<T: ?Sized> Drop for Owned<T> {
    fn drop(&mut self) {
        unsafe {
            let data = self.data.as_ptr() as *mut u8;
            self.appid.kernel.process_map_or((), self.appid, |process| {
                process.free(data);
            });
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

impl Allocator {
    pub fn alloc<T>(&mut self, data: T) -> Result<Owned<T>, Error> {
        unsafe {
            self.appid
                .kernel
                .process_map_or(Err(Error::NoSuchApp), self.appid, |process| {
                    process.alloc(size_of::<T>(), align_of::<T>()).map_or(
                        Err(Error::OutOfMemory),
                        |arr| {
                            let ptr = arr.as_mut_ptr() as *mut T;
                            // We use `ptr::write` to avoid `Drop`ping the uninitialized memory in
                            // case `T` implements the `Drop` trait.
                            write(ptr, data);
                            // Unchecked is safe as we just created this
                            let data = NonNull::new_unchecked(ptr);
                            Ok(Owned::new(data, self.appid))
                        },
                    )
                })
        }
    }
}

pub struct Borrowed<'a, T: 'a + ?Sized> {
    data: &'a mut T,
    appid: AppId,
}

impl<T: 'a + ?Sized> Borrowed<'a, T> {
    pub fn new(data: &'a mut T, appid: AppId) -> Borrowed<'a, T> {
        Borrowed {
            data: data,
            appid: appid,
        }
    }

    pub fn appid(&self) -> AppId {
        self.appid
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
    crate fn new(kernel: &'static Kernel, grant_index: usize) -> Grant<T> {
        Grant {
            kernel: kernel,
            grant_num: grant_index,
            ptr: PhantomData,
        }
    }

    pub fn grant(&self, appid: AppId) -> Option<AppliedGrant<T>> {
        appid.kernel.process_map_or(None, appid, |process| {
            if let Some(grant_ptr) = process.grant_ptr(self.grant_num) {
                NonNull::new(grant_ptr).map(|grant| AppliedGrant {
                    appid: appid,
                    grant: grant.cast::<T>(),
                    _phantom: PhantomData,
                })
            } else {
                None
            }
        })
    }

    pub fn enter<F, R>(&self, appid: AppId, fun: F) -> Result<R, Error>
    where
        F: FnOnce(&mut Borrowed<T>, &mut Allocator) -> R,
        R: Copy,
    {
        unsafe {
            appid
                .kernel
                .process_map_or(Err(Error::NoSuchApp), appid, |process| {
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
                    if let Some(untyped_grant_ptr_ref) = process.grant_ptr(self.grant_num) {
                        let typed_grant_ptr = *untyped_grant_ptr_ref as *mut T;
                        // If the pointer at that location is NULL then the grant
                        // memory needs to be allocated.
                        let grant_ptr = if (typed_grant_ptr).is_null() {
                            process
                                .alloc(size_of::<T>(), align_of::<T>())
                                .map(|root_arr| {
                                    let new_grant = root_arr.as_mut_ptr() as *mut T;
                                    // Initialize the grant contents using ptr::write, to
                                    // ensure that we don't try to drop the contents of
                                    // uninitialized memory when T implements Drop.
                                    write(new_grant, Default::default());
                                    // Record the location in the grant pointer.
                                    *untyped_grant_ptr_ref = new_grant as *mut u8;
                                    // Return the newly allocated and intialized grant
                                    new_grant
                                })
                        } else {
                            Some(typed_grant_ptr)
                        };

                        // If the grant region already exists or there was enough
                        // memory to allocate it, call the passed in closure with
                        // the borrowed grant region.
                        grant_ptr.map_or(Err(Error::OutOfMemory), move |grant_ptr| {
                            let mut grant = Borrowed::new(&mut *grant_ptr, appid);
                            let mut allocator = Allocator { appid: appid };
                            let res = fun(&mut grant, &mut allocator);
                            Ok(res)
                        })
                    } else {
                        Err(Error::InactiveApp)
                    }
                })
        }
    }

    pub fn each<F>(&self, fun: F)
    where
        F: Fn(&mut Owned<T>),
    {
        self.kernel.process_each(|process| {
            if let Some(grant_ptr) = process.grant_ptr(self.grant_num) {
                NonNull::new(grant_ptr).map(|grant| {
                    let mut root = Owned::new(grant.cast::<T>(), process.appid());
                    fun(&mut root);
                });
            }
        });
    }

    /// Get an iterator over all processes and their active grant regions for
    /// this particular grant.
    pub fn iter(&self) -> Iter<T> {
        Iter {
            grant: self,
            subiter: self.kernel.get_process_iter(),
        }
    }
}

pub struct Iter<'a, T: 'a + Default> {
    grant: &'a Grant<T>,
    subiter: core::iter::FilterMap<
        core::slice::Iter<'a, Option<&'static dyn ProcessType>>,
        fn(&Option<&'static dyn ProcessType>) -> Option<&'static dyn ProcessType>,
    >,
}

impl<T: Default> Iterator for Iter<'a, T> {
    type Item = AppliedGrant<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // Save a local copy of grant_num so we don't have to access `self`
        // in the closure below.
        let grant_num = self.grant.grant_num;

        // Get the next `AppId` from the kernel processes array that is setup to use this grant.
        // Since the iterator itself is saved calling this function
        // again will start where we left off.
        let res = self.subiter.find(|p| {
            // We have found a candidate process that exists in the
            // processes array. Now we have to check if this grant is setup
            // for this process. If not, we have to skip it and keep
            // looking.
            if let Some(grant_ptr) = p.grant_ptr(grant_num) {
                !grant_ptr.is_null()
            } else {
                false
            }
        });

        // Check if our find above returned another `AppId`, or if we hit the
        // end of the iterator. If we found another app, try to access its grant
        // region.
        res.map_or(None, |proc| self.grant.grant(proc.appid()))
    }
}
