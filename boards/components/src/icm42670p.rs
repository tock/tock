// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Component for the ICM-42670-P 6-axis IMU.
//!
//! I2C Interface
//!
//! Usage
//! -----
//!
//! ```rust
//! let icm42670p = components::icm42670p::Icm42670pComponent::new(sensors_i2c_bus, capsules_extra::icm42670p::BASE_ADDR, mux_alarm).finalize(
//!         components::icm42670p_component_static!(nrf52::rtc::Rtc<'static>),
//!     );
//! ```

use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules_core::virtualizers::virtual_i2c::{I2CDevice, MuxI2C};
use capsules_extra::icm42670p::Icm42670p;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::i2c;
use kernel::hil::time::Alarm;

// Setup static space for the objects.
#[macro_export]
macro_rules! icm42670p_component_static {
    ($A:ty, $I:ty $(,)?) => {{
        let buffer = kernel::static_buf!([u8; 6]);
        let i2c_device =
            kernel::static_buf!(capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, $I>);
        let icm42670p_alarm = kernel::static_buf!(
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>
        );
        let icm42670p = kernel::static_buf!(
            capsules_extra::icm42670p::Icm42670p<
                'static,
                capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>,
                capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, $I>,
            >
        );

        (icm42670p_alarm, i2c_device, icm42670p, buffer)
    }};
}

pub type Icm42670pComponentType<A, I> = capsules_extra::icm42670p::Icm42670p<'static, A, I>;

pub struct Icm42670pComponent<A: 'static + Alarm<'static>, I: 'static + i2c::I2CMaster<'static>> {
    i2c_mux: &'static MuxI2C<'static, I>,
    i2c_address: u8,
    alarm_mux: &'static MuxAlarm<'static, A>,
}

impl<A: 'static + Alarm<'static>, I: 'static + i2c::I2CMaster<'static>> Icm42670pComponent<A, I> {
    pub fn new(
        i2c_mux: &'static MuxI2C<'static, I>,
        i2c_address: u8,
        alarm_mux: &'static MuxAlarm<'static, A>,
    ) -> Icm42670pComponent<A, I> {
        Icm42670pComponent {
            i2c_mux,
            i2c_address,
            alarm_mux,
        }
    }
}

impl<A: 'static + Alarm<'static>, I: 'static + i2c::I2CMaster<'static>> Component
    for Icm42670pComponent<A, I>
{
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<I2CDevice<'static, I>>,
        &'static mut MaybeUninit<
            Icm42670p<'static, VirtualMuxAlarm<'static, A>, I2CDevice<'static, I>>,
        >,
        &'static mut MaybeUninit<[u8; 6]>,
    );
    type Output = &'static Icm42670p<'static, VirtualMuxAlarm<'static, A>, I2CDevice<'static, I>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let icm42670p_i2c = static_buffer
            .1
            .write(I2CDevice::new(self.i2c_mux, self.i2c_address));

        let buffer = static_buffer.3.write([0; 6]);

        let icm42670p_alarm = static_buffer.0.write(VirtualMuxAlarm::new(self.alarm_mux));
        icm42670p_alarm.setup();

        let icm42670p =
            static_buffer
                .2
                .write(Icm42670p::new(icm42670p_i2c, buffer, icm42670p_alarm));
        icm42670p_i2c.set_client(icm42670p);
        icm42670p_alarm.set_alarm_client(icm42670p);

        icm42670p
    }
}
