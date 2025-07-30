// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Objects guaranteed to be used by a single thread.

use crate::utilities::cells::OptionalCell;

/// A single-thread store that owns its contents.
///
/// This type wraps a value of type `T` by a single thread. Only that thread may
/// access the value, and that thread may have multiple _shared_ (`&`)
/// references to the value.
///
/// It is [`Sync`] and thus appropriate for static allocations of values that
/// are not themselves [`Sync`].
///
/// # Example
///
/// ```
/// use core::cell::Cell;
/// use kernel::utilities::single_thread_value::SingleThreadValue;
///
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
/// result, users must use interior mutability (e.g.
/// [`Cell`](core::cell::Cell), [`MapCell`](tock_cells::map_cell::MapCell), or
/// [`TakeCell`](tock_cells::take_cell::TakeCell)) to allow obtaining exclusive
/// mutable access.
///
/// # Safety
///
/// Creators of a [`SingleThreadValue`] must ensure that the object is **ONLY**
/// accessible from the single thread.
pub struct SingleThreadValue<T> {
    value: T,
    thread_id: OptionalCell<usize>,
    running_thread_id_fn: OptionalCell<fn() -> usize>,
}

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
    pub const unsafe fn new(value: T) -> Self {
        Self {
            value,
            thread_id: OptionalCell::empty(),
            running_thread_id_fn: OptionalCell::empty(),
        }
    }

    pub fn set_chip<C: crate::platform::chip::ChipThreadId>(&self) {
        self.running_thread_id_fn.set(C::running_thread_id);
        self.thread_id.set(C::running_thread_id());
    }

    /// Acquires a reference to value in [`SingleThreadValue`].
    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Option<&T>) -> R,
    {
        f(self
            .running_thread_id_fn
            .map_or(None, |running_thread_id_fn| {
                self.thread_id.map_or(None, |thread_id| {
                    if (running_thread_id_fn)() == thread_id {
                        Some(&self.value)
                    } else {
                        None
                    }
                })
            }))
    }

    /// Acquires a reference to value in [`SingleThreadValue`].
    pub fn with_valid<F>(&self, f: F)
    where
        F: FnOnce(&T),
    {
        self.running_thread_id_fn.map(|running_thread_id_fn| {
            self.thread_id.map(|thread_id| {
                if (running_thread_id_fn)() == thread_id {
                    f(&self.value);
                }
            });
        });
    }
}

unsafe impl<T> Sync for SingleThreadValue<T> {}
