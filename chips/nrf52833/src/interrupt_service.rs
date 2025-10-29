// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::hil::time::Alarm;
use nrf52::chip::Nrf52DefaultPeripherals;

/// This struct, when initialized, instantiates all peripheral drivers for the nrf52840.
///
/// If a board wishes to use only a subset of these peripherals, this
/// should not be used or imported, and a modified version should be
/// constructed manually in main.rs.
pub struct Nrf52833DefaultPeripherals<'a> {
    pub nrf52: Nrf52DefaultPeripherals<'a>,
    pub ieee802154_radio: crate::ieee802154_radio::Radio<'a>,
    pub gpio_port: crate::gpio::Port<'a, { crate::gpio::NUM_PINS }>,
}
impl Nrf52833DefaultPeripherals<'_> {
    pub unsafe fn new(
        ieee802154_radio_ack_buf: &'static mut [u8; crate::ieee802154_radio::ACK_BUF_SIZE],
        aes_ecb_buf: &'static mut [u8; 48],
    ) -> Self {
        Self {
            nrf52: Nrf52DefaultPeripherals::new(aes_ecb_buf),
            ieee802154_radio: crate::ieee802154_radio::Radio::new(ieee802154_radio_ack_buf),
            gpio_port: crate::gpio::nrf52833_gpio_create(),
        }
    }
    // Necessary for setting up circular dependencies
    pub fn init(&'static self) {
        self.ieee802154_radio.set_timer_ref(&self.nrf52.timer0);
        self.nrf52.timer0.set_alarm_client(&self.ieee802154_radio);
        kernel::deferred_call::DeferredCallClient::register(&self.ieee802154_radio);
        self.nrf52.init();
    }
}
impl kernel::platform::chip::InterruptService for Nrf52833DefaultPeripherals<'_> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            nrf52::peripheral_interrupts::GPIOTE => self.gpio_port.handle_interrupt(),
            nrf52::peripheral_interrupts::RADIO => {
                match (
                    self.ieee802154_radio.is_enabled(),
                    self.nrf52.ble_radio.is_enabled(),
                ) {
                    (false, false) => (),
                    (true, false) => self.ieee802154_radio.handle_interrupt(),
                    (false, true) => self.nrf52.ble_radio.handle_interrupt(),
                    (true, true) => kernel::debug!(
                        "nRF 802.15.4 and BLE radios cannot be simultaneously enabled!"
                    ),
                }
            }
            _ => return self.nrf52.service_interrupt(interrupt),
        }
        true
    }
}
