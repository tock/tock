// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for any moisture sensor.
//!
//! Usage
//! -----
//! ```rust
//!     let moisture = components::moisture::MoistureComponent::new(
//!         board_kernel,
//!         capsules_extra::moisture::DRIVER_NUM,
//!         chirp_moisture,
//!     )
//!     .finalize(components::moisture_component_static!(ChirpI2cMoistureType));
//! ```

use capsules_extra::moisture::MoistureSensor;
use core::mem::MaybeUninit;
use kernel::capabilities::MemoryAllocationCapability;
use kernel::component::Component;
use kernel::hil;

#[macro_export]
macro_rules! moisture_component_static {
    ($H: ty $(,)?) => {{
        kernel::static_buf!(capsules_extra::moisture::MoistureSensor<'static, $H>)
    };};
}

pub type MoistureComponentType<H> = capsules_extra::moisture::MoistureSensor<'static, H>;

pub struct MoistureComponent<
    T: 'static + hil::sensors::MoistureDriver<'static>,
    CAP: MemoryAllocationCapability + 'static,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    sensor: &'static T,
    mem_cap: CAP,
}

impl<
        T: 'static + hil::sensors::MoistureDriver<'static>,
        CAP: MemoryAllocationCapability + 'static,
    > MoistureComponent<T, CAP>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        sensor: &'static T,
        mem_cap: CAP,
    ) -> MoistureComponent<T, CAP> {
        MoistureComponent {
            board_kernel,
            driver_num,
            sensor,
            mem_cap,
        }
    }
}

impl<
        T: 'static + hil::sensors::MoistureDriver<'static>,
        CAP: MemoryAllocationCapability + 'static,
    > Component for MoistureComponent<T, CAP>
{
    type StaticInput = &'static mut MaybeUninit<MoistureSensor<'static, T>>;
    type Output = &'static MoistureSensor<'static, T>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let moisture = s.write(MoistureSensor::new(
            self.sensor,
            self.board_kernel
                .create_grant(self.driver_num, &self.mem_cap),
        ));

        hil::sensors::MoistureDriver::set_client(self.sensor, moisture);
        moisture
    }
}
