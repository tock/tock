// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::ptr;

use tock_cells::volatile_cell::VolatileCell;
use tock_registers::LocalRegisterCopy;

use crate::support;

use super::NUM_VECTORS;

/// A mechanism for synchronously managing and polling x86 interrupts.
///
/// Tock uses synchronous polling to service interrupts. This means that the kernel's main loop will
/// periodically call some function to detect and service interrupts. The reasoning for this
/// approach (as opposed to doing work directly within ISRs) is so we can lean on Rust's borrow
/// checker to avoid race conditions.
///
/// The `InterruptPoller` type provides a somewhat higher-level API for working with x86 interrupts
/// that fits well with Tock's synchronous lifecycle. It is modeled after the `Plic` API from the
/// `e310x` chip crate.
///
/// Internally, it maintains a large bitfield with a separate flag for every possible interrupt
/// vector (total of `NUM_VECTORS`). When an interrupt occurs, a very lightweight ISR is
/// responsible for setting the corresponding flag. To poll for pending interrupts from within the
/// kernel loop, we simply need to iterate over this bitfield and return the index of each active
/// bit.
///
/// Note that for reasons of safety, `InterruptPoller` is a singleton. You cannot create an instance
/// directly. Instead, you must access the singleton instance using `InterruptPoller::access`.
pub struct InterruptPoller {
    /// Tracks the pending status of each interrupt
    pending: [VolatileCell<LocalRegisterCopy<u32>>; NUM_VECTORS / 32],
}

/// The singleton poller instance
///
/// We use a `static mut` singleton so that the instance can be accessed directly from interrupt
/// handler routines.
///
/// ## Safety
///
/// As with any `static mut` item, the poller singleton must not be accessed concurrently. To
/// enforce this restriction, this module exposes two constrained methods for accessing the
/// instance: `InterruptPoller::access` and `InterruptPoller::save`.
static mut SINGLETON: InterruptPoller = InterruptPoller {
    pending: [
        VolatileCell::new(LocalRegisterCopy::new(0)),
        VolatileCell::new(LocalRegisterCopy::new(0)),
        VolatileCell::new(LocalRegisterCopy::new(0)),
        VolatileCell::new(LocalRegisterCopy::new(0)),
        VolatileCell::new(LocalRegisterCopy::new(0)),
        VolatileCell::new(LocalRegisterCopy::new(0)),
        VolatileCell::new(LocalRegisterCopy::new(0)),
        VolatileCell::new(LocalRegisterCopy::new(0)),
    ],
};

impl InterruptPoller {
    /// Provides safe access to the singleton instance of `InterruptPoller`.
    ///
    /// The given closure `f` is executed with interrupts disabled (using [`support::atomic`](crate::support::atomic)) and
    /// passed a reference to the singleton.
    pub fn access<F, R>(f: F) -> R
    where
        F: FnOnce(&InterruptPoller) -> R,
    {
        support::atomic(|| {
            // Safety: Interrupts are disabled within this closure, so we can safely access the
            //         singleton without racing against interrupt handlers.
            let poller = unsafe { &*ptr::addr_of!(SINGLETON) };

            f(poller)
        })
    }

    /// Marks that the specified interrupt as pending.
    ///
    /// ## Safety
    ///
    /// Interrupts must be disabled when this function is called. This function is _intended_ to be
    /// called from within an ISR, so hopefully this is already true.
    pub unsafe fn set_pending(num: u32) {
        // Safety: Caller ensures interrupts are disabled when this function is called, so it
        //         should be safe to access the singleton without racing against any interrupt
        //         handlers.
        let poller = unsafe { &*ptr::addr_of!(SINGLETON) };

        let arr_idx = (num / 32) as usize;
        let bit_idx = num % 32;

        let new_val = poller.pending[arr_idx].get().get() | 1 << bit_idx;
        poller.pending[arr_idx].set(LocalRegisterCopy::new(new_val));
    }

    /// Polls for the next pending interrupt.
    ///
    /// If multiple interrupts are currently pending, then the highest priority (i.e. numerically
    /// lowest) is returned.
    ///
    /// Once handled, interrupts should call `clear_pending` to clear the interrupt's pending status
    /// so that lower-priority interrupts can be serviced.
    pub fn next_pending(&self) -> Option<u32> {
        for (i, pending) in self.pending.iter().enumerate() {
            let val = pending.get().get();
            if val != 0 {
                return Some(val.trailing_zeros() + (i as u32 * 32));
            }
        }

        None
    }

    /// Clears the pending status of the specified interrupt, allowing lower priority interrupts to
    /// be serviced.
    ///
    /// Don't forget to call this method after servicing an interrupt.
    pub fn clear_pending(&self, num: u32) {
        let arr_idx = (num / 32) as usize;
        let bit_idx = num % 32;

        let new_val = self.pending[arr_idx].get().get() & !(1 << bit_idx);
        self.pending[arr_idx].set(LocalRegisterCopy::new(new_val));
    }
}
