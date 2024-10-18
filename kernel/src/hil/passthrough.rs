// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Interfaces for device pass through

/// Simple interface for pass through devices.
///
/// This should be implemented before `filter_passthrough()`
/// allows the device to be passed through to userspace.
pub trait PassThroughDevice<'a> {
    fn set_client(&self, client: &'a dyn Client);
}

/// Trait for handling callbacks from passed through devices.
///
/// This should be called whenever an interrupt occurs for the
/// device. The interrupt should be disabled and cleared then
/// the interrupt information passed to userspace via the
/// `interrupt_occurred()` upcall.
///
/// An example implementation would look like this
///
/// ```rust,ignore
///    pub fn handle_interrupt(&self) {
///        let irqs = self.registers.intstat.extract();
///
///        // Disable and clear interrupts
///        self.disable_interrupts();
///
///        // Pass the information to the device passthrough
///        // and then stop processing the interrupt
///        if self.passthrough_client.is_some() {
///            self.passthrough_client.map(|client| {
///                client.interrupt_occurred(irqs.get() as usize);
///            });
///        } else {
///             // Do normal IRQ processing
///         }
///     }
/// ```
pub trait Client {
    /// Called when an interrupt occurs. `interrupt_status`
    /// should contain the interrupt information from hardware that
    /// was cleared.
    fn interrupt_occurred(&self, interrupt_status: usize);
}
