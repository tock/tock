//! Data structure to store a list of userspace applications.

use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ops::{Deref, DerefMut};
use core::ptr::{slice_from_raw_parts_mut, write, NonNull};

use crate::callback::AppId;
use crate::callback::ProcessCallbackFactory;
use crate::process::{Error, ProcessType};
use crate::sched::Kernel;

/// Default trait for Grant contents
///
/// Compared to the Rust [`Default`] trait, this provides additional
/// information about the process the Grant is created over, as well
/// as factories for creating structures relating to a process.
pub trait GrantDefault {
    fn grant_default(
        process_id: AppId,
        callback_factory: &mut ProcessCallbackFactory,
        //appslice_factory: ProcessAppSliceFactory,
    ) -> Self;
}

/// Type that indicates a grant region has been entered and borrowed.
/// This is passed to capsules when they try to enter a grant region.
pub struct Borrowed<'a, T: 'a + ?Sized> {
    data: &'a mut T,
    appid: AppId,
}

impl<'a, T: 'a + ?Sized> Borrowed<'a, T> {
    fn new(data: &'a mut T, appid: AppId) -> Borrowed<'a, T> {
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

pub struct AppliedGrant<'a, T: 'a> {
    process: &'a dyn ProcessType,
    grant_num: usize,
    grant: &'a mut T,
    _phantom: PhantomData<T>,
}

impl<'a, T: GrantDefault> AppliedGrant<'a, T> {
    fn get_or_allocate(grant: &Grant<T>, process: &'a dyn ProcessType) -> Result<Self, Error> {
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
        // A `GrantPointer` can be set to all 1s (0xFFFFFFFFF) as a flag
        // that that grant is currently entered (aka being accessed). The actual
        // pointer address is restored when the access finishes.
        //
        // This function provides the app access to the specific
        // grant memory, and allocates the grant region in the
        // process memory if needed.

        // Get the GrantPointer to start. Since process.rs does not know
        // anything about the datatype of the grant, and the grant
        // memory may not yet be allocated, it can only return a `*mut
        // u8` here. We will eventually convert this to a `*mut T`.
        let driver_num = grant.driver_num;
        let grant_num = grant.grant_num;
        let appid = process.appid();
        if let Some(untyped_grant_ptr) = process.get_grant_ptr(grant_num) {
            // If the grant pointer is NULL then the memory for the
            // GrantRegion needs to be allocated. If the grant pointer is all 1s
            // then the grant is already enetered and we return an error. Otherwise, we can
            // convert the pointer to a `*mut T` because we know we
            // previously allocated enough memory for type T.
            unsafe {
                let typed_grant_pointer = if untyped_grant_ptr.is_null() {
                    // Allocate space in the process's memory for
                    // something of type `T` for the grant.
                    //
                    // Note: This allocation is intentionally never
                    // freed.  A grant region is valid once allocated
                    // for the lifetime of the process.
                    let alloc_size = size_of::<T>();
                    let new_region =
                        appid
                            .kernel
                            .process_map_or(Err(Error::NoSuchApp), appid, |process| {
                                process.alloc(alloc_size, align_of::<T>()).map_or(
                                    Err(Error::OutOfMemory),
                                    |buf| {
                                        // Convert untyped `*mut u8` allocation to allocated type
                                        let ptr = NonNull::cast::<T>(buf);

                                        Ok(ptr)
                                    },
                                )
                            })?;

                    // We may only ever have at most one Callback per
                    // (driver_num, callback_num, process_id) tuple in
                    // the kernel. To uphold this guarantee we only
                    // allow creating Callbacks in the Grant
                    // initialization and assume that any driver has
                    // only a single Grant region associated with it.
                    let mut callback_factory =
                        ProcessCallbackFactory::new(process.appid(), driver_num);

                    // We use `ptr::write` to avoid `Drop`ping the uninitialized memory in
                    // case `T` implements the `Drop` trait.
                    write(
                        new_region.as_ptr(),
                        T::grant_default(process.appid(), &mut callback_factory),
                    );

                    // Update the grant pointer in the process. Again,
                    // since the process struct does not know about the
                    // grant type we must use a `*mut u8` here.
                    process.set_grant_ptr(grant_num, new_region.as_ptr() as *mut u8);

                    // The allocator returns a `NonNull`, we just want
                    // the raw pointer.
                    new_region.as_ptr()
                } else if untyped_grant_ptr == (!0 as *mut u8) {
                    // Grant region currently entered, cannot enter again.
                    return Err(Error::AlreadyInUse);
                } else {
                    // Grant region previously allocated, just convert the
                    // pointer.
                    untyped_grant_ptr as *mut T
                };

                Ok(AppliedGrant {
                    process: process,
                    grant_num: grant.grant_num,
                    grant: &mut *(typed_grant_pointer as *mut T),
                    _phantom: PhantomData,
                })
            }
        } else {
            Err(Error::InactiveApp)
        }
    }

    /// Return an `AppliedGrant` for a grant in a process if that
    /// grant has already been allocated.
    ///
    /// On `Err()`, returns `Err(None)` if the grant has not been allocated
    /// for this process. Returns `Err(Some(Error))` with an appropriate
    /// error otherwise.
    fn get_if_allocated(
        grant: &Grant<T>,
        process: &'a dyn ProcessType,
    ) -> Result<Self, Option<Error>> {
        process
            .get_grant_ptr(grant.grant_num)
            .map_or(Err(None), |grant_ptr| {
                if grant_ptr.is_null() {
                    // Grant has not been allocated.
                    Err(None)
                } else if grant_ptr == (!0 as *mut u8) {
                    // A grant pointer set to 0xFFFFFFFF is a flag signaling
                    // that the grant has already been entered.
                    Err(Some(Error::AlreadyInUse))
                } else {
                    Ok(AppliedGrant {
                        process: process,
                        grant_num: grant.grant_num,
                        grant: unsafe { &mut *(grant_ptr as *mut T) },
                        _phantom: PhantomData,
                    })
                }
            })
    }

    /// Run a function with access to the contents of the grant region.
    /// This is "entering" the grant region, and the _only_ time when
    /// the contents of a grant region can be accessed.
    pub fn enter<F, R>(self, fun: F) -> R
    where
        F: FnOnce(&mut Borrowed<T>, &mut Allocator) -> R,
    {
        let mut allocator = Allocator {
            appid: self.process.appid(),
        };
        let mut root = Borrowed::new(self.grant, self.process.appid());
        // Mark the grant region as entered by replacing its grant pointer with
        // all 1s. This allows us to check whether the grant region is already entered.
        unsafe {
            self.process.set_grant_ptr(self.grant_num, !0 as *mut u8);
        }
        let res = fun(&mut root, &mut allocator);
        // Restore the grant pointer indicating that we are no longer accessing
        // the grant.
        unsafe {
            self.process
                .set_grant_ptr(self.grant_num, root.data as *mut _ as *mut u8);
        }
        res
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
    /// instead directly return `Err(Error::NoSuchApp)`.
    ///
    /// Because this function requires `&mut self`, it should be impossible to access
    /// the inner data of a given `DynamicGrant` reentrantly. Thus the reentrance detection
    /// we use for non-dynamic grants is not needed here.
    pub fn enter<F, R>(&mut self, fun: F) -> Result<R, Error>
    where
        F: FnOnce(Borrowed<'_, T>) -> R,
    {
        self.appid
            .kernel
            .process_map_or(Err(Error::NoSuchApp), self.appid, |_| {
                let data = unsafe { self.data.as_mut() };
                let borrowed = Borrowed::new(data, self.appid);
                Ok(fun(borrowed))
            })
    }
}

pub struct Allocator {
    appid: AppId,
}

impl Allocator {
    /// Allocates a new dynamic grant initialized using the given closure.
    ///
    /// The closure will be called exactly once, and the result will be used to
    /// initialize the owned value.
    ///
    /// This interface was chosen instead of a simple `alloc(val)` as it's
    /// much more likely to optimize out all stack intermediates. This
    /// helps to prevent stack overflows when allocating large values.
    ///
    /// # Panic Safety
    ///
    /// If `init` panics, the freshly allocated memory may leak.
    pub fn alloc_with<T, F>(&mut self, init: F) -> Result<DynamicGrant<T>, Error>
    where
        F: FnOnce() -> T,
    {
        unsafe {
            let ptr = self.alloc_raw()?;

            // We use `ptr::write` to avoid `Drop`ping the uninitialized memory in
            // case `T` implements the `Drop` trait.
            write(ptr.as_ptr(), init());

            Ok(DynamicGrant::new(ptr, self.appid))
        }
    }

    /// Allocates a slice of n instances of a given type. Each instance is
    /// initialized using the provided function.
    ///
    /// The provided function will be called exactly `n` times, and will be
    /// passed the index it's initializing, from `0` through `num_items - 1`.
    ///
    /// # Panic Safety
    ///
    /// If `val_func` panics, the freshly allocated memory and any values
    /// already written will be leaked.
    pub fn alloc_n_with<T, F>(
        &mut self,
        num_items: usize,
        mut val_func: F,
    ) -> Result<DynamicGrant<[T]>, Error>
    where
        F: FnMut(usize) -> T,
    {
        unsafe {
            let ptr = self.alloc_n_raw::<T>(num_items)?;

            for i in 0..num_items {
                write(ptr.as_ptr().add(i), val_func(i));
            }

            // convert `NonNull<T>` to a fat pointer `NonNull<[T]>` which includes
            // the length information. We do this here as initialization is more
            // convenient with the non-slice ptr.
            let slice_ptr =
                NonNull::new(slice_from_raw_parts_mut(ptr.as_ptr(), num_items)).unwrap();

            Ok(DynamicGrant::new(slice_ptr, self.appid))
        }
    }

    /// Allocates uninitialized memory appropriate to store a `T`, and returns a
    /// pointer to said memory. The caller is responsible for both initializing the
    /// returned memory, and dropping it properly when finished.
    unsafe fn alloc_raw<T>(&mut self) -> Result<NonNull<T>, Error> {
        self.alloc_n_raw::<T>(1)
    }

    /// Allocates space for a dynamic number of items. The caller is responsible
    /// for initializing and freeing returned memory. Returns memory appropriate
    /// for storing `num_items` contiguous instances of `T`.
    unsafe fn alloc_n_raw<T>(&mut self, num_items: usize) -> Result<NonNull<T>, Error> {
        let alloc_size = size_of::<T>()
            .checked_mul(num_items)
            .ok_or(Error::OutOfMemory)?;
        self.appid
            .kernel
            .process_map_or(Err(Error::NoSuchApp), self.appid, |process| {
                process
                    .alloc(alloc_size, align_of::<T>())
                    .map_or(Err(Error::OutOfMemory), |buf| {
                        // Convert untyped `*mut u8` allocation to allocated type
                        let ptr = NonNull::cast::<T>(buf);

                        Ok(ptr)
                    })
            })
    }
}

/// Region of process memory reserved for the kernel.
pub struct Grant<T: GrantDefault> {
    pub(crate) kernel: &'static Kernel,
    driver_num: u32,
    grant_num: usize,
    ptr: PhantomData<T>,
}

impl<T: GrantDefault> Grant<T> {
    pub(crate) fn new(kernel: &'static Kernel, driver_num: u32, grant_index: usize) -> Grant<T> {
        Grant {
            kernel: kernel,
            driver_num: driver_num,
            grant_num: grant_index,
            ptr: PhantomData,
        }
    }

    pub fn enter<F, R>(&self, appid: AppId, fun: F) -> Result<R, Error>
    where
        F: FnOnce(&mut Borrowed<T>, &mut Allocator) -> R,
    {
        appid
            .kernel
            .process_map_or(Err(Error::NoSuchApp), appid, |process| {
                let ag = AppliedGrant::get_or_allocate(self, process)?;
                Ok(ag.enter(fun))
            })
    }

    /// Call a function on every active grant region.
    /// Calling this function when a grant region is currently entered
    /// will lead to a panic.
    pub fn each<F>(&self, fun: F)
    where
        F: Fn(&mut Borrowed<T>),
    {
        for ag in self.iter() {
            ag.enter(|borrowed, _| fun(borrowed));
        }
    }

    /// Get an iterator over all processes and their active grant regions for
    /// this particular grant.
    /// Calling this function when a grant region is currently entered
    /// will lead to a panic.
    pub fn iter(&self) -> Iter<T> {
        Iter {
            grant: self,
            subiter: self.kernel.get_process_iter(),
            skip_entered_grants: false,
        }
    }

    /// Get an iterator over all processes and their active grant regions for
    /// this particular grant, except grant regions that are already currently
    /// entered.
    pub fn iter_unentered_grants(&self) -> Iter<T> {
        Iter {
            grant: self,
            subiter: self.kernel.get_process_iter(),
            skip_entered_grants: true,
        }
    }
}

pub struct Iter<'a, T: 'a + GrantDefault> {
    grant: &'a Grant<T>,
    subiter: core::iter::FilterMap<
        core::slice::Iter<'a, Option<&'static dyn ProcessType>>,
        fn(&Option<&'static dyn ProcessType>) -> Option<&'static dyn ProcessType>,
    >,
    /// Whether this iterator must visit every grant region, or if
    /// it should skip grant regions which are already entered.
    /// A grant region cannot be entered twice. Therefore, if a capsule
    /// tries to iterate over all grant regions while inside of a grant region,
    /// the kernel will `panic!()` to prevent double entry. However, if this
    /// is set, then the iterator will skip over already entered grant regions,
    /// avoiding a panic but potentially creating subtly unintended code.
    skip_entered_grants: bool,
}

impl<'a, T: GrantDefault> Iterator for Iter<'a, T> {
    type Item = AppliedGrant<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let grant = self.grant;
        // Get the next `AppId` from the kernel processes array that is setup to use this grant.
        // Since the iterator itself is saved calling this function
        // again will start where we left off.
        let skip_entered_grants = self.skip_entered_grants;
        self.subiter.find_map(
            |process| match AppliedGrant::get_if_allocated(grant, process) {
                Ok(ag) => Some(ag),
                Err(None) => None,
                Err(Some(_err)) => {
                    if skip_entered_grants {
                        // This iter was created from `iter_unentered_grants()`,
                        // so we skip entered grants to avoid double entry.
                        None
                    } else {
                        // Panic if the capsule asked us to enter an already entered
                        // grant region. This preserves type correctness (but does crash
                        // the system).
                        //
                        // This panic represents a tradeoff. While it is undesirable to have the
                        // potential for a runtime crash in this grant region code, it balances
                        // usability with type correctness. The challenge is that calling `self.apps.iter()`
                        // is a common pattern in capsules to access the grant region of every
                        // app that is using the capsule, and sometimes it is intuitive to call that
                        // inside of a `self.apps.enter(app_id, |app| {...})` closure. However, `.enter()`
                        // means that app's grant region is entered, and then a naive `.iter()` would
                        // re-enter the grant region and cause undefined behavior. We considered
                        // different options to resolve this.
                        //
                        // 1. Have `.iter()` only iterate over grant regions which are not entered.
                        //    This avoids the bug, but could lead to unexpected behavior, as `self.apps.iter()` will
                        //    do different things depending on where in a capsule it is called.
                        // 2. Have the compiler detect when `.iter()` is called when a grant region has
                        //    already been entered. We don't know of a viable way to implement this.
                        // 3. Panic if `.iter()` is called when a grant is already entered.
                        //
                        // We decided on option 3 because it balances minimizing surprises (`self.apps.iter()`
                        // will always iterate all grants) while also protecting against the bug. We expect
                        // that any code that attempts to call `self.apps.iter()` after calling `.enter()` will
                        // immediately encounter this `panic!()` and have to be refactored before any tests
                        // will be successful. Therefore, this `panic!()` should only occur at development/testing
                        // time.
                        //
                        // ## How to fix this error
                        //
                        // If you are seeing this panic, you need to refactor your capsule to not
                        // call `.iter()` or `.each()` from inside a `.enter()` closure. That is, you
                        // need to close the grant region you are currently in before trying to iterate
                        // over all grant regions.
                        panic!("Attempted to re-enter a grant region.")
                    }
                }
            },
        )
    }
}
