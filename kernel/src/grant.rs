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
//! Upcalls are stored in the dynamically allocated grant for a particular
//! Driver as well. Upcalls are stored outside of the `T` object to enable the
//! kernel to manage them and ensure the upcall swapping guarantees are met.
//!
//! The type `T` of a `Grant` is fixed in size and the number of upcalls
//! associated with a grant is fixed. That is, when a `Grant` is entered for a
//! process the resulting allocated object will be the size of `SizeOf<T>` plus
//! the size for the upcalls. If capsules need additional process-specific
//! memory for their operation, they can use an `Allocator` to request
//! additional memory from the process's grant region.
//!
//! ```text,ignore
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
//! ┌────────────────────────┐ │ grant_num: 1     │
//! │                        │ │ driver_num: 0x4  │
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
//! │ │─────────────────│  │ │     region.
//! │ │ Padding         │  │ │
//! │ │─────────────────│  │ │
//! │ │ Upcall Table    │  │ │
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
//! ```text,ignore
//!                         ┌──────────────────────────┐
//!                         │ struct Grant<T, NUM_UP> {│
//!                         │   driver_num: usize      │
//!                         │   grant_num: usize       │
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
//! GrantData wraps the     │ struct GrantData<T>   {  │◄┘
//! type and provides       │   data: &mut T           │
//! mutable access.         │ }                        │
//! GrantUpcallTable        │ struct GrantUpcallTable {│
//! provides access to      │   upcalls: [SavedUpcall] │
//! scheduling upcalls      │ }                        │
//!                         └──┬───────────────────────┘
//! The actual object T can    │
//! only be accessed inside    │ fn(mem: &GrantData, upcalls: &GrantUpcallTable)
//! the closure.               ▼
//! ```

use core::cmp;
use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ops::{Deref, DerefMut};
use core::ptr::{write, NonNull};
use core::slice;

use crate::kernel::Kernel;
use crate::process::{Error, Process, ProcessCustomGrantIdentifer, ProcessId};
use crate::upcall::{Upcall, UpcallError, UpcallId};
use crate::ErrorCode;

/// This GrantData object provides access to the memory allocated for a grant
/// for a specific process.
///
/// The GrantData type is templated on T, the actual type of the object in the
/// grant. GrantData holds a mutable reference to the type, allowing users
/// access to the object in process memory.
///
/// Capsules gain access to a GrantData object by calling `Grant::enter()`.
pub struct GrantData<'a, T: 'a + ?Sized> {
    /// The mutable reference to the actual object type stored in the grant.
    data: &'a mut T,
}

impl<'a, T: 'a + ?Sized> GrantData<'a, T> {
    /// Create a `GrantData` object to provide access to the actual object
    /// allocated for a process.
    ///
    /// Only one can GrantData per underlying object can be created at a time.
    /// Otherwise, there would be multiple mutable references to the same object
    /// which is undefined behavior.
    fn new(data: &'a mut T) -> GrantData<'a, T> {
        GrantData { data: data }
    }
}

impl<'a, T: 'a + ?Sized> Deref for GrantData<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T: 'a + ?Sized> DerefMut for GrantData<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

/// This GrantUpcallTable object provides a handle to access Upcalls stored on
/// behalf of a particular grant/driver.
///
/// Capsules gain access to a GrantUpcallTable object by calling
/// `Grant::enter()`. From there, they can schedule upcalls. No other access to
/// upcalls is provided.
///
/// It is expected that this type will only exist as a short-lived stack
/// allocation, so its size is not a significant concern.
pub struct GrantUpcallTable<'a> {
    /// The mutable reference to the actual object type stored in the grant.
    upcalls: &'a [SavedUpcall],

    /// We need to keep track of the driver number so we can properly identify
    /// the Upcall that is called. We need to keep track of its source so we can
    /// remove it if the Upcall is unsubscribed.
    driver_num: usize,

    /// A reference to the process that these upcalls are for. This is used for
    /// actually scheduling the upcalls.
    process: &'a dyn Process,
}

impl<'a> GrantUpcallTable<'a> {
    /// Create a `GrantUpcallTable` object to provide a handle for capsules to
    /// call Upcalls.
    fn new(
        upcalls: &'a [SavedUpcall],
        driver_num: usize,
        process: &'a dyn Process,
    ) -> GrantUpcallTable<'a> {
        Self {
            upcalls,
            driver_num,
            process,
        }
    }

    /// Schedule the specified upcall for the process with r0, r1, r2 as
    /// provided values.
    ///
    /// Capsules call this function to schedule upcalls, and upcalls are
    /// identified by the `subscribe_num`, which must match the subscribe number
    /// used when the upcall was originally subscribed by a process.
    /// `subscribe_num`s are indexed starting at zero.
    pub fn schedule_upcall(
        &self,
        subscribe_num: usize,
        r0: usize,
        r1: usize,
        r2: usize,
    ) -> Result<(), UpcallError> {
        // Implement `self.upcalls[subscribe_num]` without a chance of a panic.
        self.upcalls.get(subscribe_num).map_or(
            Err(UpcallError::InvalidSubscribeNum),
            |saved_upcall| {
                // We can create an `Upcall` object based on what is stored in
                // the process grant and use that to add the upcall to the
                // pending array for the process.
                let mut upcall = Upcall::new(
                    self.process.processid(),
                    UpcallId {
                        subscribe_num,
                        driver_num: self.driver_num,
                    },
                    saved_upcall.appdata,
                    saved_upcall.fn_ptr,
                );
                upcall.schedule(self.process, r0, r1, r2)
            },
        )
    }
}

/// A minimal representation of an upcall, used for storing an upcall
/// in a process' grant table without wasting memory duplicating information
/// such as process ID.
#[repr(C)]
pub(crate) struct SavedUpcall {
    pub(crate) appdata: usize,
    pub(crate) fn_ptr: Option<NonNull<()>>,
}

/// Subscribe to an upcall by saving the upcall in the grant region for the
/// process and returning the existing upcall for the same UpcallId.
pub(crate) fn subscribe(
    process: &dyn Process,
    upcall: Upcall,
) -> Result<Upcall, (Upcall, ErrorCode)> {
    let grant_num = match process.lookup_grant_from_driver_num(upcall.upcall_id.driver_num) {
        Ok(grant_num) => grant_num,
        Err(e) => return Err((upcall, e.into())),
    };

    // Check if the grant has been allocated, and if not we cannot handle the
    // subscribe call.
    if let Some(is_allocated) = process.grant_is_allocated(grant_num) {
        if !is_allocated {
            return Err((upcall, ErrorCode::NOMEM));
        }
    } else {
        // Process is no longer active, this case will never happen.
        return Err((upcall, ErrorCode::FAIL));
    }

    // Return early if no grant.
    let grant_ptr = match process.enter_grant(grant_num) {
        Ok(grant_ptr) => grant_ptr,
        Err(_) => return Err((upcall, ErrorCode::NOMEM)),
    };

    // The number of upcalls is stored first.
    //
    // # Safety
    //
    // This is safe because of how we created the grant region that starts at
    // this pointer. The grant structure does not change once it has been
    // allocated, and if we can enter the grant we know it must be allocated. We
    // verified the pointer is correctly aligned and that the first value in the
    // grant is the `usize` sized number of upcalls.
    let num_upcalls = unsafe { *(grant_ptr as *const usize) };

    // Create the saved upcalls slice from the grant memory.
    //
    // # Safety
    //
    // This is safe because of how the grant was initially allocated and that
    // because we were able to enter the grant the grant region must be valid
    // and initialized. We increment past the usize length and the next memory
    // is a slice of `SavedUpcall`s. We verified pointer alignment at
    // initialization.
    let saved_upcalls_slice = unsafe {
        slice::from_raw_parts_mut(
            grant_ptr.add(size_of::<usize>()) as *mut SavedUpcall,
            num_upcalls,
        )
    };

    // Index into the saved upcall slice to get the old upcall. Use .get in case
    // userspace passed us a bad subscribe number.
    let rval = match saved_upcalls_slice.get_mut(upcall.upcall_id.subscribe_num) {
        Some(saved_upcall) => {
            // Create an `Upcall` object with the old saved upcall.
            let old_upcall = Upcall::new(
                process.processid(),
                upcall.upcall_id,
                saved_upcall.appdata,
                saved_upcall.fn_ptr,
            );

            // Overwrite the saved upcall with the new upcall.
            saved_upcall.appdata = upcall.appdata;
            saved_upcall.fn_ptr = upcall.fn_ptr;

            // Success!
            Ok(old_upcall)
        }
        None => Err((upcall, ErrorCode::INVAL)),
    };

    // Now that we have finished modifying the grant region we need to "release"
    // the grant.
    process.leave_grant(grant_num);

    rval
}

/// An instance of a grant allocated for a particular process.
///
/// `ProcessGrant` is a handle to an instance of a grant that has been allocated
/// in a specific process's grant region. A `ProcessGrant` guarantees that the
/// memory for the grant has been allocated in the process's memory.
///
/// This is created from a `Grant` when that grant is entered for a specific
/// process.
pub struct ProcessGrant<'a, T: 'a, const NUM_UPCALLS: usize> {
    /// The process the grant is applied to.
    ///
    /// We use a reference here because instances of `ProcessGrant` are very
    /// short lived. They only exist while a `Grant` is being entered, so we can
    /// be sure the process still exists while a `ProcessGrant` exists. No
    /// `ProcessGrant` can be stored.
    process: &'a dyn Process,

    /// The syscall driver number this grant is associated with.
    driver_num: usize,

    /// The identifier of the Grant this is applied for.
    grant_num: usize,

    /// Used to keep the Rust type of the grant.
    _phantom: PhantomData<T>,
}

impl<'a, T: Default, const NUM_UPCALLS: usize> ProcessGrant<'a, T, NUM_UPCALLS> {
    /// Create a `ProcessGrant` for the given Grant in the given Process's grant
    /// region.
    ///
    /// If the grant in this process has not been setup before this will attempt
    /// to allocate the memory from the process's grant region.
    ///
    /// # Return
    ///
    /// If the grant is already allocated or could be allocated, and the process
    /// is valid, this returns `Ok(ProcessGrant)`. Otherwise it returns a
    /// relevant error.
    fn new(grant: &Grant<T, NUM_UPCALLS>, processid: ProcessId) -> Result<Self, Error> {
        // Moves non-generic code from new() into non-generic function to reduce
        // code bloat from the generic function being monomorphized, as it is
        // common to have over 50 copies of Grant::enter() in a Tock kernel (and
        // thus 50+ copies of this function). The returned Option indicates if
        // the returned pointer still needs to be initialized (in the case where
        // the grant was only just allocated).
        fn new_inner<'a>(
            grant_num: usize,
            driver_num: usize,
            grant_t_size: usize,
            grant_t_align: usize,
            num_upcalls: usize,
            processid: ProcessId,
        ) -> Result<(Option<NonNull<u8>>, &'a dyn Process), Error> {
            // Here is an example of how the grants are laid out in the grant
            // region of process's memory:
            //
            // Mem. Addr.
            // 0x0040000  ┌────────────────────────────────────
            //            │   DriverNumN    [0x1]
            //            │   GrantPointerN [0x003FFC8]
            //            │   ...
            //            │   DriverNum1    [0x60000]
            //            │   GrantPointer1 [0x003FFC0]
            //            │   DriverNum0
            //            │   GrantPointer0 [0x0000000 (NULL)]
            //            ├────────────────────────────────────
            //            │   Process Control Block
            // 0x003FFE0  ├────────────────────────────────────  Grant Region ┐
            //            │   GrantDataN                                      │
            // 0x003FFC8  ├────────────────────────────────────               │
            //            │   GrantData1                                      ▼
            // 0x003FF20  ├────────────────────────────────────
            //            │
            //            │   --unallocated--
            //            │
            //            └────────────────────────────────────
            //
            // An array of pointers (one per possible grant region) point to
            // where the actual grant memory is allocated inside of the process.
            // The grant memory is not allocated until the actual grant region
            // is actually used.

            let process = processid
                .kernel
                .get_process(processid)
                .ok_or(Error::NoSuchApp)?;

            // Check if the grant is allocated. If not, we allocate it process
            // memory first. We then create an `ProcessGrant` object for this
            // grant.
            if let Some(is_allocated) = process.grant_is_allocated(grant_num) {
                if !is_allocated {
                    // Allocate space in the process's memory for enough space
                    // for upcalls and something of type `T` for the grant.
                    //
                    // Here is an example layout of the grant allocation:
                    //
                    // Mem. Addr.
                    // 0x003FFC8  ┌────────────────────────────────────  G
                    //            │   T                                  r
                    // 0x003FFxx  ├  ─────────────────────────           a
                    //            │   Padding    (ensure T alignment)    n
                    // 0x003FFxx  ├  ─────────────────────────           t
                    //            │   SavedUpcallN                       M
                    //            │   ...                                e
                    //            │   SavedUpcall1                       m
                    //            │   SavedUpcall0                       o
                    // 0x003FF24  ├  ─────────────────────────           r
                    //            │   NumUpcalls (usize)                 y
                    // 0x003FF20  └────────────────────────────────────  1
                    //
                    // Note: This allocation is intentionally never freed. A
                    // grant region is valid once allocated for the lifetime of
                    // the process.
                    //
                    // If the grant could not be allocated this will cause the
                    // `new()` function to return with an error.

                    // For the upcalls we need one word for the number of
                    // upcalls, and then that many SavedUpcalls.
                    let upcalls_size =
                        size_of::<usize>() + (num_upcalls * size_of::<SavedUpcall>());

                    // As the number of upcalls comes first we need to make sure
                    // the num_upcalls usize is properly aligned. Then, we
                    // assume SavedUpcall is also properly aligned to the same
                    // alignment, and can go immediately after the num_upcalls
                    // usize. If that were to ever not be true this alignment
                    // and padding calculation would be wrong.
                    let upcalls_align = align_of::<usize>();

                    // Calculate the padding needed between the upcalls data and
                    // T such that T will be properly aligned assuming the grant
                    // starts at the correct alignment for an object of type T.
                    let upcalls_padding = grant_t_align - (upcalls_size % grant_t_align);

                    // Calculate the alignment to use for both the upcalls and
                    // T.
                    let alloc_align = cmp::max(upcalls_align, grant_t_align);

                    // Now we can calculate the entire size of the grant.
                    let alloc_size = upcalls_size + upcalls_padding + grant_t_size;

                    let (ptr_upcall_count, optional_ptr_first_upcall, raw_ptr_grant_nn) = process
                        .allocate_grant(grant_num, driver_num, alloc_size, alloc_align)
                        .map_or(Err(Error::OutOfMemory), |buf| {
                            // Number of upcalls.
                            let ptr_upcall_count = NonNull::cast::<usize>(buf);

                            // Upcall array.

                            // Only create the pointer to the first upcall if we
                            // actually have memory for the SavedUpcall to
                            // exist.
                            let optional_ptr_first_upcall = if num_upcalls > 0 {
                                // # Safety
                                //
                                // It is safe to construct a *u8 pointer to the
                                // start of the upcalls array because we ensured
                                // that the memory is both valid and aligned by
                                // performing the allocation.
                                let raw_ptr_upcalls =
                                    unsafe { buf.as_ptr().add(size_of::<usize>()) };
                                // # Safety
                                //
                                // We know that `raw_ptr_upcalls` is not null
                                // because it exists within a successful grant
                                // allocation.
                                let raw_ptr_upcalls_nn =
                                    unsafe { NonNull::new_unchecked(raw_ptr_upcalls) };
                                // We only construct a pointer to the first
                                // SavedUpcall in the array because the memory
                                // is not initialized yet. Also we know that
                                // there will be at least one SavedUpcall in the
                                // array. We do not create a slice because the
                                // memory is not initialized.
                                let ptr_first_upcall =
                                    NonNull::cast::<SavedUpcall>(raw_ptr_upcalls_nn);

                                Some(ptr_first_upcall)
                            } else {
                                None
                            };

                            // Get raw pointer to grant type T so that only
                            // remaining step in outer (generic) function it to
                            // cast the pointer to a pointer to T and initialize
                            // it if needed.

                            // # Safety
                            //
                            // This is safe because we ensure that this pointer
                            // remains in valid memory because of the allocation
                            // we just completed.
                            let raw_ptr_grant = unsafe {
                                buf.as_ptr().add(
                                    size_of::<usize>()
                                        + upcalls_padding
                                        + (num_upcalls * size_of::<SavedUpcall>()),
                                )
                            };
                            // # Safety
                            //
                            // We know that `raw_ptr_grant` is not null because
                            // it exists within a successful grant allocation.
                            let raw_ptr_grant_nn = unsafe { NonNull::new_unchecked(raw_ptr_grant) };

                            Ok((
                                ptr_upcall_count,
                                optional_ptr_first_upcall,
                                raw_ptr_grant_nn,
                            ))
                        })?;

                    // Initialize the grant allocation and its various fields.

                    // Number of upcalls.
                    //
                    // # Safety
                    //
                    // Writing memory at an arbitrary pointer is unsafe. We are
                    // safe to do this here because the following conditions are
                    // met:
                    //
                    // 1. The pointer address is valid. The pointer is allocated
                    //    statically in process memory, and will exist for as
                    //    long as the process does. The grant is only accessible
                    //    while the process is still valid.
                    //
                    // 2. The pointer is correctly aligned because we calculated
                    //    the alignment before calling `allocate_grant()` which
                    //    ensures the pointer is correctly aligned.
                    unsafe {
                        // Insert length of upcalls.
                        write(ptr_upcall_count.as_ptr(), num_upcalls);
                    }

                    // SavedUpcalls
                    //
                    // Only try to initialize upcalls if this grant actually has
                    // any.
                    optional_ptr_first_upcall.map(|ptr_first_upcall| {
                        // Initialize the SavedUpcalls in an explicit loop. We
                        // do not use a slice because before this runs the
                        // SavedUpcalls are not initialized and creating a slice
                        // to uninitialized memory is not safe.
                        for i in 0..num_upcalls {
                            // # Safety
                            //
                            // This is safe because we have allocated enough
                            // space for `num_upcalls` and that each SavedUpcall
                            // is at least aligned to `usize` bytes.
                            let ptr_upcall = unsafe { ptr_first_upcall.as_ptr().add(i) };
                            // # Safety
                            //
                            // This is safe because the pointer is valid, aligned,
                            // and will live as long as the process does.
                            unsafe {
                                write(
                                    ptr_upcall,
                                    SavedUpcall {
                                        appdata: 0,
                                        fn_ptr: None,
                                    },
                                );
                            }
                        }
                    });
                    // If we got here, we allocated space for a T, and outer
                    // (generic) function must initialize it.
                    Ok((Some(raw_ptr_grant_nn), process))
                } else {
                    // T was already allocated, outer function should not
                    // initialize T.
                    Ok((None, process))
                }
            } else {
                // Cannot use the grant region in any way if the process is
                // inactive.
                Err(Error::InactiveApp)
            }
        }

        // Handle the bulk of the work in a function which is not templated.
        let (opt_raw_grant_ptr_nn, process) = new_inner(
            grant.grant_num,
            grant.driver_num,
            size_of::<T>(),
            align_of::<T>(),
            NUM_UPCALLS,
            processid,
        )?;

        // We can now do the initialization of T object if necessary.
        match opt_raw_grant_ptr_nn {
            Some(allocated_ptr) => {
                // Grant type T
                //
                // # Safety
                //
                // This is safe because:
                //
                // 1. The pointer address is valid. The pointer is allocated
                //    statically in process memory, and will exist for as long
                //    as the process does. The grant is only accessible while
                //    the process is still valid.
                //
                // 2. The pointer is correctly aligned. The newly allocated
                //    grant is aligned for type T, and there is padding inserted
                //    between the upcall array and the T object such that the T
                //    object starts a multiple of `align_of<T>` from the
                //    beginning of the allocation.
                unsafe {
                    // Convert untyped `*mut u8` allocation to
                    // allocated type
                    let new_region = NonNull::cast::<T>(allocated_ptr);
                    // We use `ptr::write` to avoid `Drop`ping the uninitialized
                    // memory in case `T` implements the `Drop` trait.
                    write(new_region.as_ptr(), T::default());
                }
            }
            None => {} // Case if grant was already allocated.
        }

        // We have ensured the grant is already allocated or was just
        // allocated, so we can create and return the `ProcessGrant` type.
        Ok(ProcessGrant {
            process: process,
            driver_num: grant.driver_num,
            grant_num: grant.grant_num,
            _phantom: PhantomData,
        })
    }

    /// Return an `ProcessGrant` for a grant in a process if the process is
    /// valid and that process grant has already been allocated, or `None`
    /// otherwise.
    fn new_if_allocated(grant: &Grant<T, NUM_UPCALLS>, process: &'a dyn Process) -> Option<Self> {
        if let Some(is_allocated) = process.grant_is_allocated(grant.grant_num) {
            if is_allocated {
                Some(ProcessGrant {
                    process: process,
                    driver_num: grant.driver_num,
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

    /// Return the ProcessId of the process this ProcessGrant is associated with.
    pub fn processid(&self) -> ProcessId {
        self.process.processid()
    }

    /// Run a function with access to the memory in the related process for the
    /// related Grant. This also provides access to any associated Upcalls
    /// stored with the grant.
    ///
    /// This is "entering" the grant region, and the _only_ time when the
    /// contents of a grant region can be accessed.
    ///
    /// Note, a grant can only be entered once at a time. Attempting to call
    /// `.enter()` on a grant while it is already entered will result in a
    /// panic!()`. See the comment in `access_grant()` for more information.
    pub fn enter<F, R>(self, fun: F) -> R
    where
        F: FnOnce(&mut GrantData<T>, &GrantUpcallTable) -> R,
    {
        // # `unwrap()` Safety
        //
        // `access_grant()` can only return `None` if the grant is already
        // entered. Since we are asking for a panic!() if the grant is entered,
        // `access_grant()` function will never return `None`.
        self.access_grant(fun, true).unwrap()
    }

    /// Run a function with access to the data in the related process for the
    /// related Grant only if that grant region is not already entered. If the
    /// grant is already entered silently skip it. Also provide access to
    /// associated Upcalls.
    ///
    /// **You almost certainly should use `.enter()` rather than
    /// `.try_enter()`.**
    ///
    /// While the `.enter()` version can panic, that panic likely indicates a
    /// bug in the code and not a condition that should be handled. For example,
    /// this benign looking code is wrong:
    ///
    /// ```ignore
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
    /// However, if the example used `.try_enter()` in the iter loop, there
    /// would be no panic, but the already entered grant would be silently
    /// skipped. This can hide subtle bugs if the skipped grant is only relevant
    /// in certain cases.
    ///
    /// Therefore, only use `try_enter()` if you are sure you want to skip the
    /// already entered grant. Cases for this are rare.
    ///
    /// ## Return
    ///
    /// Returns `None` if the grant is already entered. Otherwise returns
    /// `Some(fun())`.
    pub fn try_enter<F, R>(self, fun: F) -> Option<R>
    where
        F: FnOnce(&mut GrantData<T>, &GrantUpcallTable) -> R,
    {
        self.access_grant(fun, false)
    }

    /// Run a function with access to the memory in the related process for the
    /// related Grant. Also provide this function with access to any associated
    /// Upcalls and an allocator for allocating additional memory in the
    /// process's grant region.
    ///
    /// This is "entering" the grant region, and the _only_ time when the
    /// contents of a grant region can be accessed.
    ///
    /// Note, a grant can only be entered once at a time. Attempting to call
    /// `.enter()` on a grant while it is already entered will result in a
    /// panic!()`. See the comment in `access_grant()` for more information.
    pub fn enter_with_allocator<F, R>(self, fun: F) -> R
    where
        F: FnOnce(&mut GrantData<T>, &GrantUpcallTable, &mut GrantRegionAllocator) -> R,
    {
        // # `unwrap()` Safety
        //
        // `access_grant()` can only return `None` if the grant is already
        // entered. Since we are asking for a panic!() if the grant is entered,
        // `access_grant()` function will never return `None`.
        self.access_grant_with_allocator(fun, true).unwrap()
    }

    /// Access the `ProcessGrant` memory and run a closure on the process's
    /// grant memory.
    ///
    /// If `panic_on_reenter` is `true`, this will panic if the grant region is
    /// already currently entered. If `panic_on_reenter` is `false`, this will
    /// return `None` if the grant region is entered and do nothing.
    fn access_grant<F, R>(self, fun: F, panic_on_reenter: bool) -> Option<R>
    where
        F: FnOnce(&mut GrantData<T>, &GrantUpcallTable) -> R,
    {
        // Access the grant that is in process memory. This can only fail if
        // the grant is already entered.
        let optional_grant_ptr = self
            .process
            .enter_grant(self.grant_num)
            .map_err(|_err| {
                // If we get an error it is because the grant is already
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

        // Return early if no grant. Type annotation for unsafe correctness.
        let grant_ptr: *mut u8 = if let Some(ptr) = optional_grant_ptr {
            ptr
        } else {
            return None;
        };

        // See new() for more explanation on these calculations.
        let upcalls_size = size_of::<usize>() + (NUM_UPCALLS * size_of::<SavedUpcall>());
        let grant_t_align = align_of::<T>();
        let upcalls_padding = grant_t_align - (upcalls_size % grant_t_align);

        // `grant_ptr` now refers to the special memory we store for each
        // `Grant` which contains the number of upcalls for this grant, an array
        // of upcall data, some potential padding, and then the object of type
        // T.
        //
        // To get to the correct pointer where object of type T is store, we
        // have to increment the pointer past our saved upcall state and
        // padding.
        //
        // # Safety
        //
        // This pointer is safe because it is a *u8 and we offset it the same
        // number of bytes as when the grant memory was originally allocated.
        let grant_type_ptr = unsafe { grant_ptr.add(upcalls_size + upcalls_padding) };

        // # Safety
        //
        // This pointer is safe because it is a *u8 and we offset it the correct
        // number of bytes to the start of the `SavedUpcall` array.
        let saved_upcalls_ptr = unsafe { grant_ptr.add(size_of::<usize>()) };
        // # Safety
        //
        // Creating this slice is safe because the pointer is in the grant
        // allocation that is guaranteed to still exist, a grant can only be
        // entered once at a time so this is the only mutable reference to this
        // slice, and the slice has valid SavedUpcalls which are guaranteed to
        // be initialized.
        let saved_upcalls_slice =
            unsafe { slice::from_raw_parts(saved_upcalls_ptr as *mut SavedUpcall, NUM_UPCALLS) };

        // Process only holds the grant's memory, but does not know the actual
        // type of the grant. We case the type here so that the user of the
        // grant is restricted by the type system to access this memory safely.
        //
        // # Safety
        //
        // This is safe as long as the memory at grant_ptr is correctly aligned,
        // the correct size for type `T`, is only ever cast as a `T`, and only
        // one reference to the object exists. We guarantee this because type
        // `T` cannot change, and we ensure the size and alignment are correct
        // when the grant is allocated. We ensure that only one reference can
        // ever exist by marking the grant entered in `enter_grant()`, and
        // subsequent calls to `enter_grant()` will fail.
        let grant = unsafe { &mut *(grant_type_ptr as *mut T) };

        // Create a wrapped object that is passed to the capsule.
        let mut grant_data = GrantData::new(grant);
        // Create a wrapped object that gives access to the upcalls for this
        // driver.
        let upcall_table =
            GrantUpcallTable::new(saved_upcalls_slice, self.driver_num, self.process);

        // Allow the capsule to access the grant.
        let res = fun(&mut grant_data, &upcall_table);

        // Now that the capsule has finished we need to "release" the grant.
        // This will mark it as no longer entered and allow the grant to be used
        // in the future.
        self.process.leave_grant(self.grant_num);

        Some(res)
    }

    /// Access the `ProcessGrant` memory and run a closure on the process's
    /// grant memory.
    ///
    /// If `panic_on_reenter` is `true`, this will panic if the grant region is
    /// already currently entered. If `panic_on_reenter` is `false`, this will
    /// return `None` if the grant region is entered and do nothing.
    fn access_grant_with_allocator<F, R>(self, fun: F, panic_on_reenter: bool) -> Option<R>
    where
        F: FnOnce(&mut GrantData<T>, &GrantUpcallTable, &mut GrantRegionAllocator) -> R,
    {
        // Access the grant that is in process memory. This can only fail if
        // the grant is already entered.
        let optional_grant_ptr = self
            .process
            .enter_grant(self.grant_num)
            .map_err(|_err| {
                // If we get an error it is because the grant is already
                // entered. `process.enter_grant()` can fail for several
                // reasons, but only the double enter case can happen once a
                // grant has been applied. The other errors would be detected
                // earlier (i.e. before the grant can be applied).

                // If `panic_on_reenter` is false, we skip this error and do
                // nothing with this grant.
                if !panic_on_reenter {
                    return;
                }

                // See `access_grant()` for an explanation of this panic.
                panic!("Attempted to re-enter a grant region.");
            })
            .ok();

        // # `unwrap()` Safety
        //
        // Only `unwrap()` if some, otherwise return early.
        let grant_ptr: *mut u8 = if optional_grant_ptr.is_some() {
            optional_grant_ptr.unwrap()
        } else {
            return None;
        };

        // See new() for more explanation on these calculations.
        let upcalls_size = size_of::<usize>() + (NUM_UPCALLS * size_of::<SavedUpcall>());
        let grant_t_align = align_of::<T>();
        let upcalls_padding = grant_t_align - (upcalls_size % grant_t_align);

        // # Safety
        //
        // This pointer is safe because it is a *u8 and we offset it the same
        // number of bytes as when the grant memory was originally allocated.
        let grant_type_ptr = unsafe { grant_ptr.add(upcalls_size + upcalls_padding) };

        // # Safety
        //
        // This pointer is safe because it is a *u8 and we offset it the correct
        // number of bytes to the start of the `SavedUpcall` array.
        let saved_upcalls_ptr = unsafe { grant_ptr.add(size_of::<usize>()) };
        // # Safety
        //
        // Creating this slice is safe because the pointer is in the grant
        // allocation that is guaranteed to still exist, a grant can only be
        // entered once at a time so this is the only mutable reference to this
        // slice, and the slice has valid SavedUpcalls which are guaranteed to
        // be initialized.
        let saved_upcalls_slice =
            unsafe { slice::from_raw_parts(saved_upcalls_ptr as *mut SavedUpcall, NUM_UPCALLS) };

        // Process only holds the grant's memory, but does not know the actual
        // type of the grant. We case the type here so that the user of the
        // grant is restricted by the type system to access this memory safely.
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
        let grant = unsafe { &mut *(grant_type_ptr as *mut T) };

        // Create a wrapped object that is passed to the capsule.
        let mut grant_data = GrantData::new(grant);

        // Setup an allocator in case the capsule needs additional memory in the
        // grant space.
        let mut allocator = GrantRegionAllocator {
            processid: self.process.processid(),
        };

        // Create a wrapped object that gives access to the upcalls for this
        // driver.
        let upcall_table =
            GrantUpcallTable::new(saved_upcalls_slice, self.driver_num, self.process);

        // Allow the capsule to access the grant.
        let res = fun(&mut grant_data, &upcall_table, &mut allocator);

        // Now that the capsule has finished we need to "release" the grant.
        // This will mark it as no longer entered and allow the grant to be used
        // in the future.
        self.process.leave_grant(self.grant_num);

        Some(res)
    }
}

/// Grant which was allocated from the kernel-owned grant region in a specific
/// process's memory, separately from a normal `Grant`.
///
/// A `CustomGrant` allows a capsule to allocate additional memory on behalf of
/// a process.
pub struct CustomGrant<T> {
    /// An identifier for this custom grant within a process's grant region.
    ///
    /// Here, this is an opaque reference that Process uses to access the
    /// custom grant allocation. This setup ensures that Process owns the grant
    /// memory.
    identifier: ProcessCustomGrantIdentifer,

    /// Identifier for the process where this custom grant is allocated.
    processid: ProcessId,

    /// Used to keep the Rust type of the grant.
    _phantom: PhantomData<T>,
}

impl<T> CustomGrant<T> {
    /// Creates a new `CustomGrant`.
    fn new(identifier: ProcessCustomGrantIdentifer, processid: ProcessId) -> Self {
        CustomGrant {
            identifier,
            processid,
            _phantom: PhantomData,
        }
    }

    /// Helper function to get the ProcessId from the custom grant.
    pub fn processid(&self) -> ProcessId {
        self.processid
    }

    /// Gives access to inner data within the given closure.
    ///
    /// If the process has since been restarted or crashed, or the memory is
    /// otherwise no longer present, then this function will not call the given
    /// closure, and will instead directly return `Err(Error::NoSuchApp)`.
    ///
    /// Because this function requires `&mut self`, it should be impossible to
    /// access the inner data of a given `CustomGrant` reentrantly. Thus the
    /// reentrance detection we use for non-custom grants is not needed here.
    pub fn enter<F, R>(&mut self, fun: F) -> Result<R, Error>
    where
        F: FnOnce(GrantData<'_, T>) -> R,
    {
        // Verify that the process this CustomGrant was allocated within still
        // exists.
        self.processid
            .kernel
            .process_map_or(Err(Error::NoSuchApp), self.processid, |process| {
                // App is valid.

                // Now try to access the custom grant memory.
                let grant_ptr = process.enter_custom_grant(self.identifier)?;

                // # Safety
                //
                // `grant_ptr` must be a valid pointer and there must not exist
                // any other references to the same memory. We verify the
                // pointer is valid and aligned when the memory is allocated and
                // `CustomGrant` is created. We are sure that there are no
                // other references because the only way to create a reference
                // is using this `enter()` function, and it can only be called
                // once (because of the `&mut self` requirement).
                let custom_grant = unsafe { &mut *(grant_ptr as *mut T) };
                let borrowed = GrantData::new(custom_grant);
                Ok(fun(borrowed))
            })
    }
}

/// Tool for allocating additional memory regions in a process's grant region.
///
/// This is optionally provided along with a grant so that if a capsule needs
/// per-process dynamic allocation it can allocate additional memory.
pub struct GrantRegionAllocator {
    /// The process the allocator will allocate memory from.
    processid: ProcessId,
}

impl GrantRegionAllocator {
    /// Allocates a new `CustomGrant` initialized using the given closure.
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
    pub fn alloc_with<T, F>(&mut self, init: F) -> Result<CustomGrant<T>, Error>
    where
        F: FnOnce() -> T,
    {
        let (custom_grant_identifier, typed_ptr) = self.alloc_raw::<T>()?;

        // # Safety
        //
        // Writing to this pointer is safe as long as the pointer is valid
        // and aligned. `alloc_raw()` guarantees these constraints are met.
        unsafe {
            // We use `ptr::write` to avoid `Drop`ping the uninitialized memory
            // in case `T` implements the `Drop` trait.
            write(typed_ptr.as_ptr(), init());
        }

        Ok(CustomGrant::new(custom_grant_identifier, self.processid))
    }

    /// Allocates a slice of n instances of a given type. Each instance is
    /// initialized using the provided function.
    ///
    /// The provided function will be called exactly `n` times, and will be
    /// passed the index it's initializing, from `0` through `NUM_ITEMS - 1`.
    ///
    /// # Panic Safety
    ///
    /// If `val_func` panics, the freshly allocated memory and any values
    /// already written will be leaked.
    pub fn alloc_n_with<T, F, const NUM_ITEMS: usize>(
        &mut self,
        mut init: F,
    ) -> Result<CustomGrant<[T; NUM_ITEMS]>, Error>
    where
        F: FnMut(usize) -> T,
    {
        let (custom_grant_identifier, typed_ptr) = self.alloc_n_raw::<T>(NUM_ITEMS)?;

        for i in 0..NUM_ITEMS {
            // # Safety
            //
            // The allocate function guarantees that `ptr` points to memory
            // large enough to allocate `num_items` copies of the object.
            unsafe {
                write(typed_ptr.as_ptr().add(i), init(i));
            }
        }

        Ok(CustomGrant::new(custom_grant_identifier, self.processid))
    }

    /// Allocates uninitialized grant memory appropriate to store a `T`.
    ///
    /// The caller must initialize the memory.
    ///
    /// Also returns a ProcessCustomGrantIdentifer to access the memory later.
    fn alloc_raw<T>(&mut self) -> Result<(ProcessCustomGrantIdentifer, NonNull<T>), Error> {
        self.alloc_n_raw::<T>(1)
    }

    /// Allocates space for a dynamic number of items.
    ///
    /// The caller is responsible for initializing the returned memory.
    ///
    /// Returns memory appropriate for storing `num_items` contiguous instances
    /// of `T` and a ProcessCustomGrantIdentifer to access the memory later.
    fn alloc_n_raw<T>(
        &mut self,
        num_items: usize,
    ) -> Result<(ProcessCustomGrantIdentifer, NonNull<T>), Error> {
        let (custom_grant_identifier, raw_ptr) =
            self.alloc_n_raw_inner(num_items, size_of::<T>(), align_of::<T>())?;
        let typed_ptr = NonNull::cast::<T>(raw_ptr);

        Ok((custom_grant_identifier, typed_ptr))
    }

    /// Helper to reduce code bloat by avoiding monomorphization.
    fn alloc_n_raw_inner(
        &mut self,
        num_items: usize,
        single_alloc_size: usize,
        alloc_align: usize,
    ) -> Result<(ProcessCustomGrantIdentifer, NonNull<u8>), Error> {
        let alloc_size = single_alloc_size
            .checked_mul(num_items)
            .ok_or(Error::OutOfMemory)?;
        self.processid
            .kernel
            .process_map_or(Err(Error::NoSuchApp), self.processid, |process| {
                process
                    .allocate_custom_grant(alloc_size, alloc_align)
                    .map_or(
                        Err(Error::OutOfMemory),
                        |(custom_grant_identifier, raw_ptr)| Ok((custom_grant_identifier, raw_ptr)),
                    )
            })
    }
}

/// Type for storing an object of type T in process memory that is only
/// accessible by the kernel.
///
/// A single `Grant` can allocate space for one object of type T for each
/// process on the board. Each allocated object will reside in the grant region
/// belonging to the process that the object is allocated for. The `Grant` type
/// is used to get access to `ProcessGrant`s, which are tied to a specific
/// process and provide access to the memory object allocated for that process.
pub struct Grant<T: Default, const NUM_UPCALLS: usize> {
    /// Hold a reference to the core kernel so we can iterate processes.
    pub(crate) kernel: &'static Kernel,

    /// Keep track of the syscall driver number assigned to the capsule that is
    /// using this grant. This allows us to uniquely identify upcalls stored in
    /// this grant.
    driver_num: usize,

    /// The identifier for this grant. Having an identifier allows the Process
    /// implementation to lookup the memory for this grant in the specific
    /// process.
    grant_num: usize,

    /// Used to keep the Rust type of the grant.
    ptr: PhantomData<T>,
}

impl<T: Default, const NUM_UPCALLS: usize> Grant<T, NUM_UPCALLS> {
    /// Create a new `Grant` type which allows a capsule to store
    /// process-specific data for each process in the process's memory region.
    ///
    /// This must only be called from the main kernel so that it can ensure that
    /// `grant_index` is a valid index.
    pub(crate) fn new(kernel: &'static Kernel, driver_num: usize, grant_index: usize) -> Self {
        Self {
            kernel: kernel,
            driver_num: driver_num,
            grant_num: grant_index,
            ptr: PhantomData,
        }
    }

    /// Enter the grant for a specific process.
    ///
    /// This creates a `ProcessGrant` which is a handle for a grant allocated
    /// for a specific process. Then, that `ProcessGrant` is entered and the
    /// provided closure is run with access to the memory in the grant region.
    pub fn enter<F, R>(&self, processid: ProcessId, fun: F) -> Result<R, Error>
    where
        F: FnOnce(&mut GrantData<T>, &GrantUpcallTable) -> R,
    {
        let pg = ProcessGrant::new(self, processid)?;

        // If we have managed to create an `ProcessGrant`, all we need
        // to do is actually access the memory and run the
        // capsule-provided closure. This can only fail if the grant is
        // already entered, at which point the kernel will panic.
        Ok(pg.enter(fun))
    }

    /// Enter the grant for a specific process with access to an allocator.
    ///
    /// This creates an `ProcessGrant` which is a handle for a grant allocated
    /// for a specific process. Then, that `ProcessGrant` is entered and the
    /// provided closure is run with access to the memory in the grant region.
    ///
    /// The allocator allows the caller to dynamically allocate additional
    /// memory in the process's grant region.
    pub fn enter_with_allocator<F, R>(&self, processid: ProcessId, fun: F) -> Result<R, Error>
    where
        F: FnOnce(&mut GrantData<T>, &GrantUpcallTable, &mut GrantRegionAllocator) -> R,
    {
        // Get the `ProcessGrant` for the process, possibly needing to
        // actually allocate the memory in the process's grant region to
        // do so. This can fail for a variety of reasons, and if so we
        // return the error to the capsule.
        let pg = ProcessGrant::new(self, processid)?;

        // If we have managed to create an `ProcessGrant`, all we need
        // to do is actually access the memory and run the
        // capsule-provided closure. This can only fail if the grant is
        // already entered, at which point the kernel will panic.
        Ok(pg.enter_with_allocator(fun))
    }

    /// Run a function on the grant for each active process if the grant has
    /// been allocated for that process.
    ///
    /// This will silently skip any process where the grant has not previously
    /// been allocated. This will also silently skip any invalid processes.
    ///
    /// Calling this function when an `ProcessGrant` for a process is currently
    /// entered will result in a panic.
    pub fn each<F>(&self, fun: F)
    where
        F: Fn(ProcessId, &mut GrantData<T>, &GrantUpcallTable),
    {
        // Create a the iterator across `ProcessGrant`s for each process.
        for pg in self.iter() {
            let processid = pg.processid();
            // Since we iterating, there is no return value we need to worry
            // about.
            pg.enter(|data, upcalls| fun(processid, data, upcalls));
        }
    }

    /// Get an iterator over all processes and their active grant regions for
    /// this particular grant.
    ///
    /// Calling this function when an `ProcessGrant` for a process is currently
    /// entered will result in a panic.
    pub fn iter(&self) -> Iter<T, NUM_UPCALLS> {
        Iter {
            grant: self,
            subiter: self.kernel.get_process_iter(),
        }
    }
}

/// Type to iterate `ProcessGrant`s across processes.
pub struct Iter<'a, T: 'a + Default, const NUM_UPCALLS: usize> {
    /// The grant type to use.
    grant: &'a Grant<T, NUM_UPCALLS>,

    /// Iterator over valid processes.
    subiter: core::iter::FilterMap<
        core::slice::Iter<'a, Option<&'static dyn Process>>,
        fn(&Option<&'static dyn Process>) -> Option<&'static dyn Process>,
    >,
}

impl<'a, T: Default, const NUM_UPCALLS: usize> Iterator for Iter<'a, T, NUM_UPCALLS> {
    type Item = ProcessGrant<'a, T, NUM_UPCALLS>;

    fn next(&mut self) -> Option<Self::Item> {
        let grant = self.grant;
        // Get the next `ProcessId` from the kernel processes array that is setup to
        // use this grant. Since the iterator itself is saved calling this
        // function again will start where we left off.
        self.subiter
            .find_map(|process| ProcessGrant::new_if_allocated(grant, process))
    }
}
