// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Public traits for interfaces between Tock components.

pub mod adc;
pub mod analog_comparator;
pub mod ble_advertising;
pub mod bus8080;
pub mod buzzer;
pub mod can;
pub mod crc;
pub mod dac;
pub mod date_time;
pub mod digest;
pub mod eic;
pub mod entropy;
pub mod ethernet;
pub mod flash;
pub mod gpio;
pub mod gpio_async;
pub mod hasher;
pub mod hw_debug;
pub mod i2c;
pub mod kv;
pub mod led;
pub mod log;
pub mod nonvolatile_storage;
pub mod public_key_crypto;
pub mod pwm;
pub mod radio;
pub mod rng;
pub mod screen;
pub mod sensors;
pub mod servo;
pub mod spi;
pub mod symmetric_encryption;
pub mod text_screen;
pub mod time;
pub mod touch;
pub mod uart;
pub mod usb;
pub mod usb_hid;

/// Shared interface for configuring components.
pub trait Controller {
    type Config;

    fn configure(&self, _: Self::Config);
}
