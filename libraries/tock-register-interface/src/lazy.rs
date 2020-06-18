//! Lazy global singleton.
//!
//! A safe, lazily initialized global singleton.

// This module requires libstd.
use std::cell::Cell;
use std::mem::MaybeUninit;
use std::sync::Once;

/// Lazily initialized singleton.
///
/// Since this is intended to be used as a static item, `T` must implement `Sync` to ensure that
/// the singleton can be safely shared among threads.
pub(crate) struct Lazy<T: Sync> {
    inner: Cell<MaybeUninit<T>>,
    once: Once,
}

impl<T: Sync> Lazy<T> {
    /// Creates a new `Lazy<T>` instance.
    pub const fn new() -> Lazy<T> {
        Lazy {
            inner: Cell::new(MaybeUninit::uninit()),
            once: Once::new(),
        }
    }

    /// Gets the value stored in the `Lazy` instance, initializing it by calling `f` if this is the
    /// first access.
    pub fn get<F>(&'static self, f: F) -> &'static T
    where
        F: FnOnce() -> T,
    {
        self.once
            .call_once(|| self.inner.set(MaybeUninit::new(f())));

        // Safe because inner's MaybeUninit was initialized by call_once.
        unsafe { (*self.inner.as_ptr()).as_ptr().as_ref().unwrap() }
    }
}

// Safe because inner's `Cell` and `MaybeUninit` are only modified in a thread-safe way via `Once`.
unsafe impl<T: Sync> Sync for Lazy<T> {}
