// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Component for the mock ISL29035 ambient light sensor.
//!
//! Creates a [`MockIsl29035`](capsules_extra::mock::sensors::isl29035::MockIsl29035),
//! which pretends to be a real ISL29035 chip on an I2C bus, and registers its
//! deferred call (used to imitate the I2C "transaction complete" interrupt).
//!
//! The returned sensor exposes `i2c::I2CDevice`; connect it to a mock I2C bus
//! with [`MockI2CBusDeviceComponent`](crate::mock::i2c_bus::MockI2CBusDeviceComponent).
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//! let mock_isl29035 = components::mock::isl29035::MockIsl29035Component::new()
//!     .finalize(components::mock_isl29035_component_static!());
//! mock_isl29035.set_lux(1000); // optional
//! ```

use capsules_extra::mock::sensors::isl29035::MockIsl29035;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::deferred_call::DeferredCallClient;

#[macro_export]
macro_rules! mock_isl29035_component_static {
    ($(,)?) => {{ kernel::static_buf!(capsules_extra::mock::sensors::isl29035::MockIsl29035<'static>) }};
}

pub type MockIsl29035ComponentType = MockIsl29035<'static>;

#[derive(Default)]
pub struct MockIsl29035Component {}

impl MockIsl29035Component {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for MockIsl29035Component {
    type StaticInput = &'static mut MaybeUninit<MockIsl29035<'static>>;
    type Output = &'static MockIsl29035<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let mock_isl29035 = s.write(MockIsl29035::new());
        mock_isl29035.register();
        mock_isl29035
    }
}
