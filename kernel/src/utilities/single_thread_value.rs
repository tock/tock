// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Objects guaranteed to be used by a single thread.

/// A single-thread store that owns its contents.
///
/// This type wraps a value of type `T` by a single thread. Only that thread may
/// access the value, and that thread may have multiple _shared_ (`&`)
/// references to the value.
///
/// It is [`Sync`] and thus appropriate for static allocations of values
/// that are not themselves [`Sync`].
///
/// # Example
///
/// ```
/// static FOO: SingleThreadValue<Cell<usize>> = unsafe { SingleThreadValue::new(Cell::new(123)) };
///
/// fn main() {
///   FOO.with(|foo| foo.set(foo.get() + 1));
/// }
/// ```
///
/// # Single-thread synchronization
///
/// It is possible for the same thread to get multiple, shared, references. As a
/// result, users must use other synchronization primitives (e.g. [`Cell`]
/// (core::cell::Cell),[`MapCell`](tock_cells::map_cell::MapCell),[`TakeCell`]
/// (tock_cells::take_cell::TakeCell) to allow obtaining exclusive mutable
/// access.
///
/// # Safety
///
/// Creators of a [`SingleThreadValue`] must ensure that the object is **ONLY**
/// accessible from the single thread.
pub struct SingleThreadValue<T>(T);

impl<T> SingleThreadValue<T> {
    /// Create a [`SingleThreadValue`].
    ///
    /// # Safety
    ///
    /// A [`SingleThreadValue`] must only be created when it can be guaranteed
    /// that only a single thread will access the value. Even on single-core
    /// systems with no threading runtime, there may be additional threads of
    /// execution, such as Interrupt Service Routines (ISRs) or signal
    /// handlers, that could race with the main thread if sharing a
    /// [`SingleThreadValue`]. It is safety-critical that such contexts do not
    /// access [`SingleThreadValue`]s.
    ///
    /// By convention, users should declare [`SingleThreadValue`] variables in
    /// scopes that are inaccessible to ISRs or signal handler, e.g. in
    /// module-private or function-local scopes.
    pub const unsafe fn new(val: T) -> Self {
        Self(val)
    }
}

impl<T> SingleThreadValue<T> {
    /// Acquires a reference to value in [`SingleThreadValue`].
    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(unsafe { &*self.0.get() })
    }
}

unsafe impl<T> Sync for SingleThreadValue<T> {}
