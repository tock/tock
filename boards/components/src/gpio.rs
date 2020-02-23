//! Components for GPIO pins.
//!
//! Usage
//! -----
//! ```rust
//! let gpio = components::gpio::GpioComponent::new(board_kernel).finalize(
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

use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::static_init;

#[macro_export]
macro_rules! gpio_component_helper {
    ($($P:expr),+ ) => {{
        use kernel::static_init;
        use kernel::count_expressions;
        use kernel::hil::gpio::InterruptValueWrapper;
        const NUM_PINS: usize = count_expressions!($($P),+);

        static_init!(
            [&'static dyn kernel::hil::gpio::InterruptValuePin; NUM_PINS],
            [
                $(
                    static_init!(InterruptValueWrapper, InterruptValueWrapper::new($P))
                    .finalize(),
                )*
            ]
        )
    };};
}

pub struct GpioComponent {
    board_kernel: &'static kernel::Kernel,
}

impl GpioComponent {
    pub fn new(board_kernel: &'static kernel::Kernel) -> GpioComponent {
        GpioComponent {
            board_kernel: board_kernel,
        }
    }
}

impl Component for GpioComponent {
    type StaticInput = &'static [&'static dyn kernel::hil::gpio::InterruptValuePin];
    type Output = &'static capsules::gpio::GPIO<'static>;

    unsafe fn finalize(&mut self, gpio_pins: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let gpio = static_init!(
            capsules::gpio::GPIO<'static>,
            capsules::gpio::GPIO::new(gpio_pins, self.board_kernel.create_grant(&grant_cap))
        );
        for pin in gpio_pins.iter() {
            pin.set_client(gpio);
        }

        gpio
    }
}
