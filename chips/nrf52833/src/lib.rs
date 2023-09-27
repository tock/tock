// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

#![no_std]

// FIXME: Move ieee802154_radio to an nrf528xx crate so this can access it.

pub use nrf52::{
    acomp, adc, aes, ble_radio, chip, clock, constants, crt1, ficr, i2c, init, nvmc,
    peripheral_interrupts as base_interrupts, pinmux, power, ppi, pwm, rtc, spi, temperature,
    timer, trng, uart, uicr,
};
pub mod gpio;
pub mod interrupt_service;
