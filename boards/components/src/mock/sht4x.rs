// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Component for the mock SHT4x temperature/humidity sensor.
//!
//! Creates a [`MockSht4x`](capsules_extra::mock::sensors::sht4x::MockSht4x),
//! which pretends to be a real SHT4x chip on an I2C bus, and registers its
//! deferred call (used to imitate the I2C "transaction complete" interrupt).
//!
//! The returned sensor exposes `i2c::I2CDevice`; connect it to a mock I2C bus
//! with [`MockI2CBusDeviceComponent`](crate::mock::i2c_bus::MockI2CBusDeviceComponent).
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//! let mock_sht4x = components::mock::sht4x::MockSht4xComponent::new()
//!     .finalize(components::mock_sht4x_component_static!());
//! mock_sht4x.set_temperature(2537); // optional: 25.37 C
//! ```

use capsules_extra::mock::sensors::sht4x::MockSht4x;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::deferred_call::DeferredCallClient;

#[macro_export]
macro_rules! mock_sht4x_component_static {
    ($(,)?) => {{ kernel::static_buf!(capsules_extra::mock::sensors::sht4x::MockSht4x<'static>) }};
}

pub type MockSht4xComponentType = MockSht4x<'static>;

#[derive(Default)]
pub struct MockSht4xComponent {}

impl MockSht4xComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for MockSht4xComponent {
    type StaticInput = &'static mut MaybeUninit<MockSht4x<'static>>;
    type Output = &'static MockSht4x<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let mock_sht4x = s.write(MockSht4x::new());
        mock_sht4x.register();
        mock_sht4x
    }
}
