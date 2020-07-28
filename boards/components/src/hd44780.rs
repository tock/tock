//! Components for the HD447880 LCD controller.
//!
//! Usage
//! -----
//! ```rust
//! let lcd = components::hd44780::HD44780Component::new(mux_alarm).finalize(
//!     components::hd44780_component_helper!(
//!         stm32f429zi::tim2::Tim2,
//!         // rs pin
//!         stm32f429zi::gpio::PinId::PF13.get_pin().as_ref().unwrap(),
//!         // en pin
//!         stm32f429zi::gpio::PinId::PE11.get_pin().as_ref().unwrap(),
//!         // data 4 pin
//!         stm32f429zi::gpio::PinId::PF14.get_pin().as_ref().unwrap(),
//!         // data 5 pin
//!         stm32f429zi::gpio::PinId::PE13.get_pin().as_ref().unwrap(),
//!         // data 6 pin
//!         stm32f429zi::gpio::PinId::PF15.get_pin().as_ref().unwrap(),
//!         // data 7 pin
//!         stm32f429zi::gpio::PinId::PG14.get_pin().as_ref().unwrap()
//!     )
//! );
//! ```
use capsules::hd44780::HD44780;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::time;
use kernel::hil::time::Alarm;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! hd44780_component_helper {
    ($A:ty, $rs:expr, $en: expr, $data_4_pin: expr, $data_5_pin: expr, $data_6_pin: expr, $data_7_pin: expr) => {{
        use capsules::hd44780::HD44780;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<HD44780<'static, VirtualMuxAlarm<'static, $A>>> =
            MaybeUninit::uninit();
        (
            &mut BUF1,
            &mut BUF2,
            $rs,
            $en,
            $data_4_pin,
            $data_5_pin,
            $data_6_pin,
            $data_7_pin,
        )
    };};
}

pub struct HD44780Component<A: 'static + time::Alarm<'static>> {
    alarm_mux: &'static MuxAlarm<'static, A>,
}

impl<A: 'static + time::Alarm<'static>> HD44780Component<A> {
    pub fn new(alarm_mux: &'static MuxAlarm<'static, A>) -> HD44780Component<A> {
        HD44780Component {
            alarm_mux: alarm_mux,
        }
    }
}

impl<A: 'static + time::Alarm<'static>> Component for HD44780Component<A> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<HD44780<'static, VirtualMuxAlarm<'static, A>>>,
        &'static dyn kernel::hil::gpio::Pin,
        &'static dyn kernel::hil::gpio::Pin,
        &'static dyn kernel::hil::gpio::Pin,
        &'static dyn kernel::hil::gpio::Pin,
        &'static dyn kernel::hil::gpio::Pin,
        &'static dyn kernel::hil::gpio::Pin,
    );
    type Output = &'static HD44780<'static, VirtualMuxAlarm<'static, A>>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let lcd_alarm = static_init_half!(
            static_buffer.0,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );

        let hd44780 = static_init_half!(
            static_buffer.1,
            capsules::hd44780::HD44780<'static, VirtualMuxAlarm<'static, A>>,
            capsules::hd44780::HD44780::new(
                static_buffer.2,
                static_buffer.3,
                static_buffer.4,
                static_buffer.5,
                static_buffer.6,
                static_buffer.7,
                &mut capsules::hd44780::ROW_OFFSETS,
                lcd_alarm,
            )
        );
        lcd_alarm.set_client(hd44780);

        hd44780
    }
}
