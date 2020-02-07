//! Components for collections of LEDs.
//!
//! Usage
//! -----
//! ```rust
//! let led = components::led::LedsComponent::new().finalize(components::led_component_helper!(
//!     (&nrf52840::gpio::PORT[LED_RED_PIN], capsules::led::ActivationMode::ActiveLow),
//!     (&nrf52840::gpio::PORT[LED_GREEN_PIN], capsules::led::ActivationMode::ActiveLow),
//!     (&nrf52840::gpio::PORT[LED_BLUE_PIN], capsules::led::ActivationMode::ActiveLow)
//! ));
//! ```

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules;
use kernel::component::Component;
use kernel::static_init;

#[macro_export]
macro_rules! led_component_helper {
    ($($P:expr),+ ) => {{
        use kernel::count_expressions;
        use kernel::static_init;
        const NUM_LEDS: usize = count_expressions!($($P),+);

        static_init!(
            [(
                &'static dyn kernel::hil::gpio::Pin,
                capsules::led::ActivationMode
            ); NUM_LEDS],
            [
                $($P,)*
            ]
        )
    };};
}

pub struct LedsComponent {}

impl LedsComponent {
    pub fn new() -> LedsComponent {
        LedsComponent {}
    }
}

impl Component for LedsComponent {
    type StaticInput = &'static [(
        &'static dyn kernel::hil::gpio::Pin,
        capsules::led::ActivationMode,
    )];
    type Output = &'static capsules::led::LED<'static>;

    unsafe fn finalize(self, pins: Self::StaticInput) -> Self::Output {
        static_init!(capsules::led::LED<'static>, capsules::led::LED::new(pins))
    }
}
