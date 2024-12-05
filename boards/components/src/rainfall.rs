// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for any rainfall sensor.
//!
//! Usage
//! -----
//! ```rust
//!     let rainfall = components::rainfall::RainFallComponent::new(
//!           board_kernel,
//!           capsules_extra::rainfall::DRIVER_NUM,
//!           dfrobot_rainfall,
//!       )
//!       .finalize(components::rainfall_component_static!(DFRobotRainFallType));
//! ```

use capsules_extra::rainfall::RainFallSensor;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;

#[macro_export]
macro_rules! rainfall_component_static {
    ($H: ty $(,)?) => {{
        kernel::static_buf!(capsules_extra::rainfall::RainFallSensor<'static, $H>)
    };};
}

pub type RainFallComponentType<H> = capsules_extra::rainfall::RainFallSensor<'static, H>;

pub struct RainFallComponent<T: 'static + hil::sensors::RainFallDriver<'static>> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    sensor: &'static T,
}

impl<T: 'static + hil::sensors::RainFallDriver<'static>> RainFallComponent<T> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        sensor: &'static T,
    ) -> RainFallComponent<T> {
        RainFallComponent {
            board_kernel,
            driver_num,
            sensor,
        }
    }
}

impl<T: 'static + hil::sensors::RainFallDriver<'static>> Component for RainFallComponent<T> {
    type StaticInput = &'static mut MaybeUninit<RainFallSensor<'static, T>>;
    type Output = &'static RainFallSensor<'static, T>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let rainfall = s.write(RainFallSensor::new(
            self.sensor,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));

        hil::sensors::RainFallDriver::set_client(self.sensor, rainfall);
        rainfall
    }
}
