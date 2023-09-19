// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for any proximity sensor.
//!
//! Usage
//! -----
//! ```rust
//! let proximity = ProximityComponent::new(apds9960, board_kernel, capsules_extra::proximity::DRIVER_NUM)
//!     .finalize(components::proximity_component_static!());
//! ```

use capsules_extra::proximity::ProximitySensor;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;

#[macro_export]
macro_rules! proximity_component_static {
    () => {{
        kernel::static_buf!(capsules_extra::proximity::ProximitySensor<'static>)
    };};
}

pub struct ProximityComponent<P: hil::sensors::ProximityDriver<'static> + 'static> {
    sensor: &'static P,
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl<P: hil::sensors::ProximityDriver<'static>> ProximityComponent<P> {
    pub fn new(
        sensor: &'static P,
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
    ) -> ProximityComponent<P> {
        ProximityComponent {
            sensor,
            board_kernel,
            driver_num,
        }
    }
}

impl<P: hil::sensors::ProximityDriver<'static>> Component for ProximityComponent<P> {
    type StaticInput = &'static mut MaybeUninit<ProximitySensor<'static>>;
    type Output = &'static ProximitySensor<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let proximity = s.write(ProximitySensor::new(self.sensor, grant));

        hil::sensors::ProximityDriver::set_client(self.sensor, proximity);
        proximity
    }
}
