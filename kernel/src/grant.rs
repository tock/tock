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
    pub(crate) kernel: &'static Kernel,
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

/// Grant which was dynamically allocated in a particular app's memory.
pub struct DynamicGrant<T: ?Sized> {
    data: NonNull<T>,
    appid: AppId,
}

impl<T: ?Sized> DynamicGrant<T> {
    /// Creates a new `DynamicGrant`.
    ///
    /// # Safety
    ///
    /// `data` must point to a valid, initialized `T`.
    unsafe fn new(data: NonNull<T>, appid: AppId) -> Self {
        DynamicGrant { data, appid }
    }

    pub fn appid(&self) -> AppId {
        self.appid
    }

    /// Gives access to inner data within the given closure.
    ///
    /// If the app has since been restarted or crashed, or the memory is otherwise no longer
    /// present, then this function will not call the given closure, and will
    /// instead directly return `None`.
    pub fn enter<F, R>(&mut self, fun: F) -> Option<R>
    where
        F: FnOnce(Borrowed<'_, T>) -> R,
    {
        self.appid.kernel.process_map_or(None, self.appid, |_| {
            let data = unsafe { self.data.as_mut() };
            let borrowed = Borrowed::new(data, self.appid);
            Some(fun(borrowed))
        })
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
    pub fn alloc<T: Default>(&mut self) -> Result<DynamicGrant<T>, Error> {
        unsafe {
            let ptr = self.alloc_unowned()?;
            Ok(DynamicGrant::new(ptr, self.appid))
        }
    }

    // Like `alloc`, but the caller is responsible for free-ing the allocated
    // memory, as it is not wrapped in a type that implements `Drop`
    unsafe fn alloc_unowned<T: Default>(&mut self) -> Result<NonNull<T>, Error> {
        self.appid
            .kernel
            .process_map_or(Err(Error::NoSuchApp), self.appid, |process| {
                process.alloc(size_of::<T>(), align_of::<T>()).map_or(
                    Err(Error::OutOfMemory),
                    |buf| {
                        // Convert untyped `*mut u8` allocation to allocated type
                        let ptr = NonNull::cast::<T>(buf);

                        // We use `ptr::write` to avoid `Drop`ping the uninitialized memory in
                        // case `T` implements the `Drop` trait.
                        write(ptr.as_ptr(), T::default());

                        Ok(ptr)
                    },
                )
            })
    }
}

pub struct Borrowed<'a, T: 'a + ?Sized> {
    data: &'a mut T,
    appid: AppId,
}

impl<'a, T: 'a + ?Sized> Borrowed<'a, T> {
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
    pub(crate) fn new(kernel: &'static Kernel, grant_index: usize) -> Grant<T> {
        Grant {
            kernel: kernel,
            grant_num: grant_index,
            ptr: PhantomData,
        }
    }

    pub fn grant(&self, appid: AppId) -> Option<AppliedGrant<T>> {
        appid.kernel.process_map_or(None, appid, |process| {
            if let Some(grant_ptr) = process.get_grant_ptr(self.grant_num) {
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
                //            ├────────────────────
                //            │   Process Control Block
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

                // Get the GrantPointer to start. Since process.rs does not know
                // anything about the datatype of the grant, and the grant
                // memory may not yet be allocated, it can only return a `*mut
                // u8` here. We will eventually convert this to a `*mut T`.
                if let Some(untyped_grant_ptr) = process.get_grant_ptr(self.grant_num) {
                    // This is the allocator for this process when needed
                    let mut allocator = Allocator { appid: appid };

                    // If the grant pointer is NULL then the memory for the
                    // GrantRegion needs to be allocated. Otherwise, we can
                    // convert the pointer to a `*mut T` because we know we
                    // previously allocated enough memory for type T.
                    let typed_grant_pointer = if untyped_grant_ptr.is_null() {
                        unsafe {
                            // Allocate space in the process's memory for
                            // something of type `T` for the grant.
                            //
                            // Note: This allocation is intentionally never
                            // freed.  A grant region is valid once allocated
                            // for the lifetime of the process.
                            let new_region = allocator.alloc_unowned()?;

                            // Update the grant pointer in the process. Again,
                            // since the process struct does not know about the
                            // grant type we must use a `*mut u8` here.
                            process.set_grant_ptr(self.grant_num, new_region.as_ptr() as *mut u8);

                            // The allocator returns a `NonNull`, we just want
                            // the raw pointer.
                            new_region.as_ptr()
                        }
                    } else {
                        // Grant region previously allocated, just convert the
                        // pointer.
                        untyped_grant_ptr as *mut T
                    };

                    // Dereference the typed GrantPointer to make a GrantRegion
                    // reference.
                    let region = unsafe { &mut *typed_grant_pointer };

                    // Wrap the grant reference in something that knows
                    // what app its a part of.
                    let mut borrowed_region = Borrowed::new(region, appid);

                    // Call the passed in closure with the borrowed grant region.
                    let res = fun(&mut borrowed_region, &mut allocator);
                    Ok(res)
                } else {
                    Err(Error::InactiveApp)
                }
            })
    }

    pub fn each<F>(&self, fun: F)
    where
        F: Fn(&mut Owned<T>),
    {
        self.kernel.process_each(|process| {
            if let Some(grant_ptr) = process.get_grant_ptr(self.grant_num) {
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

impl<T: Default> Iterator for Iter<'_, T> {
    type Item = AppliedGrant<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // Save a local copy of grant_num so we don't have to access `self`
        // in the closure below.
        let grant_num = self.grant.grant_num;

        // Get the next `AppId` from the kernel processes array that is setup to use this grant.
        // Since the iterator itself is saved calling this function
        // again will start where we left off.
        let res = self.subiter.find(|process| {
            // We have found a candidate process that exists in the
            // processes array. Now we have to check if this grant is setup
            // for this process. If not, we have to skip it and keep
            // looking.
            if let Some(grant_ptr) = process.get_grant_ptr(grant_num) {
                !grant_ptr.is_null()
            } else {
                false
            }
        });

        // Check if our find above returned another `AppId`, or if we hit the
        // end of the iterator. If we found another app, try to access its grant
        // region.
        res.map_or(None, |process| self.grant.grant(process.appid()))
    }
}
