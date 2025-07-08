// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Components for the DFRobot Rainfall Sensor.
//! <https://wiki.dfrobot.com/SKU_SEN0575_Gravity_Rainfall_Sensor>
//!
//! Usage
//! -----
//! ```rust
//!     let dfrobot_rainfall =
//!         components::dfrobot_rainfall_sensor::DFRobotRainFallSensorComponent::new(mux_i2c, 0x1D)
//!             .finalize(components::dfrobot_rainfall_sensor_component_static!(
//!                 apollo3::iom::Iom<'static>
//!             ));
//! ```

use capsules_core::virtualizers::virtual_alarm::MuxAlarm;
use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use capsules_core::virtualizers::virtual_i2c::{I2CDevice, MuxI2C};
use capsules_extra::dfrobot_rainfall_sensor::{DFRobotRainFall, BUFFER_SIZE};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::i2c;
use kernel::hil::time;
use kernel::hil::time::Alarm;

// Setup static space for the objects.
#[macro_export]
macro_rules! dfrobot_rainfall_sensor_component_static {
    ($A:ty, $I:ty $(,)?) => {{
        let i2c_device =
            kernel::static_buf!(capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, $I>);
        let i2c_buffer =
            kernel::static_buf!([u8; capsules_extra::dfrobot_rainfall_sensor::BUFFER_SIZE]);
        let dfrobot_rainfall_sensor = kernel::static_buf!(
            capsules_extra::dfrobot_rainfall_sensor::DFRobotRainFall<
                'static,
                VirtualMuxAlarm<'static, $A>,
                capsules_core::virtualizers::virtual_i2c::I2CDevice<$I>,
            >
        );
        let alarm = kernel::static_buf!(
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>
        );

        (i2c_device, i2c_buffer, dfrobot_rainfall_sensor, alarm)
    };};
}

pub type DFRobotRainFallSensorComponentType<A, I> =
    capsules_extra::dfrobot_rainfall_sensor::DFRobotRainFall<'static, A, I>;

pub struct DFRobotRainFallSensorComponent<
    A: 'static + time::Alarm<'static>,
    I: 'static + i2c::I2CMaster<'static>,
> {
    i2c_mux: &'static MuxI2C<'static, I>,
    i2c_address: u8,
    alarm_mux: &'static MuxAlarm<'static, A>,
}

impl<A: 'static + time::Alarm<'static>, I: 'static + i2c::I2CMaster<'static>>
    DFRobotRainFallSensorComponent<A, I>
{
    pub fn new(
        i2c: &'static MuxI2C<'static, I>,
        i2c_address: u8,
        alarm_mux: &'static MuxAlarm<'static, A>,
    ) -> Self {
        DFRobotRainFallSensorComponent {
            i2c_mux: i2c,
            i2c_address,
            alarm_mux,
        }
    }
}

impl<A: 'static + time::Alarm<'static>, I: 'static + i2c::I2CMaster<'static>> Component
    for DFRobotRainFallSensorComponent<A, I>
{
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static, I>>,
        &'static mut MaybeUninit<[u8; BUFFER_SIZE]>,
        &'static mut MaybeUninit<
            DFRobotRainFall<'static, VirtualMuxAlarm<'static, A>, I2CDevice<'static, I>>,
        >,
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
    );
    type Output =
        &'static DFRobotRainFall<'static, VirtualMuxAlarm<'static, A>, I2CDevice<'static, I>>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let dfrobot_rainfall_sensor_i2c = s.0.write(I2CDevice::new(self.i2c_mux, self.i2c_address));
        let i2c_buffer = s.1.write([0; BUFFER_SIZE]);
        let alarm = s.3.write(VirtualMuxAlarm::new(self.alarm_mux));
        alarm.setup();

        let dfrobot_rainfall_sensor = s.2.write(DFRobotRainFall::new(
            dfrobot_rainfall_sensor_i2c,
            i2c_buffer,
            alarm,
        ));

        alarm.set_alarm_client(dfrobot_rainfall_sensor);
        dfrobot_rainfall_sensor_i2c.set_client(dfrobot_rainfall_sensor);
        dfrobot_rainfall_sensor.startup();
        dfrobot_rainfall_sensor
    }
}
