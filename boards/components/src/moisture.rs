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
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;

#[macro_export]
macro_rules! moisture_component_static {
    ($H: ty $(,)?) => {{
        kernel::static_buf!(capsules_extra::moisture::MoistureSensor<'static, $H>)
    };};
}

pub type MoistureComponentType<H> = capsules_extra::moisture::MoistureSensor<'static, H>;

pub struct MoistureComponent<T: 'static + hil::sensors::MoistureDriver<'static>> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    sensor: &'static T,
}

impl<T: 'static + hil::sensors::MoistureDriver<'static>> MoistureComponent<T> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        sensor: &'static T,
    ) -> MoistureComponent<T> {
        MoistureComponent {
            board_kernel,
            driver_num,
            sensor,
        }
    }
}

impl<T: 'static + hil::sensors::MoistureDriver<'static>> Component for MoistureComponent<T> {
    type StaticInput = &'static mut MaybeUninit<MoistureSensor<'static, T>>;
    type Output = &'static MoistureSensor<'static, T>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let moisture = s.write(MoistureSensor::new(
            self.sensor,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));

        hil::sensors::MoistureDriver::set_client(self.sensor, moisture);
        moisture
    }
}
