// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Component for the LSM9DS1 Sensor
//!
//! I2C Interface
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//! let lsm9ds1 = components::lsm9ds1::Lsm9ds1I2CComponent::new(
//!     mux_i2c,
//!     None,
//!     None,
//!     board_kernel,
//!     capsules_extra::lsm9ds1::DRIVER_NUM,
//!     create_capability!(capabilities::MemoryAllocationCapability),
//! )
//! .finalize(components::lsm9ds1_i2c_component_static!(nrf52840::i2c::TWI));
//!
//! let _ = lsm9ds1.configure(
//!     capsules_extra::lsm9ds1::Lsm9ds1GyroDataRate::Lsm9ds1GyroRate119Hz,
//!     capsules_extra::lsm9ds1::Lsm9ds1GyroRange::Lsm9ds1GyroRange245Dps,
//!     capsules_extra::lsm9ds1::Lsm9ds1AccelDataRate::Lsm9ds1AccelRate119Hz,
//!     capsules_extra::lsm9ds1::Lsm9ds1AccelRange::Lsm9ds1AccelRange2G,
//!     capsules_extra::lsm9ds1::Lsm9ds1MagDataRate::Lsm9ds1MagRate20Hz,
//!     capsules_extra::lsm9ds1::Lsm9ds1MagRange::Lsm9ds1MagRange4Gauss,
//! );
//! ```

use capsules_core::virtualizers::virtual_i2c::{I2CDevice, MuxI2C};
use capsules_extra::lsm9ds1;
use capsules_extra::lsm9ds1::Lsm9ds1I2C;
use core::mem::MaybeUninit;
use kernel::capabilities::MemoryAllocationCapability;
use kernel::component::Component;
use kernel::hil::i2c;
use kernel::hil::i2c::I2CDevice as _;

// Setup static space for the objects.
#[macro_export]
macro_rules! lsm9ds1_i2c_component_static {
    ($I:ty $(,)?) => {{
        let buffer = kernel::static_buf!([u8; 8]);
        let accel_gyro_i2c =
            kernel::static_buf!(capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, $I>);
        let magnetometer_i2c =
            kernel::static_buf!(capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, $I>);
        let lsm9ds1 = kernel::static_buf!(
            capsules_extra::lsm9ds1::Lsm9ds1I2C<
                'static,
                capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, $I>,
            >
        );

        (accel_gyro_i2c, magnetometer_i2c, buffer, lsm9ds1)
    }};
}

pub struct Lsm9ds1I2CComponent<
    I: 'static + i2c::I2CMaster<'static>,
    CAP: MemoryAllocationCapability + 'static,
> {
    i2c_mux: &'static MuxI2C<'static, I>,
    accel_gyro_i2c_address: u8,
    magnetometer_i2c_address: u8,
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    mem_cap: CAP,
}

impl<I: 'static + i2c::I2CMaster<'static>, CAP: MemoryAllocationCapability + 'static>
    Lsm9ds1I2CComponent<I, CAP>
{
    pub fn new(
        i2c_mux: &'static MuxI2C<'static, I>,
        accel_gyro_i2c_address: Option<u8>,
        magnetometer_i2c_address: Option<u8>,
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        mem_cap: CAP,
    ) -> Lsm9ds1I2CComponent<I, CAP> {
        Lsm9ds1I2CComponent {
            i2c_mux,
            accel_gyro_i2c_address: accel_gyro_i2c_address
                .unwrap_or(lsm9ds1::ACCEL_GYRO_BASE_ADDRESS),
            magnetometer_i2c_address: magnetometer_i2c_address
                .unwrap_or(lsm9ds1::MAGNETOMETER_BASE_ADDRESS),
            board_kernel,
            driver_num,
            mem_cap,
        }
    }
}

impl<I: 'static + i2c::I2CMaster<'static>, CAP: MemoryAllocationCapability + 'static> Component
    for Lsm9ds1I2CComponent<I, CAP>
{
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static, I>>,
        &'static mut MaybeUninit<I2CDevice<'static, I>>,
        &'static mut MaybeUninit<[u8; 8]>,
        &'static mut MaybeUninit<Lsm9ds1I2C<'static, I2CDevice<'static, I>>>,
    );
    type Output = &'static Lsm9ds1I2C<'static, I2CDevice<'static, I>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let buffer = static_buffer.2.write([0; 8]);

        let accel_gyro_i2c = static_buffer
            .0
            .write(I2CDevice::new(self.i2c_mux, self.accel_gyro_i2c_address));
        let magnetometer_i2c = static_buffer
            .1
            .write(I2CDevice::new(self.i2c_mux, self.magnetometer_i2c_address));

        let grant = self
            .board_kernel
            .create_grant(self.driver_num, &self.mem_cap);
        let lsm9ds1 = static_buffer.3.write(Lsm9ds1I2C::new(
            accel_gyro_i2c,
            magnetometer_i2c,
            buffer,
            grant,
        ));
        accel_gyro_i2c.set_client(lsm9ds1);
        magnetometer_i2c.set_client(lsm9ds1);

        lsm9ds1
    }
}
