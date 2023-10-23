// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Component for the BMI270 Inertial Sensor.
//!
//! Usage
//! -----
//! ```rust
//! let bmi270 = BMI270Component::new(mux_i2c, 0x68).finalize(
//!     components::bmi270_component_static!(nrf5240::i2c::TWI));
//! let ninedof = components::ninedof::NineDofComponent::new(board_kernel)
//!     .finalize(components::ninedof_component_static!(bmi270));
//! ```

use capsules_core::virtualizers::virtual_i2c::{I2CDevice, MuxI2C};
use capsules_extra::bmi270::BMI270;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::i2c;

// Setup static space for the objects.
#[macro_export]
macro_rules! bmi270_component_static {
    ($I:ty $(,)?) => {{
        let i2c_device =
            kernel::static_buf!(capsules_core::virtualizers::virtual_i2c::I2CDevice<$I>);
        let buffer = kernel::static_buf!([u8; 8193]);
        let bmi270 = kernel::static_buf!(
            capsules_extra::bmi270::BMI270<
                'static,
                capsules_core::virtualizers::virtual_i2c::I2CDevice<$I>,
            >
        );

        (i2c_device, buffer, bmi270)
    };};
}

pub struct BMI270Component<I: 'static + i2c::I2CMaster<'static>> {
    i2c_mux: &'static MuxI2C<'static, I>,
    i2c_address: u8,
}

impl<I: 'static + i2c::I2CMaster<'static>> BMI270Component<I> {
    pub fn new(i2c: &'static MuxI2C<'static, I>, i2c_address: u8) -> Self {
        BMI270Component {
            i2c_mux: i2c,
            i2c_address: i2c_address,
        }
    }
}

impl<I: 'static + i2c::I2CMaster<'static>> Component for BMI270Component<I> {
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static, I>>,
        &'static mut MaybeUninit<[u8; 8193]>,
        &'static mut MaybeUninit<BMI270<'static, I2CDevice<'static, I>>>,
    );
    type Output = &'static BMI270<'static, I2CDevice<'static, I>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let bmi270_i2c = static_buffer
            .0
            .write(I2CDevice::new(self.i2c_mux, self.i2c_address));
        let buffer = static_buffer.1.write([0; 8193]);
        let bmi270 = static_buffer.2.write(BMI270::new(bmi270_i2c, buffer));

        bmi270_i2c.set_client(bmi270);
        bmi270
    }
}
