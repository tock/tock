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
use kernel::capabilities::MemoryAllocationCapability;
use kernel::component::Component;
use kernel::hil;

#[macro_export]
macro_rules! rainfall_component_static {
    ($H: ty $(,)?) => {{
        kernel::static_buf!(capsules_extra::rainfall::RainFallSensor<'static, $H>)
    };};
}

pub type RainFallComponentType<H> = capsules_extra::rainfall::RainFallSensor<'static, H>;

pub struct RainFallComponent<
    T: 'static + hil::sensors::RainFallDriver<'static>,
    CAP: MemoryAllocationCapability + 'static,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    sensor: &'static T,
    mem_cap: CAP,
}

impl<
        T: 'static + hil::sensors::RainFallDriver<'static>,
        CAP: MemoryAllocationCapability + 'static,
    > RainFallComponent<T, CAP>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        sensor: &'static T,
        mem_cap: CAP,
    ) -> RainFallComponent<T, CAP> {
        RainFallComponent {
            board_kernel,
            driver_num,
            sensor,
            mem_cap,
        }
    }
}

impl<
        T: 'static + hil::sensors::RainFallDriver<'static>,
        CAP: MemoryAllocationCapability + 'static,
    > Component for RainFallComponent<T, CAP>
{
    type StaticInput = &'static mut MaybeUninit<RainFallSensor<'static, T>>;
    type Output = &'static RainFallSensor<'static, T>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let rainfall = s.write(RainFallSensor::new(
            self.sensor,
            self.board_kernel
                .create_grant(self.driver_num, &self.mem_cap),
        ));

        hil::sensors::RainFallDriver::set_client(self.sensor, rainfall);
        rainfall
    }
}
