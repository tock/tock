//! Deferred call mechanism.
//!
//! This is a tool to allow chip peripherals to schedule "interrupts"
//! in the chip scheduler if the hardware doesn't support interrupts where
//! they are needed.

use core::cell::Cell;
use core::convert::Into;
use core::convert::TryFrom;
use core::convert::TryInto;
use core::marker::Copy;
use core::marker::PhantomData;

/// Any chip with peripherals which require deferred calls should
/// instantiate exactly one of these, and a reference to that manager should be
/// passed to all created `DeferredCall`s.
pub struct DeferredCallManager<T: Into<usize> + TryFrom<usize> + Copy> {
    v: Cell<usize>,
    _p: PhantomData<T>,
}

impl<T: Into<usize> + TryFrom<usize> + Copy> DeferredCallManager<T> {
    pub fn new() -> Self {
        Self {
            v: Cell::new(0),
            _p: PhantomData,
        }
    }

    /// Are there any pending `DeferredCall`s?
    pub fn has_tasks(&self) -> bool {
        self.v.get() != 0
    }

    /// Gets and clears the next pending `DeferredCall`
    pub fn next_pending(&self) -> Option<T> {
        let val = self.v.get();
        if val == 0 {
            None
        } else {
            let bit = val.trailing_zeros() as usize;
            let new_val = val & !(1 << bit);
            self.v.set(new_val);
            bit.try_into().ok()
        }
    }
}

/// Represents a way to generate an asynchronous call without a hardware
/// interrupt. Supports up to 32 possible deferrable tasks.
pub struct DeferredCall<T: 'static + Into<usize> + TryFrom<usize> + Copy> {
    task: T,
    mgr: &'static DeferredCallManager<T>,
}

impl<T: Into<usize> + TryFrom<usize> + Copy> DeferredCall<T> {
    /// Creates a new DeferredCall
    ///
    /// Only create one per task, preferably in the module that it will be used
    /// in. Creating more than 32 tasks on a given manager will lead to
    /// incorrect behavior.
    pub const fn new(task: T, mgr: &'static DeferredCallManager<T>) -> Self {
        DeferredCall { task, mgr }
    }

    /// Set the `DeferredCall` as pending
    pub fn set(&self) {
        self.mgr
            .v
            .set((1 << self.task.into() as usize) | self.mgr.v.get());
    }
}
