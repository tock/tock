//! Support for processes granting memory from their allocations to the kernel.
//!
//!
//!
//! ## Grant Overview
//!
//! Grants allow capsules to dynamically allocate memory from a process to hold
//! state on the process's behalf.
//!
//! Each capsule that wishes to do this needs to have a `Grant` type. `Grant`s
//! are created at boot, and each have a unique ID and a type `T`. This type
//! only allows the capsule to allocate memory from a process in the future. It
//! does not initially represent any allocated memory.
//!
//! When a capsule does wish to use its `Grant` to allocate memory from a
//! process, it must "enter" the `Grant` with a specific `ProcessId`. Entering a
//! `Grant` for a specific process instructs the core kernel to create an object
//! `T` in the process's memory space and provide the capsule with access to it.
//! If the `Grant` has not previously been entered for that process, the memory
//! for object `T` will be allocated from the "grant region" within the
//! kernel-accessible portion of the process's memory.
//!
//! If a `Grant` has never been entered for a process, the object `T` will _not_
//! be allocated in that process's grant region, even if the `Grant` has been
//! entered for other processes.
//!
//! The type `T` of a `Grant` is fixed in size. That is, when a `Grant` is
//! entered for a process the resulting allocated object will be the size of
//! `SizeOf<T>`. If capsules need additional process-specific memory for their
//! operation, they can use an `Allocator` to request additional memory from
//! the process's grant region.
//!
//! ```
//!                            ┌──────────────────┐
//!                            │                  │
//!                            │ Capsule          │
//!                            │                  │
//!                            └─┬────────────────┘
//!                              │ Capsules hold
//!                              │ references to
//!                              │ grants.
//!                              ▼
//!                            ┌──────────────────┐
//!                            │ Grant            │
//!                            │                  │
//!  Process Memory            │ Type: T          │
//! ┌────────────────────────┐ │ Number: 1        │
//! │  ...                   │ └───┬─────────────┬┘
//! ├────────────────────────┤     │Each Grant   │
//! │ Grant       ptr 0      │     │has a pointer│
//! │ Pointers    ptr 1 ───┐ │ ◄───┘per process. │
//! │             ...      │ │                   │
//! │             ptr N    │ │                   │
//! ├──────────────────────┼─┤                   │
//! │  ...                 │ │                   │
//! ├──────────────────────┼─┤                   │
//! │ Grant Region         │ │     When a Grant  │
//! │                      │ │     is allocated  │
//! │ ┌─────────────────┐  │ │     for a process │
//! │ │ Allocated Grant │  │ │ ◄─────────────────┘
//! │ │                 │  │ │     it uses memory
//! │ │  [ SizeOf<T> ]  │  │ │     from the grant
//! │ │                 │  │ │     region.
//! │ └─────────────────┘◄─┘ │
//! │                        │
//! │ ┌─────────────────┐    │
//! │ │ Custom Grant    │    │ ◄── Capsules can
//! │ │                 │    │     allocate extra
//! │ └─────────────────┘    │     memory if needed.
//! │                        │
//! ├─kernel_brk─────────────┤
//! │                        │
//! │ ...                    │
//! └────────────────────────┘
//! ```
//!
//! ## Grant Mechanisms and Types
//!
//! Here is an overview of the types used by grant.rs to implement the Grant
//! functionality in Tock:
//!
//! ```
//!                         ┌──────────────────────────┐
//!                         │ struct Grant<T> {        │
//!                         │   number: usize          │
//!                         │ }                        ├─┐
//! Entering a Grant for a  └──┬───────────────────────┘ │
//! process causes the         │                         │
//! memory for T to be         │ .enter(ProcessId)       │ .enter(ProcessId, fn)
//! allocated.                 ▼                         │
//!                         ┌──────────────────────────┐ │ For convenience,
//! ProcessGrant represents │ struct ProcessGrant<T> { │ │ allocating and getting
//! a Grant allocated for a │   number: usize          │ │ access to the T object
//! specific process.       │   process: &Process      │ │ is combined in one
//!                         │ }                        │ │ .enter() call.
//! A provided closure      └──┬───────────────────────┘ │
//! is given access to         │                         │
//! the underlying memory      │ .enter(fn)              │
//! where the T is stored.     ▼                         │
//!                         ┌──────────────────────────┐ │
//! Borrowed wraps the type │ struct Borrowed<T> {     │◄┘
//! and provides mutable    │   data: &mut T           │
//! access.                 │ }                        │
//!                         └──┬───────────────────────┘
//! The actual object T can    │
//! only be accessed inside    │ fn(mem: &Borrowed)
//! the closure.               ▼
//! ```

use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ops::{Deref, DerefMut};
use core::ptr::{write, NonNull};

use crate::process::{Error, ProcessDynamicGrantIdentifer, ProcessType};
use crate::sched::Kernel;
use crate::upcall::AppId;

/// This type provides access to the memory in a process's grant.
///
/// The grant must be entered to retrieve a `Borrowed` object. This object is
/// what capsules are provided so they can access the grant memory.
pub struct Borrowed<'a, T: 'a + ?Sized> {
    /// The mutable reference to the actual object type stored in the grant.
    data: &'a mut T,
}

impl<'a, T: 'a + ?Sized> Borrowed<'a, T> {
    /// Create a `Borrowed` object to provide to a capsule. Only one can be
    /// created at a time.
    fn new(data: &'a mut T) -> Borrowed<'a, T> {
        Borrowed { data: data }
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

/// A grant applied to a particular process.
///
/// This is created from a `Grant` to setup memory for that grant in a specific
/// process's grant region. An `AppliedGrant` guarantees that the memory for the
/// grant has been allocated in the process's memory.
pub struct AppliedGrant<'a, T: 'a> {
    /// The process the grant is applied to.
    process: &'a dyn ProcessType,

    /// The identifier of the Grant this is applied for.
    grant_num: usize,

    /// Used to keep the Rust type of the grant.
    _phantom: PhantomData<T>,
}

impl<'a, T: Default> AppliedGrant<'a, T> {
    /// Create an `AppliedGrant` for the given Grant and apply it to the given
    /// Process.
    ///
    /// If the grant in this process has not been setup before this will attempt
    /// to allocate the memory from the process's grant region.
    ///
    /// # Return
    ///
    /// If the grant is already allocated or could be allocated, and the process
    /// is valid, this returns `Ok(AppliedGrant)`. Otherwise it returns a
    /// relevant error.
    fn new(grant: &Grant<T>, process: &'a dyn ProcessType) -> Result<Self, Error> {
        // Here is an example of how the grants are laid out in the grant region
        // of process's memory:
        //
        // Mem. Addr.
        // 0x0040000  ┌────────────────────
        //            │   GrantPointerN [0x003FFC8]
        //            │   ...
        //            │   GrantPointer1 [0x003FFC0]
        //            │   GrantPointer0 [0x0000000 (NULL)]
        //            ├────────────────────
        //            │   Process Control Block
        // 0x003FFE0  ├────────────────────
        //            │   GrantRegionN
        // 0x003FFC8  ├────────────────────
        //            │   GrantRegion1
        // 0x003FFC0  ├────────────────────
        //            │
        //            │   --unallocated--
        //            │
        //            └────────────────────
        //
        // An array of pointers (one per possible grant region) point to where
        // the actual grant memory is allocated inside of the process. The grant
        // memory is not allocated until the actual grant region is actually
        // used.

        let grant_num = grant.grant_num;
        let appid = process.appid();

        // Check if the grant is allocated. If not, we allocate it process
        // memory first. We then create an `AppliedGrant` object for this grant.
        if let Some(is_allocated) = process.grant_is_allocated(grant_num) {
            if !is_allocated {
                // Allocate space in the process's memory for something of type
                // `T` for the grant.
                //
                // Note: This allocation is intentionally never freed. A grant
                // region is valid once allocated for the lifetime of the
                // process.
                //
                // If the grant could not be allocated this will cause the
                // `new()` function to return with an error.
                let alloc_size = size_of::<T>();
                let new_region =
                    appid
                        .kernel
                        .process_map_or(Err(Error::NoSuchApp), appid, |process| {
                            process
                                .allocate_grant(grant_num, alloc_size, align_of::<T>())
                                .map_or(Err(Error::OutOfMemory), |buf| {
                                    // Convert untyped `*mut u8` allocation to
                                    // allocated type
                                    let ptr = NonNull::cast::<T>(buf);
                                    Ok(ptr)
                                })
                        })?;

                // ### Safety
                //
                // Writing memory at an arbitrary pointer is unsafe. We are safe
                // to do this here because the following conditions are met:
                //
                // 1. The pointer address is valid. The pointer is allocated
                //    statically in process memory, and will exist for as long
                //    as the process does. The grant is only accessible while
                //    the process is still valid.
                //
                // 2. The pointer is correct aligned. The `allocate_grant()`
                //    function ensures the pointer is correctly aligned.
                unsafe {
                    // We use `ptr::write` to avoid `Drop`ping the uninitialized
                    // memory in case `T` implements the `Drop` trait.
                    write(new_region.as_ptr(), T::default());
                }
            }

            // We have ensured the grant is already allocated or was just
            // allocated, so we can create and return the `AppliedGrant` type.
            Ok(AppliedGrant {
                process: process,
                grant_num: grant_num,
                _phantom: PhantomData,
            })
        } else {
            // Cannot use the grant region in any way if the process is
            // inactive.
            Err(Error::InactiveApp)
        }
    }

    /// Return an `AppliedGrant` for a grant in a process if the process is
    /// valid and that grant has already been allocated, or `None` otherwise.
    fn new_if_allocated(grant: &Grant<T>, process: &'a dyn ProcessType) -> Option<Self> {
        if let Some(is_allocated) = process.grant_is_allocated(grant.grant_num) {
            if is_allocated {
                Some(AppliedGrant {
                    process: process,
                    grant_num: grant.grant_num,
                    _phantom: PhantomData,
                })
            } else {
                // Grant has not been allocated.
                None
            }
        } else {
            // Process is invalid.
            None
        }
    }

    /// Return the AppId of the process this AppliedGrant is applied for.
    pub fn appid(&self) -> AppId {
        self.process.appid()
    }

    /// Run a function with access to the contents of the grant region.
    ///
    /// This is "entering" the grant region, and the _only_ time when the
    /// contents of a grant region can be accessed.
    ///
    /// Note, a grant can only be entered once at a time. Attempting to call
    /// `.enter()` on a grant while it is already entered will result in a
    /// panic!()`. See the comment in `access_grant()` for more information.
    pub fn enter<F, R>(self, fun: F) -> R
    where
        F: FnOnce(&mut Borrowed<T>, &mut Allocator) -> R,
    {
        // # `unwrap()` Safety
        //
        // `access_grant()` can only return `None` if the grant is already
        // entered. Since we are asking for a panic!() if the grant is entered,
        // `access_grant()` function will never return `None`.
        self.access_grant(fun, true).unwrap()
    }

    /// Run a function with access to the contents of the grant region only if
    /// that grant region is not already entered. If the grant is already
    /// entered silently skip it.
    ///
    /// **You almost certainly should use `.enter()` rather than
    /// `.enter_if_unentered()`.**
    ///
    /// While the `.enter()` version can panic, that panic likely indicates a
    /// bug in the code and not a condition that should be handled. For example,
    /// this benign looking code is wrong:
    ///
    /// ```
    /// self.apps.enter(thisapp, |app_grant, _| {
    ///     // Update state in the grant region of `thisapp`. Also, mark that
    ///     // `thisapp` needs to run again.
    ///     app_grant.runnable = true;
    ///
    ///     // Now, check all apps to see if any are ready to run.
    ///     let mut work_left_to_do = false;
    ///     self.apps.iter().each(|other_app| {
    ///         other_app.enter(|other_app_grant, _| { // ERROR! This leads to a
    ///             if other_app_grant.runnable {      // grant being entered
    ///                 work_left_to_do = true;        // twice!
    ///             }
    ///         })
    ///     })
    /// })
    /// ```
    ///
    /// The example is wrong because it tries to iterate across all grant
    /// regions while one of them is already entered. This will lead to a grant
    /// region being entered twice which violates Rust's memory restrictions and
    /// is undefined behavior.
    ///
    /// However, since the example uses `.enter()` on the iteration, Tock will
    /// panic when the grant is entered for the second time, notifying the
    /// developer that something is wrong. The fix is to exit out of the first
    /// `.enter()` before attempting to iterate over the grant for all
    /// processes.
    ///
    /// However, if the example used `.enter_if_unentered()` in the iter loop,
    /// there would be no panic, but the already entered grant would be silently
    /// skipped. This can hide subtle bugs if the skipped grant is only relevant
    /// in certain cases.
    ///
    /// Therefore, only use `enter_if_unentered()` if you are sure you want to
    /// skip the already entered grant. Cases for this are rare.
    ///
    /// ## Return
    ///
    /// Returns `None` if the grant is already entered. Otherwise returns
    /// `Some(fun())`.
    pub fn enter_if_unentered<F, R>(self, fun: F) -> Option<R>
    where
        F: FnOnce(&mut Borrowed<T>, &mut Allocator) -> R,
    {
        self.access_grant(fun, false)
    }

    /// Access the `AppliedGrant` memory and run a closure on the process's
    /// grant memory.
    ///
    /// If `panic_on_reenter` is `true`, this will panic if the grant region is
    /// already currently entered. If `panic_on_reenter` is `false`, this will
    /// return `None` if the grant region is entered and do nothing.
    fn access_grant<F, R>(self, fun: F, panic_on_reenter: bool) -> Option<R>
    where
        F: FnOnce(&mut Borrowed<T>, &mut Allocator) -> R,
    {
        // Access the grant that is in process memory. This can only fail if
        // the grant is already entered.
        let optional_grant_ptr = self
            .process
            .enter_grant(self.grant_num)
            .map_err(|_err| {
                // If we get an error it is causeless the grant is already
                // entered. `process.enter_grant()` can fail for several
                // reasons, but only the double enter case can happen once a
                // grant has been applied. The other errors would be detected
                // earlier (i.e. before the grant can be applied).

                // If `panic_on_reenter` is false, we skip this error and do
                // nothing with this grant.
                if !panic_on_reenter {
                    return;
                }

                // If `enter_grant` fails, we panic!() to notify the developer
                // that they tried to enter the same grant twice which is
                // prohibited because it would result in two mutable references
                // existing for the same memory. This preserves type correctness
                // (but does crash the system).
                //
                // ## Explanation and Rationale
                //
                // This panic represents a tradeoff. While it is undesirable to
                // have the potential for a runtime crash in this grant region
                // code, it balances usability with type correctness. The
                // challenge is that calling `self.apps.iter()` is a common
                // pattern in capsules to access the grant region of every app
                // that is using the capsule, and sometimes it is intuitive to
                // call that inside of a `self.apps.enter(app_id, |app| {...})`
                // closure. However, `.enter()` means that app's grant region is
                // entered, and then a naive `.iter()` would re-enter the grant
                // region and cause undefined behavior. We considered different
                // options to resolve this.
                //
                // 1. Have `.iter()` only iterate over grant regions which are
                //    not entered. This avoids the bug, but could lead to
                //    unexpected behavior, as `self.apps.iter()` will do
                //    different things depending on where in a capsule it is
                //    called.
                // 2. Have the compiler detect when `.iter()` is called when a
                //    grant region has already been entered. We don't know of a
                //    viable way to implement this.
                // 3. Panic if `.iter()` is called when a grant is already
                //    entered.
                //
                // We decided on option 3 because it balances minimizing
                // surprises (`self.apps.iter()` will always iterate all grants)
                // while also protecting against the bug. We expect that any
                // code that attempts to call `self.apps.iter()` after calling
                // `.enter()` will immediately encounter this `panic!()` and
                // have to be refactored before any tests will be successful.
                // Therefore, this `panic!()` should only occur at
                // development/testing time.
                //
                // ## How to fix this error
                //
                // If you are seeing this panic, you need to refactor your
                // capsule to not call `.iter()` or `.each()` from inside a
                // `.enter()` closure. That is, you need to close the grant
                // region you are currently in before trying to iterate over all
                // grant regions.
                panic!("Attempted to re-enter a grant region.");
            })
            .ok();

        // # `unwrap()` Safety
        //
        // Only `unwrap()` if some, otherwise return early.
        let grant_ptr = if optional_grant_ptr.is_some() {
            optional_grant_ptr.unwrap()
        } else {
            return None;
        };

        // Process only holds the grant's memory, but does not know the actual type
        // of the grant. We case the type here so that the user of the grant is restricted
        // by the type system to access this memory safely.
        //
        // # Safety
        //
        // This is safe as long as the memory at grant_ptr is correctly aligned,
        // the correct size for type `T`, is only every cast as a `T`, and only
        // one reference to the object exists. We guarantee this because type
        // `T` cannot change, and we ensure the size and alignment are correct
        // when the grant is allocated. We ensure that only one reference can
        // ever exist by marking the grant entered in `enter_grant()`, and
        // subsequent calls to `enter_grant()` will fail.
        let grant = unsafe { &mut *(grant_ptr as *mut T) };

        // Setup an allocator in case the capsule needs additional memory in the
        // grant space.
        let mut allocator = Allocator {
            appid: self.process.appid(),
        };

        // Create a wrapped object that is passed to the capsule.
        let mut root = Borrowed::new(grant);

        // Allow the capsule to access the grant.
        let res = fun(&mut root, &mut allocator);

        // Now that the capsule has finished we need to "release" the grant.
        // This will mark it as no longer entered and allow the grant to be used
        // in the future.
        self.process.leave_grant(self.grant_num);

        Some(res)
    }
}

/// Grant which was dynamically allocated from the kernel-owned grant region in
/// the process's memory separately from a normal `Grant`.
///
/// A `DynamicGrant` allows a capsule to allocate additional memory on behalf of
/// a process and store the reference to the new memory in the original
/// `AppliedGrant`.
pub struct DynamicGrant<T> {
    /// An identifier for this dynamic grant within a process's grant region.
    ///
    /// Here, this is an opaque reference that Process uses to access the
    /// dynamic grant allocation. This setup ensures that Process owns the grant
    /// memory.
    identifier: ProcessDynamicGrantIdentifer,

    /// Identifier for the process where this dynamic grant is allocated.
    appid: AppId,

    /// Used to keep the Rust type of the grant.
    _phantom: PhantomData<T>,
}

impl<T> DynamicGrant<T> {
    /// Creates a new `DynamicGrant`.
    fn new(identifier: ProcessDynamicGrantIdentifer, appid: AppId) -> Self {
        DynamicGrant {
            identifier,
            appid,
            _phantom: PhantomData,
        }
    }

    /// Helper function to get the AppId from the dynamic grant.
    pub fn appid(&self) -> AppId {
        self.appid
    }

    /// Gives access to inner data within the given closure.
    ///
    /// If the app has since been restarted or crashed, or the memory is
    /// otherwise no longer present, then this function will not call the given
    /// closure, and will instead directly return `Err(Error::NoSuchApp)`.
    ///
    /// Because this function requires `&mut self`, it should be impossible to
    /// access the inner data of a given `DynamicGrant` reentrantly. Thus the
    /// reentrance detection we use for non-dynamic grants is not needed here.
    pub fn enter<F, R>(&mut self, fun: F) -> Result<R, Error>
    where
        F: FnOnce(Borrowed<'_, T>) -> R,
    {
        // Verify that the process this DynamicGrant was allocated within still
        // exists.
        self.appid
            .kernel
            .process_map_or(Err(Error::NoSuchApp), self.appid, |process| {
                // App is valid.

                // Now try to access the dynamic grant memory.
                let grant_ptr = process.enter_dynamic_grant(self.identifier)?;

                // # Safety
                //
                // `grant_ptr` must be a valid pointer and there must not exist
                // any other references to the same memory. We verify the
                // pointer is valid and aligned when the memory is allocated and
                // `DynamicGrant` is created. We are sure that there are no
                // other references because the only way to create a reference
                // is using this `enter()` function, and it can only be called
                // once (because of the `&mut self` requirement).
                let dynamic_grant = unsafe { &mut *(grant_ptr as *mut T) };
                let borrowed = Borrowed::new(dynamic_grant);
                Ok(fun(borrowed))
            })
    }
}

/// Tool for allocating additional memory regions in a process's grant region.
///
/// This is provided along with a grant so that if a capsule needs per-process
/// dynamic allocation it can allocate additional memory.
pub struct Allocator {
    /// The process the allocator will allocate memory from.
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
        let (dynamic_grant_identifier, typed_ptr) = self.alloc_raw::<T>()?;

        // # Safety
        //
        // Writing to this pointer is safe as long as the pointer is valid
        // and aligned. `alloc_raw()` guarantees these constraints are met.
        unsafe {
            // We use `ptr::write` to avoid `Drop`ping the uninitialized memory
            // in case `T` implements the `Drop` trait.
            write(typed_ptr.as_ptr(), init());
        }

        Ok(DynamicGrant::new(dynamic_grant_identifier, self.appid))
    }

    // /// Allocates a slice of n instances of a given type. Each instance is
    // /// initialized using the provided function.
    // ///
    // /// The provided function will be called exactly `n` times, and will be
    // /// passed the index it's initializing, from `0` through `num_items - 1`.
    // ///
    // /// # Panic Safety
    // ///
    // /// If `val_func` panics, the freshly allocated memory and any values
    // /// already written will be leaked.
    // pub fn alloc_n_with<T, F>(
    //     &mut self,
    //     num_items: usize,
    //     mut init: F,
    // ) -> Result<DynamicGrant<[T]>, Error>
    // where
    //     F: FnMut(usize) -> T,
    // {
    //     let (dynamic_grant_identifier, typed_ptr) = self.alloc_n_raw::<T>(num_items)?;

    //     for i in 0..num_items {
    //         // # Safety
    //         //
    //         // The allocate function guarantees that `ptr` points to memory
    //         // large enough to allocate `num_items` copies of the object.
    //         unsafe {
    //             write(typed_ptr.as_ptr().add(i), init(i));
    //         }
    //     }

    //     // // convert `NonNull<T>` to a fat pointer `NonNull<[T]>` which includes
    //     // // the length information. We do this here as initialization is more
    //     // // convenient with the non-slice ptr.
    //     // let slice_ptr = NonNull::new(slice_from_raw_parts_mut(typed_ptr.as_ptr(), num_items)).unwrap();

    //     Ok(DynamicGrant::new(dynamic_grant_identifier, self.appid))
    // }

    /// Allocates uninitialized grant memory appropriate to store a `T`.
    ///
    /// The caller must initialize the memory.
    ///
    /// Also returns a ProcessDynamicGrantIdentifer to access the memory later.
    fn alloc_raw<T>(&mut self) -> Result<(ProcessDynamicGrantIdentifer, NonNull<T>), Error> {
        self.alloc_n_raw::<T>(1)
    }

    /// Allocates space for a dynamic number of items.
    ///
    /// The caller is responsible for initializing the returned memory.
    ///
    /// Returns memory appropriate for storing `num_items` contiguous instances
    /// of `T` and a ProcessDynamicGrantIdentifer to access the memory later.
    fn alloc_n_raw<T>(
        &mut self,
        num_items: usize,
    ) -> Result<(ProcessDynamicGrantIdentifer, NonNull<T>), Error> {
        let alloc_size = size_of::<T>()
            .checked_mul(num_items)
            .ok_or(Error::OutOfMemory)?;
        self.appid
            .kernel
            .process_map_or(Err(Error::NoSuchApp), self.appid, |process| {
                process
                    .allocate_dynamic_grant(alloc_size, align_of::<T>())
                    .map_or(
                        Err(Error::OutOfMemory),
                        |(dynamic_grant_identifier, raw_ptr)| {
                            // Convert untyped `*mut u8` allocation to allocated
                            // type
                            let typed_ptr = NonNull::cast::<T>(raw_ptr);

                            Ok((dynamic_grant_identifier, typed_ptr))
                        },
                    )
            })
    }
}

/// Type for storing an object of type T in process memory that is only
/// accessible by the kernel.
pub struct Grant<T: Default> {
    /// Hold a reference to the core kernel so we can iterate processes.
    pub(crate) kernel: &'static Kernel,

    /// The identifier for this grant. Having an identifier allows the Process
    /// implementation to lookup the memory for this grant in the specific
    /// process.
    grant_num: usize,

    /// Used to keep the Rust type of the grant.
    ptr: PhantomData<T>,
}

impl<T: Default> Grant<T> {
    /// Create a new `Grant` type which allows a capsule to store
    /// process-specific data for each process in the process's memory region.
    ///
    /// This must only be called from the main kernel so that it can ensure that
    /// `grant_index` is a valid index.
    pub(crate) fn new(kernel: &'static Kernel, grant_index: usize) -> Grant<T> {
        Grant {
            kernel: kernel,
            grant_num: grant_index,
            ptr: PhantomData,
        }
    }

    /// Enter the grant for a specific process.
    ///
    /// This creates an `AppliedGrant` which is the grant region for just one
    /// process. Then, that `AppliedGrant` is entered and the provided closure
    /// is run with access to the memory in the grant region.
    pub fn enter<F, R>(&self, appid: AppId, fun: F) -> Result<R, Error>
    where
        F: FnOnce(&mut Borrowed<T>, &mut Allocator) -> R,
    {
        // Verify that this process actually exists.
        appid
            .kernel
            .process_map_or(Err(Error::NoSuchApp), appid, |process| {
                // Get the `AppliedGrant` for the process, possibly needing to
                // actually allocate the memory in the process's grant region to
                // do so. This can fail for a variety of reasons, and if so we
                // return the error to the capsule.
                let ag = AppliedGrant::new(self, process)?;

                // If we have managed to create an `AppliedGrant`, all we need
                // to do is actually access the memory and run the
                // capsule-provided closure. This can only fail if the grant is
                // already entered, at which point the kernel will panic.
                Ok(ag.enter(fun))
            })
    }

    /// Call a function on the grant for each active process if the grant has
    /// been allocated for that process.
    ///
    /// This will silently skip any process where the grant has not previously
    /// been allocated. This will also silently skip any invalid processes.
    ///
    /// Calling this function when an `AppliedGrant` for a process is currently
    /// entered will result in a panic.
    pub fn each<F>(&self, fun: F)
    where
        F: Fn(AppId, &mut Borrowed<T>),
    {
        // Create a the iterator across `AppliedGrant`s for each process.
        for ag in self.iter() {
            let appid = ag.appid();
            // Since we iterating, there is no return value we need to worry
            // about.
            ag.enter(|borrowed, _| fun(appid, borrowed));
        }
    }

    /// Get an iterator over all processes and their active grant regions for
    /// this particular grant.
    ///
    /// Calling this function when an `AppliedGrant` for a process is currently
    /// entered will result in a panic.
    pub fn iter(&self) -> Iter<T> {
        Iter {
            grant: self,
            subiter: self.kernel.get_process_iter(),
        }
    }
}

/// Type to iterate `AppliedGrant`s across processes.
pub struct Iter<'a, T: 'a + Default> {
    /// The grant type to use.
    grant: &'a Grant<T>,

    /// Iterator over valid processes.
    subiter: core::iter::FilterMap<
        core::slice::Iter<'a, Option<&'static dyn ProcessType>>,
        fn(&Option<&'static dyn ProcessType>) -> Option<&'static dyn ProcessType>,
    >,
}

impl<'a, T: Default> Iterator for Iter<'a, T> {
    type Item = AppliedGrant<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let grant = self.grant;
        // Get the next `AppId` from the kernel processes array that is setup to
        // use this grant. Since the iterator itself is saved calling this
        // function again will start where we left off.
        self.subiter
            .find_map(|process| AppliedGrant::new_if_allocated(grant, process))
    }
}
