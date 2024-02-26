// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use capsules_core::virtualizers::virtual_i2c::{I2CDevice, MuxI2C};
use capsules_extra::nmea_i2c::{I2cNmea, I2C_BUFFER_LEN};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::i2c;

// Setup static space for the objects.
#[macro_export]
macro_rules! nmea_i2c_component_static {
    ($I:ty $(,)?) => {{
        let i2c_device =
            kernel::static_buf!(capsules_core::virtualizers::virtual_i2c::I2CDevice<$I>);
        let buffer = kernel::static_buf!([u8; capsules_extra::nmea_i2c::I2C_BUFFER_LEN]);
        let nmea_i2c = kernel::static_buf!(
            capsules_extra::nmea_i2c::I2cNmea<
                capsules_core::virtualizers::virtual_i2c::I2CDevice<$I>,
            >
        );

        (i2c_device, buffer, nmea_i2c)
    };};
}

pub struct I2cNmeaComponent<I: 'static + i2c::I2CMaster<'static>> {
    i2c_mux: &'static MuxI2C<'static, I>,
    i2c_address: u8,
}

impl<I: 'static + i2c::I2CMaster<'static>> I2cNmeaComponent<I> {
    pub fn new(i2c: &'static MuxI2C<'static, I>, i2c_address: u8) -> Self {
        I2cNmeaComponent {
            i2c_mux: i2c,
            i2c_address,
        }
    }
}

impl<I: 'static + i2c::I2CMaster<'static>> Component for I2cNmeaComponent<I> {
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static, I>>,
        &'static mut MaybeUninit<[u8; I2C_BUFFER_LEN]>,
        &'static mut MaybeUninit<I2cNmea<'static, I2CDevice<'static, I>>>,
    );
    type Output = &'static I2cNmea<'static, I2CDevice<'static, I>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let i2c_device = static_buffer
            .0
            .write(I2CDevice::new(self.i2c_mux, self.i2c_address));
        let buffer = static_buffer.1.write([0; I2C_BUFFER_LEN]);
        let nmea_i2c = static_buffer.2.write(I2cNmea::new(i2c_device, buffer));

        i2c_device.set_client(nmea_i2c);
        nmea_i2c
    }
}
