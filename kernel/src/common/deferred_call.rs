//! Deferred call mechanism.
//!
//! This is a tool to allow chip peripherals to schedule "interrupts"
//! in the chip scheduler if the hardware doesn't support interrupts where
//! they are needed.

use core::convert::Into;
use core::convert::TryFrom;
use core::convert::TryInto;
use core::marker::Copy;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;

static DEFERRED_CALL: AtomicUsize = AtomicUsize::new(0);

/// Are there any pending `DeferredCall`s?
pub fn has_tasks() -> bool {
    DEFERRED_CALL.load(Ordering::Relaxed) != 0
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
        // DEFERRED_CALL.fetch_or(1 << self.0.into() as usize, Ordering::Relaxed);
        let val = DEFERRED_CALL.load(Ordering::Relaxed);
        let new_val = val | (1 << self.0.into());
        DEFERRED_CALL.store(new_val, Ordering::Relaxed);
    }

    /// Gets and clears the next pending `DeferredCall`
    pub fn next_pending() -> Option<T> {
        let val = DEFERRED_CALL.load(Ordering::Relaxed);
        if val == 0 {
            None
        } else {
            let bit = val.trailing_zeros() as usize;
            let new_val = val & !(1 << bit);
            DEFERRED_CALL.store(new_val, Ordering::Relaxed);
            bit.try_into().ok()
        }
    }
}
