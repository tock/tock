//! Data structures for passing application memory to the kernel.
//!
//! A Tock process can pass read-write or read-only buffers into the kernel for
//! it to use. The kernel checks that read-write buffers exist within a
//! process's RAM address space, and that read-only buffers exist either within
//! its RAM or flash address space. These buffers are shared with the
//! `allow_readwrite()` and `allow_readonly()` system calls. Refer to [TRD 104
//! -- Syscalls][1] for more information.
//!
//! A read-write and read-only call is mapped to the high-level Rust types
//! [`ReadWriteProcessBuffer`] and [`ReadOnlyProcessBuffer`] respectively. The
//! memory regions can be accessed through the [`ReadableProcessBuffer`] and
//! [`WriteableProcessBuffer`] traits, implemented on the process buffer
//! structs.
//!
//! Each access to the buffer structs requires a liveness check to ensure that
//! the process memory is still valid. For a more traditional interface, users
//! can convert buffers into [`ReadableProcessSlice`] or
//! [`WriteableProcessSlice`] and use these for the lifetime of their
//! operations. Users cannot hold live-lived references to these slices,
//! however.
//!
//! [1]: https://github.com/tock/tock/blob/master/doc/reference/trd104-syscalls.md

use core::iter::Iterator;
use core::marker::PhantomData;
use core::ops::{Deref, Range, RangeFrom, RangeTo};
use core::ptr::NonNull;

use crate::capabilities;
use crate::process::{self, ProcessId};
use crate::ErrorCode;

// ---------- PROCESS BUFFER TRAITS --------------------------------------------

/// A readable region of userspace process memory.
///
/// This trait can be used to gain read-only access to memory regions wrapped in
/// either a [`ReadOnlyProcessBuffer`] or a [`ReadWriteProcessBuffer`] type.
pub trait ReadableProcessBuffer {
    /// Length of the memory region.
    ///
    /// If the process is no longer alive and the memory has been reclaimed,
    /// this method must return 0.
    ///
    /// # Default Process Buffer
    ///
    /// A default instance of a process buffer must return 0.
    fn len(&self) -> usize;

    /// Pointer to the first byte of the userspace memory region.
    ///
    /// If the length of the initially shared memory region (irrespective of the
    /// return value of [`len`](ReadableProcessBuffer::len)) is 0, this function
    /// returns a pointer to address `0x0`. This is because processes may allow
    /// buffers with length 0 to share no memory with the kernel. Because these
    /// buffers have zero length, they may have any pointer value. However,
    /// these _dummy addresses_ should not be leaked, so this method returns 0
    /// for zero-length slices.
    ///
    /// # Default Process Buffer
    ///
    /// A default instance of a process buffer must return a pointer
    /// to address `0x0`.
    fn ptr(&self) -> *const u8;

    /// Applies a function to the (read only) process slice reference pointed to
    /// by the process buffer.
    ///
    /// If the process is no longer alive and the memory has been reclaimed,
    /// this method must return `Err(process::Error::NoSuchApp)`.
    ///
    /// # Default Process Buffer
    ///
    /// A default instance of a process buffer must return
    /// `Err(process::Error::NoSuchApp)` without executing the passed closure.
    fn enter<'a, F, R>(&'a self, fun: F) -> Result<R, process::Error>
    where
        F: FnOnce(ReadableProcessSlice<'a>) -> R;
}

/// A readable and writeable region of userspace process memory.
///
/// This trait can be used to gain read-write access to memory regions wrapped
/// in a [`ReadWriteProcessBuffer`].
///
/// This is a supertrait of [`ReadableProcessBuffer`], which features methods
/// allowing mutable access.
pub trait WriteableProcessBuffer: ReadableProcessBuffer {
    /// Applies a function to the mutable process slice reference pointed to by
    /// the [`ReadWriteProcessBuffer`].
    ///
    /// If the process is no longer alive and the memory has been reclaimed,
    /// this method must return `Err(process::Error::NoSuchApp)`.
    ///
    /// # Default Process Buffer
    ///
    /// A default instance of a process buffer must return
    /// `Err(process::Error::NoSuchApp)` without executing the passed closure.
    fn mut_enter<'a, F, R>(&'a self, fun: F) -> Result<R, process::Error>
    where
        F: FnOnce(WriteableProcessSlice<'a>) -> R;
}

// ---------- PROCESS BUFFER TYPES ---------------------------------------------

/// Read-only buffer shared by a userspace process.
///
/// This struct is provided to capsules when a process `allow`s a particular
/// section of its memory to the kernel and gives the kernel read access to this
/// memory.
///
/// It can be used to obtain a [`ReadableProcessSlice`], which uses operations
/// on raw pointers to access the covered memory. This is because a userspace
/// can `allow` overlapping sections of memory into different
/// [`ReadableProcessSlice`]. Having at least one mutable Rust slice along with
/// read-only slices to overlapping memory in Rust violates Rust's aliasing
/// rules. Raw pointer accesses avoid this issue, as Rust cannot statically
/// determine two pointers to be within the same allocation and apply aliasing
/// optimizations. Still, a memory barrier prior to switching to userspace is
/// required, as the compiler is free to reorder reads and writes, even through
/// raw pointer operations.
pub struct ReadOnlyProcessBuffer {
    ptr: *const u8,
    len: usize,
    process_id: Option<ProcessId>,
}

impl ReadOnlyProcessBuffer {
    /// Construct a new [`ReadOnlyProcessBuffer`] over a given pointer and
    /// length.
    ///
    /// # Safety requirements
    ///
    /// Refer to the safety requirements of
    /// [`ReadOnlyProcessBuffer::new_external`].
    // TODO: refactor this method to return an Option<Self>, enforcing the
    // required invariant of `ptr != 0 || len == 0`. This is currently enforced
    // through ProcessStandard::build_readonly_process_buffer, but is better to
    // be encoded in the type system when constructing this struct.
    pub(crate) unsafe fn new(ptr: *const u8, len: usize, process_id: ProcessId) -> Self {
        ReadOnlyProcessBuffer {
            ptr,
            len,
            process_id: Some(process_id),
        }
    }

    /// Construct a new [`ReadOnlyProcessBuffer`] over a given pointer
    /// and length.
    ///
    /// Publicly accessible constructor, which requires the
    /// [`capabilities::ExternalProcessCapability`] capability. This is provided
    /// to allow implementations of the [`Process`](crate::process::Process)
    /// trait outside of the `kernel` crate.
    ///
    /// # Safety requirements
    ///
    /// If the length is `0`, an arbitrary pointer may be passed into `ptr`. It
    /// does not necessarily have to point to allocated memory, nor does it have
    /// to meet [Rust's pointer validity
    /// requirements](https://doc.rust-lang.org/core/ptr/index.html#safety).
    /// [`ReadOnlyProcessBuffer`] will ensure that all [`ReadableProcessSlice`]s
    /// with a length of `0` be constructed over a valid (but not necessarily
    /// allocated) base pointer.
    ///
    /// If the length is not `0`, the memory region of `[ptr; ptr + len)` must
    /// be valid memory of the process of the given [`ProcessId`]. The memory
    /// region must not contain address `0`, wrap the memory space or be larger
    /// than `isize::MAX`. It must be allocated and and accessible over the
    /// entire lifetime of the [`ReadOnlyProcessBuffer`]. It must not point to
    /// memory outside of the process' accessible memory range, or point (in
    /// part) to other processes or kernel memory. The `ptr` must meet [Rust's
    /// requirements for pointer
    /// validity](https://doc.rust-lang.org/core/ptr/index.html#safety), in
    /// particular it must have a minimum alignment of
    /// `core::mem::align_of::<u8>()` on the respective platform. It must point
    /// to memory mapped as _readable_ and optionally _writable_ and
    /// _executable_.
    pub unsafe fn new_external(
        ptr: *const u8,
        len: usize,
        process_id: ProcessId,
        _cap: &dyn capabilities::ExternalProcessCapability,
    ) -> Self {
        Self::new(ptr, len, process_id)
    }

    /// Consumes the ReadOnlyProcessBuffer, returning its constituent pointer
    /// and size. This ensures that there cannot simultaneously be both a
    /// `ReadOnlyProcessBuffer` and a pointer to its internal data.
    ///
    /// `consume` can be used when the kernel needs to pass the underlying
    /// values across the kernel-to-user boundary (e.g., in return values to
    /// system calls).
    pub(crate) fn consume(self) -> (*const u8, usize) {
        (self.ptr, self.len)
    }
}

impl ReadableProcessBuffer for ReadOnlyProcessBuffer {
    fn len(&self) -> usize {
        self.process_id
            .map_or(0, |pid| pid.kernel.process_map_or(0, pid, |_| self.len))
    }

    fn ptr(&self) -> *const u8 {
        if self.len == 0 {
            0x0 as *const u8
        } else {
            self.ptr
        }
    }

    fn enter<'a, F, R>(&'a self, fun: F) -> Result<R, process::Error>
    where
        F: FnOnce(ReadableProcessSlice<'a>) -> R,
    {
        match self.process_id {
            None => Err(process::Error::NoSuchApp),
            Some(pid) => pid
                .kernel
                .process_map_or(Err(process::Error::NoSuchApp), pid, |_| {
                    // ## Safety
                    //
                    // While the ReadOnlyProcessBuffer must refuse construction
                    // with a null-pointer and a non-zero length, it may still
                    // contain a null-pointer. This is because it must be able
                    // to represent all valid read-only allow buffer
                    // descriptions passed from userspace. Nonetheless, a Rust
                    // slice (or any Rust allocation) cannot be located at
                    // address zero, even zero-sized types, and especially not a
                    // NonNull. Thus, apply the following rules:
                    //
                    // 1. if the buffer is of non-zero length, it must have a
                    //    non-null pointer (refer to
                    //    ReadableProcessBuffer::new_external). Construct a
                    //    NonNull based on that pointer.
                    //
                    // 2. if the buffer is of zero-length, it may have a null
                    //    pointer. Regardless, in the spirit of the
                    //    ReadableProcessSlice::ptr method, we don't have to
                    //    expose the proper pointer provided by the application,
                    //    as this does not have any meaning for a zero-sized
                    //    slice (not bounds checked, etc.). Thus, always return
                    //    a NonNull::dangling in this case.
                    //
                    // Through the basic assumption of non zero-length buffer
                    // having a properly aligned, non-null pointer, the
                    // following use of NonNull::new_unchecked is safe here:
                    let ptr = if self.len > 0 {
                        unsafe { NonNull::new_unchecked(self.ptr as *mut u8) }
                    } else {
                        NonNull::dangling()
                    };

                    // ## Safety
                    //
                    // `kernel.process_map_or()` validates that the process
                    // still exists and its memory is still valid. In
                    // particular, `Process` tracks the "high water mark" of
                    // memory that the process has `allow`ed to the
                    // kernel. Because `Process` does not feature an API to move
                    // the "high water mark" down again, which would be called
                    // once a `ProcessBuffer` has been passed back into the
                    // kernel, a given `Process` implementation must assume that
                    // the memory described by a once-allowed `ProcessBuffer` is
                    // still in use, and thus will not permit the process to
                    // free any memory after it has been `allow`ed to the kernel
                    // once. This guarantees that the memory pointed to by the
                    // buffer is safe to dereference here. For more information,
                    // refer to the comment and subsequent discussion on
                    // tock/tock#2632[1].
                    //
                    // [1]: https://github.com/tock/tock/pull/2632#issuecomment-869974365
                    Ok(fun(unsafe { ReadableProcessSlice::new(ptr, self.len) }))
                }),
        }
    }
}

// TODO: remove, along with making the ProcessId non-optional and moving it to
// the front. This should save us a usize in some cases (e.g.
// Option<ReadOnlyProcessBuffer>), as ProcessId must start with a non-null field
// (&'static Kernel) and thus ReadOnlyProcessBuffer will also start with a
// non-null field.
impl Default for ReadOnlyProcessBuffer {
    fn default() -> Self {
        ReadOnlyProcessBuffer {
            ptr: 0x0 as *mut u8,
            len: 0,
            process_id: None,
        }
    }
}

/// Provides access to a ReadOnlyProcessBuffer with a restricted lifetime.
///
/// This automatically dereferences into a ReadOnlyProcessBuffer.
pub struct ReadOnlyProcessBufferRef<'a> {
    buf: ReadOnlyProcessBuffer,
    _phantom: PhantomData<&'a ()>,
}

impl ReadOnlyProcessBufferRef<'_> {
    /// Construct a new [`ReadOnlyProcessBufferRef`] over a given pointer and
    /// length with a lifetime derived from the caller.
    ///
    /// # Safety requirements
    ///
    /// Refer to the safety requirements of
    /// [`ReadOnlyProcessBuffer::new_external`]. The derived lifetime can help
    /// enforce the invariant that this incoming pointer may only be access for
    /// a certain duration.
    pub(crate) unsafe fn new(ptr: *const u8, len: usize, process_id: ProcessId) -> Self {
        Self {
            buf: ReadOnlyProcessBuffer::new(ptr, len, process_id),
            _phantom: PhantomData,
        }
    }
}

impl Deref for ReadOnlyProcessBufferRef<'_> {
    type Target = ReadOnlyProcessBuffer;
    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

/// Read-writable buffer shared by a userspace process.
///
/// This struct is provided to capsules when a process `allows` a particular
/// section of its memory to the kernel and gives the kernel read and write
/// access to this memory.
///
/// It can be used to obtain a [`WriteableProcessSlice`], which uses operations
/// on raw pointers to access the covered memory. This is because a userspace
/// can `allow` overlapping sections of memory into different
/// [`ReadableProcessSlice`]. Having at least one mutable Rust slice along with
/// read-only or other mutable slices to overlapping memory in Rust violates
/// Rust's aliasing rules. Raw pointer accesses avoid this issue, as Rust cannot
/// statically determine two pointers to be within the same allocation and apply
/// aliasing optimizations. Still, a memory barrier prior to switching to
/// userspace is required, as the compiler is free to reorder reads and writes,
/// even through raw pointer operations.
pub struct ReadWriteProcessBuffer {
    ptr: *mut u8,
    len: usize,
    process_id: Option<ProcessId>,
}

impl ReadWriteProcessBuffer {
    /// Construct a new [`ReadWriteProcessBuffer`] over a given
    /// pointer and length.
    ///
    /// # Safety requirements
    ///
    /// Refer to the safety requirements of
    /// [`ReadWriteProcessBuffer::new_external`].
    // TODO: refactor this method to return an Option<Self>, enforcing the
    // required invariant of `ptr != 0 || len == 0`. This is currently enforced
    // through ProcessStandard::build_readonly_process_buffer, but is better to
    // be encoded in the type system when constructing this struct.
    pub(crate) unsafe fn new(ptr: *mut u8, len: usize, process_id: ProcessId) -> Self {
        ReadWriteProcessBuffer {
            ptr,
            len,
            process_id: Some(process_id),
        }
    }

    /// Construct a new [`ReadWriteProcessBuffer`] over a given
    /// pointer and length.
    ///
    /// Publicly accessible constructor, which requires the
    /// [`capabilities::ExternalProcessCapability`] capability. This is provided
    /// to allow implementations of the [`Process`](crate::process::Process)
    /// trait outside of the `kernel` crate.
    ///
    /// # Safety requirements
    ///
    /// If the length is `0`, an arbitrary pointer may be passed into `ptr`. It
    /// does not necessarily have to point to allocated memory, nor does it have
    /// to meet [Rust's pointer validity
    /// requirements](https://doc.rust-lang.org/core/ptr/index.html#safety).
    /// [`ReadWriteProcessBuffer`] will ensure that all
    /// [`ReadableProcessSlice`]s and [`WriteableProcessSlice`]s with a length
    /// of `0` be constructed over a valid (but not necessarily allocated) base
    /// pointer.
    ///
    /// If the length is not `0`, the memory region of `[ptr; ptr + len)` must
    /// be valid memory of the process of the given [`ProcessId`]. The memory
    /// region must not contain address `0`, wrap the memory space or be larger
    /// than `isize::MAX`. It must be allocated and and accessible over the
    /// entire lifetime of the [`ReadWriteProcessBuffer`]. It must not point to
    /// memory outside of the process' accessible memory range, or point (in
    /// part) to other processes or kernel memory. The `ptr` must meet [Rust's
    /// requirements for pointer
    /// validity](https://doc.rust-lang.org/core/ptr/index.html#safety), in
    /// particular it must have a minimum alignment of
    /// `core::mem::align_of::<u8>()` on the respective platform. It must point
    /// to memory mapped as _readable_ and _writable_ and optionally
    /// _executable_.
    pub unsafe fn new_external(
        ptr: *mut u8,
        len: usize,
        process_id: ProcessId,
        _cap: &dyn capabilities::ExternalProcessCapability,
    ) -> Self {
        Self::new(ptr, len, process_id)
    }

    /// Consumes the ReadWriteProcessBuffer, returning its constituent pointer
    /// and size. This ensures that there cannot simultaneously be both a
    /// `ReadWriteProcessBuffer` and a pointer to its internal data.
    ///
    /// `consume` can be used when the kernel needs to pass the underlying
    /// values across the kernel-to-user boundary (e.g., in return values to
    /// system calls).
    pub(crate) fn consume(self) -> (*mut u8, usize) {
        (self.ptr, self.len)
    }

    /// This is a `const` version of `Default::default` with the same
    /// semantics.
    ///
    /// Having a const initializer allows initializing a fixed-size array with
    /// default values without the struct being marked `Copy` as such:
    ///
    /// ```
    /// use kernel::processbuffer::ReadWriteProcessBuffer;
    /// const DEFAULT_RWPROCBUF_VAL: ReadWriteProcessBuffer
    ///     = ReadWriteProcessBuffer::const_default();
    /// let my_array = [DEFAULT_RWPROCBUF_VAL; 12];
    /// ```
    pub const fn const_default() -> Self {
        Self {
            ptr: 0x0 as *mut u8,
            len: 0,
            process_id: None,
        }
    }
}

impl ReadableProcessBuffer for ReadWriteProcessBuffer {
    fn len(&self) -> usize {
        self.process_id
            .map_or(0, |pid| pid.kernel.process_map_or(0, pid, |_| self.len))
    }

    fn ptr(&self) -> *const u8 {
        if self.len == 0 {
            0x0 as *const u8
        } else {
            self.ptr
        }
    }

    fn enter<'a, F, R>(&'a self, fun: F) -> Result<R, process::Error>
    where
        F: FnOnce(ReadableProcessSlice<'a>) -> R,
    {
        match self.process_id {
            None => Err(process::Error::NoSuchApp),
            Some(pid) => pid
                .kernel
                .process_map_or(Err(process::Error::NoSuchApp), pid, |_| {
                    // ## Safety
                    //
                    // While the ReadWriteProcessBuffer must refuse construction
                    // with a null-pointer and a non-zero length, it may still
                    // contain a null-pointer. This is because it must be able
                    // to represent all valid read-only allow buffer
                    // descriptions passed from userspace. Nonetheless, a Rust
                    // slice (or any Rust allocation) cannot be located at
                    // address zero, even zero-sized types, and especially not a
                    // NonNull. Thus, apply the following rules:
                    //
                    // 1. if the buffer is of non-zero length, it must have a
                    //    non-null pointer (refer to
                    //    ReadableProcessBuffer::new_external). Construct a
                    //    NonNull based on that pointer.
                    //
                    // 2. if the buffer is of zero-length, it may have a null
                    //    pointer. Regardless, in the spirit of the
                    //    ReadableProcessSlice::ptr method, we don't have to
                    //    expose the proper pointer provided by the application,
                    //    as this does not have any meaning for a zero-sized
                    //    slice (not bounds checked, etc.). Thus, always return
                    //    a NonNull::dangling in this case.
                    //
                    // Through the basic assumption of non zero-length buffer
                    // having a properly aligned, non-null pointer, the
                    // following use of NonNull::new_unchecked is safe here:
                    let ptr = if self.len > 0 {
                        unsafe { NonNull::new_unchecked(self.ptr) }
                    } else {
                        NonNull::dangling()
                    };

                    // ## Safety
                    //
                    // `kernel.process_map_or()` validates that the process
                    // still exists and its memory is still valid. In
                    // particular, `Process` tracks the "high water mark" of
                    // memory that the process has `allow`ed to the
                    // kernel. Because `Process` does not feature an API to move
                    // the "high water mark" down again, which would be called
                    // once a `ProcessBuffer` has been passed back into the
                    // kernel, a given `Process` implementation must assume that
                    // the memory described by a once-allowed `ProcessBuffer` is
                    // still in use, and thus will not permit the process to
                    // free any memory after it has been `allow`ed to the kernel
                    // once. This guarantees that the memory pointed to by the
                    // buffer is safe to dereference here. For more information,
                    // refer to the comment and subsequent discussion on
                    // tock/tock#2632[1].
                    //
                    // [1]: https://github.com/tock/tock/pull/2632#issuecomment-869974365
                    Ok(fun(unsafe { ReadableProcessSlice::new(ptr, self.len) }))
                }),
        }
    }
}

impl WriteableProcessBuffer for ReadWriteProcessBuffer {
    fn mut_enter<'a, F, R>(&'a self, fun: F) -> Result<R, process::Error>
    where
        F: FnOnce(WriteableProcessSlice<'a>) -> R,
    {
        match self.process_id {
            None => Err(process::Error::NoSuchApp),
            Some(pid) => pid
                .kernel
                .process_map_or(Err(process::Error::NoSuchApp), pid, |_| {
                    // ## Safety
                    //
                    // While the ReadWriteProcessBuffer must refuse construction
                    // with a null-pointer and a non-zero length, it may still
                    // contain a null-pointer. This is because it must be able
                    // to represent all valid read-only allow buffer
                    // descriptions passed from userspace. Nonetheless, a Rust
                    // slice (or any Rust allocation) cannot be located at
                    // address zero, even zero-sized types, and especially not a
                    // NonNull. Thus, apply the following rules:
                    //
                    // 1. if the buffer is of non-zero length, it must have a
                    //    non-null pointer (refer to
                    //    ReadableProcessBuffer::new_external). Construct a
                    //    NonNull based on that pointer.
                    //
                    // 2. if the buffer is of zero-length, it may have a null
                    //    pointer. Regardless, in the spirit of the
                    //    ReadableProcessSlice::ptr method, we don't have to
                    //    expose the proper pointer provided by the application,
                    //    as this does not have any meaning for a zero-sized
                    //    slice (not bounds checked, etc.). Thus, always return
                    //    a NonNull::dangling in this case.
                    //
                    // Through the basic assumption of non zero-length buffer
                    // having a properly aligned, non-null pointer, the
                    // following use of NonNull::new_unchecked is safe here:
                    let ptr = if self.len > 0 {
                        unsafe { NonNull::new_unchecked(self.ptr) }
                    } else {
                        NonNull::dangling()
                    };

                    // ## Safety
                    //
                    // `kernel.process_map_or()` validates that the process
                    // still exists and its memory is still valid. In
                    // particular, `Process` tracks the "high water mark" of
                    // memory that the process has `allow`ed to the
                    // kernel. Because `Process` does not feature an API to move
                    // the "high water mark" down again, which would be called
                    // once a `ProcessBuffer` has been passed back into the
                    // kernel, a given `Process` implementation must assume that
                    // the memory described by a once-allowed `ProcessBuffer` is
                    // still in use, and thus will not permit the process to
                    // free any memory after it has been `allow`ed to the kernel
                    // once. This guarantees that the memory pointed to by the
                    // buffer is safe to dereference here. For more information,
                    // refer to the comment and subsequent discussion on
                    // tock/tock#2632[1].
                    //
                    // [1]: https://github.com/tock/tock/pull/2632#issuecomment-869974365
                    Ok(fun(unsafe { WriteableProcessSlice::new(ptr, self.len) }))
                }),
        }
    }
}

// TODO: remove, along with making the ProcessId non-optional and moving it to
// the front. This should save us a usize in some cases (e.g.
// Option<ReadWriteProcessBuffer>), as ProcessId must start with a non-null field
// (&'static Kernel) and thus ReadWriteProcessBuffer will also start with a
// non-null field.
impl Default for ReadWriteProcessBuffer {
    fn default() -> Self {
        Self::const_default()
    }
}

/// Provides access to a ReadWriteProcessBuffer with a restricted lifetime.
///
/// This automatically dereferences into a ReadWriteProcessBuffer.
pub struct ReadWriteProcessBufferRef<'a> {
    buf: ReadWriteProcessBuffer,
    _phantom: PhantomData<&'a ()>,
}

impl ReadWriteProcessBufferRef<'_> {
    /// Construct a new [`ReadWriteProcessBufferRef`] over a given pointer and
    /// length with a lifetime derived from the caller.
    ///
    /// # Safety requirements
    ///
    /// Refer to the safety requirements of
    /// [`ReadWriteProcessBuffer::new_external`]. The derived lifetime can help
    /// enforce the invariant that this incoming pointer may only be access for
    /// a certain duration.
    pub(crate) unsafe fn new(ptr: *mut u8, len: usize, process_id: ProcessId) -> Self {
        Self {
            buf: ReadWriteProcessBuffer::new(ptr, len, process_id),
            _phantom: PhantomData,
        }
    }
}

impl Deref for ReadWriteProcessBufferRef<'_> {
    type Target = ReadWriteProcessBuffer;
    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

/// A shareable region of userspace memory.
///
/// This trait can be used to gain read-write access to memory regions wrapped
/// in a ProcessBuffer type.
// We currently don't need any special functionality in the kernel for this
// type so we alias it as `ReadWriteProcessBuffer`.
pub type UserspaceReadableProcessBuffer = ReadWriteProcessBuffer;

// --------- PROCESS SLICES ----------------------------------------------------

/// Indexing operations on [`ReadableProcessSlice`]s and
/// [`WriteableProcessSlice`]s.
///
/// Because process slices are not proper Rust slice references, constituent of
/// a pointer and length, but rather structs containing this data, the Rust
/// index operator along with the [`Index`](core::ops::Index) trait cannot be
/// used, as it fundamentally requires an indexing operation to return a
/// reference.
///
/// Thus, provide a custom trait, used to implement a common `get` method to
/// subslice or index into process buffers.
pub trait ProcessSliceIndex<I> {
    type Output;

    fn get(&self, idx: I) -> Option<Self::Output>;
}

/// Safe abstraction over a readable byte in process memory.
///
/// The memory made accessible through a [`ReadableProcessByte`] must never be
/// written to by the kernel, over the entire lifetime of this
/// instance. However, it may either exist in flash (read-only memory) or RAM
/// (read-writeable memory). Consequently, a process may `allow` memory
/// overlapping with a [`ReadOnlyProcessBuffer`] also simultaneously through a
/// [`ReadWriteProcessBuffer`]. Hence, the kernel can have two references to the
/// same memory, where one can lead to mutation of the memory
/// contents. Therefore, the kernel must use raw pointer operations to
/// dereference this memory, to avoid violating Rust's aliasing rules.
///
/// This wrapper is transient, as the underlying buffer must be checked to point
/// to valid process memory each time an instance is created. This is enforced
/// through the associated lifetime.
pub struct ReadableProcessByte<'a> {
    ptr: NonNull<u8>,
    _lt: PhantomData<&'a ()>,
}

impl<'a> ReadableProcessByte<'a> {
    unsafe fn new(ptr: NonNull<u8>) -> Self {
        ReadableProcessByte {
            ptr,
            _lt: PhantomData,
        }
    }

    /// Retrieve the contents of the [`ReadableProcessByte`].
    #[inline]
    pub fn get(&self) -> u8 {
        unsafe { core::ptr::read(self.ptr.as_ptr()) }
    }
}

/// Readable and accessible slice of memory of a process buffer.
///
/// The only way to obtain this struct is through a [`ReadWriteProcessBuffer`]
/// or [`ReadOnlyProcessBuffer`], or based on an immutably borrowed Rust `&[u8]`
/// slice reference through the trait implementations of `From<&[u8]>` and
/// `From<&mut [u8]>`.
///
/// Slices provide a convenient, traditional interface to process memory. These
/// slices are transient, as the underlying buffer must be checked to point to
/// valid process memory each time a slice is created. This is enforced through
/// the associated lifetime.
///
/// The memory over which a [`ReadableProcessSlice`] exists must never be
/// written to by the kernel. However, it may either exist in flash (read-only
/// memory) or RAM (read-writeable memory). Consequently, a process may `allow`
/// memory overlapping with a [`ReadOnlyProcessBuffer`] also simultaneously
/// through a [`ReadWriteProcessBuffer`]. Hence, the kernel can have two
/// references to the same memory, where one can lead to mutation of the memory
/// contents. Therefore, the kernel must use raw pointer operations to
/// dereference this memory, to avoid violating Rust's aliasing rules.
pub struct ReadableProcessSlice<'a> {
    ptr: NonNull<u8>,
    len: usize,
    _lt: PhantomData<&'a ()>,
}

impl<'a> From<&'a [u8]> for ReadableProcessSlice<'a> {
    /// Borrow an immutable slice reference `&[u8]` into a
    /// `ReadableProcessSlice`.
    ///
    /// Allow an immutable slice reference `&[u8]` of lifetime `'a` to be
    /// borrowed into a `ReadableProcessSlice` of lifetime `'a`. This is to
    /// allow client code to be authored once and accept either `&'a [u8]` or
    /// `ReadableProcessBuffer<'a>`.
    fn from(val: &'a [u8]) -> Self {
        // # Safety
        //
        // This function immutably borrows `val` for the lifetime 'a, which is
        // further attached to the returned [`ReadableProcessSlice`]. As a
        // consequence, the original aliasing rules and lifetime constraints of
        // the slice hold.
        //
        // Furthermore, the provided Rust slice is guaranteed to have a non-null
        // pointer which is must to be valid for reads of the slice length *
        // mem::size_of::<u8>(), (based on the safety requirements of
        // `core::slice::from_raw_parts_mut`). Thus using NonNull::new_unchecked
        // is safe to use here.
        //
        // In addition to that, this function must adhere to all safety
        // requirements of [`ReadableProcessSlice::new_external`]:
        //
        // - By mutably borrowing `val` for the entire lifetime of the returned
        //   [`ReadableProcessSlice`] it is ensured that no other accessible
        //   Rust allocation is overlapping with the slice's memory region.
        //
        // - As documented with [core::slice::from_raw_parts], the entire memory
        //   range covered by a slice must be contained within a single
        //   allocated object, which in Rust cannot wrap around the address
        //   space. TODO: source!
        //
        //   Furthermore, Rust slices are, as are all other Rust allocations,
        //   limited to contain `isize::MAX` elements. Even more strict, slices
        //   are limited to contain `isize::MAX` _bytes_.
        //
        // For these reasons, the supplied `ptr` and `len` arguments satisfy the
        // constraints imposed by [`ReadableProcessSlice::new_external`].
        ReadableProcessSlice {
            ptr: unsafe { NonNull::new_unchecked(val.as_ptr() as *mut u8) },
            len: val.len(),
            _lt: PhantomData,
        }
    }
}

impl<'a> From<&'a mut [u8]> for ReadableProcessSlice<'a> {
    /// Borrow a mutable slice reference `&mut [u8]` into a
    /// `ReadableProcessSlice`.
    ///
    /// Allow an mutable slice reference `&mut [u8]` of lifetime `'a` to be
    /// borrowed into a `ReadableProcessSlice` of lifetime `'a`. This is to
    /// allow client code to be authored once and accept either `&'a mut [u8]`
    /// or `ReadableProcessBuffer<'a>`.
    #[inline]
    fn from(val: &'a mut [u8]) -> Self {
        ReadableProcessSlice::from(val as &'a [u8])
    }
}

impl<'a> ReadableProcessSlice<'a> {
    /// Construct a new [`ReadableProcessSlice`] of lifetime `'a`.
    ///
    /// This is a fundamentally unsafe operation.
    ///
    /// # Safety
    ///
    /// The constructed [`ReadableProcessSlice`] will make any memory in the
    /// bounds of `[ptr, ptr + len)` read-accessible through the returned
    /// object. Callers must ensure that no Rust aliasing rules be violated, in
    /// particular that no other accessible Rust allocation (e.g. through a
    /// mutable reference) is overlapping with this memory region.
    ///
    /// The provided length cannot overflow an isize, for reasons outlined in
    /// [`ptr::offset`](https://doc.rust-lang.org/stable/core/primitive.pointer.html#method.offset).
    /// If this condition is violated, the construction of a
    /// [`ReadableProcessSlice`] can lead to undefined behavior. Furthermore,
    /// the memory region must not wrap the address space, that is the address
    /// resulting from a wrapping-addition of `ptr + len` must always be
    /// strictly greater or equal than that of `ptr`.
    unsafe fn new(ptr: NonNull<u8>, len: usize) -> ReadableProcessSlice<'a> {
        ReadableProcessSlice {
            ptr,
            len,
            _lt: PhantomData,
        }
    }

    /// Copy the contents of a [`ReadableProcessSlice`] into a mutable
    /// slice reference.
    ///
    /// The length of `self` must be the same as `dest`, otherwise an error of
    /// `ErrorCode::SIZE` is returned. Subslicing can be used to obtain a slice
    /// of matching length.
    pub fn copy_to_slice(&self, dest: &mut [u8]) -> Result<(), ErrorCode> {
        // Method implemetation adopted from the
        // core::slice::copy_from_slice method implementation:
        // https://doc.rust-lang.org/src/core/slice/mod.rs.html#3034-3036

        if self.len() != dest.len() {
            Err(ErrorCode::SIZE)
        } else {
            // _If_ this turns out to not be efficiently optimized, it
            // should be possible to use a ptr::copy_nonoverlapping here
            // given we have exclusive mutable access to the destination
            // slice which will never be in process memory, and the layout
            // of &[ReadableProcessByte] is guaranteed to be compatible to
            // &[u8].
            for (i, b) in self.iter().enumerate() {
                dest[i] = b.get();
            }
            Ok(())
        }
    }

    /// Length of the [`WriteableProcessSlice`] memory region, in bytes.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Iterate over the [`ReadableProcessSlice`] memory region bytes.
    pub fn iter(&self) -> impl Iterator<Item = ReadableProcessByte<'a>> {
        unsafe { ReadableProcessSliceIter::new(self.ptr, self.len) }
    }

    /// Iterate over chunks the [`ReadableProcessSlice`] memory region bytes.
    ///
    /// `chunk_size` specifies the maximum number of bytes provided in each
    /// iteration. The last iteration may yield only a partial chunk, holding
    /// less than `chunk_size` bytes.
    pub fn chunks(&self, chunk_size: usize) -> impl Iterator<Item = ReadableProcessSlice<'a>> {
        unsafe { ReadableProcessSliceChunks::new(self.ptr, self.len, chunk_size) }
    }
}

impl<'a> ProcessSliceIndex<usize> for ReadableProcessSlice<'a> {
    type Output = ReadableProcessByte<'a>;

    #[inline]
    fn get(&self, idx: usize) -> Option<Self::Output> {
        // This is essentially a copy of the implementation of
        // <SliceIndex<[T]> for usize>::get()
        if idx < self.len {
            Some(unsafe {
                ReadableProcessByte::new(NonNull::new_unchecked(self.ptr.as_ptr().add(idx)))
            })
        } else {
            None
        }
    }
}

impl<'a> ProcessSliceIndex<Range<usize>> for ReadableProcessSlice<'a> {
    type Output = ReadableProcessSlice<'a>;

    #[inline]
    fn get(&self, range: Range<usize>) -> Option<Self::Output> {
        // This is essentially a copy of the implementation of
        // <SliceIndex<[T]> for Range<usize>>::get()
        if range.start > range.end || range.end > self.len {
            None
        } else {
            Some(ReadableProcessSlice {
                ptr: unsafe { NonNull::new_unchecked(self.ptr.as_ptr().add(range.start)) },
                len: range.end - range.start,
                _lt: PhantomData,
            })
        }
    }
}

impl<'a> ProcessSliceIndex<RangeFrom<usize>> for ReadableProcessSlice<'a> {
    type Output = ReadableProcessSlice<'a>;

    #[inline]
    fn get(&self, range: RangeFrom<usize>) -> Option<Self::Output> {
        ProcessSliceIndex::<Range<usize>>::get(self, range.start..self.len)
    }
}

impl<'a> ProcessSliceIndex<RangeTo<usize>> for ReadableProcessSlice<'a> {
    type Output = ReadableProcessSlice<'a>;

    #[inline]
    fn get(&self, range: RangeTo<usize>) -> Option<Self::Output> {
        ProcessSliceIndex::<Range<usize>>::get(self, 0..range.end)
    }
}

/// Iterator over bytes of a [`ReadableProcessSlice`].
///
/// Obtainable through [`ReadableProcessSlice::iter`]. Use subslicing with
/// [`ProcessSliceIndex`] to adjust the range of bytes iterated over.
pub struct ReadableProcessSliceIter<'a> {
    current: NonNull<u8>,
    remaining: usize,
    _lt: PhantomData<&'a ()>,
}

impl<'a> ReadableProcessSliceIter<'a> {
    unsafe fn new(base_ptr: NonNull<u8>, len: usize) -> Self {
        ReadableProcessSliceIter {
            current: base_ptr,
            remaining: len,
            _lt: PhantomData,
        }
    }
}

impl<'a> Iterator for ReadableProcessSliceIter<'a> {
    type Item = ReadableProcessByte<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining != 0 {
            let byte_handle = unsafe { ReadableProcessByte::new(self.current) };

            // Move the pointer forward, and decrement the number of remaining
            // iterations.
            self.current = unsafe { NonNull::new_unchecked(self.current.as_ptr().add(1)) };
            self.remaining -= 1;

            Some(byte_handle)
        } else {
            None
        }
    }
}

/// Iterator over chunks of bytes of a [`ReadableProcessSlice`].
///
/// Obtainable through [`ReadableProcessSlice::chunks`]. Use subslicing with
/// [`ProcessSliceIndex`] to adjust the range of bytes iterated over.
///
/// Each iteration will yield either a `None` or a `Some(ReadableProcessSlice)`,
/// containing `chunk_size` elements as specified in the
/// [`chunks`](ReadableProcessSlice::chunks) invocation. The last iteration may
/// yield a partial chunk, containing less than `chunk_size` elements.
pub struct ReadableProcessSliceChunks<'a> {
    current_ptr: NonNull<u8>,
    remaining: usize,
    chunk_size: usize,
    _lt: PhantomData<&'a ()>,
}

impl<'a> ReadableProcessSliceChunks<'a> {
    unsafe fn new(base_ptr: NonNull<u8>, len: usize, chunk_size: usize) -> Self {
        ReadableProcessSliceChunks {
            current_ptr: base_ptr,
            remaining: len,
            chunk_size,
            _lt: PhantomData,
        }
    }
}

impl<'a> Iterator for ReadableProcessSliceChunks<'a> {
    type Item = ReadableProcessSlice<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining != 0 {
            let current_chunk_size = core::cmp::min(self.chunk_size, self.remaining);
            let current_chunk =
                unsafe { ReadableProcessSlice::new(self.current_ptr, current_chunk_size) };

            // Move the pointer forward, and decrement the number of remaining
            // iterations.
            self.current_ptr = unsafe {
                NonNull::new_unchecked(self.current_ptr.as_ptr().add(current_chunk_size))
            };
            self.remaining -= current_chunk_size;

            Some(current_chunk)
        } else {
            None
        }
    }
}

/// Safe abstraction over a writeable byte in process memory.
///
/// A process may `allow` memory overlapping with a [`ReadWriteProcessBuffer`]
/// also simultaneously through a second [`ReadWriteProcessBuffer`]. Hence, the
/// kernel can have two references to the same memory, where one can lead to
/// mutation of the memory contents. Therefore, the kernel must use raw pointer
/// operations to dereference this memory, to avoid violating Rust's aliasing
/// rules.
///
/// This wrapper is transient, as the underlying buffer must be checked to point
/// to valid process memory each time an instance is created. This is enforced
/// through the associated lifetime.
pub struct WriteableProcessByte<'a> {
    ptr: NonNull<u8>,
    _lt: PhantomData<&'a ()>,
}

impl<'a> WriteableProcessByte<'a> {
    unsafe fn new(ptr: NonNull<u8>) -> Self {
        WriteableProcessByte {
            ptr,
            _lt: PhantomData,
        }
    }

    /// Retrieve the contents of the [`WriteableProcessByte`].
    #[inline]
    pub fn get(&self) -> u8 {
        unsafe { core::ptr::read(self.ptr.as_ptr()) }
    }

    /// Set the value of the [`WriteableProcessByte`].
    #[inline]
    pub fn set(&self, val: u8) {
        unsafe { core::ptr::write(self.ptr.as_ptr(), val) }
    }
}

/// Read-writeable and accessible slice of memory of a process buffer.
///
/// The only way to obtain this struct is through a [`ReadWriteProcessBuffer`],
/// or based on a mutably borrowed Rust `&mut [u8]` slice reference through the
/// trait implementations `From<&mut [u8]>`.
///
/// Slices provide a convenient, traditional interface to process memory. These
/// slices are transient, as the underlying buffer must be checked to point to
/// valid process memory each time a slice is created. This is enforced through
/// the associated lifetime.
///
/// A process may `allow` memory overlapping with a [`ReadWriteProcessBuffer`]
/// also simultaneously through a second [`ReadWriteProcessBuffer`]. Hence, the
/// kernel can have two references to the same memory, where one can lead to
/// mutation of the memory contents. Therefore, the kernel must use raw pointer
/// operations to dereference this memory, to avoid violating Rust's aliasing
/// rules.
pub struct WriteableProcessSlice<'a> {
    ptr: NonNull<u8>,
    len: usize,
    _lt: PhantomData<&'a ()>,
}

impl<'a> From<&'a mut [u8]> for WriteableProcessSlice<'a> {
    /// Borrow a mutable slice reference `&mut [u8]` into a
    /// `WriteableProcessSlice`.
    ///
    /// Allow a mutable slice reference `&mut [u8]` of lifetime `'a` to be
    /// borrowed into a `WriteableProcessSlice` of lifetime `'a`. This is to
    /// allow client code to be authored once and accept either `&'a mut [u8]`
    /// or `WriteableProcessBuffer<'a>`.
    fn from(val: &'a mut [u8]) -> Self {
        // # Safety
        //
        // This function mutably borrows `val` for the lifetime 'a, which is
        // further attached to the returned [`WriteableProcessSlice`]. As a
        // consequence the original aliasing rules and lifetime constraints of
        // the slice hold.
        //
        // Furthermore, the provided Rust slice is guaranteed to have a non-null
        // pointer which is must to be valid for reads of the slice length *
        // mem::size_of::<u8>(), (based on the safety requirements of
        // `core::slice::from_raw_parts_mut`). Thus using NonNull::new_unchecked
        // is safe to use here.
        //
        // In addition to that, this function must adhere to all safety
        // requirements of [`WriteableProcessSlice::new_external`]:
        //
        // - By mutably borrowing `val` for the entire lifetime of the returned
        //   [`WriteableProcessSlice`] it is ensured that no other accessible
        //   Rust allocation is overlapping with the slice's memory region.
        //
        // - As documented with [core::slice::from_raw_parts], the entire memory
        //   range covered by a slice must be contained within a single
        //   allocated object, which in Rust cannot wrap around the address
        //   space. TODO: source!
        //
        //   Furthermore, Rust slices are, as are all other Rust allocations,
        //   limited to contain `isize::MAX` elements. Even more strict, slices
        //   are limited to contain `isize::MAX` _bytes_.
        //
        // For these reasons, the supplied `ptr` and `len` arguments satisfy the
        // constraints imposed by [`WriteableProcessSlice::new_external`].
        unsafe { WriteableProcessSlice::new(NonNull::new_unchecked(val.as_mut_ptr()), val.len()) }
    }
}

impl<'a> WriteableProcessSlice<'a> {
    /// Construct a new [`WriteableProcessSlice`] of lifetime `'a`.
    ///
    /// This is a fundamentally unsafe operation.
    ///
    /// # Safety
    ///
    /// The constructed [`WriteableProcessSlice`] will make any memory in the
    /// bounds of `[ptr, ptr + len)` write-accessible through the returned
    /// object. Callers must ensure that no Rust aliasing rules be violated, in
    /// particular that no other accessible Rust allocation (e.g. through an
    /// immutable or mutable reference) is overlapping with this memory region.
    ///
    /// The provided length cannot overflow an isize, for reasons outlined in
    /// [`ptr::offset`](https://doc.rust-lang.org/stable/core/primitive.pointer.html#method.offset).
    /// If this condition is violated, the construction of a
    /// [`WriteableProcessSlice`] can lead to undefined behavior. Furthermore,
    /// the memory region must not wrap the address space, that is the address
    /// resulting from a wrapping-addition of `ptr + len` must always be
    /// strictly greater or equal than that of `ptr`.
    unsafe fn new(ptr: NonNull<u8>, len: usize) -> WriteableProcessSlice<'a> {
        WriteableProcessSlice {
            ptr,
            len,
            _lt: PhantomData,
        }
    }

    /// Copy the contents of a [`WriteableProcessSlice`] into a mutable
    /// slice reference.
    ///
    /// The length of `self` must be the same as `dest`, otherwise an error of
    /// `ErrorCode::SIZE` is returned. Subslicing can be used to obtain a slice
    /// of matching length.
    pub fn copy_to_slice(&self, dest: &mut [u8]) -> Result<(), ErrorCode> {
        // Method implemetation adopted from the
        // core::slice::copy_from_slice method implementation:
        // https://doc.rust-lang.org/src/core/slice/mod.rs.html#3034-3036

        if self.len() != dest.len() {
            Err(ErrorCode::SIZE)
        } else {
            // _If_ this turns out to not be efficiently optimized, it
            // should be possible to use a ptr::copy_nonoverlapping here
            // given we have exclusive mutable access to the destination
            // slice which will never be in process memory, and the layout
            // of &[Cell<u8>] is guaranteed to be compatible to &[u8].
            self.iter()
                .zip(dest.iter_mut())
                .for_each(|(src, dst)| *dst = src.get());
            Ok(())
        }
    }

    /// Copy the contents of a slice of bytes into a [`WriteableProcessSlice`].
    ///
    /// The length of `src` must be the same as `self`, otherwise an error of
    /// `ErrorCode::SIZE` is returned. Subslicing can be used to obtain a slice
    /// of matching length.
    pub fn copy_from_slice(&self, src: &[u8]) -> Result<(), ErrorCode> {
        // Method implemetation adopted from the
        // core::slice::copy_from_slice method implementation:
        // https://doc.rust-lang.org/src/core/slice/mod.rs.html#3034-3036

        if self.len() != src.len() {
            Err(ErrorCode::SIZE)
        } else {
            // _If_ this turns out to not be efficiently optimized, it
            // should be possible to use a ptr::copy_nonoverlapping here
            // given we have exclusive mutable access to the destination
            // slice which will never be in process memory, and the layout
            // of &[Cell<u8>] is guaranteed to be compatible to &[u8].
            src.iter()
                .zip(self.iter())
                .for_each(|(src, dst)| dst.set(*src));
            Ok(())
        }
    }

    /// Length of the [`WriteableProcessSlice`] memory region, in bytes.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Iterate over the [`WriteableProcessSlice`] memory region bytes.
    pub fn iter(&self) -> impl Iterator<Item = WriteableProcessByte> {
        unsafe { WriteableProcessSliceIter::new(self.ptr, self.len) }
    }

    /// Iterate over chunks the [`WriteableProcessSlice`] memory region bytes.
    ///
    /// `chunk_size` specifies the maximum number of bytes provided in each
    /// iteration. The last iteration may yield only a partial chunk, holding
    /// less than `chunk_size` bytes.
    pub fn chunks(
        &self,
        chunk_size: usize,
    ) -> impl core::iter::Iterator<Item = WriteableProcessSlice<'a>> {
        unsafe { WriteableProcessSliceChunks::new(self.ptr, self.len, chunk_size) }
    }
}

impl<'a> ProcessSliceIndex<usize> for WriteableProcessSlice<'a> {
    type Output = WriteableProcessByte<'a>;

    #[inline]
    fn get(&self, idx: usize) -> Option<Self::Output> {
        // This is essentially a copy of the implementation of
        // <SliceIndex<[T]> for usize>::get_mut()
        if idx < self.len {
            Some(unsafe {
                WriteableProcessByte::new(NonNull::new_unchecked(self.ptr.as_ptr().add(idx)))
            })
        } else {
            None
        }
    }
}

impl<'a> ProcessSliceIndex<Range<usize>> for WriteableProcessSlice<'a> {
    type Output = WriteableProcessSlice<'a>;

    #[inline]
    fn get(&self, range: Range<usize>) -> Option<Self::Output> {
        // This is essentially a copy of the implementation of
        // <SliceIndex<[T]> for Range<usize>>::get_mut()
        if range.start > range.end || range.end > self.len {
            None
        } else {
            Some(WriteableProcessSlice {
                ptr: unsafe { NonNull::new_unchecked(self.ptr.as_ptr().add(range.start)) },
                len: range.end - range.start,
                _lt: PhantomData,
            })
        }
    }
}

impl<'a> ProcessSliceIndex<RangeFrom<usize>> for WriteableProcessSlice<'a> {
    type Output = WriteableProcessSlice<'a>;

    #[inline]
    fn get(&self, range: RangeFrom<usize>) -> Option<Self::Output> {
        ProcessSliceIndex::<Range<usize>>::get(self, range.start..self.len)
    }
}

impl<'a> ProcessSliceIndex<RangeTo<usize>> for WriteableProcessSlice<'a> {
    type Output = WriteableProcessSlice<'a>;

    #[inline]
    fn get(&self, range: RangeTo<usize>) -> Option<Self::Output> {
        ProcessSliceIndex::<Range<usize>>::get(self, 0..range.end)
    }
}

/// Iterator over bytes of a [`ReadableProcessSlice`].
///
/// Obtainable through [`ReadableProcessSlice::iter`]. Use subslicing with
/// [`ProcessSliceIndex`] to adjust the range of bytes iterated over.
pub struct WriteableProcessSliceIter<'a> {
    current: NonNull<u8>,
    remaining: usize,
    _lt: PhantomData<&'a ()>,
}

impl<'a> WriteableProcessSliceIter<'a> {
    unsafe fn new(base_ptr: NonNull<u8>, len: usize) -> Self {
        WriteableProcessSliceIter {
            current: base_ptr,
            remaining: len,
            _lt: PhantomData,
        }
    }
}

impl<'a> Iterator for WriteableProcessSliceIter<'a> {
    type Item = WriteableProcessByte<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining != 0 {
            let byte_handle = unsafe { WriteableProcessByte::new(self.current) };

            // Move the pointer forward, and decrement the number of remaining
            // iterations.
            //
            // # Safety
            //
            // TODO
            self.current = unsafe { NonNull::new_unchecked(self.current.as_ptr().add(1)) };
            self.remaining -= 1;

            Some(byte_handle)
        } else {
            None
        }
    }
}

/// Iterator over chunks of bytes of a [`WriteableProcessSlice`].
///
/// Obtainable through [`WriteableProcessSlice::chunks`]. Use subslicing with
/// [`ProcessSliceIndex`] to adjust the range of bytes iterated over.
///
/// Each iteration will yield either a `None` or a
/// `Some(WriteableProcessSlice)`, containing `chunk_size` elements as specified
/// in the [`chunks`](WriteableProcessSlice::chunks) invocation. The last
/// iteration may yield a partial chunk, containing less than `chunk_size`
/// elements.
pub struct WriteableProcessSliceChunks<'a> {
    current_ptr: NonNull<u8>,
    remaining: usize,
    chunk_size: usize,
    _lt: PhantomData<&'a ()>,
}

impl<'a> WriteableProcessSliceChunks<'a> {
    unsafe fn new(base_ptr: NonNull<u8>, len: usize, chunk_size: usize) -> Self {
        WriteableProcessSliceChunks {
            current_ptr: base_ptr,
            remaining: len,
            chunk_size,
            _lt: PhantomData,
        }
    }
}

impl<'a> Iterator for WriteableProcessSliceChunks<'a> {
    type Item = WriteableProcessSlice<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining != 0 {
            let current_chunk_size = core::cmp::min(self.chunk_size, self.remaining);
            let current_chunk =
                unsafe { WriteableProcessSlice::new(self.current_ptr, current_chunk_size) };

            // Move the pointer forward, and decrement the number of remaining
            // iterations.
            self.current_ptr = unsafe {
                NonNull::new_unchecked(self.current_ptr.as_ptr().add(current_chunk_size))
            };
            self.remaining -= current_chunk_size;

            Some(current_chunk)
        } else {
            None
        }
    }
}
