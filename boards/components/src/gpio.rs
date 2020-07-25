//! Components for GPIO pins.
//!
//!
//! Usage
//! -----
//! ```rust
//! let gpio = components::gpio::GpioComponent::new(
//!     board_kernel,
//!     components::gpio_component_helper!(
//!         nrf52840::gpio::GPIOPin,
//!         // left side of the USB plug
//!         0 => &nrf52840::gpio::PORT[Pin::P0_13],
//!         1 => &nrf52840::gpio::PORT[Pin::P0_15],
//!         2 => &nrf52840::gpio::PORT[Pin::P0_17],
//!         3 => &nrf52840::gpio::PORT[Pin::P0_20],
//!         4 => &nrf52840::gpio::PORT[Pin::P0_22],
//!         5 => &nrf52840::gpio::PORT[Pin::P0_24],
//!         6 => &nrf52840::gpio::PORT[Pin::P1_00],
//!         7 => &nrf52840::gpio::PORT[Pin::P0_09],
//!         8 => &nrf52840::gpio::PORT[Pin::P0_10],
//!         // right side of the USB plug
//!         9 => &nrf52840::gpio::PORT[Pin::P0_31],
//!         10 => &nrf52840::gpio::PORT[Pin::P0_29],
//!         11 => &nrf52840::gpio::PORT[Pin::P0_02],
//!         12 => &nrf52840::gpio::PORT[Pin::P1_15],
//!         13 => &nrf52840::gpio::PORT[Pin::P1_13],
//!         14 => &nrf52840::gpio::PORT[Pin::P1_10],
//!         // Below the PCB
//!         15 => &nrf52840::gpio::PORT[Pin::P0_26],
//!         16 => &nrf52840::gpio::PORT[Pin::P0_04],
//!         17 => &nrf52840::gpio::PORT[Pin::P0_11],
//!         18 => &nrf52840::gpio::PORT[Pin::P0_14],
//!         19 => &nrf52840::gpio::PORT[Pin::P1_11],
//!         20 => &nrf52840::gpio::PORT[Pin::P1_07],
//!         21 => &nrf52840::gpio::PORT[Pin::P1_01],
//!         22 => &nrf52840::gpio::PORT[Pin::P1_04],
//!         23 => &nrf52840::gpio::PORT[Pin::P1_02]
//!     ),
//! ).finalize(components::gpio_component_buf!(nrf52840::gpio::GPIOPin));
//! ```

use capsules::gpio::GPIO;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::gpio;
use kernel::hil::gpio::InterruptWithValue;
use kernel::static_init_half;

#[macro_export]
macro_rules! gpio_component_helper_max_pin {
    () => { 0usize };
    ($a: expr, $b: expr, $($tail:expr,)*) => { $crate::gpio_component_helper_max_pin! (max ($a, $b), $($tail,)*) };
    ($a: expr,) => { $a };
}

#[macro_export]
/// Pins are declared using the following format:
///     number => pin
///
/// Any pin numbers that are skipped will be declared as None
/// and using them from user space will return ENODEVICE
macro_rules! gpio_component_helper {
    (
        $Pin:ty,
        $($nr:literal => $pin:expr),*
    ) => {{
        use kernel::count_expressions;
        use kernel::hil::gpio::InterruptValueWrapper;
        use kernel::static_init;

        const fn max (a: usize, b: usize) -> usize {
            [a, b][(a < b) as usize]
        }

        const NUM_PINS: usize = $crate::gpio_component_helper_max_pin! ($($nr,)*) + 1;

        let mut pins = static_init!(
            [Option<&'static InterruptValueWrapper<'static, $Pin>>; NUM_PINS],
            [None; NUM_PINS]
        );

        $(
            pins[$nr] = Some(static_init!(InterruptValueWrapper<$Pin>, InterruptValueWrapper::new($pin)).finalize());
        )*

        pins
    };};
}

#[macro_export]
macro_rules! gpio_component_buf {
    ($Pin:ty) => {{
        use capsules::gpio::GPIO;
        use core::mem::MaybeUninit;
        static mut BUF: MaybeUninit<GPIO<'static, $Pin>> = MaybeUninit::uninit();
        &mut BUF
    };};
}

pub struct GpioComponent<IP: 'static + gpio::InterruptPin<'static>> {
    board_kernel: &'static kernel::Kernel,
    gpio_pins: &'static [Option<&'static gpio::InterruptValueWrapper<'static, IP>>],
}

impl<IP: 'static + gpio::InterruptPin<'static>> GpioComponent<IP> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        gpio_pins: &'static [Option<&'static gpio::InterruptValueWrapper<'static, IP>>],
    ) -> Self {
        Self {
            board_kernel: board_kernel,
            gpio_pins,
        }
    }
}

impl<IP: 'static + gpio::InterruptPin<'static>> Component for GpioComponent<IP> {
    type StaticInput = &'static mut MaybeUninit<GPIO<'static, IP>>;
    type Output = &'static GPIO<'static, IP>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let gpio = static_init_half!(
            static_buffer,
            GPIO<'static, IP>,
            GPIO::new(self.gpio_pins, self.board_kernel.create_grant(&grant_cap))
        );
        for maybe_pin in self.gpio_pins.iter() {
            if let Some(pin) = maybe_pin {
                pin.set_client(gpio);
            }
        }

        gpio
    }
}
