// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Component for any barometer sensor.
//!
//! Usage
//! -----
//! ```rust
//! let pressure = PressureComponent::new(board_kernel, nrf52::pressure::PRES)
//!     .finalize(components::pressure_component_static!());
//! ```

use capsules_extra::pressure::PressureSensor;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;

#[macro_export]
macro_rules! pressure_component_static {
    ($T:ty $(,)?) => {{
        kernel::static_buf!(capsules_extra::pressure::PressureSensor<'static, $T>)
    };};
}

pub struct PressureComponent<T: 'static + hil::sensors::PressureDriver<'static>> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    pressure_sensor: &'static T,
}

impl<T: 'static + hil::sensors::PressureDriver<'static>> PressureComponent<T> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        pressure_sensor: &'static T,
    ) -> PressureComponent<T> {
        PressureComponent {
            board_kernel,
            driver_num,
            pressure_sensor,
        }
    }
}

impl<T: 'static + hil::sensors::PressureDriver<'static>> Component for PressureComponent<T> {
    type StaticInput = &'static mut MaybeUninit<PressureSensor<'static, T>>;
    type Output = &'static PressureSensor<'static, T>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let pressure = static_buffer.write(PressureSensor::new(
            self.pressure_sensor,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));

        hil::sensors::PressureDriver::set_client(self.pressure_sensor, pressure);

        pressure
    }
}
