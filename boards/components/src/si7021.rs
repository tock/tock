//! Components for the SI7021 Temperature/Humidity Sensor.
//!
//! This provides the SI7021Component which provides access to the SI7021 over
//! I2C.
//!
//! Usage
//! -----
//! ```rust
//! let si7021 = SI7021Component::new(mux_i2c, mux_alarm, 0x40).finalize(
//!     components::si7021_component_static!(sam4l::ast::Ast));
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

use core::mem::MaybeUninit;

use core_capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core_capsules::virtual_i2c::{I2CDevice, MuxI2C};
use extra_capsules::si7021::SI7021;
use kernel::component::Component;
use kernel::hil::time::{self, Alarm};

// Setup static space for the objects.
#[macro_export]
macro_rules! si7021_component_static {
    ($A:ty $(,)?) => {{
        let alarm = kernel::static_buf!(core_capsules::virtual_alarm::VirtualMuxAlarm<'static, $A>);
        let i2c_device = kernel::static_buf!(core_capsules::virtual_i2c::I2CDevice<'static>);
        let si7021 = kernel::static_buf!(
            extra_capsules::si7021::SI7021<
                'static,
                core_capsules::virtual_alarm::VirtualMuxAlarm<'static, $A>,
            >
        );
        let buffer = kernel::static_buf!([u8; 14]);

        (alarm, i2c_device, si7021, buffer)
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

impl<A: 'static + time::Alarm<'static>> Component for SI7021Component<A> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<I2CDevice<'static>>,
        &'static mut MaybeUninit<SI7021<'static, VirtualMuxAlarm<'static, A>>>,
        &'static mut MaybeUninit<[u8; 14]>,
    );
    type Output = &'static SI7021<'static, VirtualMuxAlarm<'static, A>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let si7021_i2c = static_buffer
            .1
            .write(I2CDevice::new(self.i2c_mux, self.i2c_address));

        let si7021_alarm = static_buffer.0.write(VirtualMuxAlarm::new(self.alarm_mux));
        si7021_alarm.setup();

        let buffer = static_buffer.3.write([0; 14]);

        let si7021 = static_buffer
            .2
            .write(SI7021::new(si7021_i2c, si7021_alarm, buffer));

        si7021_i2c.set_client(si7021);
        si7021_alarm.set_alarm_client(si7021);
        si7021
    }
}
