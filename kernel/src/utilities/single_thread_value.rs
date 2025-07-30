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
///   // Provide the SingleThreadValue with a method to inspect the current thread.
///   FOO.set_chip();
///
///   // Attempt to access the value. Passes an Option to the closure.
///   FOO.with(|foo_option| foo_option.map(|foo| foo.set(foo.get() + 1)));
///
///   // Attempt to access the value. Closure is only invoked if permitted.
///   FOO.with_valid(|foo| foo.set(foo.get() + 2));
/// }
/// ```
///
/// After creating the [`SingleThreadValue`] and before trying to access the
/// wrapped value, the [`SingleThreadValue`] must have its [`set_chip()`]
/// method called. Failing to set the mechanism which can identify threads
/// will prevent any access to the wrapped value.
///
/// # Single-thread Synchronization
///
/// It is possible for the same thread to get multiple, shared, references. As a
/// result, users must use interior mutability (e.g.
/// [`Cell`](core::cell::Cell), [`MapCell`](tock_cells::map_cell::MapCell), or
/// [`TakeCell`](tock_cells::take_cell::TakeCell)) to allow obtaining exclusive
/// mutable access.
///
/// # Guaranteeing Single-Thread Access
///
/// [`SingleThreadValue`] is safe because it guarantees that the value is only
/// ever accessed from the same thread that created it. To do this,
/// [`SingleThreadValue`] inspects the currently running thread on every access
/// to the wrapped value. If the active thread is different than the original
/// thread then the caller will not be able to access the value.
///
/// This requires that the system provides a correct implementation of
/// [`ChipThreadId`] to identify the currently executing thread. Internally,
/// [`SingleThreadValue`] uses the[`ChipThreadId`] trait to identify the
/// current thread and compares it against the original thread.
//
// # Implementation Trade-Offs
//
// Correctly implementing a type which can be safely shared within a single
// thread requires balancing the following trade-offs:
//
// 1. Automated enforcement (either by the compiler or at runtime) vs.
//    programmer-checked correctness.
// 2. Runtime overhead vs. compile-time checks.
// 3. Simplicity for users vs. easy-to-make mistakes.
//
// Ideally, the guarantees this type provides would be verified at compile time
// and be simple for users to use. However, as of July 2025, we don't know how
// to realistically meet those goals.
//
// This approach first chooses to have automated enforcement rather than have
// correctness (and avoiding unsoundness) based only on carefully written code
// and scrutiny during code reviews. This is generally consistent with the Tock
// ethos with using Rust and APIs to enforce correctness, even if the check
// occurs at runtime.
//
// To implement the automated check, this type uses runtime checks. We don't
// know a realistic way to move those checks to compile time.
//
// One downside to these decisions is the need to pass in the thread identifier
// after the type is created. While this is conceptually similar to providing a
// callback client after an object is constructed, which is widely used in
// Tock, forgetting this operation will lead to the runtime check always
// failing and will break the desired functionality of using the wrapped value.
// However, passing in the type later is needed because the thread identifier
// is likely not available where users want to construct this type.
//
// Ultimately, these trade-offs, runtime checks and an interface where missing a
// call leads to failures, were deemed acceptable and worth the upside
// (guaranteed soundness). In part, this is because this type should be used
// and accessed sparingly within Tock. The complexity of creating this type
// stems from its general risky-ness: enabling sharing static global variables.
// While necessary for certain use cases within Tock, this should not be used
// generally.
pub struct SingleThreadValue<T> {
    value: T,
    thread_id: OptionalCell<usize>,
    running_thread_id_fn: OptionalCell<fn() -> usize>,
}

impl<T> SingleThreadValue<T> {
    /// Create a [`SingleThreadValue`].
    ///
    /// Note, the value will not be accessible immediately after `new()` runs.
    /// To provide the single-thread guaranteed, [`SingleThreadValue`] needs a
    /// reference to a [`ChipThreadId`] implementation, provided by
    /// [`set_chip()`]. Since in many cases a suitable implementation of
    /// [`ChipThreadId`] is not available when [`SingleThreadValue::new()`] is
    /// called, the [`ChipThreadId`] implementation is provided later.
    pub const fn new(value: T) -> Self {
        Self {
            value,
            thread_id: OptionalCell::empty(),
            running_thread_id_fn: OptionalCell::empty(),
        }
    }

    /// Assign the [`ChipThreadId`] implementation.
    ///
    /// This stores the method that can identify the currently executing thread.
    /// This method is used to determine if an attempted access is permitted or
    /// not.
    pub fn set_chip<C: crate::platform::chip::ChipThreadId>(&self) {
        if self.thread_id.is_none() {
            self.running_thread_id_fn.set(C::running_thread_id);
            self.thread_id.set(C::running_thread_id());
        }
    }

    /// Attempt to acquire a reference to the wrapped value.
    ///
    /// The provided function `f` is passed an optional reference to the wrapped
    /// value. If the access is permitted, meaning the currently executing
    /// thread is the thread that owns the [`SingleThreadValue`], then the
    /// argument will be `Some(&T)`. If the access is not permitted the
    /// argument will be `None`.
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

    /// Acquire a reference to the wrapped value only if permitted.
    ///
    /// The provided function `f` is only executed and given a reference to the
    /// wrapped value if the access is permitted. Otherwise, `f` is not
    /// executed.
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

/// Mark that [`SingleThreadValue`] is [`Sync`] to enable multiple accesses.
///
/// # Safety
///
/// This is safe because [`SingleThreadValue`] enforces that the shared value
/// is only ever accessed from a single thread.
unsafe impl<T> Sync for SingleThreadValue<T> {}
