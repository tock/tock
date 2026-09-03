// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Components for the mock I2C bus ("reverse" I2C virtualizer).
//!
//! [`MockI2CBus`](capsules_extra::mock::i2c_bus::MockI2CBus) presents an
//! `i2c::I2CMaster` bus upward (so the normal `MuxI2C` + device drivers stack
//! on it unchanged) and dispatches each transaction to the attached mock
//! device whose address matches.
//!
//! Two components:
//!
//! 1. [`MockI2CBusComponent`] creates the bus.
//! 2. [`MockI2CBusDeviceComponent`] attaches one mock device to the bus at a
//!    given address (and registers the bus as that device's completion-callback
//!    client).
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//! let mock_sht4x = components::mock::sht4x::MockSht4xComponent::new()
//!     .finalize(components::mock_sht4x_component_static!());
//!
//! let i2c_bus = components::mock::i2c_bus::MockI2CBusComponent::new()
//!     .finalize(components::mock_i2c_bus_component_static!());
//!
//! components::mock::i2c_bus::MockI2CBusDeviceComponent::new(
//!     i2c_bus,
//!     mock_sht4x,
//!     capsules_extra::mock::sensors::sht4x::BASE_ADDR,
//! )
//! .finalize(components::mock_i2c_bus_device_component_static!());
//!
//! // Stack the normal I2C virtualizer on top of the mock bus:
//! let mux_i2c = components::i2c::I2CMuxComponent::new(i2c_bus, None)
//!     .finalize(components::i2c_mux_component_static!(
//!         capsules_extra::mock::i2c_bus::MockI2CBus<'static>
//!     ));
//! ```

use capsules_extra::mock::i2c_bus::{I2CBusDevice, MockI2CBus, MockI2CDevice};
use core::mem::MaybeUninit;
use kernel::component::Component;

#[macro_export]
macro_rules! mock_i2c_bus_component_static {
    ($(,)?) => {{ kernel::static_buf!(capsules_extra::mock::i2c_bus::MockI2CBus<'static>) }};
}

#[macro_export]
macro_rules! mock_i2c_bus_device_component_static {
    ($(,)?) => {{ kernel::static_buf!(capsules_extra::mock::i2c_bus::I2CBusDevice<'static>) }};
}

pub type MockI2CBusComponentType = MockI2CBus<'static>;
pub type MockI2CBusDeviceComponentType = I2CBusDevice<'static>;

//------------------------------------------------------------------------------
// The bus itself.
//------------------------------------------------------------------------------

#[derive(Default)]
pub struct MockI2CBusComponent {}

impl MockI2CBusComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for MockI2CBusComponent {
    type StaticInput = &'static mut MaybeUninit<MockI2CBus<'static>>;
    type Output = &'static MockI2CBus<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(MockI2CBus::new())
    }
}

//------------------------------------------------------------------------------
// Attaching one device to the bus.
//------------------------------------------------------------------------------

pub struct MockI2CBusDeviceComponent<D: MockI2CDevice<'static> + 'static> {
    bus: &'static MockI2CBus<'static>,
    device: &'static D,
    address: u8,
}

impl<D: MockI2CDevice<'static> + 'static> MockI2CBusDeviceComponent<D> {
    pub fn new(bus: &'static MockI2CBus<'static>, device: &'static D, address: u8) -> Self {
        Self {
            bus,
            device,
            address,
        }
    }
}

impl<D: MockI2CDevice<'static> + 'static> Component for MockI2CBusDeviceComponent<D> {
    type StaticInput = &'static mut MaybeUninit<I2CBusDevice<'static>>;
    type Output = &'static I2CBusDevice<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let node = s.write(I2CBusDevice::new(self.device, self.address));
        self.bus.add_device(node);
        self.device.set_i2c_client(self.bus);
        node
    }
}
