// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Components for the HS3003 Temperature/Humidity Sensor.
//!
//! Usage
//! -----
//! ```rust
//! let hs3003 = Hs3003Component::new(mux_i2c, mux_alarm, 0x44).finalize(
//!     components::hs3003_component_static!(sam4l::ast::Ast));
//! let temperature = components::temperature::TemperatureComponent::new(board_kernel, hs3003).finalize(());
//! let humidity = components::humidity::HumidityComponent::new(board_kernel, hs3003).finalize(());
//! ```

use capsules_core::virtualizers::virtual_i2c::{I2CDevice, MuxI2C};
use capsules_extra::hs3003::Hs3003;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::i2c;

// Setup static space for the objects.
#[macro_export]
macro_rules! hs3003_component_static {
    ($I:ty $(,)?) => {{
        let i2c_device =
            kernel::static_buf!(capsules_core::virtualizers::virtual_i2c::I2CDevice<$I>);
        let buffer = kernel::static_buf!([u8; 5]);
        let hs3003 = kernel::static_buf!(
            capsules_extra::hs3003::Hs3003<
                'static,
                capsules_core::virtualizers::virtual_i2c::I2CDevice<$I>,
            >
        );

        (i2c_device, buffer, hs3003)
    };};
}

pub struct Hs3003Component<I: 'static + i2c::I2CMaster<'static>> {
    i2c_mux: &'static MuxI2C<'static, I>,
    i2c_address: u8,
}

impl<I: 'static + i2c::I2CMaster<'static>> Hs3003Component<I> {
    pub fn new(i2c: &'static MuxI2C<'static, I>, i2c_address: u8) -> Self {
        Hs3003Component {
            i2c_mux: i2c,
            i2c_address: i2c_address,
        }
    }
}

impl<I: 'static + i2c::I2CMaster<'static>> Component for Hs3003Component<I> {
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static, I>>,
        &'static mut MaybeUninit<[u8; 5]>,
        &'static mut MaybeUninit<Hs3003<'static, I2CDevice<'static, I>>>,
    );
    type Output = &'static Hs3003<'static, I2CDevice<'static, I>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let hs3003_i2c = static_buffer
            .0
            .write(I2CDevice::new(self.i2c_mux, self.i2c_address));
        let buffer = static_buffer.1.write([0; 5]);
        let hs3003 = static_buffer.2.write(Hs3003::new(hs3003_i2c, buffer));

        hs3003_i2c.set_client(hs3003);
        hs3003
    }
}
