// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Components for the TMP431 Temperature Sensor.
//!
//! Usage
//! -----
//! ```rust
//! let tmp431 = components::tmp431::Tmp431SMBusComponent::new(
//!     thermometer, alarm_mux, board_kernel, driver_num
//! )
//! .finalize(components::tmp431_component_static!(Thermometer, Alarm));
//!
//! let temp = components::temperature::TemperatureComponent::new(
//!    board_kernel,
//!    capsules_extra::temperature::DRIVER_NUM,
//!    tmp431,
//! )
//! .finalize(components::temperature_component_static!(
//!    capsules_extra::tmp431::Tmp431SMBus<Thermometer, VirtualMuxAlarm<Alarm>
//! ));
//! ```

use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules_extra::tmp431::Tmp431SMBus;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::i2c::{I2CClient, I2CHwMasterClient, SMBusDevice};
use kernel::hil::time::Alarm;

// Setup static space for the objects.
#[macro_export]
macro_rules! tmp431_component_static {
    ($A:ty, $Thermometer:ty $(,)?) => {{
        let alarm = kernel::static_buf!(VirtualMuxAlarm<'static, $A>);
        let i2c_buf = kernel::static_buf!([u8; 2]);
        let tmp431 = kernel::static_buf!(
            capsules_extra::tmp431::Tmp431SMBus<
                'static,
                $Thermometer,
                VirtualMuxAlarm<'static, $A>,
            >
        );

        (alarm, i2c_buf, tmp431)
    };};
}

pub trait SetThermometerClient<'a> {
    fn set_client(&self, thermometer_client: &'a dyn I2CClient);
}

pub struct Tmp431SMBusComponent<
    Thermometer: SMBusDevice + I2CHwMasterClient + SetThermometerClient<'static> + 'static,
    A: Alarm<'static> + 'static,
> {
    thermometer: &'static Thermometer,
    mux_alarm: &'static MuxAlarm<'static, A>,
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl<
        Thermometer: SMBusDevice + I2CHwMasterClient + SetThermometerClient<'static> + 'static,
        A: Alarm<'static> + 'static,
    > Tmp431SMBusComponent<Thermometer, A>
{
    pub fn new(
        thermometer: &'static Thermometer,
        mux_alarm: &'static MuxAlarm<'static, A>,
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
    ) -> Self {
        Tmp431SMBusComponent {
            thermometer,
            mux_alarm,
            board_kernel,
            driver_num,
        }
    }
}

impl<
        Thermometer: SMBusDevice + I2CHwMasterClient + SetThermometerClient<'static> + 'static,
        A: Alarm<'static> + 'static,
    > Component for Tmp431SMBusComponent<Thermometer, A>
{
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<[u8; 2]>,
        &'static mut MaybeUninit<Tmp431SMBus<'static, Thermometer, VirtualMuxAlarm<'static, A>>>,
    );
    type Output = &'static Tmp431SMBus<'static, Thermometer, VirtualMuxAlarm<'static, A>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let (alarm_buf, i2c_buf, tmp431_buf) = static_buffer;

        let alarm = alarm_buf.write(VirtualMuxAlarm::new(self.mux_alarm));
        alarm.setup();

        let i2c_buf = i2c_buf.write([0; 2]);

        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let tmp431 = tmp431_buf.write(Tmp431SMBus::new(self.thermometer, alarm, grant, i2c_buf));
        tmp431.initialize();
        self.thermometer.set_client(tmp431);

        tmp431
    }
}
