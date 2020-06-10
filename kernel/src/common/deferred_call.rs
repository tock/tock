//! Deferred call mechanism.
//!
//! This is a tool to allow chip peripherals to schedule "interrupts"
//! in the chip scheduler if the hardware doesn't support interrupts where
//! they are needed.

use core::cell::UnsafeCell;
use core::convert::Into;
use core::convert::TryFrom;
use core::convert::TryInto;
use core::intrinsics;
use core::marker::Copy;
use core::marker::Sync;

/// AtomicUsize with no CAS operations that works on targets that have "no atomic
/// support" according to their specification. This makes it work on thumbv6
/// platforms.
///
/// Borrowed from https://github.com/japaric/heapless/blob/master/src/ring_buffer/mod.rs
/// See: https://github.com/japaric/heapless/commit/37c8b5b63780ed8811173dc1ec8859cd99efa9ad
struct AtomicUsize {
    v: UnsafeCell<usize>,
}

impl AtomicUsize {
    pub(crate) const fn new(v: usize) -> AtomicUsize {
        AtomicUsize {
            v: UnsafeCell::new(v),
        }
    }

    pub(crate) fn load_relaxed(&self) -> usize {
        unsafe { intrinsics::atomic_load_relaxed(self.v.get()) }
    }

    pub(crate) fn store_relaxed(&self, val: usize) {
        unsafe { intrinsics::atomic_store_relaxed(self.v.get(), val) }
    }

    pub(crate) fn fetch_or_relaxed(&self, val: usize) {
        unsafe { intrinsics::atomic_store_relaxed(self.v.get(), self.load_relaxed() | val) }
    }
}

unsafe impl Sync for AtomicUsize {}

static DEFERRED_CALL: AtomicUsize = AtomicUsize::new(0);

/// Are there any pending `DeferredCall`s?
pub fn has_tasks() -> bool {
    DEFERRED_CALL.load_relaxed() != 0
}

/// Represents a way to generate an asynchronous call without a hardware
/// interrupt. Supports up to 32 possible deferrable tasks.
pub struct DeferredCall<T>(T);

impl<T: Into<usize> + TryFrom<usize> + Copy> DeferredCall<T> {
    /// Creates a new DeferredCall
    ///
    /// Only create one per task, preferably in the module that it will be used
    /// in.
    pub const unsafe fn new(task: T) -> Self {
        DeferredCall(task)
    }

    /// Set the `DeferredCall` as pending
    pub fn set(&self) {
        DEFERRED_CALL.fetch_or_relaxed(1 << self.0.into() as usize);
    }

    /// Gets and clears the next pending `DeferredCall`
    pub fn next_pending() -> Option<T> {
        let val = DEFERRED_CALL.load_relaxed();
        if val == 0 {
            None
        } else {
            let bit = val.trailing_zeros() as usize;
            let new_val = val & !(1 << bit);
            DEFERRED_CALL.store_relaxed(new_val);
            bit.try_into().ok()
        }
    }
}
