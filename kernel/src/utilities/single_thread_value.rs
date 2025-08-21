// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! A container for objects accessible to a single thread.

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;

use crate::platform::chip::ThreadIdProvider;

/// Stages of binding a [`SingleThreadValue`] to a given thread.
///
/// The [`SingleThreadValue`] starts out not being bound to, and thus not being
/// usable by any thread. Only after its construction will it be bound to a
/// particular thread, using either the [`SingleThreadValue::bind_to_thread`],
/// or the [`SingleThreadValue::bind_to_thread_unsafe`] methods.
///
/// For other these and other methods to know whether a [`SingleThreadValue`]
/// has been bound to a thread already, and thus whether its `thread_id_and_fn`
/// field is initialized and holds a stable, it contains an `AtomicUsize` type
/// to indicate its current "binding stage". Over its lifetime, this stage value
/// is strictly increasing, transitioning through its variants as per their
/// documentation.
#[repr(usize)]
enum BoundToThreadStage {
    /// This state means that the [`SingleThreadValue`] has not been bound to a
    /// particular thread yet, and its `thread_id_and_fn` field is not
    /// initialized.
    ///
    /// For the `bind_to_thread` and `bind_to_thread_unsafe` methods, this value
    /// further means that the value can be bound to the currently running
    /// thread.
    Unbound = 0,

    /// To bind a [`SingleThreadValue`] to the currently running thread, the
    /// `bind_to_thread` function atomically transitions it from the `Unbound`
    /// stage to the `Binding` stage, through a compare-exchange operation.
    ///
    /// When this stage is active, the `thread_id_and_fn` are not initialized,
    /// similar to `Unbound`. However, it guards other, concurrent calls to
    /// `bind_to_thread` from attempting to bind the value to another thread
    /// concurrently.
    ///
    /// Targets without support for compare-exchange atomic operations on
    /// `usize` types can utilize `bind_to_thread_unsafe` instead: this method
    /// requires callers to guarantee that there are no concurrent calls to
    /// either `bind_to_thread` OR `bind_to_thread_unsafe`. Thus, it can skip
    /// this intermediate state: it can directly transition from `Unbound` into
    /// `Bound`.
    #[allow(dead_code)]
    Binding = 1,

    /// This value indicates that the [`SingleThreadValue`] is bound to a
    /// particular thread. The `thread_id_and_fn` value is initialized and
    /// sealed: it must not be modified any longer beyond this point, and no
    /// mutable references may exist to it. The `Bound` state cannot be
    /// transitioned out of.
    Bound = 2,
}

/// A container for objects accessible to a single thread.
///
/// This type wraps a value of type `T: ?Sync`, accessible to only a single
/// thread. Only that thread can obtain references to the contained value, and
/// that thread may obtain multiple _shared_ (`&`) references to the value
/// concurrently.
///
/// This container is [`Sync`], regardless of whether `T` is `Sync`. This is
/// similar to the standard library's `LocalKey`, and thus appropriate for
/// static allocations of values that are not themselves [`Sync`]. However,
/// unlike `LocalKey`, it only holds a value for a single thread, determined at
/// runtime based on the first call to [`SingleThreadValue::bind_to_thread`].
///
/// # Example
///
/// ```
/// use core::cell::Cell;
/// use kernel::utilities::single_thread_value::SingleThreadValue;
///
/// // Binding to a thread requires a "ThreadIdProvider", used to query the
/// // thread ID of the currently running thread at runtime:
/// enum DummyThreadIdProvider {}
/// unsafe impl kernel::platform::chip::ThreadIdProvider for DummyThreadIdProvider {
///     fn running_thread_id() -> usize {
///         // Return the current thread id. We return a constant for
///         // demonstration purposes, but doing so in practice is unsound:
///         42
///     }
/// }
///
/// static FOO: SingleThreadValue<Cell<usize>> = SingleThreadValue::new(Cell::new(123));
///
/// fn main() {
///     // Bind the value contained in the `SingleThreadValue` to the currently
///     // running thread:
///     FOO.bind_to_thread::<DummyThreadIdProvider>();
///
///     // Attempt to access the value. Returns `Some(&T)` if running from the
///     // thread that the `SingleThreadValue` is bound to:
///     let foo_ref = FOO.get().unwrap();
///     foo_ref.set(foo_ref.get() + 1);
/// }
/// ```
///
/// After creating the [`SingleThreadValue`] and before trying to access the
/// wrapped value, the [`SingleThreadValue`] must have its
/// [`bind_to_thread`](SingleThreadValue::bind_to_thread) method called. Failing
/// to bind it to a thread will prevent any access to the wrapped value.
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
/// ever accessed from a single thread. To do this, [`SingleThreadValue`]
/// inspects the currently running thread on every access to the wrapped
/// value. If the active thread is different than the original thread then the
/// caller will not be able to access the value.
///
/// This requires that the system provides a correct implementation of
/// [`ThreadIdProvider`] to identify the currently executing thread. Internally,
/// [`SingleThreadValue`] uses the [`ThreadIdProvider::running_thread_id`]
/// function to identify the current thread and compares it against the thread
/// ID that the contained value is bound to.
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
// One downside to these decisions is the need to bind the value to a thread
// _after_ the type is created. While this is conceptually similar to providing
// a callback client after an object is constructed, which is widely used in
// Tock, forgetting this operation will lead to the runtime check always failing
// and will break the desired functionality of using the wrapped value.
// However, passing in the type later is needed because the thread identifier is
// likely not available where users want to construct this type.
//
// Ultimately, these trade-offs, runtime checks and an interface where missing a
// call leads to failures, were deemed acceptable and worth the upside
// (guaranteed soundness). In part, this is because this type should be used and
// accessed sparingly within Tock. The complexity of creating this type stems
// from its general risky-ness: enabling sharing static global variables. While
// necessary for certain use cases within Tock, this should not be used lightly.
pub struct SingleThreadValue<T> {
    /// The contained value, made accessible to a single thread only.
    value: T,

    /// Shared atomic state to indicate whether this type is already bound to a
    /// particular thread, or in the process of being bound to a particular
    /// thread. Assumes values of [`BoundToThreadStage`]. Consider that type's
    /// documentation for how `bound_to_thread` is used.
    bound_to_thread: AtomicUsize,

    /// Context used to determine which thread ID this type is bound to, and how
    /// to determine the currently running thread ID.
    ///
    /// This value must only be used in accordance with the rules around
    /// [`BoundToThreadStage`]. Consider that type's documentation for more
    /// information. Whether this value is initialized, and when it is safe to
    /// read and write depend on the value of `bound_to_thread`.
    thread_id_and_fn: UnsafeCell<MaybeUninit<(fn() -> usize, usize)>>,
}

/// Mark that [`SingleThreadValue`] is [`Sync`] to enable multiple accesses.
///
/// # Safety
///
/// This is safe because [`SingleThreadValue`] enforces that the shared value
/// is only ever accessed from a single thread.
unsafe impl<T> Sync for SingleThreadValue<T> {}

impl<T> SingleThreadValue<T> {
    /// Create a [`SingleThreadValue`].
    ///
    /// Note, the value will not be accessible immediately after `new()` runs.
    /// It must first be bound to a particular thread, using the
    /// [`bind_to_thread`](SingleThreadValue::bind_to_thread) or
    /// [`bind_to_thread_unsafe`](SingleThreadValue::bind_to_thread_unsafe) methods.
    pub const fn new(value: T) -> Self {
        Self {
            value,
            bound_to_thread: AtomicUsize::new(BoundToThreadStage::Unbound as usize),
            thread_id_and_fn: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Bind this [`SingleThreadValue`] to the currently running thread.
    ///
    /// If this [`SingleThreadValue`] is not already bound to a thread, or if it
    /// is not currently in the process of binding to a thread, then this binds
    /// the [`SingleThreadValue`] to the current thread.
    ///
    /// It further records the [`ThreadIdProvider::running_thread_id`] function
    /// reference, and uses this function to determine the currently running
    /// thread for any future queries.
    ///
    /// Returns `true` if this invocation successfully bound the value to the
    /// current thread. Otherwise, if the value was already bound to this same
    /// or another thread, or is concurrently being bound to a thread, it
    /// returns `false`.
    ///
    /// This method requires the target to support atomic operations
    /// (namely, `compare_exchange`) on `usize`-sized values, and thus
    /// relies on the `cfg(target_has_atomic = "ptr")` conditional.
    #[cfg(target_has_atomic = "ptr")]
    pub fn bind_to_thread<P: ThreadIdProvider>(&self) -> bool {
        // For the check whether we're already bound to a thread, `Relaxed`
        // ordering is fine: we don't actually care about the value in
        // `thread_id_and_fn`, and don't need previous writes to it to be
        // visible to this thread.
        //
        // Perform a compare-exchange on the `bound_to_thread` value. If this
        // operation is successful, the [`SingleThreadValue`] was in the
        // `Unbound` stage, but is now in the `Binding` stage. This "reserves"
        // to be initialized: other concurrent accesses observing a `Binding`
        // stage must not assume that `thread_id_and_fn` is initialized, but
        // also will not attempt to being initialization themselves:
        if self
            .bound_to_thread
            .compare_exchange(
                // Expected current value:
                BoundToThreadStage::Unbound as usize,
                // New value:
                BoundToThreadStage::Binding as usize,
                // Success memory ordering:
                Ordering::Relaxed,
                // Failure memory ordering:
                Ordering::Relaxed,
            )
            .is_err()
        {
            return false;
        }

        // Great, we have reserved the value for initialization!
        //
        // Write the current thread ID, and the function symbol used to query
        // the currently running thread ID.
        let ptr_thread_id_and_fn = self.thread_id_and_fn.get();

        // # Safety
        //
        // We must ensure that there are no (mutable) aliases or concurrent
        // reads or writes of the thread_id_and_fn value.
        //
        // This value is accessed in three functions: `bind_to_thread_unsafe`,
        // `bind_to_thread`, and `bound_to_current_thread`:
        //
        // - `bind_to_thread_unsafe`: Callers of that function guarantee that
        //   there are no concurrent invocations of it together with our
        //   current function, `bind_to_thread`. Thus, no concurrent call to
        //   that other function can hold an alias, or perform a read or write
        //   of this value.
        //
        // - `bind_to_thread`: Before obtaining a reference to the
        //   `thread_id_and_fn` value or reading/writing it, this function
        //   performs a compare-exchange operation on the `bound_to_thread`
        //   value, ensuring that it is currently `Unbound`, and atomtically
        //   transitioning it towards `Binding`.
        //
        //   Our own compare-exchange operation on this value was successful. As
        //   `Binding` cannot transition back to `Unbound`, there can only be a
        //   single `bind_to_thread` call to successfully perform this
        //   compare-exchange operation per[`SingleThreadValue`].
        //
        //   If another concurrent call had performed this compare-exchange
        //   successfully, our own operation would have failed. Given that we
        //   successfully performed this operation, all other concurrent calls
        //   to this function must fail this operation instead, and thus will
        //   not take this if-branch, and therefore will not attempt to access
        //   the `thread_id_and_fn` value.
        //
        // - `bound_to_current_thread`: This function is allowed to run
        //   concurrently with `bind_to_thread`. However, it only accesses the
        //   `thread_id_and_fn` value when the `bound_to_thread` atomic
        //   contains `Bound`.
        //
        //   The compare and swap has transitioned this value from `Unbound` to
        //   `Binding`, and no concurrent call can further *modify*
        //   `bound_to_thread` other than our own function instance. This
        //   currently running function will only transition the
        //   `bound_to_thread` value from `Binding` to `Bound` once it has
        //   initialized `thread_id_and_fn` and all references to it cease to
        //   exist. Thus, no concurrent calls to `bound_to_current_thread` will
        //   read the `thread_id_and_fn` before that point.
        //
        // Hence this operation is sound.
        unsafe {
            *ptr_thread_id_and_fn =
                MaybeUninit::new((P::running_thread_id, P::running_thread_id()));
        }

        // When initializing the `SingleThreadValue`, we must use `Release`
        // ordering on the `bound_to_thread` store. This ensures that any
        // subsequent atomic load of this value with at least `Acquire`
        // ordering constraints will observe the previous write to
        // `thread_id_and_fn`).
        self.bound_to_thread
            .store(BoundToThreadStage::Bound as usize, Ordering::Release);

        // We have successfully bound this `SingleThreadValue` to the
        // currently running thread:
        true
    }

    /// Bind this [`SingleThreadValue`] to the currently running thread.
    ///
    /// If this [`SingleThreadValue`] is not already bound to a thread, or if it
    /// is not currently in the process of binding to a thread, then this binds
    /// the [`SingleThreadValue`] to the current thread.
    ///
    /// It further records the [`ThreadIdProvider::running_thread_id`] function
    /// reference, and uses this function to determine the currently running
    /// thread for any future queries.
    ///
    /// Returns `true` if this invocation successfully bound the value to the
    /// current thread. Otherwise, if the value was already bound to this same
    /// or another thread, it returns `false`.
    ///
    /// This method is `unsafe`, and does not require the target to support
    /// atomic operations (namely, `compare_exchange`) on `usize`-sized values.
    ///
    /// # Safety
    ///
    /// Callers of this function must ensure that this function is never called
    /// concurrently with other calls to
    /// [`bind_to_thread`](SingleThreadValue::bind_to_thread) or
    /// [`bind_to_thread_unsafe`](SingleThreadValue::bind_to_thread_unsafe) on
    /// the same [`SingleThreadValue`] instance.
    pub unsafe fn bind_to_thread_unsafe<P: ThreadIdProvider>(&self) -> bool {
        // For the check whether we're already bound to a thread, `Relaxed`
        // ordering is fine: we don't actually care about the value in
        // `thread_id_and_fn`, and don't need previous writes to it to be
        // visible to this thread.
        if self.bound_to_thread.load(Ordering::Relaxed) != BoundToThreadStage::Unbound as usize {
            // This value is already bound to a thread
            return false;
        }

        // Write the current thread ID, and the function symbol used to
        // query the currently running thread ID.
        let ptr_thread_id_and_fn = self.thread_id_and_fn.get();

        // # Safety
        //
        // We must ensure that there are no (mutable) aliases or concurrent
        // reads or writes of the `thread_id_and_fn` value.
        //
        // This value is accessed in three functions: `bind_to_thread`,
        // `bind_to_thread_unsafe`, and `bound_to_current_thread`:
        //
        // - `bind_to_thread` & `bind_to_thread_unsafe`: Callers of this current
        //   function guarantee that there are no concurrent invocations of
        //   either `bind_to_thread` or `bind_to_thread_unsafe` on the same
        //   `SingleThreadValue` object. Thus, no concurrent call to either of
        //   these functions can hold an alias, or perform a read or write of
        //   the `thread_id_and_fn` value.
        //
        // - `bound_to_current_thread`: This function is allowed to run
        //   concurrently with `bind_to_thread`. However, it only accesses this
        //   value when the `bound_to_thread` atomic contains `Bound`.
        //   `bound_to_current_thread` never writes to `bound_to_thread`.
        //
        //   This current function has, before taking this if-branch, checked
        //   that `bound_to_thread` is currently `Unbound`, and will continue
        //   to be `Unbound` for the duration of the write to
        //   `thread_id_and_fn`, as there are no concurrent calls to
        //   `bind_to_thread` or `bind_to_thread_unsafe`. It only transitions
        //   `bound_to_thread` to `Bound` after completing the initialization
        //   of `thread_id_and_fn` and all references to that value cease to
        //   exist.
        //
        // Thus, this operation is sound.
        unsafe {
            *ptr_thread_id_and_fn =
                MaybeUninit::new((P::running_thread_id, P::running_thread_id()));
        }

        // When initializing the `SingleThreadValue`, we must use `Release`
        // ordering on the `bound_to_thread` store. This ensures that any
        // subsequent atomic load of this value with at least `Acquire`
        // ordering constraints will observe the previous write to
        // `thread_id_and_fn`).
        self.bound_to_thread
            .store(BoundToThreadStage::Bound as usize, Ordering::Release);

        // We have successfully bound this `SingleThreadValue` to the
        // currently running thread:
        true
    }

    /// Check whether this `SingleThreadValue` instance is bound to the
    /// currently running thread ID.
    ///
    /// Returns `true` if the value is bound to the thread that is currently
    /// running (as determined by the implementation of `ThreadIdProvider` used
    /// in [`bind_to_thread`](SingleThreadValue::bind_to_thread) or
    /// [`bind_to_thread_unsafe`](SingleThreadValue::bind_to_thread_unsafe)).
    /// Returns `false` when a different thread is executing, when it is not
    /// bound to any thread yet, or currently in the process of being bound to a
    /// thread.
    pub fn bound_to_current_thread(&self) -> bool {
        // This function loads the `thread_id_and_fn` field, written by either
        // `bind_to_thread` or `bind_to_thread_unsafe` before setting
        // `bound_to_thread` to `Bound` with `Release` ordering constraints. To
        // make those writes visible, we need to load the `bound_to_thread`
        // value with at least `Acquire` ordering constraints:
        if self.bound_to_thread.load(Ordering::Acquire) != BoundToThreadStage::Bound as usize {
            // The `SingleThreadValue` value is not yet bound to any thread:
            return false;
        }

        // The `SingleThreadValue` value is bound to _a_ thread, check whether
        // it is the currently running thread:
        let ptr_thread_id_and_fn: *mut MaybeUninit<(fn() -> usize, usize)> =
            self.thread_id_and_fn.get();

        // # Safety
        //
        // We must ensure that there are no (mutable) aliases or concurrent
        // reads or writes of the thread_id_and_fn value.
        //
        // This value is accessed in three functions: `bind_to_thread`,
        // `bind_to_thread_unsafe`, and `bound_to_current_thread`:
        //
        // - `bind_to_thread` / `bind_to_thread_unsafe`: Before creating a
        //   reference to, reading from, or writing to the `bound_to_thread`
        //   value, both functions check that `bound_to_thread` is not already
        //   set to `Bound`. However, when reaching if-branch in this current
        //   function, we have observed that `bound_to_thread` is set to
        //   `Bound`. The state of `Bound` cannot be transitioned out of. Thus,
        //   `bound_to_thread` will never be written again.
        //
        //   `bound_to_thread` is only ever set to `Bound`, *after*
        //   `thread_id_and_fn` has been initialized, and all references to
        //   `thread_id_and_fn` cease to exist.
        //
        //   When reaching this point, neither `bind_to_thread` nor
        //   `bind_to_thread_unsafe` will ever read from, write to, or create a
        //   reference to `thread_id_and_fn` again.
        //
        // - `bound_to_current_thread`: This function is allowed to run
        //   concurrently with other calls to `bound_to_current_thread`.
        //
        //   However, this function only creates shared / immutable references
        //   to this value that are never written. Multiple shared references
        //   to this value are allowed to exist, so long as they are not
        //   aliased by another exclusive / mutable reference, and the
        //   underlying memory is not modified otherwise.
        //
        //   The only functions writing to `thread_id_and_fn` are
        //   `bind_to_thread` and `bind_to_thread_unsafe`. As stated above,
        //   when reaching this point, neither functions will ever write to
        //   `thread_id_and_fn` again, and all of their internal references
        //   have ceased to exist.
        //
        // Thus, this operation is sound.
        let maybe_thread_id_and_fn: &MaybeUninit<(fn() -> usize, usize)> =
            unsafe { &*(ptr_thread_id_and_fn as *const _) };

        // # Safety
        //
        // Both `bind_to_thread` and `bind_to_thread_unsafe` are guaranteed to
        // have initialized `thread_id_and_fn` *before* setting
        // `bound_to_thread` to `Bound` with `Release` ordering constraints.
        //
        // In this if-branch, `bound_to_thread` has been observed to be `Bound`,
        // a state that cannot be transitioned out of. We have loaded this
        // value with `Acquire` ordering constraints, making all values written
        // by other threads before a write to `bound_to_thread` with `Release`
        // constraints visible to this thread.
        //
        // Thus, we can safely rely on `thread_id_and_fn` to be initialized:
        let (running_thread_id_fn, bound_thread_id) =
            unsafe { maybe_thread_id_and_fn.assume_init() };

        // Finally, check if the thread this `SingleThreadValue` is bound to
        // is the running thread ID:
        bound_thread_id == running_thread_id_fn()
    }

    /// Obtain a reference to the contained value.
    ///
    /// This function checks whether the [`SingleThreadValue`] is bound to the
    /// currently running thread and, if so, returns a shared / immutable
    /// reference to its contained value.
    pub fn get(&self) -> Option<&T> {
        if self.bound_to_current_thread() {
            Some(&self.value)
        } else {
            None
        }
    }
}
