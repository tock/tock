//! Components for GPIO pins.
//!
//! Usage
//! -----
//! ```rust
//! let gpio = components::gpio::GpioComponent::new(
//!     board_kernel,
//!     components::gpio_component_helper!(
//!         sam4l::gpio::GPIOPin,
//!         &nrf52840::gpio::PORT[GPIO_D2],
//!         &nrf52840::gpio::PORT[GPIO_D3],
//!         &nrf52840::gpio::PORT[GPIO_D4],
//!         &nrf52840::gpio::PORT[GPIO_D5],
//!         &nrf52840::gpio::PORT[GPIO_D6],
//!         &nrf52840::gpio::PORT[GPIO_D7],
//!         &nrf52840::gpio::PORT[GPIO_D8],
//!         &nrf52840::gpio::PORT[GPIO_D9],
//!         &nrf52840::gpio::PORT[GPIO_D10]
//!     )
//! )
//! .finalize(gpio_component_buf!(sam4l::gpio::GPIOPin));
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
macro_rules! gpio_helper {
    (($id:literal =>  $P:expr)) => {
        Some(static_init!(InterruptValueWrapper, InterruptValueWrapper::new($P)).finalize())
    };
    ((@gap)) => {
        None
    };
}

#[macro_export]
macro_rules! gpio_component_helper {
    ($Pin:ty, $($P:expr),+ ) => {{
        use kernel::static_init;
        use kernel::count_expressions;
        use kernel::hil::gpio::InterruptValueWrapper;
        use components::gpio_helper;
        const NUM_PINS: usize = count_expressions!($($P),+);

        static_init!(
            [&'static InterruptValueWrapper<'static, $Pin>; NUM_PINS],
            [
                $(
                    static_init!(InterruptValueWrapper<$Pin>, InterruptValueWrapper::new($P))
                    .finalize(),
                )*
            ]
        )
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
    gpio_pins: &'static [&'static gpio::InterruptValueWrapper<'static, IP>],
}

impl<IP: 'static + gpio::InterruptPin> GpioComponent<IP> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        gpio_pins: &'static [&'static gpio::InterruptValueWrapper<'static, IP>],
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
        for pin in self.gpio_pins.iter() {
            pin.set_client(gpio);
        }

        gpio
    }
}
