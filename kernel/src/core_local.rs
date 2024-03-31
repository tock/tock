// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Core local storage

use core::cell::UnsafeCell;

/// A core-local store that owns its contents.
///
/// This type wraps a core-specific value of type `T`. It precludes
/// access to the same value from different cores/threads, but allows
/// multiple _shared_ (`&`) references from the same core/thread.
///
/// It is [`Sync`] and thus appropriate for static allocations of values
/// that are not themselves [`Sync`].
///
/// # Example
///
/// ```
/// static FOO: CoreLocal<Cell<usize>> = unsafe { CoreLocal::new_single_core(Cell::new(123)) };
///
/// fn main() {
///   FOO.with(|foo| foo.set(foo.get() + 1));
/// }
/// ```
///
/// # Single-thread synchronization
///
/// Though there is no potential for races across cores, it is
/// possible for code on the same core to get multiple, shared,
/// references. As a result, users must use other synchronization
/// primitives (e.g. [`Cell`](core::cell::Cell),
/// [`MapCell`](tock_cells::map_cell::MapCell),
/// [`TakeCell`](tock_cells::take_cell::TakeCell) to allow obtaining
/// exclusive mutable access.
///
/// # Safety
///
/// Creators of a [`CoreLocal`] must ensure that they are only
/// accessible from contexts that meet the requirements of the
/// [`CoreLocal`] implementation used. See constructors (`new_*`
/// functions) for details on safety requirements.
pub struct CoreLocal<T>(UnsafeCell<T>);

impl<T> CoreLocal<T> {
    /// Create a [`CoreLocal`] on a single-core system.
    ///
    /// # Safety
    ///
    /// A [`CoreLocal`] must only be created with this constructor on
    /// systems where a single-core has exclusive access to the
    /// declared value and without preemtive threads.
    ///
    /// Even on single-core systems with no thread runtime, there may
    /// be additional threads of execution, such as Interrupt Service
    /// Routines (ISRs) or signal handlers, that could race with the
    /// main thread if sharing a [`CoreLocal`]. It is safety-critical
    /// that such contexts do not access [`CoreLocal`]s.
    ///
    /// By convetion, users should declare [`CoreLocal`] variables in
    /// scopes that are inaccessible to ISRs or signal handler,
    /// e.g. in module-private or function-local scopes.
    pub const unsafe fn new_single_core(val: T) -> Self {
        CoreLocal(UnsafeCell::new(val))
    }
}

impl<T> CoreLocal<T> {
    /// Aquires a reference to value in [`CoreLocal`].
    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(unsafe { &*self.0.get() })
    }
}

unsafe impl<T> Sync for CoreLocal<T> {}
