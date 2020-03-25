//! Components for GPIO pins.
//!
//! Usage
//! -----
//! ```rust
//! let gpio = components::gpio::HD44780Component::new(board_kernel).finalize(
//!     components::gpio_component_helper!(
//!     &nrf52840::gpio::PORT[GPIO_D2],
//!     &nrf52840::gpio::PORT[GPIO_D3],
//!     &nrf52840::gpio::PORT[GPIO_D4],
//!     &nrf52840::gpio::PORT[GPIO_D5],
//!     &nrf52840::gpio::PORT[GPIO_D6],
//!     &nrf52840::gpio::PORT[GPIO_D7],
//!     &nrf52840::gpio::PORT[GPIO_D8],
//!     &nrf52840::gpio::PORT[GPIO_D9],
//!     &nrf52840::gpio::PORT[GPIO_D10]
//! ));
//! ```
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::static_init;
use kernel::hil::time::Alarm;


pub struct HD44780Component {
    board_kernel: &'static kernel::Kernel,
}

impl HD44780Component {
    pub fn new(board_kernel: &'static kernel::Kernel) -> HD44780Component {
        HD44780Component {
            board_kernel: board_kernel,
        }
    }
}

impl<A: Alarm<'a>> Component for HD44780Component {
    type Output = &'static capsules::hd44780::HD44780<
        'static,
        VirtualMuxAlarm<'static, stm32f4xx::tim2::Tim2<'static>>,
    >;

    unsafe fn finalize(
        self, 
        rs_pin: &'static dyn kernel::hil::gpio::Pin,
        en_pin: &'static dyn kernel::hil::gpio::Pin,
        data_4_pin: &'static dyn kernel::hil::gpio::Pin,
        data_5_pin: &'static dyn kernel::hil::gpio::Pin,
        data_6_pin: &'static dyn kernel::hil::gpio::Pin,
        data_7_pin: &'static dyn kernel::hil::gpio::Pin,
        command_buffer: &'static mut [u8],
        row_offsets: &'static mut [u8],
        alarm: &'static A,
    ) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_lcd = self.board_kernel.create_grant(&grant_cap);
    
        let hd44780 = static_init!(
            capsules::hd44780::HD44780<'static, VirtualMuxAlarm<'static, stm32f4xx::tim2::Tim2>>,
            capsules::hd44780::HD44780::new(
                rs_pin,
                en_pin,
                data_4_pin,
                data_5_pin,
                data_6_pin,
                data_7_pin, 
                command_buffer,
                row_offsets,
                alarm,
                grant_lcd,
            )
        );

        hd44780
    }
}
