// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! LiteX Event Manager
//!
//! Documentation on the different LiteX event source, which all
//! behave differently, can be found in the LiteX repository under
//! [`litex/soc/interconnect/csr_eventmanager.py`](https://github.com/enjoy-digital/litex/blob/master/litex/soc/interconnect/csr_eventmanager.py).

use crate::litex_registers::{Read, ReadWrite, UIntLike};
use core::marker::PhantomData;

/// LiteX event manager abstraction
///
/// A LiteX event manager combines and manages event sources of a
/// LiteX core / peripheral. The event manager itself is connected to
/// an interrupt source in the CPU.
///
/// This is an abstraction over an instance of a LiteX EventManager,
/// which is exposed to the operating system using three configuration
/// status registers (LiteX CSRs), as part of the core's /
/// peripheral's configuration status registers bank.
pub struct LiteXEventManager<'a, T, S, P, E>
where
    T: UIntLike,
    S: Read<T>,
    P: ReadWrite<T>,
    E: ReadWrite<T>,
{
    status: &'a S,
    pending: &'a P,
    enable: &'a E,
    _base_type: PhantomData<T>,
}

impl<'a, T, S, P, E> LiteXEventManager<'a, T, S, P, E>
where
    T: UIntLike,
    S: Read<T>,
    P: ReadWrite<T>,
    E: ReadWrite<T>,
{
    pub const fn new(status: &'a S, pending: &'a P, enable: &'a E) -> Self {
        LiteXEventManager {
            status,
            pending,
            enable,
            _base_type: PhantomData,
        }
    }

    /// Disable / suppress all event sources connected to the LiteX
    /// event manager.
    ///
    /// This will prevent any of the event sources from asserting the
    /// event manager's CPU interrupt.
    pub fn disable_all(&self) {
        self.enable.set(T::zero());
    }

    /// Enable all event sources connected to the LiteX
    /// event manager.
    ///
    /// This will make any asserted (pending) event source assert the
    /// event manager's CPU interrupt.
    pub fn enable_all(&self) {
        self.enable.set(T::max());
    }

    /// Disable / suppress an event source connected to the LiteX
    /// event manager.
    ///
    /// This will prevent the specific event source from asserting the
    /// event manager's CPU interrupt.
    ///
    /// The event is addressed by its index in the event manager's
    /// registers (starting at 0).
    pub fn disable_event(&self, index: usize) {
        self.enable.set(self.enable.get() & !(T::one() << index));
    }

    /// Enable an event source connected to the LiteX event manager
    ///
    /// This will assert the event manager's CPU interrupt if this or
    /// any other event source is asserted (pending).
    ///
    /// The event is addressed by its index in the event manager's
    /// registers (starting at 0).
    pub fn enable_event(&self, index: usize) {
        self.enable.set(self.enable.get() | (T::one() << index));
    }

    /// Check whether an event is enabled.
    ///
    /// This checks whether an event source may assert the event
    /// manager's CPU interrupt.
    ///
    /// The event is addressed by its index in the event manager's
    /// registers (starting at 0).
    pub fn event_enabled(&self, index: usize) -> bool {
        self.enable.get() & (T::one() << index) != T::zero()
    }

    /// Get all enabled events.
    ///
    /// The enabled events are encoded as bits in the returned integer
    /// type, starting from the least significant bit for the first
    /// event source (index 0), where a `1` means enabled and `0`
    /// means disabled (suppressed).
    pub fn events_enabled(&self) -> T {
        self.enable.get()
    }

    /// Get the input signal to an event source.
    ///
    /// This returns whether an event source input is currently
    /// asserted. This is independent of whether the event is actually
    /// enabled or pending.
    ///
    /// The event source is addressed by its index in the event
    /// manager's registers (starting at 0).
    pub fn event_source_input(&self, index: usize) -> bool {
        self.status.get() & (T::one() << index) != T::zero()
    }

    /// Check whether any event source is pending.
    ///
    /// This returns whether any event source is claiming to be
    /// pending. This is irrespective of whether an event source has a
    /// specific input or is enabled.
    ///
    /// An example for an event source which can be pending
    /// irrespective of the current input is an "EventSourceProcess",
    /// which triggers on a falling edge of the input and stays
    /// pending until cleared.
    pub fn any_event_pending(&self) -> bool {
        self.pending.get() != T::zero()
    }

    /// Check whether an event source is pending.
    ///
    /// This returns whether an event source is claiming to be
    /// pending. This is irrespective of whether an event source has a
    /// specific input or is enabled.
    ///
    /// An example for an event source which can be pending
    /// irrespective of the current input is an "EventSourceProcess",
    /// which triggers on a falling edge of the input and stays
    /// pending until cleared.
    ///
    /// The event source is addressed by its index in the event
    /// manager's registers (starting at 0).
    pub fn event_pending(&self, index: usize) -> bool {
        self.pending.get() & (T::one() << index) != T::zero()
    }

    /// Get all pending events.
    ///
    /// The pending events are encoded as bits in the returned integer
    /// type, starting from the least significant bit for the first
    /// event source (index 0), where a `1` means that the event
    /// source is pending.
    pub fn events_pending(&self) -> T {
        self.pending.get()
    }

    /// Check whether an event source is asserting the event manager's
    /// CPU interrupt (both enabled and pending).
    ///
    ///
    /// The event source is addressed by its index in the event
    /// manager's registers (starting at 0).
    pub fn event_asserted(&self, index: usize) -> bool {
        self.event_enabled(index) && self.event_pending(index)
    }

    /// Get the next asserted event, starting from 0.
    ///
    /// If an asserted event was found, its index is returned. Otherwise, None
    /// is returned.
    ///
    /// This method works by ANDing the enabled and pending bits and using the
    /// trailing_zeros intrinsic (of which there may be an optimized version
    /// with special instructions). Thus this is faster than a naive, loop-based
    /// version.
    pub fn next_asserted(&self) -> Option<usize> {
        let ev_bits = core::mem::size_of::<T>() * 8;
        let enabled = self.events_enabled();
        let pending = self.events_pending();
        let asserted = enabled & pending;

        // If there are no interrupts pending (asserted == 0), T::trailing_zeros
        // will return the number of bits in T, in which case we need to return
        // None.
        let trailing_zeros = T::trailing_zeros(asserted) as usize;
        if trailing_zeros == ev_bits {
            None
        } else {
            Some(trailing_zeros)
        }
    }

    /// Clear a pending event source.
    ///
    /// This operation may have side effects in the device (for
    /// instance, acknowledge the reception of a UART data word).
    ///
    /// It is not guaranteed that the event source will be no longer
    /// pending after clearing (for instance when used with FIFOs and
    /// pending data, or with an "EventSourceLevel" which can only be
    /// cleared by driving the input signal low).
    pub fn clear_event(&self, index: usize) {
        self.pending.set(T::one() << index);
    }

    /// Clear all pending event sources.
    ///
    /// This operation may have side effects in the device (for
    /// instance, acknowledge the reception of a UART data word).
    ///
    /// It is not guaranteed that the event sources will be no longer
    /// pending after clearing (for instance when used with FIFOs and
    /// pending data, or with an "EventSourceLevel" which can only be
    /// cleared by driving the input signal low).
    pub fn clear_all(&self) {
        self.pending.set(T::max());
    }
}
