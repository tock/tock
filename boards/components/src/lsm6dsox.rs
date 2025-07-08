// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for the LSM6DSOXTR Sensor
//!
//! Usage
//! ------
//!
//! ```rust
//! let lsm6dsoxtr = components::lsm6dsox::Lsm6dsoxtrI2CComponent::new(
//!     mux_i2c,
//!     capsules_extra::lsm6dsoxtr::ACCELEROMETER_BASE_ADDRESS,
//!     board_kernel,
//!     capsules_extra::lsm6dsoxtr::DRIVER_NUM,
//! )
//! .finalize(components::lsm6ds_i2c_component_static!());
//!
//! let _ = lsm6dsoxtr
//!          .configure(
//!              capsules_extra::lsm6ds_definitions::LSM6DSOXGyroDataRate::LSM6DSOX_GYRO_RATE_12_5_HZ,
//!              capsules_extra::lsm6ds_definitions::LSM6DSOXAccelDataRate::LSM6DSOX_ACCEL_RATE_12_5_HZ,
//!              capsules_extra::lsm6ds_definitions::LSM6DSOXAccelRange::LSM6DSOX_ACCEL_RANGE_2_G,
//!              capsules_extra::lsm6ds_definitions::LSM6DSOXTRGyroRange::LSM6DSOX_GYRO_RANGE_250_DPS,
//!              true,
//!          )
//!          .map_err(|e| panic!("ERROR Failed LSM6DSOXTR sensor configuration ({:?})", e));
//! ```
//! Author: Cristiana Andrei <cristiana.andrei@stud.fils.upb.ro>

use capsules_core::virtualizers::virtual_i2c::{I2CDevice, MuxI2C};
use capsules_extra::lsm6dsoxtr::Lsm6dsoxtrI2C;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::i2c;

// Setup static space for the objects.
#[macro_export]
macro_rules! lsm6ds_i2c_component_static {
    ($I:ty $(,)?) => {{
        let buffer = kernel::static_buf!([u8; 8]);
        let i2c_device =
            kernel::static_buf!(capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, $I>);
        let lsm6dsoxtr = kernel::static_buf!(
            capsules_extra::lsm6dsoxtr::Lsm6dsoxtrI2C<
                'static,
                capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, $I>,
            >
        );

        (i2c_device, buffer, lsm6dsoxtr)
    };};
}

pub struct Lsm6dsoxtrI2CComponent<I: 'static + i2c::I2CMaster<'static>> {
    i2c_mux: &'static MuxI2C<'static, I>,
    i2c_address: u8,
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl<I: 'static + i2c::I2CMaster<'static>> Lsm6dsoxtrI2CComponent<I> {
    pub fn new(
        i2c_mux: &'static MuxI2C<'static, I>,
        i2c_address: u8,
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
    ) -> Lsm6dsoxtrI2CComponent<I> {
        Lsm6dsoxtrI2CComponent {
            i2c_mux,
            i2c_address,
            board_kernel,
            driver_num,
        }
    }
}

impl<I: 'static + i2c::I2CMaster<'static>> Component for Lsm6dsoxtrI2CComponent<I> {
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static, I>>,
        &'static mut MaybeUninit<[u8; 8]>,
        &'static mut MaybeUninit<Lsm6dsoxtrI2C<'static, I2CDevice<'static, I>>>,
    );
    type Output = &'static Lsm6dsoxtrI2C<'static, I2CDevice<'static, I>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let lsm6dsox_i2c = static_buffer
            .0
            .write(I2CDevice::new(self.i2c_mux, self.i2c_address));
        let buffer = static_buffer.1.write([0; 8]);

        let lsm6dsox = static_buffer
            .2
            .write(Lsm6dsoxtrI2C::new(lsm6dsox_i2c, buffer, grant));
        lsm6dsox_i2c.set_client(lsm6dsox);

        lsm6dsox
    }
}
