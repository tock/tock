// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Components for the MLX90614 IR Temperature Sensor.
//!
//! Usage
//! -----
//! ```rust
//! let mlx90614 = components::mlx90614::Mlx90614I2CComponent::new(mux_i2c, i2c_addr,
//! board_kernel)
//!    .finalize(components::mlx90614_component_static!());
//!
//! let temp = static_init!(
//!        capsules_extra::temperature::TemperatureSensor<'static>,
//!        capsules_extra::temperature::TemperatureSensor::new(mlx90614,
//!                                                 grant_temperature));
//! kernel::hil::sensors::TemperatureDriver::set_client(mlx90614, temp);
//! ```

use capsules_core::virtualizers::virtual_i2c::{MuxI2C, SMBusDevice};
use capsules_extra::mlx90614::Mlx90614SMBus;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::i2c::{self, NoSMBus};

// Setup static space for the objects.
#[macro_export]
macro_rules! mlx90614_component_static {
    () => {{
        let i2c_device = kernel::static_buf!(capsules_core::virtualizers::virtual_i2c::SMBusDevice);
        let buffer = kernel::static_buf!([u8; 14]);
        let mlx90614 = kernel::static_buf!(capsules_extra::mlx90614::Mlx90614SMBus<'static>);

        (i2c_device, buffer, mlx90614)
    };};
}

pub struct Mlx90614SMBusComponent<
    I: 'static + i2c::I2CMaster<'static>,
    S: 'static + i2c::SMBusMaster<'static> = NoSMBus,
> {
    i2c_mux: &'static MuxI2C<'static, I, S>,
    i2c_address: u8,
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl<I: 'static + i2c::I2CMaster<'static>, S: 'static + i2c::SMBusMaster<'static>>
    Mlx90614SMBusComponent<I, S>
{
    pub fn new(
        i2c: &'static MuxI2C<'static, I, S>,
        i2c_address: u8,
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
    ) -> Self {
        Mlx90614SMBusComponent {
            i2c_mux: i2c,
            i2c_address: i2c_address,
            board_kernel,
            driver_num,
        }
    }
}

impl<I: 'static + i2c::I2CMaster<'static>, S: 'static + i2c::SMBusMaster<'static>> Component
    for Mlx90614SMBusComponent<I, S>
{
    type StaticInput = (
        &'static mut MaybeUninit<SMBusDevice<'static, I, S>>,
        &'static mut MaybeUninit<[u8; 14]>,
        &'static mut MaybeUninit<Mlx90614SMBus<'static>>,
    );
    type Output = &'static Mlx90614SMBus<'static>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let mlx90614_smbus = static_buffer
            .0
            .write(SMBusDevice::new(self.i2c_mux, self.i2c_address));
        let buffer = static_buffer.1.write([0; 14]);
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let mlx90614 = static_buffer.2.write(Mlx90614SMBus::new(
            mlx90614_smbus,
            buffer,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));

        mlx90614_smbus.set_client(mlx90614);
        mlx90614
    }
}
