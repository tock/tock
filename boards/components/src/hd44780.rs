//! Components for the HD447880 LCD controller.
//!
//! Usage
//! -----
//! ```rust
//! let height: u8 = 2;
//! let width: u8 = 16;
//! let lcd = components::hd44780::HD44780Component::new(mux_alarm,
//!                                                      width,
//!                                                      height,
//!                                                      // rs pin
//!                                                      gpio_ports.pins[5][13].as_ref().unwrap(),
//!                                                      // en pin
//!                                                      gpio_ports.pins[4][11].as_ref().unwrap(),
//!                                                      // data 4 pin
//!                                                      gpio_ports.pins[5][14].as_ref().unwrap(),
//!                                                      // data 5 pin
//!                                                      gpio_ports.pins[4][13].as_ref().unwrap(),
//!                                                      // data 6 pin
//!                                                      gpio_ports.pins[5][15].as_ref().unwrap(),
//!                                                      // data 7 pin
//!                                                      gpio_ports.pins[6][14].as_ref().unwrap())
//!     .finalize(
//!     components::hd44780_component_static!(
//!         stm32f429zi::tim2::Tim2,
//!
//!
//!     )
//! );
//! ```

use core::mem::MaybeUninit;
use core_capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use extra_capsules::hd44780::HD44780;
use kernel::component::Component;
use kernel::hil::time;
use kernel::hil::time::Alarm;

// Setup static space for the objects.
#[macro_export]
macro_rules! hd44780_component_static {
    ($A:ty $(,)?) => {{
        let alarm = kernel::static_buf!(core_capsules::virtual_alarm::VirtualMuxAlarm<'static, $A>);
        let hd44780 = kernel::static_buf!(
            extra_capsules::hd44780::HD44780<
                'static,
                core_capsules::virtual_alarm::VirtualMuxAlarm<'static, $A>,
            >
        );
        let buffer = kernel::static_buf!([u8; extra_capsules::hd44780::BUF_LEN]);

        (alarm, hd44780, buffer)
    };};
}

pub struct HD44780Component<A: 'static + time::Alarm<'static>> {
    alarm_mux: &'static MuxAlarm<'static, A>,
    width: u8,
    height: u8,
    rs: &'static dyn kernel::hil::gpio::Pin,
    en: &'static dyn kernel::hil::gpio::Pin,
    data_4_pin: &'static dyn kernel::hil::gpio::Pin,
    data_5_pin: &'static dyn kernel::hil::gpio::Pin,
    data_6_pin: &'static dyn kernel::hil::gpio::Pin,
    data_7_pin: &'static dyn kernel::hil::gpio::Pin,
}

impl<A: 'static + time::Alarm<'static>> HD44780Component<A> {
    pub fn new(
        alarm_mux: &'static MuxAlarm<'static, A>,
        width: u8,
        height: u8,
        rs: &'static dyn kernel::hil::gpio::Pin,
        en: &'static dyn kernel::hil::gpio::Pin,
        data_4_pin: &'static dyn kernel::hil::gpio::Pin,
        data_5_pin: &'static dyn kernel::hil::gpio::Pin,
        data_6_pin: &'static dyn kernel::hil::gpio::Pin,
        data_7_pin: &'static dyn kernel::hil::gpio::Pin,
    ) -> HD44780Component<A> {
        HD44780Component {
            alarm_mux,
            width,
            height,
            rs,
            en,
            data_4_pin,
            data_5_pin,
            data_6_pin,
            data_7_pin,
        }
    }
}

impl<A: 'static + time::Alarm<'static>> Component for HD44780Component<A> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<HD44780<'static, VirtualMuxAlarm<'static, A>>>,
        &'static mut MaybeUninit<[u8; extra_capsules::hd44780::BUF_LEN]>,
    );
    type Output = &'static HD44780<'static, VirtualMuxAlarm<'static, A>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let lcd_alarm = static_buffer.0.write(VirtualMuxAlarm::new(self.alarm_mux));
        lcd_alarm.setup();

        let buffer = static_buffer.2.write([0; extra_capsules::hd44780::BUF_LEN]);

        let hd44780 = static_buffer.1.write(extra_capsules::hd44780::HD44780::new(
            self.rs,
            self.en,
            self.data_4_pin,
            self.data_5_pin,
            self.data_6_pin,
            self.data_7_pin,
            buffer,
            lcd_alarm,
            self.width,
            self.height,
        ));
        lcd_alarm.set_alarm_client(hd44780);

        hd44780
    }
}
