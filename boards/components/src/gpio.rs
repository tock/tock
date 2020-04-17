//! Components for GPIO pins.
//!
//! Usage
//! -----
//! ```rust
//! let gpio = components::gpio::GpioComponent::new(
//!     board_kernel,
//!     components::gpio_component_helper!(
//!         // pin number => pin reference
//!         sam4l::gpio::GPIOPin,
//!         2 => &nrf52840::gpio::PORT[GPIO_D2],
//!         3 => &nrf52840::gpio::PORT[GPIO_D3],
//!         4 => &nrf52840::gpio::PORT[GPIO_D4],
//!         5 => &nrf52840::gpio::PORT[GPIO_D5],
//!         6 => &nrf52840::gpio::PORT[GPIO_D6],
//!         7 => &nrf52840::gpio::PORT[GPIO_D7],
//!         8 => &nrf52840::gpio::PORT[GPIO_D8],
//!         9 => &nrf52840::gpio::PORT[GPIO_D9],
//!         10 => &nrf52840::gpio::PORT[GPIO_D10]
//!     )
//! ).finalize(gpio_component_buf!(sam4l::gpio::GPIOPin));;
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

pub struct GpioComponent<IP: 'static + gpio::InterruptPin> {
    board_kernel: &'static kernel::Kernel,
    gpio_pins: &'static [Option<&'static gpio::InterruptValueWrapper<'static, IP>>],
}

impl<IP: 'static + gpio::InterruptPin> GpioComponent<IP> {
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

impl<IP: 'static + gpio::InterruptPin> Component for GpioComponent<IP> {
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
            if let Some (pin) = maybe_pin {
                pin.set_client(gpio);
            }
        }

        gpio
    }
}
