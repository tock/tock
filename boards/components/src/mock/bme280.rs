// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Component for the mock BME280 humidity/pressure/temperature sensor.
//!
//! Creates a [`MockBme280`](capsules_extra::mock::sensors::bme280::MockBme280),
//! which pretends to be a real BME280 chip on an I2C bus, and registers its
//! deferred call (used to imitate the I2C "transaction complete" interrupt).
//!
//! The returned sensor exposes `i2c::I2CDevice`; connect it to a mock I2C bus
//! with [`MockI2CBusDeviceComponent`](crate::mock::i2c_bus::MockI2CBusDeviceComponent).
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//! let mock_bme280 = components::mock::bme280::MockBme280Component::new()
//!     .finalize(components::mock_bme280_component_static!());
//! mock_bme280.set_pressure(1023); // optional: 1023 hPa
//! ```

use capsules_extra::mock::sensors::bme280::MockBme280;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::deferred_call::DeferredCallClient;

#[macro_export]
macro_rules! mock_bme280_component_static {
    ($(,)?) => {{ kernel::static_buf!(capsules_extra::mock::sensors::bme280::MockBme280<'static>) }};
}

pub type MockBme280ComponentType = MockBme280<'static>;

#[derive(Default)]
pub struct MockBme280Component {}

impl MockBme280Component {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for MockBme280Component {
    type StaticInput = &'static mut MaybeUninit<MockBme280<'static>>;
    type Output = &'static MockBme280<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let mock_bme280 = s.write(MockBme280::new());
        mock_bme280.register();
        mock_bme280
    }
}
