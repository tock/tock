// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Data structures for passing application memory to the kernel.
//!
//! A Tock process can pass read-write or read-only buffers into the
//! kernel for it to use. The kernel checks that read-write buffers
//! exist within a process's RAM address space, and that read-only
//! buffers exist either within its RAM or flash address space. These
//! buffers are shared with the allow_read_write() and
//! allow_read_only() system calls.
//!
//! A read-write and read-only call is mapped to the high-level Rust
//! types [`ReadWriteProcessBuffer`] and [`ReadOnlyProcessBuffer`]
//! respectively. The memory regions can be accessed through the
//! [`ReadableProcessBuffer`] and [`WriteableProcessBuffer`] traits,
//! implemented on the process buffer structs.
//!
//! Each access to the buffer structs requires a liveness check to ensure that
//! the process memory is still valid. For a more traditional interface, users
//! can convert buffers into [`ReadableProcessSlice`] or
//! [`WriteableProcessSlice`] and use these for the lifetime of their
//! operations. Users cannot hold live-lived references to these slices,
//! however.

use core::cell::Cell;
use core::marker::PhantomData;
use core::ops::{Deref, Index, Range, RangeFrom, RangeTo};

use crate::capabilities;
use crate::process::{self, ProcessId};
use crate::ErrorCode;

/// Convert a process buffer's internal representation to a
/// [`ReadableProcessSlice`].
///
/// This function will automatically convert zero-length process
/// buffers into valid zero-sized Rust slices regardless of the value
/// of `ptr`.
///
/// # Safety requirements
///
/// In the case of `len != 0`, the memory `[ptr; ptr + len)` must be
/// within a single process' address space, and `ptr` must be
/// nonzero. This memory region must be mapped as _readable_, and
/// optionally _writable_ and _executable_. It must be allocated
/// within a single process' address space for the entire lifetime
/// `'a`.
///
/// It is sound for multiple overlapping [`ReadableProcessSlice`]s or
/// [`WriteableProcessSlice`]s to be in scope at the same time.
unsafe fn raw_processbuf_to_roprocessslice<'a>(
    ptr: *const u8,
    len: usize,
) -> &'a ReadableProcessSlice {
    // Transmute a reference to a slice of Cell<u8>s into a reference
    // to a ReadableProcessSlice. This is possible as
    // ReadableProcessSlice is a #[repr(transparent)] wrapper around a
    // [ReadableProcessByte], which is a #[repr(transparent)] wrapper
    // around a [Cell<u8>], which is a #[repr(transparent)] wrapper
    // around an [UnsafeCell<u8>], which finally #[repr(transparent)]
    // wraps a [u8]
    core::mem::transmute::<&[u8], &ReadableProcessSlice>(
        // Rust has very strict requirements on pointer validity[1]
        // which also in part apply to accesses of length 0. We allow
        // an application to supply arbitrary pointers if the buffer
        // length is 0, but this is not allowed for Rust slices. For
        // instance, a null pointer is _never_ valid, not even for
        // accesses of size zero.
        //
        // To get a pointer which does not point to valid (allocated)
        // memory, but is safe to construct for accesses of size zero,
        // we must call NonNull::dangling(). The resulting pointer is
        // guaranteed to be well-aligned and uphold the guarantees
        // required for accesses of size zero.
        //
        // [1]: https://doc.rust-lang.org/core/ptr/index.html#safety
        match len {
            0 => core::slice::from_raw_parts(core::ptr::NonNull::<u8>::dangling().as_ptr(), 0),
            _ => core::slice::from_raw_parts(ptr, len),
        },
    )
}

/// Convert an process buffers's internal representation to a
/// [`WriteableProcessSlice`].
///
/// This function will automatically convert zero-length process
/// buffers into valid zero-sized Rust slices regardless of the value
/// of `ptr`.
///
/// # Safety requirements
///
/// In the case of `len != 0`, the memory `[ptr; ptr + len)` must be
/// within a single process' address space, and `ptr` must be
/// nonzero. This memory region must be mapped as _readable_ and
/// _writable_, and optionally _executable_. It must be allocated
/// within a single process' address space for the entire lifetime
/// `'a`.
///
/// No other mutable or immutable Rust reference pointing to an
/// overlapping memory region, which is not also created over
/// `UnsafeCell`, may exist over the entire lifetime `'a`. Even though
/// this effectively returns a slice of [`Cell`]s, writing to some
/// memory through a [`Cell`] while another reference is in scope is
/// unsound. Because a process is free to modify its memory, this is
/// -- in a broader sense -- true for all process memory.
///
/// However, it is sound for multiple overlapping
/// [`ReadableProcessSlice`]s or [`WriteableProcessSlice`]s to be in
/// scope at the same time.
unsafe fn raw_processbuf_to_rwprocessslice<'a>(
    ptr: *mut u8,
    len: usize,
) -> &'a WriteableProcessSlice {
    // Transmute a reference to a slice of Cell<u8>s into a reference
    // to a ReadableProcessSlice. This is possible as
    // ReadableProcessSlice is a #[repr(transparent)] wrapper around a
    // [ReadableProcessByte], which is a #[repr(transparent)] wrapper
    // around a [Cell<u8>], which is a #[repr(transparent)] wrapper
    // around an [UnsafeCell<u8>], which finally #[repr(transparent)]
    // wraps a [u8]
    core::mem::transmute::<&[u8], &WriteableProcessSlice>(
        // Rust has very strict requirements on pointer validity[1]
        // which also in part apply to accesses of length 0. We allow
        // an application to supply arbitrary pointers if the buffer
        // length is 0, but this is not allowed for Rust slices. For
        // instance, a null pointer is _never_ valid, not even for
        // accesses of size zero.
        //
        // To get a pointer which does not point to valid (allocated)
        // memory, but is safe to construct for accesses of size zero,
        // we must call NonNull::dangling(). The resulting pointer is
        // guaranteed to be well-aligned and uphold the guarantees
        // required for accesses of size zero.
        //
        // [1]: https://doc.rust-lang.org/core/ptr/index.html#safety
        match len {
            0 => core::slice::from_raw_parts_mut(core::ptr::NonNull::<u8>::dangling().as_ptr(), 0),
            _ => core::slice::from_raw_parts_mut(ptr, len),
        },
    )
}

/// A readable region of userspace process memory.
///
/// This trait can be used to gain read-only access to memory regions
/// wrapped in either a [`ReadOnlyProcessBuffer`] or a
/// [`ReadWriteProcessBuffer`] type.
///
/// # Safety
///
/// This is an `unsafe trait` as users of this trait need to trust that the
/// implementation of [`ReadableProcessBuffer::ptr`] is correct. Implementors of
/// this trait must ensure that the [`ReadableProcessBuffer::ptr`] method
/// follows the semantics and invariants described in its documentation.
pub unsafe trait ReadableProcessBuffer {
    /// Length of the memory region.
    ///
    /// If the process is no longer alive and the memory has been
    /// reclaimed, this method must return 0.
    ///
    /// # Default Process Buffer
    ///
    /// A default instance of a process buffer must return 0.
    fn len(&self) -> usize;

    /// Pointer to the first byte of the userspace-allowed memory region.
    ///
    /// If [`ReadableProcessBuffer::len`] returns a non-zero value,
    /// then this method is guaranteed to return a pointer to the
    /// start address of a memory region (of length returned by
    /// `len`), allowable by a userspace process, and allowed to the
    /// kernel for read operations. The memory region must not be
    /// written to through this pointer.
    ///
    /// If the length of the initially shared memory region
    /// (irrespective of the return value of
    /// [`len`](ReadableProcessBuffer::len)) is 0, this function
    /// returns a pointer to address `0x0`. This is because processes
    /// may allow zero-length buffer to share no memory with the
    /// kernel. Because these buffers have zero length, they may have
    /// any arbitrary pointer value. However, these "dummy addresses"
    /// should not be leaked, so this method returns 0 for zero-length
    /// slices. Care must be taken to not create a Rust (slice)
    /// reference to a null-pointer, as that is unsound.
    ///
    /// Users of this pointer must not produce any mutable aliasing, such as by
    /// creating a reference from this pointer concurrently with calling
    /// [`WriteableProcessBuffer::mut_enter`].
    ///
    /// # Default Process Buffer
    ///
    /// A default instance of a process buffer must return a pointer
    /// to address `0x0`.
    fn ptr(&self) -> *const u8;

    /// Applies a function to the (read only) process slice reference
    /// pointed to by the process buffer.
    ///
    /// If the process is no longer alive and the memory has been
    /// reclaimed, this method must return
    /// `Err(process::Error::NoSuchApp)`.
    ///
    /// # Default Process Buffer
    ///
    /// A default instance of a process buffer must return
    /// `Err(process::Error::NoSuchApp)` without executing the passed
    /// closure.
    fn enter<F, R>(&self, fun: F) -> Result<R, process::Error>
    where
        F: FnOnce(&ReadableProcessSlice) -> R;
}

/// A readable and writeable region of userspace process memory.
///
/// This trait can be used to gain read-write access to memory regions
/// wrapped in a [`ReadWriteProcessBuffer`].
///
/// This is a supertrait of [`ReadableProcessBuffer`], which features
/// methods allowing mutable access.
///
/// # Safety
///
/// This is an `unsafe trait` as users of this trait need to trust that the
/// implementation of [`WriteableProcessBuffer::mut_ptr`] is
/// correct.
///
/// Implementors of this trait must ensure that the
/// [`WriteableProcessBuffer::mut_ptr`] method follows the semantics and
/// invariants described in its documentation, and that the length of the
/// [`WriteableProcessBuffer`] is identical to the value returned by the
/// [`ReadableProcessBuffer::len`] supertrait method.
///
/// Additionally, when using the default implementation of `mut_ptr` provided by
/// this trait, implementors guarantee that the readable pointer returned by
/// [`ReadableProcessBuffer::ptr`] points to the same read-write allowed shared
/// memory region as described by the [`WriteableProcessBuffer`], and that
/// writes through the pointer returned by [`ReadableProcessBuffer::ptr`] are
/// sound for [`ReadableProcessBuffer::len`] bytes, notwithstanding any aliasing
/// requirements.
pub unsafe trait WriteableProcessBuffer: ReadableProcessBuffer {
    /// Pointer to the first byte of the userspace-allowed memory region.
    ///
    /// If [`ReadableProcessBuffer::len`] returns a non-zero value,
    /// then this method is guaranteed to return a pointer to the
    /// start address of a memory region (of length returned by
    /// `len`), allowable by a userspace process, and allowed to the
    /// kernel for read or write operations.
    ///
    /// If the length of the initially shared memory region
    /// (irrespective of the return value of
    /// [`len`](ReadableProcessBuffer::len)) is 0, this function
    /// returns a pointer to address `0x0`. This is because processes
    /// may allow zero-length buffer to share no memory with the
    /// kernel. Because these buffers have zero length, they may have
    /// any arbitrary pointer value. However, these "dummy addresses"
    /// should not be leaked, so this method returns 0 for zero-length
    /// slices. Care must be taken to not create a Rust (slice)
    /// reference to a null-pointer, as that is unsound.
    ///
    /// Users of this pointer must not produce any mutable aliasing, such as by
    /// creating a reference from this pointer concurrently with calling
    /// [`WriteableProcessBuffer::mut_enter`].
    ///
    /// # Default Process Buffer
    ///
    /// A default instance of a process buffer must return a pointer
    /// to address `0x0`.
    fn mut_ptr(&self) -> *mut u8 {
        ReadableProcessBuffer::ptr(self) as *mut u8
    }

    /// Applies a function to the mutable process slice reference
    /// pointed to by the [`ReadWriteProcessBuffer`].
    ///
    /// If the process is no longer alive and the memory has been
    /// reclaimed, this method must return
    /// `Err(process::Error::NoSuchApp)`.
    ///
    /// # Default Process Buffer
    ///
    /// A default instance of a process buffer must return
    /// `Err(process::Error::NoSuchApp)` without executing the passed
    /// closure.
    fn mut_enter<F, R>(&self, fun: F) -> Result<R, process::Error>
    where
        F: FnOnce(&WriteableProcessSlice) -> R;
}

/// Read-only buffer shared by a userspace process.
///
/// This struct is provided to capsules when a process `allow`s a
/// particular section of its memory to the kernel and gives the
/// kernel read access to this memory.
///
/// It can be used to obtain a [`ReadableProcessSlice`], which is
/// based around a slice of [`Cell`]s. This is because a userspace can
/// `allow` overlapping sections of memory into different
/// [`ReadableProcessSlice`]. Having at least one mutable Rust slice
/// along with read-only slices to overlapping memory in Rust violates
/// Rust's aliasing rules. A slice of [`Cell`]s avoids this issue by
/// explicitly supporting interior mutability. Still, a memory barrier
/// prior to switching to userspace is required, as the compiler is
/// free to reorder reads and writes, even through [`Cell`]s.
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
    /// [`capabilities::ExternalProcessCapability`] capability. This
    /// is provided to allow implementations of the
    /// [`Process`](crate::process::Process) trait outside of the
    /// `kernel` crate.
    ///
    /// # Safety requirements
    ///
    /// If the length is `0`, an arbitrary pointer may be passed into
    /// `ptr`. It does not necessarily have to point to allocated
    /// memory, nor does it have to meet [Rust's pointer validity
    /// requirements](https://doc.rust-lang.org/core/ptr/index.html#safety).
    /// [`ReadOnlyProcessBuffer`] must ensure that all Rust slices
    /// with a length of `0` must be constructed over a valid (but not
    /// necessarily allocated) base pointer.
    ///
    /// If the length is not `0`, the memory region of `[ptr; ptr +
    /// len)` must be valid memory of the process of the given
    /// [`ProcessId`]. It must be allocated and and accessible over
    /// the entire lifetime of the [`ReadOnlyProcessBuffer`]. It must
    /// not point to memory outside of the process' accessible memory
    /// range, or point (in part) to other processes or kernel
    /// memory. The `ptr` must meet [Rust's requirements for pointer
    /// validity](https://doc.rust-lang.org/core/ptr/index.html#safety),
    /// in particular it must have a minimum alignment of
    /// `core::mem::align_of::<u8>()` on the respective platform. It
    /// must point to memory mapped as _readable_ and optionally
    /// _writable_ and _executable_.
    pub unsafe fn new_external(
        ptr: *const u8,
        len: usize,
        process_id: ProcessId,
        _cap: &dyn capabilities::ExternalProcessCapability,
    ) -> Self {
        Self::new(ptr, len, process_id)
    }

    /// Consumes the ReadOnlyProcessBuffer, returning its constituent
    /// pointer and size. This ensures that there cannot
    /// simultaneously be both a `ReadOnlyProcessBuffer` and a pointer
    /// to its internal data.
    ///
    /// `consume` can be used when the kernel needs to pass the
    /// underlying values across the kernel-to-user boundary (e.g., in
    /// return values to system calls).
    pub(crate) fn consume(self) -> (*const u8, usize) {
        (self.ptr, self.len)
    }
}

unsafe impl ReadableProcessBuffer for ReadOnlyProcessBuffer {
    /// Return the length of the buffer in bytes.
    fn len(&self) -> usize {
        self.process_id
            .map_or(0, |pid| pid.kernel.process_map_or(0, pid, |_| self.len))
    }

    /// Return the pointer to the start of the buffer.
    fn ptr(&self) -> *const u8 {
        if self.len == 0 {
            core::ptr::null::<u8>()
        } else {
            self.ptr
        }
    }

    /// Access the contents of the buffer in a closure.
    ///
    /// This verifies the process is still valid before accessing the underlying
    /// memory.
    fn enter<F, R>(&self, fun: F) -> Result<R, process::Error>
    where
        F: FnOnce(&ReadableProcessSlice) -> R,
    {
        match self.process_id {
            None => Err(process::Error::NoSuchApp),
            Some(pid) => pid
                .kernel
                .process_map_or(Err(process::Error::NoSuchApp), pid, |_| {
                    // Safety: `kernel.process_map_or()` validates that
                    // the process still exists and its memory is still
                    // valid. In particular, `Process` tracks the "high water
                    // mark" of memory that the process has `allow`ed to the
                    // kernel. Because `Process` does not feature an API to
                    // move the "high water mark" down again, which would be
                    // called once a `ProcessBuffer` has been passed back into
                    // the kernel, a given `Process` implementation must assume
                    // that the memory described by a once-allowed
                    // `ProcessBuffer` is still in use, and thus will not
                    // permit the process to free any memory after it has
                    // been `allow`ed to the kernel once. This guarantees
                    // that the buffer is safe to convert into a slice
                    // here. For more information, refer to the
                    // comment and subsequent discussion on tock/tock#2632:
                    // https://github.com/tock/tock/pull/2632#issuecomment-869974365
                    Ok(fun(unsafe {
                        raw_processbuf_to_roprocessslice(self.ptr, self.len)
                    }))
                }),
        }
    }
}

impl Default for ReadOnlyProcessBuffer {
    fn default() -> Self {
        ReadOnlyProcessBuffer {
            ptr: core::ptr::null_mut::<u8>(),
            len: 0,
            process_id: None,
        }
    }
}

/// Provides access to a [`ReadOnlyProcessBuffer`] with a restricted lifetime.
/// This automatically dereferences into a ReadOnlyProcessBuffer
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
    /// [`ReadOnlyProcessBuffer::new_external`]. The derived lifetime can
    /// help enforce the invariant that this incoming pointer may only
    /// be access for a certain duration.
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
/// This struct is provided to capsules when a process `allows` a
/// particular section of its memory to the kernel and gives the
/// kernel read and write access to this memory.
///
/// It can be used to obtain a [`WriteableProcessSlice`], which is
/// based around a slice of [`Cell`]s. This is because a userspace can
/// `allow` overlapping sections of memory into different
/// [`WriteableProcessSlice`]. Having at least one mutable Rust slice
/// along with read-only or other mutable slices to overlapping memory
/// in Rust violates Rust's aliasing rules. A slice of [`Cell`]s
/// avoids this issue by explicitly supporting interior
/// mutability. Still, a memory barrier prior to switching to
/// userspace is required, as the compiler is free to reorder reads
/// and writes, even through [`Cell`]s.
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
    /// [`capabilities::ExternalProcessCapability`] capability. This
    /// is provided to allow implementations of the
    /// [`Process`](crate::process::Process) trait outside of the
    /// `kernel` crate.
    ///
    /// # Safety requirements
    ///
    /// If the length is `0`, an arbitrary pointer may be passed into
    /// `ptr`. It does not necessarily have to point to allocated
    /// memory, nor does it have to meet [Rust's pointer validity
    /// requirements](https://doc.rust-lang.org/core/ptr/index.html#safety).
    /// [`ReadWriteProcessBuffer`] must ensure that all Rust slices
    /// with a length of `0` must be constructed over a valid (but not
    /// necessarily allocated) base pointer.
    ///
    /// If the length is not `0`, the memory region of `[ptr; ptr +
    /// len)` must be valid memory of the process of the given
    /// [`ProcessId`]. It must be allocated and and accessible over
    /// the entire lifetime of the [`ReadWriteProcessBuffer`]. It must
    /// not point to memory outside of the process' accessible memory
    /// range, or point (in part) to other processes or kernel
    /// memory. The `ptr` must meet [Rust's requirements for pointer
    /// validity](https://doc.rust-lang.org/core/ptr/index.html#safety),
    /// in particular it must have a minimum alignment of
    /// `core::mem::align_of::<u8>()` on the respective platform. It
    /// must point to memory mapped as _readable_ and optionally
    /// _writable_ and _executable_.
    pub unsafe fn new_external(
        ptr: *mut u8,
        len: usize,
        process_id: ProcessId,
        _cap: &dyn capabilities::ExternalProcessCapability,
    ) -> Self {
        Self::new(ptr, len, process_id)
    }

    /// Consumes the ReadWriteProcessBuffer, returning its constituent
    /// pointer and size. This ensures that there cannot
    /// simultaneously be both a `ReadWriteProcessBuffer` and a pointer to
    /// its internal data.
    ///
    /// `consume` can be used when the kernel needs to pass the
    /// underlying values across the kernel-to-user boundary (e.g., in
    /// return values to system calls).
    pub(crate) fn consume(self) -> (*mut u8, usize) {
        (self.ptr, self.len)
    }

    /// This is a `const` version of `Default::default` with the same
    /// semantics.
    ///
    /// Having a const initializer allows initializing a fixed-size
    /// array with default values without the struct being marked
    /// `Copy` as such:
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

unsafe impl ReadableProcessBuffer for ReadWriteProcessBuffer {
    /// Return the length of the buffer in bytes.
    fn len(&self) -> usize {
        self.process_id
            .map_or(0, |pid| pid.kernel.process_map_or(0, pid, |_| self.len))
    }

    /// Return the pointer to the start of the buffer.
    fn ptr(&self) -> *const u8 {
        if self.len == 0 {
            core::ptr::null::<u8>()
        } else {
            self.ptr
        }
    }

    /// Access the contents of the buffer in a closure.
    ///
    /// This verifies the process is still valid before accessing the underlying
    /// memory.
    fn enter<F, R>(&self, fun: F) -> Result<R, process::Error>
    where
        F: FnOnce(&ReadableProcessSlice) -> R,
    {
        match self.process_id {
            None => Err(process::Error::NoSuchApp),
            Some(pid) => pid
                .kernel
                .process_map_or(Err(process::Error::NoSuchApp), pid, |_| {
                    // Safety: `kernel.process_map_or()` validates that
                    // the process still exists and its memory is still
                    // valid. In particular, `Process` tracks the "high water
                    // mark" of memory that the process has `allow`ed to the
                    // kernel. Because `Process` does not feature an API to
                    // move the "high water mark" down again, which would be
                    // called once a `ProcessBuffer` has been passed back into
                    // the kernel, a given `Process` implementation must assume
                    // that the memory described by a once-allowed
                    // `ProcessBuffer` is still in use, and thus will not
                    // permit the process to free any memory after it has
                    // been `allow`ed to the kernel once. This guarantees
                    // that the buffer is safe to convert into a slice
                    // here. For more information, refer to the
                    // comment and subsequent discussion on tock/tock#2632:
                    // https://github.com/tock/tock/pull/2632#issuecomment-869974365
                    Ok(fun(unsafe {
                        raw_processbuf_to_roprocessslice(self.ptr, self.len)
                    }))
                }),
        }
    }
}

unsafe impl WriteableProcessBuffer for ReadWriteProcessBuffer {
    fn mut_enter<F, R>(&self, fun: F) -> Result<R, process::Error>
    where
        F: FnOnce(&WriteableProcessSlice) -> R,
    {
        match self.process_id {
            None => Err(process::Error::NoSuchApp),
            Some(pid) => pid
                .kernel
                .process_map_or(Err(process::Error::NoSuchApp), pid, |_| {
                    // Safety: `kernel.process_map_or()` validates that
                    // the process still exists and its memory is still
                    // valid. In particular, `Process` tracks the "high water
                    // mark" of memory that the process has `allow`ed to the
                    // kernel. Because `Process` does not feature an API to
                    // move the "high water mark" down again, which would be
                    // called once a `ProcessBuffer` has been passed back into
                    // the kernel, a given `Process` implementation must assume
                    // that the memory described by a once-allowed
                    // `ProcessBuffer` is still in use, and thus will not
                    // permit the process to free any memory after it has
                    // been `allow`ed to the kernel once. This guarantees
                    // that the buffer is safe to convert into a slice
                    // here. For more information, refer to the
                    // comment and subsequent discussion on tock/tock#2632:
                    // https://github.com/tock/tock/pull/2632#issuecomment-869974365
                    Ok(fun(unsafe {
                        raw_processbuf_to_rwprocessslice(self.ptr, self.len)
                    }))
                }),
        }
    }
}

impl Default for ReadWriteProcessBuffer {
    fn default() -> Self {
        Self::const_default()
    }
}

/// Provides access to a [`ReadWriteProcessBuffer`] with a restricted lifetime.
/// This automatically dereferences into a ReadWriteProcessBuffer
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
    /// [`ReadWriteProcessBuffer::new_external`]. The derived lifetime can
    /// help enforce the invariant that this incoming pointer may only
    /// be access for a certain duration.
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
/// This trait can be used to gain read-write access to memory regions
/// wrapped in a ProcessBuffer type.
// We currently don't need any special functionality in the kernel for this
// type so we alias it as `ReadWriteProcessBuffer`.
pub type UserspaceReadableProcessBuffer = ReadWriteProcessBuffer;

/// Equivalent of the Rust core library's
/// [`SliceIndex`](core::slice::SliceIndex) type for process slices.
///
/// This helper trait is used to abstract over indexing operators into
/// process slices, and is used to "overload" the `.get()` methods
/// such that it can be called with multiple different indexing
/// operators.
///
/// While we can use the core library's `SliceIndex` trait, parameterized over
/// our own `ProcessSlice` types, this trait includes mandatory methods that are
/// undesirable for the process buffer infrastructure, such as unchecked or
/// mutable index operations. Furthermore, implementing it requires the
/// `slice_index_methods` nightly feature. Thus we vendor our own, small variant
/// of this trait.
pub trait ProcessSliceIndex<PB: ?Sized>: private_process_slice_index::Sealed {
    type Output: ?Sized;
    fn get(self, slice: &PB) -> Option<&Self::Output>;
    fn index(self, slice: &PB) -> &Self::Output;
}

// Analog to `private_slice_index` from
// https://github.com/rust-lang/rust/blob/a1eceec00b2684f947481696ae2322e20d59db60/library/core/src/slice/index.rs#L149
mod private_process_slice_index {
    use core::ops::{Range, RangeFrom, RangeTo};

    pub trait Sealed {}

    impl Sealed for usize {}
    impl Sealed for Range<usize> {}
    impl Sealed for RangeFrom<usize> {}
    impl Sealed for RangeTo<usize> {}
}

/// Read-only wrapper around a [`Cell`]
///
/// This type is used in providing the [`ReadableProcessSlice`]. The
/// memory over which a [`ReadableProcessSlice`] exists must never be
/// written to by the kernel. However, it may either exist in flash
/// (read-only memory) or RAM (read-writeable memory). Consequently, a
/// process may `allow` memory overlapping with a
/// [`ReadOnlyProcessBuffer`] also simultaneously through a
/// [`ReadWriteProcessBuffer`]. Hence, the kernel can have two
/// references to the same memory, where one can lead to mutation of
/// the memory contents. Therefore, the kernel must use [`Cell`]s
/// around the bytes shared with userspace, to avoid violating Rust's
/// aliasing rules.
///
/// This read-only wrapper around a [`Cell`] only exposes methods
/// which are safe to call on a process-shared read-only `allow`
/// memory.
#[repr(transparent)]
pub struct ReadableProcessByte {
    cell: Cell<u8>,
}

impl ReadableProcessByte {
    #[inline]
    pub fn get(&self) -> u8 {
        self.cell.get()
    }
}

/// Readable and accessible slice of memory of a process buffer.
///
///
/// The only way to obtain this struct is through a
/// [`ReadWriteProcessBuffer`] or [`ReadOnlyProcessBuffer`].
///
/// Slices provide a more convenient, traditional interface to process
/// memory. These slices are transient, as the underlying buffer must
/// be checked each time a slice is created. This is usually enforced
/// by the anonymous lifetime defined by the creation of the slice.
#[repr(transparent)]
pub struct ReadableProcessSlice {
    slice: [ReadableProcessByte],
}

fn cast_byte_slice_to_process_slice(byte_slice: &[ReadableProcessByte]) -> &ReadableProcessSlice {
    // As ReadableProcessSlice is a transparent wrapper around its inner type,
    // [ReadableProcessByte], we can safely transmute a reference to the inner
    // type as a reference to the outer type with the same lifetime.
    unsafe { core::mem::transmute::<&[ReadableProcessByte], &ReadableProcessSlice>(byte_slice) }
}

// Allow a u8 slice to be viewed as a ReadableProcessSlice to allow client code
// to be authored once and accept either [u8] or ReadableProcessSlice.
impl<'a> From<&'a [u8]> for &'a ReadableProcessSlice {
    fn from(val: &'a [u8]) -> Self {
        // # Safety
        //
        // The layout of a [u8] and ReadableProcessSlice are guaranteed to be
        // the same. This also extends the lifetime of the buffer, so aliasing
        // rules are thus maintained properly.
        unsafe { core::mem::transmute(val) }
    }
}

// Allow a mutable u8 slice to be viewed as a ReadableProcessSlice to allow
// client code to be authored once and accept either [u8] or
// ReadableProcessSlice.
impl<'a> From<&'a mut [u8]> for &'a ReadableProcessSlice {
    fn from(val: &'a mut [u8]) -> Self {
        // # Safety
        //
        // The layout of a [u8] and ReadableProcessSlice are guaranteed to be
        // the same. This also extends the mutable lifetime of the buffer, so
        // aliasing rules are thus maintained properly.
        unsafe { core::mem::transmute(val) }
    }
}

impl ReadableProcessSlice {
    /// Copy the contents of a [`ReadableProcessSlice`] into a mutable
    /// slice reference.
    ///
    /// The length of `self` must be the same as `dest`. Subslicing
    /// can be used to obtain a slice of matching length.
    ///
    /// # Panics
    ///
    /// This function will panic if `self.len() != dest.len()`.
    pub fn copy_to_slice(&self, dest: &mut [u8]) {
        // The panic code path was put into a cold function to not
        // bloat the call site.
        #[inline(never)]
        #[cold]
        #[track_caller]
        fn len_mismatch_fail(dst_len: usize, src_len: usize) -> ! {
            panic!(
                "source slice length ({}) does not match destination slice length ({})",
                src_len, dst_len,
            );
        }

        if self.copy_to_slice_or_err(dest).is_err() {
            len_mismatch_fail(dest.len(), self.len());
        }
    }

    /// Copy the contents of a [`ReadableProcessSlice`] into a mutable
    /// slice reference.
    ///
    /// The length of `self` must be the same as `dest`. Subslicing
    /// can be used to obtain a slice of matching length.
    pub fn copy_to_slice_or_err(&self, dest: &mut [u8]) -> Result<(), ErrorCode> {
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
            for (i, b) in self.slice.iter().enumerate() {
                dest[i] = b.get();
            }
            Ok(())
        }
    }

    /// Return the length of the slice in bytes.
    pub fn len(&self) -> usize {
        self.slice.len()
    }

    /// Return an iterator over the bytes of the slice.
    pub fn iter(&self) -> core::slice::Iter<'_, ReadableProcessByte> {
        self.slice.iter()
    }

    /// Iterate the slice in chunks.
    pub fn chunks(
        &self,
        chunk_size: usize,
    ) -> impl core::iter::Iterator<Item = &ReadableProcessSlice> {
        self.slice
            .chunks(chunk_size)
            .map(cast_byte_slice_to_process_slice)
    }

    /// Access a portion of the slice with bounds checking. If the access is not
    /// within the slice then `None` is returned.
    pub fn get<I: ProcessSliceIndex<Self>>(
        &self,
        index: I,
    ) -> Option<&<I as ProcessSliceIndex<Self>>::Output> {
        index.get(self)
    }

    /// Access a portion of the slice with bounds checking. If the access is not
    /// within the slice then `None` is returned.
    #[deprecated = "Use ReadableProcessSlice::get instead"]
    pub fn get_from(&self, range: RangeFrom<usize>) -> Option<&ReadableProcessSlice> {
        range.get(self)
    }

    /// Access a portion of the slice with bounds checking. If the access is not
    /// within the slice then `None` is returned.
    #[deprecated = "Use ReadableProcessSlice::get instead"]
    pub fn get_to(&self, range: RangeTo<usize>) -> Option<&ReadableProcessSlice> {
        range.get(self)
    }
}

impl ProcessSliceIndex<ReadableProcessSlice> for usize {
    type Output = ReadableProcessByte;

    fn get(self, slice: &ReadableProcessSlice) -> Option<&Self::Output> {
        slice.slice.get(self)
    }

    fn index(self, slice: &ReadableProcessSlice) -> &Self::Output {
        &slice.slice[self]
    }
}

impl ProcessSliceIndex<ReadableProcessSlice> for Range<usize> {
    type Output = ReadableProcessSlice;

    fn get(self, slice: &ReadableProcessSlice) -> Option<&Self::Output> {
        slice.slice.get(self).map(cast_byte_slice_to_process_slice)
    }

    fn index(self, slice: &ReadableProcessSlice) -> &Self::Output {
        cast_byte_slice_to_process_slice(&slice.slice[self])
    }
}

impl ProcessSliceIndex<ReadableProcessSlice> for RangeFrom<usize> {
    type Output = ReadableProcessSlice;

    fn get(self, slice: &ReadableProcessSlice) -> Option<&Self::Output> {
        slice.slice.get(self).map(cast_byte_slice_to_process_slice)
    }

    fn index(self, slice: &ReadableProcessSlice) -> &Self::Output {
        cast_byte_slice_to_process_slice(&slice.slice[self])
    }
}

impl ProcessSliceIndex<ReadableProcessSlice> for RangeTo<usize> {
    type Output = ReadableProcessSlice;

    fn get(self, slice: &ReadableProcessSlice) -> Option<&Self::Output> {
        slice.slice.get(self).map(cast_byte_slice_to_process_slice)
    }

    fn index(self, slice: &ReadableProcessSlice) -> &Self::Output {
        cast_byte_slice_to_process_slice(&slice.slice[self])
    }
}

impl<I: ProcessSliceIndex<Self>> Index<I> for ReadableProcessSlice {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        index.index(self)
    }
}

/// Read-writeable and accessible slice of memory of a process buffer
///
/// The only way to obtain this struct is through a
/// [`ReadWriteProcessBuffer`].
///
/// Slices provide a more convenient, traditional interface to process
/// memory. These slices are transient, as the underlying buffer must
/// be checked each time a slice is created. This is usually enforced
/// by the anonymous lifetime defined by the creation of the slice.
#[repr(transparent)]
pub struct WriteableProcessSlice {
    slice: [Cell<u8>],
}

fn cast_cell_slice_to_process_slice(cell_slice: &[Cell<u8>]) -> &WriteableProcessSlice {
    // # Safety
    //
    // As WriteableProcessSlice is a transparent wrapper around its inner type,
    // [Cell<u8>], we can safely transmute a reference to the inner type as the
    // outer type with the same lifetime.
    unsafe { core::mem::transmute(cell_slice) }
}

// Allow a mutable u8 slice to be viewed as a WritableProcessSlice to allow
// client code to be authored once and accept either [u8] or
// WriteableProcessSlice.
impl<'a> From<&'a mut [u8]> for &'a WriteableProcessSlice {
    fn from(val: &'a mut [u8]) -> Self {
        // # Safety
        //
        // The layout of a [u8] and WriteableProcessSlice are guaranteed to be
        // the same. This also extends the mutable lifetime of the buffer, so
        // aliasing rules are thus maintained properly.
        unsafe { core::mem::transmute(val) }
    }
}

impl WriteableProcessSlice {
    /// Copy the contents of a [`WriteableProcessSlice`] into a mutable
    /// slice reference.
    ///
    /// The length of `self` must be the same as `dest`. Subslicing
    /// can be used to obtain a slice of matching length.
    ///
    /// # Panics
    ///
    /// This function will panic if `self.len() != dest.len()`.
    pub fn copy_to_slice(&self, dest: &mut [u8]) {
        // The panic code path was put into a cold function to not
        // bloat the call site.
        #[inline(never)]
        #[cold]
        #[track_caller]
        fn len_mismatch_fail(dst_len: usize, src_len: usize) -> ! {
            panic!(
                "source slice length ({}) does not match destination slice length ({})",
                src_len, dst_len,
            );
        }

        if self.copy_to_slice_or_err(dest).is_err() {
            len_mismatch_fail(dest.len(), self.len());
        }
    }

    /// Copy the contents of a [`WriteableProcessSlice`] into a mutable
    /// slice reference.
    ///
    /// The length of `self` must be the same as `dest`. Subslicing
    /// can be used to obtain a slice of matching length.
    pub fn copy_to_slice_or_err(&self, dest: &mut [u8]) -> Result<(), ErrorCode> {
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
            self.slice
                .iter()
                .zip(dest.iter_mut())
                .for_each(|(src, dst)| *dst = src.get());
            Ok(())
        }
    }

    /// Copy the contents of a slice of bytes into a [`WriteableProcessSlice`].
    ///
    /// The length of `src` must be the same as `self`. Subslicing can
    /// be used to obtain a slice of matching length.
    ///
    /// # Panics
    ///
    /// This function will panic if `src.len() != self.len()`.
    pub fn copy_from_slice(&self, src: &[u8]) {
        // Method implemetation adopted from the
        // core::slice::copy_from_slice method implementation:
        // https://doc.rust-lang.org/src/core/slice/mod.rs.html#3034-3036

        // The panic code path was put into a cold function to not
        // bloat the call site.
        #[inline(never)]
        #[cold]
        #[track_caller]
        fn len_mismatch_fail(dst_len: usize, src_len: usize) -> ! {
            panic!(
                "src slice len ({}) != dest slice len ({})",
                src_len, dst_len,
            );
        }

        if self.copy_from_slice_or_err(src).is_err() {
            len_mismatch_fail(self.len(), src.len());
        }
    }

    /// Copy the contents of a slice of bytes into a [`WriteableProcessSlice`].
    ///
    /// The length of `src` must be the same as `self`. Subslicing can
    /// be used to obtain a slice of matching length.
    pub fn copy_from_slice_or_err(&self, src: &[u8]) -> Result<(), ErrorCode> {
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
                .zip(self.slice.iter())
                .for_each(|(src, dst)| dst.set(*src));
            Ok(())
        }
    }

    /// Return the length of the slice in bytes.
    pub fn len(&self) -> usize {
        self.slice.len()
    }

    /// Return an iterator over the slice.
    pub fn iter(&self) -> core::slice::Iter<'_, Cell<u8>> {
        self.slice.iter()
    }

    /// Iterate over the slice in chunks.
    pub fn chunks(
        &self,
        chunk_size: usize,
    ) -> impl core::iter::Iterator<Item = &WriteableProcessSlice> {
        self.slice
            .chunks(chunk_size)
            .map(cast_cell_slice_to_process_slice)
    }

    /// Access a portion of the slice with bounds checking. If the access is not
    /// within the slice then `None` is returned.
    pub fn get<I: ProcessSliceIndex<Self>>(
        &self,
        index: I,
    ) -> Option<&<I as ProcessSliceIndex<Self>>::Output> {
        index.get(self)
    }

    /// Access a portion of the slice with bounds checking. If the access is not
    /// within the slice then `None` is returned.
    #[deprecated = "Use WriteableProcessSlice::get instead"]
    pub fn get_from(&self, range: RangeFrom<usize>) -> Option<&WriteableProcessSlice> {
        range.get(self)
    }

    /// Access a portion of the slice with bounds checking. If the access is not
    /// within the slice then `None` is returned.
    #[deprecated = "Use WriteableProcessSlice::get instead"]
    pub fn get_to(&self, range: RangeTo<usize>) -> Option<&WriteableProcessSlice> {
        range.get(self)
    }
}

impl ProcessSliceIndex<WriteableProcessSlice> for usize {
    type Output = Cell<u8>;

    fn get(self, slice: &WriteableProcessSlice) -> Option<&Self::Output> {
        slice.slice.get(self)
    }

    fn index(self, slice: &WriteableProcessSlice) -> &Self::Output {
        &slice.slice[self]
    }
}

impl ProcessSliceIndex<WriteableProcessSlice> for Range<usize> {
    type Output = WriteableProcessSlice;

    fn get(self, slice: &WriteableProcessSlice) -> Option<&Self::Output> {
        slice.slice.get(self).map(cast_cell_slice_to_process_slice)
    }

    fn index(self, slice: &WriteableProcessSlice) -> &Self::Output {
        cast_cell_slice_to_process_slice(&slice.slice[self])
    }
}

impl ProcessSliceIndex<WriteableProcessSlice> for RangeFrom<usize> {
    type Output = WriteableProcessSlice;

    fn get(self, slice: &WriteableProcessSlice) -> Option<&Self::Output> {
        slice.slice.get(self).map(cast_cell_slice_to_process_slice)
    }

    fn index(self, slice: &WriteableProcessSlice) -> &Self::Output {
        cast_cell_slice_to_process_slice(&slice.slice[self])
    }
}

impl ProcessSliceIndex<WriteableProcessSlice> for RangeTo<usize> {
    type Output = WriteableProcessSlice;

    fn get(self, slice: &WriteableProcessSlice) -> Option<&Self::Output> {
        slice.slice.get(self).map(cast_cell_slice_to_process_slice)
    }

    fn index(self, slice: &WriteableProcessSlice) -> &Self::Output {
        cast_cell_slice_to_process_slice(&slice.slice[self])
    }
}

impl<I: ProcessSliceIndex<Self>> Index<I> for WriteableProcessSlice {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        index.index(self)
    }
}
