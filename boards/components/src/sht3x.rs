//! Component for the SHT3x sensor.
//!
//! I2C Interface
//!
//! Usage
//! -----
//!
//! With the default i2c address
//! ```rust
//! let sht3x = components::sht3x::SHT3xComponent::new(sensors_i2c_bus, mux_alarm).finalize(
//!         components::sht3x_component_helper!(nrf52::rtc::Rtc<'static>),
//!     );
//! sht3x.reset();
//! ```
//!
//! With a specified i2c address
//! ```rust
//! let sht3x = components::sht3x::SHT3xComponent::new(sensors_i2c_bus, mux_alarm).finalize(
//!         components::sht3x_component_helper!(nrf52::rtc::Rtc<'static>, capsules::sht3x::BASE_ADDR),
//!     );
//! sht3x.reset();
//! ```

use capsules::sht3x::SHT3x;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_i2c::MuxI2C;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::time::Alarm;

use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! sht3x_component_helper {
    ($A:ty) => {{
        use capsules::sht3x;
        $crate::sht3x_component_helper!($A, sht3x::BASE_ADDR)
    }};

    // used for specifically stating the i2c address
    // as some boards (like nrf52) require a shift
    ($A:ty, $address: expr) => {{
        use capsules::sht3x::SHT3x;
        use capsules::virtual_i2c::I2CDevice;
        use core::mem::MaybeUninit;

        static mut BUFFER: [u8; 6] = [0; 6];

        static mut sht3x: MaybeUninit<SHT3x<'static, VirtualMuxAlarm<'static, $A>>> =
            MaybeUninit::uninit();
        static mut sht3x_alarm: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        (&mut sht3x_alarm, &mut BUFFER, &mut sht3x, $address)
    }};
}

pub struct SHT3xComponent<A: 'static + Alarm<'static>> {
    i2c_mux: &'static MuxI2C<'static>,
    alarm_mux: &'static MuxAlarm<'static, A>,
}

impl<A: 'static + Alarm<'static>> SHT3xComponent<A> {
    pub fn new(
        i2c_mux: &'static MuxI2C<'static>,
        alarm_mux: &'static MuxAlarm<'static, A>,
    ) -> SHT3xComponent<A> {
        SHT3xComponent { i2c_mux, alarm_mux }
    }
}

impl<A: 'static + Alarm<'static>> Component for SHT3xComponent<A> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut [u8],
        &'static mut MaybeUninit<SHT3x<'static, VirtualMuxAlarm<'static, A>>>,
        u8,
    );
    type Output = &'static SHT3x<'static, VirtualMuxAlarm<'static, A>>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let sht3x_i2c = crate::i2c::I2CComponent::new(self.i2c_mux, static_buffer.3)
            .finalize(crate::i2c_component_helper!());

        let sht3x_alarm = static_init_half!(
            static_buffer.0,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );

        let sht3x = static_init_half!(
            static_buffer.2,
            SHT3x<'static, VirtualMuxAlarm<'static, A>>,
            SHT3x::new(sht3x_i2c, static_buffer.1, sht3x_alarm)
        );
        sht3x_i2c.set_client(sht3x);
        sht3x_alarm.set_alarm_client(sht3x);

        sht3x
    }
}
