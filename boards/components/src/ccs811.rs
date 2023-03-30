// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Components for the BME280 Humidity, Pressure and Temperature Sensor.
//!
//! Usage
//! -----
//! ```rust
//!     let ccs811 =
//!         Ccs811Component::new(mux_i2c, 0x77).finalize(components::ccs811_component_static!());
//!     let temperature = components::temperature::TemperatureComponent::new(
//!         board_kernel,
//!         capsules_extra::temperature::DRIVER_NUM,
//!         ccs811,
//!     )
//!     .finalize(());
//!     let humidity = components::humidity::HumidityComponent::new(
//!         board_kernel,
//!         capsules_extra::humidity::DRIVER_NUM,
//!         ccs811,
//!     )
//!     .finalize(());
//! ```

use capsules_core::virtualizers::virtual_i2c::{I2CDevice, MuxI2C};
use capsules_extra::ccs811::Ccs811;
use core::mem::MaybeUninit;
use kernel::component::Component;

// Setup static space for the objects.
#[macro_export]
macro_rules! ccs811_component_static {
    () => {{
        let i2c_device = kernel::static_buf!(capsules_core::virtualizers::virtual_i2c::I2CDevice);
        let buffer = kernel::static_buf!([u8; 6]);
        let ccs811 = kernel::static_buf!(capsules_extra::ccs811::Ccs811<'static>);

        (i2c_device, buffer, ccs811)
    };};
}

pub struct Ccs811Component {
    i2c_mux: &'static MuxI2C<'static>,
    i2c_address: u8,
}

impl Ccs811Component {
    pub fn new(i2c: &'static MuxI2C<'static>, i2c_address: u8) -> Self {
        Ccs811Component {
            i2c_mux: i2c,
            i2c_address,
        }
    }
}

impl Component for Ccs811Component {
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static>>,
        &'static mut MaybeUninit<[u8; 6]>,
        &'static mut MaybeUninit<Ccs811<'static>>,
    );
    type Output = &'static Ccs811<'static>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let ccs811_i2c = static_buffer
            .0
            .write(I2CDevice::new(self.i2c_mux, self.i2c_address));
        let buffer = static_buffer.1.write([0; 6]);
        let ccs811 = static_buffer.2.write(Ccs811::new(ccs811_i2c, buffer));
        kernel::deferred_call::DeferredCallClient::register(ccs811);

        ccs811_i2c.set_client(ccs811);
        ccs811.startup();
        ccs811
    }
}
