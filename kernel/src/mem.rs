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

use core::slice;

use crate::capabilities;
use crate::AppId;

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
    /// reclaimed, this method must return 0.
    ///
    /// # Default AppSlice
    ///
    /// A default instance of an AppSlice must return the passed
    /// default value without executing the closure.
    fn map_or<F, R>(&self, default: R, fun: F) -> R
    where
        R: Copy,
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
    /// reclaimed, this method must return 0.
    ///
    /// # Default AppSlice
    ///
    /// A default instance of an AppSlice must return the passed
    /// default value without executing the closure.
    fn mut_map_or<F, R>(&self, default: R, fun: F) -> R
    where
        R: Copy,
        F: FnOnce(&mut [u8]) -> R;
}

/// Read-writable memory region of a process, shared with the kernel
pub struct ReadWriteAppSlice {
    ptr: *mut u8,
    len: usize,

    // TODO: For improved efficiency this should use a dummy instance
    // of an AppId when constructed with Default::default(), as the
    // Option will allocate another full word.
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
}

impl Default for ReadWriteAppSlice {
    fn default() -> Self {
        ReadWriteAppSlice {
            ptr: 0x0 as *mut u8,
            len: 0,
            process_id: None,
        }
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
        R: Copy,
        F: FnOnce(&mut [u8]) -> R,
    {
        self.process_id.map_or(default, |pid| {
            pid.kernel.process_map_or(default, pid, |_| {
                let slice = unsafe { slice::from_raw_parts_mut(self.ptr, self.len) };
                fun(slice)
            })
        })
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
        R: Copy,
        F: FnOnce(&[u8]) -> R,
    {
        self.process_id.map_or(default, |pid| {
            pid.kernel.process_map_or(default, pid, |_| {
                let slice = unsafe { slice::from_raw_parts(self.ptr, self.len) };
                fun(slice)
            })
        })
    }
}

/// Read-only memory region of a process, shared with the kernel
pub struct ReadOnlyAppSlice {
    ptr: *const u8,
    len: usize,

    // TODO: For improved efficiency this should use a dummy instance
    // of an AppId when constructed with Default::default(), as the
    // Option will allocate another full word.
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
        R: Copy,
        F: FnOnce(&[u8]) -> R,
    {
        self.process_id.map_or(default, |pid| {
            pid.kernel.process_map_or(default, pid, |_| {
                let slice = unsafe { slice::from_raw_parts(self.ptr, self.len) };
                fun(slice)
            })
        })
    }
}

// ---------- TOCK 1.x LEGACY INTERFACES ----------
// TODO: Remove prior to releasing Tock 2.0

pub(crate) mod legacy {
    use core::marker::PhantomData;
    use core::ptr::NonNull;
    use core::slice;

    use crate::capabilities;
    use crate::AppId;

    /// Type for specifying an AppSlice is shared with the kernel.
    #[derive(Debug)]
    pub struct SharedReadWrite;

    /// Base type for an AppSlice that holds the raw pointer to the memory region
    /// the app shared with the kernel.
    pub(crate) struct AppPtr<L, T> {
        ptr: NonNull<T>,
        process: AppId,
        _phantom: PhantomData<L>,
    }

    impl<L, T> AppPtr<L, T> {
        /// Safety: Trusts that `ptr` points to userspace memory in `appid`
        unsafe fn new(ptr: NonNull<T>, appid: AppId) -> AppPtr<L, T> {
            AppPtr {
                ptr: ptr,
                process: appid,
                _phantom: PhantomData,
            }
        }
    }

    // TODO: Remove prior to releasing Tock 2.0
    /// Tock 1.x legacy AppSlice type
    ///
    /// Buffer of memory shared from an app to the kernel.
    ///
    /// This is the type created after an app calls the `allow` syscall.
    pub struct AppSlice<L, T> {
        ptr: AppPtr<L, T>,
        len: usize,
    }

    impl<L, T> AppSlice<L, T> {
        /// Safety: Trusts that `ptr` + `len` is a buffer in the memory region owned
        /// by `appid` and that no other references to that memory range exist.
        pub(crate) unsafe fn new(ptr: NonNull<T>, len: usize, appid: AppId) -> AppSlice<L, T> {
            AppSlice {
                ptr: AppPtr::new(ptr, appid),
                len: len,
            }
        }

        /// Safety: Trusts that `ptr` + `len` is a buffer in the memory region owned
        /// by `appid` and that no other references to that memory range exist.
        ///
        /// This constructor is public but protected with a capability to enable
        /// external implementations of `ProcessType` to create `AppSlice`s.
        pub unsafe fn new_external(
            ptr: NonNull<T>,
            len: usize,
            appid: AppId,
            _capability: &dyn capabilities::ExternalProcessCapability,
        ) -> AppSlice<L, T> {
            AppSlice {
                ptr: AppPtr::new(ptr, appid),
                len: len,
            }
        }

        /// Number of bytes in the `AppSlice`.
        ///
        /// If the app died, has restarted, or its AppId identifier
        /// changed for any other reason, return an accessible length of
        /// zero, consistent with the [`AsRef`](struct.AppSlice.html#impl-AsRef<[T]>)
        /// and [`AsMut`](struct.AppSlice.html#impl-AsMut<[T]>) implementations.
        pub fn len(&self) -> usize {
            self.ptr
                .process
                .kernel
                .process_map_or(0, self.ptr.process, |_| self.len)
        }

        /// Get the raw pointer to the buffer. This will be a pointer inside of the
        /// app's memory region.
        pub fn ptr(&self) -> *const T {
            self.ptr.ptr.as_ptr()
        }

        /// Provide access to one app's AppSlice to another app. This is used for
        /// IPC.
        pub(crate) unsafe fn expose_to(&self, appid: AppId) -> bool {
            if appid != self.ptr.process {
                self.ptr
                    .process
                    .kernel
                    .process_map_or(false, appid, |process| {
                        process
                            .add_mpu_region(self.ptr() as *const u8, self.len(), self.len())
                            .is_some()
                    })
            } else {
                false
            }
        }

        /// Returns an iterator over the slice
        ///
        /// See
        /// [`std::slice::iter()`](https://doc.rust-lang.org/std/primitive.slice.html#method.iter).
        ///
        /// Internally this uses
        /// [`AsRef`](struct.AppSlice.html#impl-AsRef<[T]>), hence when
        /// the app dies, restarts or the
        /// [`AppId`](crate::callback::AppId) changes for any other
        /// reason, the iterator will be of zero length.
        pub fn iter(&self) -> slice::Iter<T> {
            self.as_ref().iter()
        }

        /// Returns an iterator that allows modifying each value
        ///
        /// See
        /// [`std::slice::iter_mut()`](https://doc.rust-lang.org/std/primitive.slice.html#method.iter_mut).
        ///
        /// Internally this uses
        /// [`AsMut`](struct.AppSlice.html#impl-AsMut<[T]>), hence when
        /// the app dies, restarts or the
        /// [`AppId`](crate::callback::AppId) changes for any other
        /// reason, the iterator will be of zero length.
        pub fn iter_mut(&mut self) -> slice::IterMut<T> {
            self.as_mut().iter_mut()
        }

        /// Iterate over `chunk_size` elements at a time, starting at the
        /// beginning of the AppSlice.
        ///
        /// See
        /// [`std::slice::chunks()`](https://doc.rust-lang.org/std/primitive.slice.html#method.chunks).
        ///
        /// Internally this uses
        /// [`AsRef`](struct.AppSlice.html#impl-AsRef<[T]>), hence when
        /// the app dies, restarts or the
        /// [`AppId`](crate::callback::AppId) changes for any other
        /// reason, a [`Chunks`](core::slice::Chunks) iterator of zero length will
        /// be returned.
        pub fn chunks(&self, size: usize) -> slice::Chunks<T> {
            self.as_ref().chunks(size)
        }

        /// Mutably iterate over `chunk_size` elements at a time, starting at the
        /// beginning of the AppSlice.
        ///
        /// See
        /// [`std::slice::chunks_mut()`](https://doc.rust-lang.org/std/primitive.slice.html#method.chunks_mut).
        ///
        /// Internally this uses
        /// [`AsMut`](struct.AppSlice.html#impl-AsMut<[T]>), hence when
        /// the app dies, restarts or the
        /// [`AppId`](crate::callback::AppId) changes for any other
        /// reason, a [`ChunksMut`](core::slice::ChunksMut) iterator of zero length will
        /// be returned.
        ///
        /// # Panics
        ///
        /// Panics if `chunk_size` is 0.
        pub fn chunks_mut(&mut self, size: usize) -> slice::ChunksMut<T> {
            self.as_mut().chunks_mut(size)
        }
    }

    impl<L, T> AsRef<[T]> for AppSlice<L, T> {
        /// Get a slice reference over the userspace buffer
        ///
        /// This first checks whether the app died, restarted, or its
        /// AppId identifier changed for any other reason. In this case, a
        /// slice of length zero is returned.
        fn as_ref(&self) -> &[T] {
            self.ptr
                .process
                .kernel
                .process_map_or(&[], self.ptr.process, |_| unsafe {
                    slice::from_raw_parts(self.ptr.ptr.as_ref(), self.len)
                })
        }
    }

    impl<L, T> AsMut<[T]> for AppSlice<L, T> {
        /// Get a mutable slice reference over the userspace buffer
        ///
        /// This first checks whether the app died, restarted, or its
        /// AppId identifier changed for any other reason. In this case, a
        /// slice of length zero is returned.
        fn as_mut(&mut self) -> &mut [T] {
            self.ptr
                .process
                .kernel
                .process_map_or(&mut [], self.ptr.process, |_| unsafe {
                    slice::from_raw_parts_mut(self.ptr.ptr.as_mut(), self.len)
                })
        }
    }
}
