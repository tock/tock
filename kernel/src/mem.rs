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
//! types [`ReadWriteAppSlice`] and [`ReadOnlyAppSlice`]
//! respectively. The memory regions can be accessed through the
//! [`Read`] and [`ReadWrite`] traits, implemented on the
//! AppSlice-structs.

use crate::capabilities;
use crate::AppId;

/// Convert an AppSlice's internal representation to a Rust slice.
///
/// This function will automatically convert zero-length AppSlices
/// into valid zero-sized Rust slices regardless of the value of
/// `ptr`.
///
/// # Safety requirements
///
/// In the case of `len != 0`, the memory `[ptr; ptr + len)` must be
/// within a single process' address space, and `ptr` must be
/// nonzero. This memory region must be mapped as _readable_, and
/// optionally _writable_ and _executable_. It must be allocated
/// within a single process' address space for the entire lifetime
/// `'a`. No mutable slice pointing to an overlapping memory region
/// may exist over the entire lifetime `'a`.
unsafe fn raw_appslice_to_slice<'a>(ptr: *const u8, len: usize) -> &'a [u8] {
    use core::ptr::NonNull;
    use core::slice::from_raw_parts;

    // Rust has very strict requirements on pointer validity[1] which
    // also in part apply to accesses of length 0. We allow an
    // application to supply arbitrary pointers if the buffer length is
    // 0, but this is not allowed for Rust slices. For instance, a null
    // pointer is _never_ valid, not even for accesses of size zero.
    //
    // To get a pointer which does not point to valid (allocated) memory, but
    // is safe to construct for accesses of size zero, we must call
    // NonNull::dangling(). The resulting pointer is guaranteed to be well-aligned
    // and uphold the guarantees required for accesses of size zero.
    //
    // [1]: https://doc.rust-lang.org/core/ptr/index.html#safety
    match len {
        0 => from_raw_parts(NonNull::<u8>::dangling().as_ptr(), 0),
        _ => from_raw_parts(ptr, len),
    }
}

/// Convert an AppSlice's internal representation to a mutable Rust
/// slice.
///
/// This function will automatically convert zero-length appslices
/// into valid zero-sized Rust slices regardless of the value of
/// `ptr`.
///
/// # Safety requirements
///
/// In the case of `len != 0`, the memory `[ptr; ptr + len)` must be
/// within a single process' address space, and `ptr` must be
/// nonzero. This memory region must be mapped as _readable_ and
/// _writable_, and optionally _executable_. It must be allocated
/// within a single process' address space for the entire lifetime
/// `'a`. No other mutable or immutable slice pointing to an
/// overlapping memory region may exist over the entire lifetime `'a`.
unsafe fn raw_appslice_to_slice_mut<'a>(ptr: *mut u8, len: usize) -> &'a mut [u8] {
    use core::ptr::NonNull;
    use core::slice::from_raw_parts_mut;

    // See documentation on [`raw_appslice_to_slice`] for Rust slice &
    // pointer validity requirements
    match len {
        0 => from_raw_parts_mut(NonNull::<u8>::dangling().as_ptr(), 0),
        _ => from_raw_parts_mut(ptr, len),
    }
}

/// A readable region of userspace memory.
///
/// This trait can be used to gain read-only access to memory regions
/// wrapped in an AppSlice type.
pub trait Read {
    /// Length of the memory region.
    ///
    /// If the process is no longer alive and the memory has been
    /// reclaimed, this method must return 0.
    ///
    /// # Default AppSlice
    ///
    /// A default instance of an AppSlice must return 0.
    fn len(&self) -> usize;

    /// Pointer to the userspace memory region.
    ///
    /// If the length of the initially shared memory region
    /// (irrespective of the return value of [`len`](Read::len)) is 0,
    /// this function must return a pointer to address `0x0`. This is
    /// because processes allow buffers with length 0 to reclaim
    /// shared memory with the kernel and are allowed to specify _any_
    /// address, even if it is not contained within their address
    /// space. These _dummy addresses_ should not be leaked to outside
    /// code.
    ///
    /// # Default AppSlice
    ///
    /// A default instance of an AppSlice must return a pointer to
    /// address `0x0`.
    fn ptr(&self) -> *const u8;

    /// Applies a function to the (read only) slice reference pointed
    /// to by the AppSlice.
    ///
    /// If the process is no longer alive and the memory has been
    /// reclaimed, this method must return the default value.
    ///
    /// # Default AppSlice
    ///
    /// A default instance of an AppSlice must return the passed
    /// default value without executing the closure.
    fn map_or<F, R>(&self, default: R, fun: F) -> R
    where
        F: FnOnce(&[u8]) -> R;
}

/// A readable and writable region of userspace memory.
///
/// This trait can be used to gain read-write access to memory regions
/// wrapped in an AppSlice type.
///
/// This is a supertrait of [`Read`], which features further methods.
pub trait ReadWrite: Read {
    /// Mutable pointer to the userspace memory region
    ///
    /// If the length of the initially shared memory region
    /// (irrespective of the return value of [`len`](Read::len)) is 0,
    /// this function must return a pointer to address `0x0`. This is
    /// because processes allow buffers with length 0 to reclaim
    /// shared memory with the kernel and are allowed to specify _any_
    /// address, even if it is not contained within their address
    /// space. These _dummy addresses_ should not be leaked to outside
    /// code.
    fn mut_ptr(&self) -> *mut u8;

    /// Applies a function to the mutable slice reference pointed to
    /// by the AppSlice.
    ///
    /// If the process is no longer alive and the memory has been
    /// reclaimed, this method must return the default value.
    ///
    /// # Default AppSlice
    ///
    /// A default instance of an AppSlice must return the passed
    /// default value without executing the closure.
    fn mut_map_or<F, R>(&self, default: R, fun: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R;
}

/// Read-writable memory region of a process, shared with the kernel
pub struct ReadWriteAppSlice {
    ptr: *mut u8,
    len: usize,
    process_id: Option<AppId>,
}

impl ReadWriteAppSlice {
    pub(crate) unsafe fn new(ptr: *mut u8, len: usize, process_id: AppId) -> Self {
        ReadWriteAppSlice {
            ptr,
            len,
            process_id: Some(process_id),
        }
    }

    pub unsafe fn new_external(
        ptr: *mut u8,
        len: usize,
        process_id: AppId,
        _cap: &dyn capabilities::ExternalProcessCapability,
    ) -> Self {
        Self::new(ptr, len, process_id)
    }

    pub(crate) fn consume(self) -> (*mut u8, usize) {
        (self.ptr, self.len)
    }

    /// This is a `const` version of `Default::default` with the same semantics.
    ///
    /// Having a const initializer allows initializing a fixed-size array with default values
    /// without the struct being marked `Copy` as such:
    ///
    /// ```
    /// use kernel::ReadWriteAppSlice;
    /// const DEFAULT_RWAPPSLICE_VAL: ReadWriteAppSlice = ReadWriteAppSlice::const_default();
    /// let my_array = [DEFAULT_RWAPPSLICE_VAL; 12];
    /// ```
    pub const fn const_default() -> Self {
        Self {
            ptr: 0x0 as *mut u8,
            len: 0,
            process_id: None,
        }
    }
}

impl Default for ReadWriteAppSlice {
    fn default() -> Self {
        Self::const_default()
    }
}

impl ReadWrite for ReadWriteAppSlice {
    fn mut_ptr(&self) -> *mut u8 {
        if self.len == 0 {
            0x0 as *mut u8
        } else {
            self.ptr
        }
    }

    fn mut_map_or<F, R>(&self, default: R, fun: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        match self.process_id {
            None => default,
            Some(pid) => pid.kernel.process_map_or(default, pid, |_| {
                // Safety: `kernel.process_map_or()` validates that the process still exists
                // and its memory is still valid. `Process` tracks the "high water mark" of
                // memory that the process has `allow`ed to the kernel, and will not permit
                // the process to free any memory after it has been `allow`ed. This guarantees
                // that the buffer is safe to convert into a slice here.
                fun(unsafe { raw_appslice_to_slice_mut(self.ptr, self.len) })
            }),
        }
    }
}

impl Read for ReadWriteAppSlice {
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

    fn map_or<F, R>(&self, default: R, fun: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        match self.process_id {
            None => default,
            Some(pid) => pid.kernel.process_map_or(default, pid, |_| {
                // Safety: `kernel.process_map_or()` validates that the process still exists
                // and its memory is still valid. `Process` tracks the "high water mark" of
                // memory that the process has `allow`ed to the kernel, and will not permit
                // the process to free any memory after it has been `allow`ed. This guarantees
                // that the buffer is safe to convert into a slice here.
                fun(unsafe { raw_appslice_to_slice(self.ptr, self.len) })
            }),
        }
    }
}

/// Read-only memory region of a process, shared with the kernel
pub struct ReadOnlyAppSlice {
    ptr: *const u8,
    len: usize,
    process_id: Option<AppId>,
}

impl ReadOnlyAppSlice {
    pub(crate) unsafe fn new(ptr: *const u8, len: usize, process_id: AppId) -> Self {
        ReadOnlyAppSlice {
            ptr,
            len,
            process_id: Some(process_id),
        }
    }

    pub unsafe fn new_external(
        ptr: *const u8,
        len: usize,
        process_id: AppId,
        _cap: &dyn capabilities::ExternalProcessCapability,
    ) -> Self {
        Self::new(ptr, len, process_id)
    }

    pub(crate) fn consume(self) -> (*const u8, usize) {
        (self.ptr, self.len)
    }
}

impl Default for ReadOnlyAppSlice {
    fn default() -> Self {
        ReadOnlyAppSlice {
            ptr: 0x0 as *mut u8,
            len: 0,
            process_id: None,
        }
    }
}

impl Read for ReadOnlyAppSlice {
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

    fn map_or<F, R>(&self, default: R, fun: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        match self.process_id {
            None => default,
            Some(pid) => pid.kernel.process_map_or(default, pid, |_| {
                // Safety: `kernel.process_map_or()` validates that the process still exists
                // and its memory is still valid. `Process` tracks the "high water mark" of
                // memory that the process has `allow`ed to the kernel, and will not permit
                // the process to free any memory after it has been `allow`ed. This guarantees
                // that the buffer is safe to convert into a slice here.
                fun(unsafe { raw_appslice_to_slice(self.ptr, self.len) })
            }),
        }
    }
}
