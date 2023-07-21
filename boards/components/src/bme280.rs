// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Components for the BME280 Humidity, Pressure and Temperature Sensor.
//!
//! Usage
//! -----
//! ```rust
//!     let bme280 =
//!         Bme280Component::new(mux_i2c, 0x77).finalize(components::bme280_component_static!());
//!     let temperature = components::temperature::TemperatureComponent::new(
//!         board_kernel,
//!         capsules_extra::temperature::DRIVER_NUM,
//!         bme280,
//!     )
//!     .finalize(components::temperature_component_static!());
//!     let humidity = components::humidity::HumidityComponent::new(
//!         board_kernel,
//!         capsules_extra::humidity::DRIVER_NUM,
//!         bme280,
//!     )
//!     .finalize(components::humidity_component_static!());
//! ```

use capsules_core::virtualizers::virtual_i2c::{I2CDevice, MuxI2C};
use capsules_extra::bme280::Bme280;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::i2c;

// Setup static space for the objects.
#[macro_export]
macro_rules! bme280_component_static {
    ($I:ty $(,)?) => {{
        let i2c_device =
            kernel::static_buf!(capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, $I>);
        let i2c_buffer = kernel::static_buf!([u8; 26]);
        let bme280 = kernel::static_buf!(capsules_extra::bme280::Bme280<'static>);

        (i2c_device, i2c_buffer, bme280)
    };};
}

pub struct Bme280Component<I: 'static + i2c::I2CMaster<'static>> {
    i2c_mux: &'static MuxI2C<'static, I>,
    i2c_address: u8,
}

impl<I: 'static + i2c::I2CMaster<'static>> Bme280Component<I> {
    pub fn new(i2c: &'static MuxI2C<'static, I>, i2c_address: u8) -> Self {
        Bme280Component {
            i2c_mux: i2c,
            i2c_address: i2c_address,
        }
    }
}

impl<I: 'static + i2c::I2CMaster<'static>> Component for Bme280Component<I> {
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static, I>>,
        &'static mut MaybeUninit<[u8; 26]>,
        &'static mut MaybeUninit<Bme280<'static>>,
    );
    type Output = &'static Bme280<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let bme280_i2c = s.0.write(I2CDevice::new(self.i2c_mux, self.i2c_address));
        let i2c_buffer = s.1.write([0; 26]);

        let bme280 = s.2.write(Bme280::new(bme280_i2c, i2c_buffer));

        bme280_i2c.set_client(bme280);
        bme280.startup();
        bme280
    }
}
