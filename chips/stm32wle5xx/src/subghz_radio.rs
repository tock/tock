// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Sub-GHz Radio Virtual GPIOs for the STM32WLE5xx.
//!
//! The Stm3wle5xx includes a sub-ghz spi radio peripheral
//! on the same soc. Typically, these radios expose gpios
//! for interrupt signaling, chip select, and busy status. However,
//! the stm32wle5xx does not expose these as physical gpios.
//! The Tock Radiolib library expects these to be available as
//! gpios. This module provides virtual gpio implementations
//! for these functionalities so that they can be exposed to userspace
//! through the gpio capsule.
//!
//! Radiolib takes a similar approach in their stm32wle5xx port:
//! https://github.com/jgromes/RadioLib/issues/588.

use core::cell::Cell;

use kernel::{
    hil::gpio::{Configure, Input, Interrupt, Output},
    utilities::cells::OptionalCell,
};

/// Trait to expose a gpio-like interface for subghz radio
/// functionality that is not tied to a physical GPIO pin.
pub trait VirtualGpio<'a> {
    fn read(&self) -> bool;
    fn write(&self, val: u32);
    fn set_client(&'a self, _client: &'a dyn kernel::hil::gpio::Client) {
        // Default implementation does nothing.
    }
    fn disable_interrupts(&self) {
        // Default implementation does nothing.
    }
    fn enable_interrupts(&self) {
        // Default implementation does nothing.
    }
}

/// SPI chip select for the Sub-GHz radio.
///
/// The `subghzspicr` register of the `PWR` peripheral
/// determines the NSS state.
pub struct NSS {
    pwr: &'static crate::pwr::Pwr,
}

impl VirtualGpio<'_> for NSS {
    fn read(&self) -> bool {
        self.pwr.is_set_nss()
    }

    fn write(&self, val: u32) {
        if val == 0 {
            self.pwr.clear_nss();
        } else if val == 1 {
            self.pwr.set_nss();
        } else {
            // Do nothing. The valid values are only 0 or 1.
        }
    }
}

/// Interrupt line for the Sub-GHz radio.
pub struct SubGhzRadioInterrupt<'a> {
    client: OptionalCell<&'a dyn kernel::hil::gpio::Client>,
    interrupt_disabled: Cell<bool>,
}

impl SubGhzRadioInterrupt<'_> {
    pub fn new() -> Self {
        SubGhzRadioInterrupt {
            client: OptionalCell::empty(),
            interrupt_disabled: Cell::new(false),
        }
    }

    pub fn handle_interrupt(&self) {
        if self.interrupt_disabled.get() {
            return;
        }

        // notify client
        self.client.map(|client| {
            client.fired();
        });

        self.disable_interrupts();
    }
}

impl<'a> VirtualGpio<'a> for SubGhzRadioInterrupt<'a> {
    fn read(&self) -> bool {
        // The Sub-Ghz radio interrupt is level triggered
        // and cannot be cleared except by issuing a subghzspi
        // command to the subghz radio. Because of this, we mask
        // the interrupt in the interrupt handler and perform the
        // check here to see if any other interrupts are pending.
        unsafe {
            cortexm4::nvic::next_pending_with_mask((u128::MAX, !(1 << crate::nvic::RADIO_IRQ)))
                .is_some_and(|_| true)
        }
    }
    fn write(&self, _val: u32) {
        // Read-only, write does nothing.
    }

    fn set_client(&self, _client: &'a dyn kernel::hil::gpio::Client) {
        self.client.replace(_client);
    }

    fn disable_interrupts(&self) {
        self.interrupt_disabled.set(true);
    }

    fn enable_interrupts(&self) {
        self.interrupt_disabled.set(false);
    }
}

/// Busy line for the Sub-GHz radio.
pub struct SubGhzRadioBusy {
    pwr: &'static crate::pwr::Pwr,
}

impl SubGhzRadioBusy {
    pub fn new(pwr: &'static crate::pwr::Pwr) -> Self {
        SubGhzRadioBusy { pwr }
    }
}

impl VirtualGpio<'_> for SubGhzRadioBusy {
    fn read(&self) -> bool {
        self.pwr.is_rfbusys()
    }

    fn write(&self, _val: u32) {
        // Read-only, write does nothing.
    }
}

/// SubGhzRadio Virtual Gpio to be used by capsules.
pub struct SubGhzRadioVirtualGpio<'a> {
    reader: &'a dyn VirtualGpio<'a>,
}

impl<'a> SubGhzRadioVirtualGpio<'a> {
    pub fn new(reader: &'a dyn VirtualGpio<'a>) -> Self {
        SubGhzRadioVirtualGpio { reader }
    }
}

impl Input for SubGhzRadioVirtualGpio<'_> {
    fn read(&self) -> bool {
        self.reader.read()
    }
}

impl Output for SubGhzRadioVirtualGpio<'_> {
    fn clear(&self) {
        // do nothing
    }

    fn set(&self) {
        // do nothing
    }

    fn toggle(&self) -> bool {
        // do nothing
        false
    }
}

// To pass this as a GPIO to capsules, we need to implement
// the GPIO traits, but most of them are no-ops since
// there is no physical pin to manipulate.
impl<'a> Interrupt<'a> for SubGhzRadioVirtualGpio<'a> {
    fn disable_interrupts(&self) {
        self.reader.disable_interrupts();
    }

    fn enable_interrupts(&self, _mode: kernel::hil::gpio::InterruptEdge) {
        self.reader.enable_interrupts();
    }

    fn is_pending(&self) -> bool {
        unsafe {
            cortexm4::nvic::next_pending_with_mask((u128::MAX, !(1 << crate::nvic::RADIO_IRQ)))
                .is_some_and(|_| true)
        }
    }

    fn set_client(&self, client: &'a dyn kernel::hil::gpio::Client) {
        self.reader.set_client(client);
    }
}

// The Configure trait must be implemented for the GPIO capsule. All methods
// are no-ops and return values are garbage values.
//
// We cannot simply leave these functions as `unimplemented!()` as this would
// allow an application to potentially panic the kernel (if they were to call
// the given functionality indirectly through the GPIO capsule).
impl Configure for SubGhzRadioVirtualGpio<'_> {
    fn configuration(&self) -> kernel::hil::gpio::Configuration {
        kernel::hil::gpio::Configuration::Other
    }

    fn deactivate_to_low_power(&self) {
        // do nothing
    }

    fn disable_input(&self) -> kernel::hil::gpio::Configuration {
        kernel::hil::gpio::Configuration::Other
    }

    fn disable_output(&self) -> kernel::hil::gpio::Configuration {
        kernel::hil::gpio::Configuration::Other
    }

    fn floating_state(&self) -> kernel::hil::gpio::FloatingState {
        kernel::hil::gpio::FloatingState::PullNone
    }

    fn set_floating_state(&self, _state: kernel::hil::gpio::FloatingState) {
        // do nothing
    }

    fn is_input(&self) -> bool {
        false
    }

    fn is_output(&self) -> bool {
        false
    }

    fn make_input(&self) -> kernel::hil::gpio::Configuration {
        kernel::hil::gpio::Configuration::Other
    }

    fn make_output(&self) -> kernel::hil::gpio::Configuration {
        kernel::hil::gpio::Configuration::Other
    }
}
