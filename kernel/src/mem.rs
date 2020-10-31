//! Data structure for passing application memory to the kernel.

use core::marker::PhantomData;
use core::ptr::NonNull;
use core::slice;

use crate::callback::ProcessId;
use crate::capabilities;

/// Type for specifying an AppSlice is hidden from the kernel.
#[derive(Debug)]
pub struct Private;

/// Type for specifying an AppSlice is shared with the kernel.
#[derive(Debug)]
pub struct Shared;

/// Base type for an AppSlice that holds the raw pointer to the memory region
/// the app shared with the kernel.
pub(crate) struct AppPtr<L, T> {
    ptr: NonNull<T>,
    process: ProcessId,
    _phantom: PhantomData<L>,
}

impl<L, T> AppPtr<L, T> {
    /// Safety: Trusts that `ptr` points to userspace memory in `appid`
    unsafe fn new(ptr: NonNull<T>, appid: ProcessId) -> AppPtr<L, T> {
        AppPtr {
            ptr: ptr,
            process: appid,
            _phantom: PhantomData,
        }
    }
}

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
    pub(crate) unsafe fn new(ptr: NonNull<T>, len: usize, appid: ProcessId) -> AppSlice<L, T> {
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
        appid: ProcessId,
        _capability: &dyn capabilities::ExternalProcessCapability,
    ) -> AppSlice<L, T> {
        AppSlice {
            ptr: AppPtr::new(ptr, appid),
            len: len,
        }
    }

    /// Number of bytes in the `AppSlice`.
    ///
    /// If the app died, has restarted, or its ProcessId identifier
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
    pub(crate) unsafe fn expose_to(&self, appid: ProcessId) -> bool {
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
    /// [`ProcessId`](crate::callback::ProcessId) changes for any other
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
    /// [`ProcessId`](crate::callback::ProcessId) changes for any other
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
    /// [`ProcessId`](crate::callback::ProcessId) changes for any other
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
    /// [`ProcessId`](crate::callback::ProcessId) changes for any other
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
    /// ProcessId identifier changed for any other reason. In this case, a
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
    /// ProcessId identifier changed for any other reason. In this case, a
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
