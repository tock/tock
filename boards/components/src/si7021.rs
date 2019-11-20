//! Components for the SI7021 Temperature/Humidity Sensor.
//!
//! This provides three Components, SI7021Component, which provides
//! access to the SI7021 over I2C, TemperatureComponent, which
//! provides a temperature system call driver, and HumidityComponent,
//! which provides a humidity system call driver. SI7021Component is
//! a parameter to both TemperatureComponent and HumidityComponent.
//!
//! Usage
//! -----
//! ```rust
//! let si7021 = SI7021Component::new(mux_i2c, mux_alarm, 0x40).finalize(
//!     components::si7021_component_helper!(sam4l::ast::Ast));
//! let temp = TemperatureComponent::new(board_kernel, si7021).finalize(());
//! let humidity = HumidityComponent::new(board_kernel, si7021).finalize(());
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules::humidity::HumiditySensor;
use capsules::si7021::SI7021;
use capsules::temperature::TemperatureSensor;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_i2c::{I2CDevice, MuxI2C};
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::hil::time::{self, Alarm};
use kernel::static_init;

use crate::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! si7021_component_helper {
    ($A:ty) => {{
        use capsules::si7021::SI7021;
        static mut BUF1: Option<VirtualMuxAlarm<'static, $A>> = None;
        static mut BUF2: Option<SI7021<'static, VirtualMuxAlarm<'static, $A>>> = None;
        (&mut BUF1, &mut BUF2)
    };};
}

pub struct SI7021Component<A: 'static + time::Alarm<'static>> {
    i2c_mux: &'static MuxI2C<'static>,
    alarm_mux: &'static MuxAlarm<'static, A>,
    i2c_address: u8,
}

impl<A: 'static + time::Alarm<'static>> SI7021Component<A> {
    pub fn new(
        i2c: &'static MuxI2C<'static>,
        alarm: &'static MuxAlarm<'static, A>,
        i2c_address: u8,
    ) -> Self {
        SI7021Component {
            i2c_mux: i2c,
            alarm_mux: alarm,
            i2c_address: i2c_address,
        }
    }
}

static mut I2C_BUF: [u8; 14] = [0; 14];

impl<A: 'static + time::Alarm<'static>> Component for SI7021Component<A> {
    type StaticInput = (
        &'static mut Option<VirtualMuxAlarm<'static, A>>,
        &'static mut Option<SI7021<'static, VirtualMuxAlarm<'static, A>>>,
    );
    type Output = &'static SI7021<'static, VirtualMuxAlarm<'static, A>>;

    unsafe fn finalize(&mut self, static_buffer: Self::StaticInput) -> Self::Output {
        let si7021_i2c = static_init!(I2CDevice, I2CDevice::new(self.i2c_mux, self.i2c_address));
        let si7021_alarm = static_init_half!(
            static_buffer.0,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );
        let si7021 = static_init_half!(
            static_buffer.1,
            SI7021<'static, VirtualMuxAlarm<'static, A>>,
            SI7021::new(si7021_i2c, si7021_alarm, &mut I2C_BUF)
        );

        si7021_i2c.set_client(si7021);
        si7021_alarm.set_client(si7021);
        si7021
    }
}

pub struct TemperatureComponent<A: 'static + time::Alarm<'static>> {
    board_kernel: &'static kernel::Kernel,
    si7021: &'static SI7021<'static, VirtualMuxAlarm<'static, A>>,
}

impl<A: 'static + time::Alarm<'static>> TemperatureComponent<A> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        si: &'static SI7021<'static, VirtualMuxAlarm<'static, A>>,
    ) -> TemperatureComponent<A> {
        TemperatureComponent {
            board_kernel: board_kernel,
            si7021: si,
        }
    }
}

impl<A: 'static + time::Alarm<'static>> Component for TemperatureComponent<A> {
    type StaticInput = ();
    type Output = &'static TemperatureSensor<'static>;

    unsafe fn finalize(&mut self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let temp = static_init!(
            TemperatureSensor<'static>,
            TemperatureSensor::new(self.si7021, self.board_kernel.create_grant(&grant_cap))
        );

        hil::sensors::TemperatureDriver::set_client(self.si7021, temp);
        temp
    }
}

pub struct HumidityComponent<A: 'static + time::Alarm<'static>> {
    board_kernel: &'static kernel::Kernel,
    si7021: &'static SI7021<'static, VirtualMuxAlarm<'static, A>>,
}

impl<A: 'static + time::Alarm<'static>> HumidityComponent<A> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        si: &'static SI7021<'static, VirtualMuxAlarm<'static, A>>,
    ) -> HumidityComponent<A> {
        HumidityComponent {
            board_kernel: board_kernel,
            si7021: si,
        }
    }
}

impl<A: 'static + time::Alarm<'static>> Component for HumidityComponent<A> {
    type StaticInput = ();
    type Output = &'static HumiditySensor<'static>;

    unsafe fn finalize(&mut self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let hum = static_init!(
            HumiditySensor<'static>,
            HumiditySensor::new(self.si7021, self.board_kernel.create_grant(&grant_cap))
        );

        hil::sensors::HumidityDriver::set_client(self.si7021, hum);
        hum
    }
}
