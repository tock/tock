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
//! Upcalls and allowed buffer references are stored in the dynamically
//! allocated grant for a particular Driver as well. Upcalls and allowed buffer
//! references are stored outside of the `T` object to enable the kernel to
//! manage them and ensure swapping guarantees are met.
//!
//! The type `T` of a `Grant` is fixed in size and the number of upcalls and
//! allowed buffers associated with a grant is fixed. That is, when a `Grant` is
//! entered for a process the resulting allocated object will be the size of
//! `SizeOf<T>` plus the size for the structure to hold upcalls and allowed
//! buffer references. If capsules need additional process-specific memory for
//! their operation, they can use an `Allocator` to request additional memory
//! from the process's grant region.
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
//! │ │ GrantKernelData │  │ │
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
//!                         │ struct Grant<T, ...> {   │
//!                         │   driver_num: usize      │
//!                         │   grant_num: usize       │
//!                         │ }                        ├───┐
//! Entering a Grant for a  └──┬───────────────────────┘   │
//! process causes the         │                           │
//! memory for T to be         │ .enter(ProcessId)         │ .enter(ProcessId, fn)
//! allocated.                 ▼                           │
//!                         ┌──────────────────────────┐   │ For convenience,
//! ProcessGrant represents │ struct ProcessGrant<T> { │   │ allocating and getting
//! a Grant allocated for a │   number: usize          │   │ access to the T object
//! specific process.       │   process: &Process      │   │ is combined in one
//!                         │ }                        │   │ .enter() call.
//! A provided closure      └──┬───────────────────────┘   │
//! is given access to         │                           │
//! the underlying memory      │ .enter(fn)                │
//! where the T is stored.     ▼                           │
//!                         ┌────────────────────────────┐ │
//! GrantData wraps the     │ struct GrantData<T>   {    │◄┘
//! type and provides       │   data: &mut T             │
//! mutable access.         │ }                          │
//! GrantKernelData         │ struct GrantKernelData {   │
//! provides access to      │   upcalls: [SavedUpcall]   │
//! scheduling upcalls      │   allow_ro: [SavedAllowRo] │
//! and process buffers.    │   allow_rw: [SavedAllowRW] │
//!                         │ }                          │
//!                         └──┬─────────────────────────┘
//! The actual object T can    │
//! only be accessed inside    │ fn(mem: &GrantData, kernel_data: &GrantKernelData)
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
use crate::processbuffer::{ReadOnlyProcessBuffer, ReadWriteProcessBuffer};
use crate::processbuffer::{ReadOnlyProcessBufferRef, ReadWriteProcessBufferRef};
use crate::upcall::{Upcall, UpcallError, UpcallId};
use crate::ErrorCode;

/// Tracks how many upcalls a grant instance supports automatically.
pub trait UpcallSize {
    /// The number of upcalls the grant supports.
    const COUNT: u8;
}

/// Specifies how many upcalls a grant instance supports automatically.
pub struct UpcallCount<const NUM: u8>;
impl<const NUM: u8> UpcallSize for UpcallCount<NUM> {
    const COUNT: u8 = NUM;
}

/// Tracks how many read-only allows a grant instance supports automatically.
pub trait AllowRoSize {
    /// The number of read-only allows the grant supports.
    const COUNT: u8;
}

/// Specifies how many read-only allows a grant instance supports automatically.
pub struct AllowRoCount<const NUM: u8>;
impl<const NUM: u8> AllowRoSize for AllowRoCount<NUM> {
    const COUNT: u8 = NUM;
}

/// Tracks how many read-write allows a grant instance supports automatically.
pub trait AllowRwSize {
    /// The number of read-write allows the grant supports.
    const COUNT: u8;
}

/// Specifies how many read-write allows a grant instance supports
/// automatically.
pub struct AllowRwCount<const NUM: u8>;
impl<const NUM: u8> AllowRwSize for AllowRwCount<NUM> {
    const COUNT: u8 = NUM;
}

/// Helper that calculated offsets within the kernel owned memory (i.e. the
/// non-T part of grant).
///
/// Example layout of full grant belonging to a single app and driver:
///
/// ```text,ignore
/// 0x003FFC8  ┌────────────────────────────────────┐
///            │   T                                |
/// 0x003FFxx  ├  ───────────────────────── ┐ K     |
///            │   Padding (ensure T aligns)| e     |
/// 0x003FF44  ├  ───────────────────────── | r     |
///            │   SavedAllowRwN            | n     |
///            │   ...                      | e     | G
///            │   SavedAllowRw1            | l     | r
///            │   SavedAllowRw0            |       | a
/// 0x003FF44  ├  ───────────────────────── | O     | n
///            │   SavedAllowRoN            | w     | t
///            │   ...                      | n     |
///            │   SavedAllowRo1            | e     | M
///            │   SavedAllowRo0            | d     | e
/// 0x003FF30  ├  ───────────────────────── |       | m
///            │   SavedUpcallN             | D     | o
///            │   ...                      | a     | r
///            │   SavedUpcall1             | t     | y
///            │   SavedUpcall0             | a     |
/// 0x003FF24  ├  ───────────────────────── |       |
///            │   Counters (usize)         |       |
/// 0x003FF20  └────────────────────────────────────┘
/// ```
///
/// The counters structure is composed as:
///
/// ```text,ignore
/// 0             1             2             3         bytes
/// |-------------|-------------|-------------|-------------|
/// | # Upcalls   | # RO Allows | # RW Allows | [unused]    |
/// |-------------|-------------|-------------|-------------|
/// ```
struct KernelManagedLayout {
    counters_ptr: *mut usize,
    upcalls_array: *mut SavedUpcall,
    allow_ro_array: *mut SavedAllowRo,
    allow_rw_array: *mut SavedAllowRw,
}

/// Represents the number of the upcall elements in the kernel owned section of
/// the grant.
#[derive(Copy, Clone)]
struct UpcallItems(u8);
/// Represents the number of the read-only allow elements in the kernel owned
/// section of the grant.
#[derive(Copy, Clone)]
struct AllowRoItems(u8);
/// Represents the number of the read-write allow elements in the kernel owned
/// section of the grant.
#[derive(Copy, Clone)]
struct AllowRwItems(u8);
/// Represents the size data (in bytes) T within the grant.
#[derive(Copy, Clone)]
struct GrantDataSize(usize);
/// Represents the alignment of data T within the grant.
#[derive(Copy, Clone)]
struct GrantDataAlign(usize);

impl KernelManagedLayout {
    /// Reads the specified pointer as the base of the kernel owned grant region
    /// that has previously been initialized.
    ///
    /// # Safety
    ///
    /// The incoming base pointer must be well aligned and already contain
    /// initialized data in the expected form. There must not be any other
    /// `KernelManagedLayout` for the given `base_ptr` at the same time,
    /// otherwise multiple mutable references to the same upcall/allow slices
    /// could be created.
    unsafe fn read_from_base(base_ptr: *mut u8) -> Self {
        let counters_ptr = base_ptr as *mut usize;
        let counters_val = counters_ptr.read();

        // Parse the counters field for each of the fields
        let upcalls_num = (counters_val & 0xFF) as u8;
        let allow_ro_num = ((counters_val >> 8) & 0xFF) as u8;

        // Skip over the counter usize, then the stored array of `SavedAllowRo`
        // items and `SavedAllowRw` items.
        let upcalls_array = counters_ptr.add(1) as *mut SavedUpcall;
        let allow_ro_array = upcalls_array.add(upcalls_num as usize) as *mut SavedAllowRo;
        let allow_rw_array = allow_ro_array.add(allow_ro_num as usize) as *mut SavedAllowRw;

        Self {
            counters_ptr,
            upcalls_array,
            allow_ro_array,
            allow_rw_array,
        }
    }

    /// Creates a layout from the specified pointer and lengths of arrays and
    /// initializes the kernel owned portion of the layout.
    ///
    /// # Safety
    ///
    /// The incoming base pointer must be well aligned and reference enough
    /// memory to hold the entire kernel managed grant structure. There must
    /// not be any other `KernelManagedLayout` for
    /// the given `base_ptr` at the same time, otherwise multiple mutable
    /// references to the same upcall/allow slices could be created.
    unsafe fn initialize_from_counts(
        base_ptr: *mut u8,
        upcalls_num_val: UpcallItems,
        allow_ro_num_val: AllowRoItems,
        allow_rw_num_val: AllowRwItems,
    ) -> Self {
        let counters_ptr = base_ptr as *mut usize;

        // Create the counters usize value by correctly packing the various
        // counts into 8 bit fields.
        let counter: usize = upcalls_num_val.0 as usize
            | ((allow_ro_num_val.0 as usize) << 8)
            | ((allow_rw_num_val.0 as usize) << 16);

        let upcalls_array = counters_ptr.add(1) as *mut SavedUpcall;
        let allow_ro_array = upcalls_array.add(upcalls_num_val.0.into()) as *mut SavedAllowRo;
        let allow_rw_array = allow_ro_array.add(allow_ro_num_val.0.into()) as *mut SavedAllowRw;

        counters_ptr.write(counter.into());
        write_default_array(upcalls_array, upcalls_num_val.0.into());
        write_default_array(allow_ro_array, allow_ro_num_val.0.into());
        write_default_array(allow_rw_array, allow_rw_num_val.0.into());

        Self {
            counters_ptr,
            upcalls_array,
            allow_ro_array,
            allow_rw_array,
        }
    }

    /// Returns the entire grant size including the kernel owned memory,
    /// padding, and data for T. Requires that grant_t_align be a power of 2,
    /// which is guaranteed from align_of rust calls.
    fn grant_size(
        upcalls_num: UpcallItems,
        allow_ro_num: AllowRoItems,
        allow_rw_num: AllowRwItems,
        grant_t_size: GrantDataSize,
        grant_t_align: GrantDataAlign,
    ) -> usize {
        let kernel_managed_size = size_of::<usize>()
            + upcalls_num.0 as usize * size_of::<SavedUpcall>()
            + allow_ro_num.0 as usize * size_of::<SavedAllowRo>()
            + allow_rw_num.0 as usize * size_of::<SavedAllowRw>();
        // We know that grant_t_align is a power of 2, so we can make a mask
        // that will save only the remainder bits.
        let grant_t_align_mask = grant_t_align.0 - 1;
        // Determine padding to get to the next multiple of grant_t_align by
        // taking the remainder and subtracting that from the alignment, then
        // ensuring a full alignment value maps to 0.
        let padding =
            (grant_t_align.0 - (kernel_managed_size & grant_t_align_mask)) & grant_t_align_mask;
        kernel_managed_size + padding + grant_t_size.0
    }

    /// Returns the alignment of the entire grant region based on the alignment
    /// of data T.
    fn grant_align(grant_t_align: GrantDataAlign) -> usize {
        // The kernel owned memory all aligned to usize. We need to use the
        // higher of the two alignment to ensure our padding calculations work
        // for any alignment of T.
        cmp::max(align_of::<usize>(), grant_t_align.0)
    }

    /// Returns the offset for the grant data t within the entire grant region.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the specified base pointer is aligned to at
    /// least the alignment of T and points to a grant that is of size
    /// grant_size bytes.
    unsafe fn offset_of_grant_data_t(
        base_ptr: *mut u8,
        grant_size: usize,
        grant_t_size: GrantDataSize,
    ) -> NonNull<u8> {
        // The location of the grant data T is the last element in the entire
        // grant region. Caller must verify that memory is accessible and well
        // aligned to T.
        let grant_t_size_usize: usize = grant_t_size.0;
        NonNull::new_unchecked(base_ptr.add(grant_size - grant_t_size_usize))
    }

    /// Read an 8 bit value from the counter field offset by the specified
    /// number of bits. This is a helper function for reading the counter field.
    fn get_counter_offset(&self, offset_bits: usize) -> usize {
        // # Safety
        //
        // Creating a `KernelManagedLayout` object requires that the pointers
        // are well aligned and point to valid memory.
        let counters_val = unsafe { self.counters_ptr.read() };
        (counters_val >> offset_bits) & 0xFF
    }

    /// Return the number of upcalls stored by the core kernel for this grant.
    fn get_upcalls_number(&self) -> usize {
        self.get_counter_offset(0)
    }

    /// Return the number of read-only allow buffers stored by the core kernel
    /// for this grant.
    fn get_allow_ro_number(&self) -> usize {
        self.get_counter_offset(8)
    }

    /// Return the number of read-write allow buffers stored by the core kernel
    /// for this grant.
    fn get_allow_rw_number(&self) -> usize {
        self.get_counter_offset(16)
    }

    /// Return mutable access to the slice of stored upcalls for this grant.
    /// This is necessary for storing a new upcall.
    fn get_upcalls_slice(&mut self) -> &mut [SavedUpcall] {
        // # Safety
        //
        // Creating a `KernelManagedLayout` object ensures that the pointer to
        // the upcall array is valid.
        unsafe { slice::from_raw_parts_mut(self.upcalls_array, self.get_upcalls_number()) }
    }

    /// Return mutable access to the slice of stored read-only allow buffers for
    /// this grant. This is necessary for storing a new read-only allow.
    fn get_allow_ro_slice(&mut self) -> &mut [SavedAllowRo] {
        // # Safety
        //
        // Creating a `KernelManagedLayout` object ensures that the pointer to
        // the RO allow array is valid.
        unsafe { slice::from_raw_parts_mut(self.allow_ro_array, self.get_allow_ro_number()) }
    }

    /// Return mutable access to the slice of stored read-write allow buffers
    /// for this grant. This is necessary for storing a new read-write allow.
    fn get_allow_rw_slice(&mut self) -> &mut [SavedAllowRw] {
        // # Safety
        //
        // Creating a `KernelManagedLayout` object ensures that the pointer to
        // the RW allow array is valid.
        unsafe { slice::from_raw_parts_mut(self.allow_rw_array, self.get_allow_rw_number()) }
    }

    /// Return slices to the kernel managed upcalls and allow buffers. This
    /// permits using upcalls and allow buffers when a capsule is accessing a
    /// grant.
    fn get_resource_slices(&self) -> (&[SavedUpcall], &[SavedAllowRo], &[SavedAllowRw]) {
        // # Safety
        //
        // Creating a `KernelManagedLayout` object ensures that the pointer to
        // the upcall array is valid.
        let upcall_slice =
            unsafe { slice::from_raw_parts(self.upcalls_array, self.get_upcalls_number()) };

        // # Safety
        //
        // Creating a `KernelManagedLayout` object ensures that the pointer to
        // the RO allow array is valid.
        let allow_ro_slice =
            unsafe { slice::from_raw_parts(self.allow_ro_array, self.get_allow_ro_number()) };

        // # Safety
        //
        // Creating a `KernelManagedLayout` object ensures that the pointer to
        // the RW allow array is valid.
        let allow_rw_slice =
            unsafe { slice::from_raw_parts(self.allow_rw_array, self.get_allow_rw_number()) };

        (upcall_slice, allow_ro_slice, allow_rw_slice)
    }
}

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

/// This GrantKernelData object provides a handle to access upcalls and process
/// buffers stored on behalf of a particular grant/driver.
///
/// Capsules gain access to a GrantKernelData object by calling
/// `Grant::enter()`. From there, they can schedule upcalls or access process
/// buffers.
///
/// It is expected that this type will only exist as a short-lived stack
/// allocation, so its size is not a significant concern.
pub struct GrantKernelData<'a> {
    /// A reference to the actual upcall slice stored in the grant.
    upcalls: &'a [SavedUpcall],

    /// A reference to the actual read only allow slice stored in the grant.
    allow_ro: &'a [SavedAllowRo],

    /// A reference to the actual read write allow slice stored in the grant.
    allow_rw: &'a [SavedAllowRw],

    /// We need to keep track of the driver number so we can properly identify
    /// the Upcall that is called. We need to keep track of its source so we can
    /// remove it if the Upcall is unsubscribed.
    driver_num: usize,

    /// A reference to the process that these upcalls are for. This is used for
    /// actually scheduling the upcalls.
    process: &'a dyn Process,
}

impl<'a> GrantKernelData<'a> {
    /// Create a `GrantKernelData` object to provide a handle for capsules to
    /// call Upcalls.
    fn new(
        upcalls: &'a [SavedUpcall],
        allow_ro: &'a [SavedAllowRo],
        allow_rw: &'a [SavedAllowRw],
        driver_num: usize,
        process: &'a dyn Process,
    ) -> GrantKernelData<'a> {
        Self {
            upcalls,
            allow_ro,
            allow_rw,
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
        r: (usize, usize, usize),
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
                upcall.schedule(self.process, r.0, r.1, r.2)
            },
        )
    }

    /// Returns a lifetime limited reference to the requested
    /// `ReadOnlyProcessBuffer`.
    ///
    /// The `ReadOnlyProcessBuffer` is only valid for as long as this object is
    /// valid, i.e. the lifetime of the app enter closure.
    ///
    /// If the specified allow number is invalid, then a AddressOutOfBounds will
    /// be returned. This returns a process::Error to allow for easy chaining of
    /// this function with the ReadOnlyProcessBuffer::enter function with
    /// `and_then`.
    pub fn get_readonly_processbuffer(
        &self,
        allow_ro_num: usize,
    ) -> Result<ReadOnlyProcessBufferRef, crate::process::Error> {
        self.allow_ro.get(allow_ro_num).map_or(
            Err(crate::process::Error::AddressOutOfBounds),
            |saved_ro| {
                // # Safety
                //
                // This is the saved process buffer data has been validated to
                // be wholly contained within this process before it was stored.
                // The lifetime of the ReadOnlyProcessBuffer is bound to the
                // lifetime of self, which correctly limits dereferencing this
                // saved pointer to only when it is valid.
                unsafe {
                    Ok(ReadOnlyProcessBufferRef::new(
                        saved_ro.ptr,
                        saved_ro.len,
                        self.process.processid(),
                    ))
                }
            },
        )
    }

    /// Returns a lifetime limited reference to the requested
    /// `ReadWriteProcessBuffer`.
    ///
    /// The ReadWriteProcessBuffer is only value for as long as this object is
    /// valid, i.e. the lifetime of the app enter closure.
    ///
    /// If the specified allow number is invalid, then a AddressOutOfBounds will
    /// be return. This returns a process::Error to allow for easy chaining of
    /// this function with the `ReadWriteProcessBuffer::enter()` function with
    /// `and_then`.
    pub fn get_readwrite_processbuffer(
        &self,
        allow_rw_num: usize,
    ) -> Result<ReadWriteProcessBufferRef, crate::process::Error> {
        self.allow_rw.get(allow_rw_num).map_or(
            Err(crate::process::Error::AddressOutOfBounds),
            |saved_rw| {
                // # Safety
                //
                // This is the saved process buffer data has been validated to
                // be wholly contained within this process before it was stored.
                // The lifetime of the ReadWriteProcessBuffer is bound to the
                // lifetime of self, which correctly limits dereferencing this
                // saved pointer to only when it is valid.
                unsafe {
                    Ok(ReadWriteProcessBufferRef::new(
                        saved_rw.ptr,
                        saved_rw.len,
                        self.process.processid(),
                    ))
                }
            },
        )
    }
}

/// A minimal representation of an upcall, used for storing an upcall in a
/// process' grant table without wasting memory duplicating information such as
/// process ID.
#[repr(C)]
#[derive(Default)]
struct SavedUpcall {
    appdata: usize,
    fn_ptr: Option<NonNull<()>>,
}

/// A minimal representation of a read-only allow from app, used for storing a
/// read-only allow in a process' kernel managed grant space without wasting
/// memory duplicating information such as process ID.
#[repr(C)]
struct SavedAllowRo {
    ptr: *const u8,
    len: usize,
}

impl Default for SavedAllowRo {
    fn default() -> Self {
        Self {
            ptr: core::ptr::null(),
            len: 0,
        }
    }
}

/// A minimal representation of a read-write allow from app, used for storing a
/// read-write allow in a process' kernel managed grant space without wasting
/// memory duplicating information such as process ID.
#[repr(C)]
struct SavedAllowRw {
    ptr: *mut u8,
    len: usize,
}

impl Default for SavedAllowRw {
    fn default() -> Self {
        Self {
            ptr: core::ptr::null_mut(),
            len: 0,
        }
    }
}

/// Write the default value of T to every element of the array.
///
/// # Safety
///
/// The pointer must be well aligned and point to allocated memory that is
/// writable for `size_of::<T> * num` bytes. No Rust references may exist to
/// memory in the address range spanned by `base..base+num` at the time this
/// function is called. The memory does not need to be initialized yet. If it
/// already does contain initialized memory, then those contents will be
/// overwritten without being `Drop`ed first.
unsafe fn write_default_array<T: Default>(base: *mut T, num: usize) {
    for i in 0..num {
        base.add(i).write(T::default());
    }
}

/// Lifetime of guard represents the lifetime a grant is held "open". On Drop,
/// we leave grant.
///
/// This protects against calling `grant.enter()` without calling the
/// corresponding `grant.leave()`, perhaps due to accidentally using the `?`
/// operator to return early.
struct GrantEnterLifetimeGuard<'a> {
    /// Leaving a grant is handled through the process implementation, so must
    /// keep a reference to the relevant process.
    process: &'a dyn Process,
    /// The grant number of the entered grant that we want to ensure we leave
    /// properly.
    grant_num: usize,
}

impl Drop for GrantEnterLifetimeGuard<'_> {
    fn drop(&mut self) {
        self.process.leave_grant(self.grant_num);
    }
}

/// Enters the grant for the specified process. Caller must hold on to the grant
/// lifetime guard while they accessing the memory in the layout (second
/// element).
fn enter_grant_kernel_managed(
    process: &dyn Process,
    driver_num: usize,
) -> Result<(GrantEnterLifetimeGuard, KernelManagedLayout), ErrorCode> {
    let grant_num = process.lookup_grant_from_driver_num(driver_num)?;

    // Check if the grant has been allocated, and if not we cannot enter this
    // grant.
    match process.grant_is_allocated(grant_num) {
        Some(true) => { /* Allocated, nothing to do */ }
        Some(false) => return Err(ErrorCode::NOMEM),
        None => return Err(ErrorCode::FAIL),
    };

    // Return early if no grant.
    let grant_base_ptr = process.enter_grant(grant_num).or(Err(ErrorCode::NOMEM))?;
    // # Safety
    //
    // We know that this pointer is well aligned and initialized with meaningful
    // data when the grant region was allocated.
    let layout = unsafe { KernelManagedLayout::read_from_base(grant_base_ptr) };
    Ok((GrantEnterLifetimeGuard { process, grant_num }, layout))
}

/// Subscribe to an upcall by saving the upcall in the grant region for the
/// process and returning the existing upcall for the same UpcallId.
pub(crate) fn subscribe(
    process: &dyn Process,
    upcall: Upcall,
) -> Result<Upcall, (Upcall, ErrorCode)> {
    // Enter grant and keep it open until _grant_open goes out of scope.
    let (_grant_open, mut layout) =
        match enter_grant_kernel_managed(process, upcall.upcall_id.driver_num) {
            Ok(val) => val,
            Err(e) => return Err((upcall, e)),
        };

    // Create the saved upcalls slice from the grant memory.
    //
    // # Safety
    //
    // This is safe because of how the grant was initially allocated and that
    // because we were able to enter the grant the grant region must be valid
    // and initialized. We are also holding the grant open until `_grant_open`
    // goes out of scope.
    let saved_upcalls_slice = layout.get_upcalls_slice();

    // Index into the saved upcall slice to get the old upcall. Use .get in case
    // userspace passed us a bad subscribe number.
    match saved_upcalls_slice.get_mut(upcall.upcall_id.subscribe_num) {
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
    }
}

/// Stores the specified read-only process buffer in the kernel managed grant
/// region for this process and driver. The previous read-only process buffer
/// stored at the same allow_num id is returned.
pub(crate) fn allow_ro(
    process: &dyn Process,
    driver_num: usize,
    allow_num: usize,
    buffer: ReadOnlyProcessBuffer,
) -> Result<ReadOnlyProcessBuffer, (ReadOnlyProcessBuffer, ErrorCode)> {
    // Enter grant and keep it open until `_grant_open` goes out of scope.
    let (_grant_open, mut layout) = match enter_grant_kernel_managed(process, driver_num) {
        Ok(val) => val,
        Err(e) => return Err((buffer, e)),
    };

    // Create the saved allow ro slice from the grant memory.
    //
    // # Safety
    //
    // This is safe because of how the grant was initially allocated and that
    // because we were able to enter the grant the grant region must be valid
    // and initialized. We are also holding the grant open until _grant_open
    // goes out of scope.
    let saved_allow_ro_slice = layout.get_allow_ro_slice();

    // Index into the saved slice to get the old value. Use .get in case
    // userspace passed us a bad allow number.
    match saved_allow_ro_slice.get_mut(allow_num) {
        Some(saved) => {
            // # Safety
            //
            // The pointer has already been validated to be within application
            // memory before storing the values in the saved slice.
            let old_allow =
                unsafe { ReadOnlyProcessBuffer::new(saved.ptr, saved.len, process.processid()) };

            // Replace old values with current buffer.
            let (ptr, len) = buffer.consume();
            saved.ptr = ptr;
            saved.len = len;

            // Success!
            Ok(old_allow)
        }
        None => Err((buffer, ErrorCode::INVAL)),
    }
}

/// Stores the specified read-write process buffer in the kernel managed grant
/// region for this process and driver. The previous read-write process buffer
/// stored at the same allow_num id is returned.
pub(crate) fn allow_rw(
    process: &dyn Process,
    driver_num: usize,
    allow_num: usize,
    buffer: ReadWriteProcessBuffer,
) -> Result<ReadWriteProcessBuffer, (ReadWriteProcessBuffer, ErrorCode)> {
    // Enter grant and keep it open until `_grant_open` goes out of scope.
    let (_grant_open, mut layout) = match enter_grant_kernel_managed(process, driver_num) {
        Ok(val) => val,
        Err(e) => return Err((buffer, e)),
    };

    // Create the saved allow rw slice from the grant memory.
    //
    // # Safety
    //
    // This is safe because of how the grant was initially allocated and that
    // because we were able to enter the grant the grant region must be valid
    // and initialized. We are also holding the grant open until `_grant_open`
    // goes out of scope.
    let saved_allow_rw_slice = layout.get_allow_rw_slice();

    // Index into the saved slice to get the old value. Use .get in case
    // userspace passed us a bad allow number.
    match saved_allow_rw_slice.get_mut(allow_num) {
        Some(saved) => {
            // # Safety
            //
            // The pointer has already been validated to be within application
            // memory before storing the values in the saved slice.
            let old_allow =
                unsafe { ReadWriteProcessBuffer::new(saved.ptr, saved.len, process.processid()) };

            // Replace old values with current buffer.
            let (ptr, len) = buffer.consume();
            saved.ptr = ptr;
            saved.len = len;

            // Success!
            Ok(old_allow)
        }
        None => Err((buffer, ErrorCode::INVAL)),
    }
}

/// An instance of a grant allocated for a particular process.
///
/// `ProcessGrant` is a handle to an instance of a grant that has been allocated
/// in a specific process's grant region. A `ProcessGrant` guarantees that the
/// memory for the grant has been allocated in the process's memory.
///
/// This is created from a `Grant` when that grant is entered for a specific
/// process.
pub struct ProcessGrant<
    'a,
    T: 'a,
    Upcalls: UpcallSize,
    AllowROs: AllowRoSize,
    AllowRWs: AllowRwSize,
> {
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

    /// Used to store Rust types for grant.
    _phantom: PhantomData<(T, Upcalls, AllowROs, AllowRWs)>,
}

impl<'a, T: Default, Upcalls: UpcallSize, AllowROs: AllowRoSize, AllowRWs: AllowRwSize>
    ProcessGrant<'a, T, Upcalls, AllowROs, AllowRWs>
{
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
    fn new(
        grant: &Grant<T, Upcalls, AllowROs, AllowRWs>,
        processid: ProcessId,
    ) -> Result<Self, Error> {
        // Moves non-generic code from new() into non-generic function to reduce
        // code bloat from the generic function being monomorphized, as it is
        // common to have over 50 copies of Grant::enter() in a Tock kernel (and
        // thus 50+ copies of this function). The returned Option indicates if
        // the returned pointer still needs to be initialized (in the case where
        // the grant was only just allocated).
        fn new_inner<'a>(
            grant_num: usize,
            driver_num: usize,
            grant_t_size: GrantDataSize,
            grant_t_align: GrantDataAlign,
            num_upcalls: UpcallItems,
            num_allow_ros: AllowRoItems,
            num_allow_rws: AllowRwItems,
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
                    // Calculate the alignment and size for entire grant region.
                    let alloc_align = KernelManagedLayout::grant_align(grant_t_align);
                    let alloc_size = KernelManagedLayout::grant_size(
                        num_upcalls,
                        num_allow_ros,
                        num_allow_rws,
                        grant_t_size,
                        grant_t_align,
                    );

                    // Allocate grant, the memory is still uninitialized though.
                    let grant_ptr = process
                        .allocate_grant(grant_num, driver_num, alloc_size, alloc_align)
                        .ok_or(Error::OutOfMemory)?
                        .as_ptr();

                    // Create a layout from the counts we have and initialize
                    // all memory so it is valid in the future to read as a
                    // reference.
                    //
                    // # Safety
                    //
                    // - The grant base pointer is well aligned, yet does not
                    //   have initialized data.
                    // - The pointer points to a large enough space to correctly
                    //   write to is guaranteed by alloc of size
                    //   `KernelManagedLayout::grant_size`.
                    // - There are no proper rust references that map to these
                    //   addresses.
                    unsafe {
                        let _layout = KernelManagedLayout::initialize_from_counts(
                            grant_ptr,
                            num_upcalls,
                            num_allow_ros,
                            num_allow_rws,
                        );
                    }

                    // # Safety
                    //
                    // The grant pointer points to an alloc that is alloc_size
                    // large and is at least as aligned as grant_t_align.
                    unsafe {
                        Ok((
                            Some(KernelManagedLayout::offset_of_grant_data_t(
                                grant_ptr,
                                alloc_size,
                                grant_t_size,
                            )),
                            process,
                        ))
                    }
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
            GrantDataSize(size_of::<T>()),
            GrantDataAlign(align_of::<T>()),
            UpcallItems(Upcalls::COUNT),
            AllowRoItems(AllowROs::COUNT),
            AllowRwItems(AllowRWs::COUNT),
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
                    // Convert untyped `*mut u8` allocation to allocated type.
                    let new_region = NonNull::cast::<T>(allocated_ptr);
                    // We use `ptr::write` to avoid `Drop`ping the uninitialized
                    // memory in case `T` implements the `Drop` trait.
                    write(new_region.as_ptr(), T::default());
                }
            }
            None => {} // Case if grant was already allocated.
        }

        // We have ensured the grant is already allocated or was just allocated,
        // so we can create and return the `ProcessGrant` type.
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
    fn new_if_allocated(
        grant: &Grant<T, Upcalls, AllowROs, AllowRWs>,
        process: &'a dyn Process,
    ) -> Option<Self> {
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
    /// related Grant. This also provides access to any associated Upcalls and
    /// allowed buffers stored with the grant.
    ///
    /// This is "entering" the grant region, and the _only_ time when the
    /// contents of a grant region can be accessed.
    ///
    /// Note, a grant can only be entered once at a time. Attempting to call
    /// `.enter()` on a grant while it is already entered will result in a
    /// panic!()`. See the comment in `access_grant()` for more information.
    pub fn enter<F, R>(self, fun: F) -> R
    where
        F: FnOnce(&mut GrantData<T>, &GrantKernelData) -> R,
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
        F: FnOnce(&mut GrantData<T>, &GrantKernelData) -> R,
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
        F: FnOnce(&mut GrantData<T>, &GrantKernelData, &mut GrantRegionAllocator) -> R,
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
        F: FnOnce(&mut GrantData<T>, &GrantKernelData) -> R,
    {
        self.access_grant_with_allocator(
            |grant_data, kernel_data, _allocator| fun(grant_data, kernel_data),
            panic_on_reenter,
        )
    }

    /// Access the `ProcessGrant` memory and run a closure on the process's
    /// grant memory.
    ///
    /// If `panic_on_reenter` is `true`, this will panic if the grant region is
    /// already currently entered. If `panic_on_reenter` is `false`, this will
    /// return `None` if the grant region is entered and do nothing.
    fn access_grant_with_allocator<F, R>(self, fun: F, panic_on_reenter: bool) -> Option<R>
    where
        F: FnOnce(&mut GrantData<T>, &GrantKernelData, &mut GrantRegionAllocator) -> R,
    {
        // Access the grant that is in process memory. This can only fail if
        // the grant is already entered.
        let grant_ptr = self
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
            .ok()?;
        // Ensure we leave this grant when _grant_open goes out of scope
        let _grant_open = GrantEnterLifetimeGuard {
            process: self.process,
            grant_num: self.grant_num,
        };

        let grant_t_align = GrantDataAlign(align_of::<T>());
        let grant_t_size = GrantDataSize(size_of::<T>());

        let alloc_size = KernelManagedLayout::grant_size(
            UpcallItems(Upcalls::COUNT),
            AllowRoItems(AllowROs::COUNT),
            AllowRwItems(AllowRWs::COUNT),
            grant_t_size,
            grant_t_align,
        );

        // Parse layout of entire grant allocation using the known base pointer.
        //
        // # Safety
        //
        // Grant pointer is well aligned and points to initialized data.
        let layout = unsafe { KernelManagedLayout::read_from_base(grant_ptr) };

        // Get references to all of the saved upcall data.
        //
        // # Safety
        //
        // - Pointer is well aligned and initialized with data from Self::new()
        //   call.
        // - Data will not be modified externally while this immutable reference
        //   is alive.
        // - Data is accessible for the entire duration of this immutable
        //   reference.
        // - No other mutable reference to this memory exists concurrently.
        //   Mutable reference to this memory are only created through the
        //   kernel in the syscall interface which is serialized in time with
        //   this call.
        let (saved_upcalls_slice, saved_allow_ro_slice, saved_allow_rw_slice) =
            layout.get_resource_slices();
        let grant_data = unsafe {
            KernelManagedLayout::offset_of_grant_data_t(grant_ptr, alloc_size, grant_t_size)
                .cast()
                .as_mut()
        };

        // Create a wrapped objects that are passed to functor.
        let mut grant_data = GrantData::new(grant_data);
        let kernel_data = GrantKernelData::new(
            saved_upcalls_slice,
            saved_allow_ro_slice,
            saved_allow_rw_slice,
            self.driver_num,
            self.process,
        );
        // Setup an allocator in case the capsule needs additional memory in the
        // grant space.
        let mut allocator = GrantRegionAllocator {
            processid: self.process.processid(),
        };

        // Call functor and pass back value.
        Some(fun(&mut grant_data, &kernel_data, &mut allocator))
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
pub struct Grant<T: Default, Upcalls: UpcallSize, AllowROs: AllowRoSize, AllowRWs: AllowRwSize> {
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

    /// Used to store the Rust types for grant.
    ptr: PhantomData<(T, Upcalls, AllowROs, AllowRWs)>,
}

impl<T: Default, Upcalls: UpcallSize, AllowROs: AllowRoSize, AllowRWs: AllowRwSize>
    Grant<T, Upcalls, AllowROs, AllowRWs>
{
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
        F: FnOnce(&mut GrantData<T>, &GrantKernelData) -> R,
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
        F: FnOnce(&mut GrantData<T>, &GrantKernelData, &mut GrantRegionAllocator) -> R,
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
    pub fn each<F>(&self, mut fun: F)
    where
        F: FnMut(ProcessId, &mut GrantData<T>, &GrantKernelData),
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
    pub fn iter(&self) -> Iter<T, Upcalls, AllowROs, AllowRWs> {
        Iter {
            grant: self,
            subiter: self.kernel.get_process_iter(),
        }
    }
}

/// Type to iterate `ProcessGrant`s across processes.
pub struct Iter<
    'a,
    T: 'a + Default,
    Upcalls: UpcallSize,
    AllowROs: AllowRoSize,
    AllowRWs: AllowRwSize,
> {
    /// The grant type to use.
    grant: &'a Grant<T, Upcalls, AllowROs, AllowRWs>,

    /// Iterator over valid processes.
    subiter: core::iter::FilterMap<
        core::slice::Iter<'a, Option<&'static dyn Process>>,
        fn(&Option<&'static dyn Process>) -> Option<&'static dyn Process>,
    >,
}

impl<'a, T: Default, Upcalls: UpcallSize, AllowROs: AllowRoSize, AllowRWs: AllowRwSize> Iterator
    for Iter<'a, T, Upcalls, AllowROs, AllowRWs>
{
    type Item = ProcessGrant<'a, T, Upcalls, AllowROs, AllowRWs>;

    fn next(&mut self) -> Option<Self::Item> {
        let grant = self.grant;
        // Get the next `ProcessId` from the kernel processes array that is
        // setup to use this grant. Since the iterator itself is saved calling
        // this function again will start where we left off.
        self.subiter
            .find_map(|process| ProcessGrant::new_if_allocated(grant, process))
    }
}
