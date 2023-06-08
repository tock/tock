// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Components for the HTS221 Temperature/Humidity Sensor.
//!
//! Usage
//! -----
//! ```rust
//! let hts221 = Hts221Component::new(mux_i2c, mux_alarm, 0x5f).finalize(
//!     components::hts221_component_static!(sam4l::ast::Ast));
//! let temperature = components::temperature::TemperatureComponent::new(board_kernel, hts221).finalize(());
//! let humidity = components::humidity::HumidityComponent::new(board_kernel, hts221).finalize(());
//! ```

use capsules_core::virtualizers::virtual_i2c::{I2CDevice, MuxI2C};
use capsules_extra::hts221::Hts221;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::i2c;

// Setup static space for the objects.
#[macro_export]
macro_rules! hts221_component_static {
    ($I:ty $(,)?) => {{
        let i2c_device =
            kernel::static_buf!(capsules_core::virtualizers::virtual_i2c::I2CDevice<$I>);
        let buffer = kernel::static_buf!([u8; 17]);
        let hts221 = kernel::static_buf!(
            capsules_extra::hts221::Hts221<
                'static,
                capsules_core::virtualizers::virtual_i2c::I2CDevice<$I>,
            >
        );

        (i2c_device, buffer, hts221)
    };};
}

pub struct Hts221Component<I: 'static + i2c::I2CMaster<'static>> {
    i2c_mux: &'static MuxI2C<'static, I>,
    i2c_address: u8,
}

impl<I: 'static + i2c::I2CMaster<'static>> Hts221Component<I> {
    pub fn new(i2c: &'static MuxI2C<'static, I>, i2c_address: u8) -> Self {
        Hts221Component {
            i2c_mux: i2c,
            i2c_address: i2c_address,
        }
    }
}

impl<I: 'static + i2c::I2CMaster<'static>> Component for Hts221Component<I> {
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static, I>>,
        &'static mut MaybeUninit<[u8; 17]>,
        &'static mut MaybeUninit<Hts221<'static, I2CDevice<'static, I>>>,
    );
    type Output = &'static Hts221<'static, I2CDevice<'static, I>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let hts221_i2c = static_buffer
            .0
            .write(I2CDevice::new(self.i2c_mux, self.i2c_address));
        let buffer = static_buffer.1.write([0; 17]);
        let hts221 = static_buffer.2.write(Hts221::new(hts221_i2c, buffer));

        hts221_i2c.set_client(hts221);
        hts221
    }
}
