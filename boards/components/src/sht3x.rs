//! Component for the SHT3x sensor.
//!
//! I2C Interface
//!
//! Usage
//! -----
//!
//! ```rust
//! let sht3x = components::sht3x::SHT3xComponent::new(sensors_i2c_bus, extra_capsules::sht3x::BASE_ADDR, mux_alarm).finalize(
//!         components::sht3x_component_static!(nrf52::rtc::Rtc<'static>),
//!     );
//! sht3x.reset();
//! ```

use core::mem::MaybeUninit;
use core_capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core_capsules::virtual_i2c::{I2CDevice, MuxI2C};
use extra_capsules::sht3x::SHT3x;
use kernel::component::Component;
use kernel::hil::time::Alarm;

// Setup static space for the objects.
#[macro_export]
macro_rules! sht3x_component_static {
    ($A:ty $(,)?) => {{
        let buffer = kernel::static_buf!([u8; 6]);
        let i2c_device = kernel::static_buf!(core_capsules::virtual_i2c::I2CDevice<'static>);
        let sht3x_alarm =
            kernel::static_buf!(core_capsules::virtual_alarm::VirtualMuxAlarm<'static, $A>);
        let sht3x = kernel::static_buf!(
            extra_capsules::sht3x::SHT3x<
                'static,
                core_capsules::virtual_alarm::VirtualMuxAlarm<'static, $A>,
            >
        );

        (sht3x_alarm, i2c_device, sht3x, buffer)
    };};
}

pub struct SHT3xComponent<A: 'static + Alarm<'static>> {
    i2c_mux: &'static MuxI2C<'static>,
    i2c_address: u8,
    alarm_mux: &'static MuxAlarm<'static, A>,
}

impl<A: 'static + Alarm<'static>> SHT3xComponent<A> {
    pub fn new(
        i2c_mux: &'static MuxI2C<'static>,
        i2c_address: u8,
        alarm_mux: &'static MuxAlarm<'static, A>,
    ) -> SHT3xComponent<A> {
        SHT3xComponent {
            i2c_mux,
            i2c_address,
            alarm_mux,
        }
    }
}

impl<A: 'static + Alarm<'static>> Component for SHT3xComponent<A> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<I2CDevice<'static>>,
        &'static mut MaybeUninit<SHT3x<'static, VirtualMuxAlarm<'static, A>>>,
        &'static mut MaybeUninit<[u8; 6]>,
    );
    type Output = &'static SHT3x<'static, VirtualMuxAlarm<'static, A>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let sht3x_i2c = static_buffer
            .1
            .write(I2CDevice::new(self.i2c_mux, self.i2c_address));

        let buffer = static_buffer.3.write([0; 6]);

        let sht3x_alarm = static_buffer.0.write(VirtualMuxAlarm::new(self.alarm_mux));
        sht3x_alarm.setup();

        let sht3x = static_buffer
            .2
            .write(SHT3x::new(sht3x_i2c, buffer, sht3x_alarm));
        sht3x_i2c.set_client(sht3x);
        sht3x_alarm.set_alarm_client(sht3x);

        sht3x
    }
}
