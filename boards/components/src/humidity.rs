// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for any humidity sensor.
//!
//! Usage
//! -----
//! ```rust
//! let humidity = HumidityComponent::new(board_kernel, nrf52::humidity::TEMP)
//!     .finalize(components::humidity_component_static!());
//! ```

use capsules_extra::humidity::HumiditySensor;
use core::mem::MaybeUninit;
use kernel::capabilities::MemoryAllocationCapability;
use kernel::component::Component;
use kernel::hil;

#[macro_export]
macro_rules! humidity_component_static {
    ($H: ty $(,)?) => {{
        kernel::static_buf!(capsules_extra::humidity::HumiditySensor<'static, $H>)
    };};
}

pub type HumidityComponentType<H> = capsules_extra::humidity::HumiditySensor<'static, H>;

pub struct HumidityComponent<
    T: 'static + hil::sensors::HumidityDriver<'static>,
    CAP: MemoryAllocationCapability + 'static,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    sensor: &'static T,
    mem_cap: CAP,
}

impl<T: 'static + hil::sensors::HumidityDriver<'static>, CAP: MemoryAllocationCapability + 'static>
    HumidityComponent<T, CAP>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        sensor: &'static T,
        mem_cap: CAP,
    ) -> HumidityComponent<T, CAP> {
        HumidityComponent {
            board_kernel,
            driver_num,
            sensor,
            mem_cap,
        }
    }
}

impl<T: 'static + hil::sensors::HumidityDriver<'static>, CAP: MemoryAllocationCapability + 'static>
    Component for HumidityComponent<T, CAP>
{
    type StaticInput = &'static mut MaybeUninit<HumiditySensor<'static, T>>;
    type Output = &'static HumiditySensor<'static, T>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let humidity = s.write(HumiditySensor::new(
            self.sensor,
            self.board_kernel
                .create_grant(self.driver_num, &self.mem_cap),
        ));

        hil::sensors::HumidityDriver::set_client(self.sensor, humidity);
        humidity
    }
}
