// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use nrf52::chip::Nrf52DefaultPeripherals;

/// This struct, when initialized, instantiates all peripheral drivers for the nrf52840.
///
/// If a board wishes to use only a subset of these peripherals, this
/// should not be used or imported, and a modified version should be
/// constructed manually in main.rs.
pub struct Nrf52832DefaultPeripherals<'a> {
    pub nrf52: Nrf52DefaultPeripherals<'a>,
    pub gpio_port: crate::gpio::Port<'a, { crate::gpio::NUM_PINS }>,
}
impl Nrf52832DefaultPeripherals<'_> {
    pub unsafe fn new(aes_ecb_buf: &'static mut [u8; 48]) -> Self {
        Self {
            nrf52: Nrf52DefaultPeripherals::new(aes_ecb_buf),
            gpio_port: crate::gpio::nrf52832_gpio_create(),
        }
    }
    // Necessary for setting up circular dependencies
    pub fn init(&'static self) {
        self.nrf52.init();
    }
}
impl kernel::platform::chip::InterruptService for Nrf52832DefaultPeripherals<'_> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            nrf52::peripheral_interrupts::GPIOTE => self.gpio_port.handle_interrupt(),
            _ => return self.nrf52.service_interrupt(interrupt),
        }
        true
    }
}
