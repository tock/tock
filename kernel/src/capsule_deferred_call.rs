//! Deferred call mechanism.
//!
//! This is a tool to allow capsules to schedule "interrupts"
//! in the chip scheduler if the hardware doesn't support interrupts where
//! they are needed.

use crate::deferred_call::AtomicUsize;
use core::convert::Into;
use core::convert::TryFrom;
use core::convert::TryInto;
use core::marker::Copy;

static CAPSULE_DEFERRED_CALL: AtomicUsize = AtomicUsize::new(0);

/// Are there any pending `CapsuleDeferredCall`s?
pub fn has_tasks() -> bool {
    CAPSULE_DEFERRED_CALL.load_relaxed() != 0
}

/// Represents a way to generate an asynchronous call without a hardware
/// interrupt. Supports up to 32 possible deferrable tasks.
pub struct CapsuleDeferredCall<T>(T);

impl<T: Into<usize> + TryFrom<usize> + Copy> CapsuleDeferredCall<T> {
    /// Creates a new CapsuleDeferredCall
    ///
    /// Only create one per task, preferably in the module that it will be used
    /// in.
    pub const fn new(task: T) -> Self {
        CapsuleDeferredCall(task)
    }

    /// Set the `CapsuleDeferredCall` as pending
    pub fn set(&self) {
        CAPSULE_DEFERRED_CALL.fetch_or_relaxed(1 << self.0.into() as usize);
    }

    /// Gets and clears the next pending `CapsuleDeferredCall`
    pub fn next_pending() -> Option<T> {
        let val = CAPSULE_DEFERRED_CALL.load_relaxed();
        if val == 0 {
            None
        } else {
            let bit = val.trailing_zeros() as usize;
            let new_val = val & !(1 << bit);
            CAPSULE_DEFERRED_CALL.store_relaxed(new_val);
            bit.try_into().ok()
        }
    }
}

pub trait CapsuleTask: Into<usize> + TryFrom<usize> + Copy {}

pub trait DeferredCallMapper {
    type CAT: CapsuleTask;
    fn service_deferred_call(&self, task: Self::CAT) -> bool;
}
