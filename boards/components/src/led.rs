//! Components for collections of LEDs.
//!
//! Usage
//! -----
//! ```rust
//! let led = components::led::LedsComponent::new().finalize(components::led_component_helper!(
//!     (&nrf52840::gpio::PORT[LED_RED_PIN], kernel::hil::gpio::ActivationMode::ActiveLow),
//!     (&nrf52840::gpio::PORT[LED_GREEN_PIN], kernel::hil::gpio::ActivationMode::ActiveLow),
//!     (&nrf52840::gpio::PORT[LED_BLUE_PIN], kernel::hil::gpio::ActivationMode::ActiveLow)
//! ));
//! ```

use capsules::led::LED;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::static_init_half;

#[macro_export]
macro_rules! led_component_helper {
    ($Pin:ty, $($P:expr),+ ) => {{
        use kernel::count_expressions;
        use kernel::static_init;
        const NUM_LEDS: usize = count_expressions!($($P),+);

        static_init!(
            [(
                &'static $Pin,
                kernel::hil::gpio::ActivationMode
            ); NUM_LEDS],
            [
                $($P,)*
            ]
        )
    };};
}

#[macro_export]
macro_rules! led_component_buf {
    ($Pin:ty) => {{
        use capsules::led::LED;
        use core::mem::MaybeUninit;
        static mut BUF: MaybeUninit<LED<'static, $Pin>> = MaybeUninit::uninit();
        &mut BUF
    };};
}

pub struct LedsComponent<P: 'static + kernel::hil::gpio::Pin> {
    pins: &'static [(&'static P, kernel::hil::gpio::ActivationMode)],
}

impl<P: 'static + kernel::hil::gpio::Pin> LedsComponent<P> {
    pub fn new(pins: &'static [(&'static P, kernel::hil::gpio::ActivationMode)]) -> Self {
        Self { pins }
    }
}

impl<P: 'static + kernel::hil::gpio::Pin> Component for LedsComponent<P> {
    type StaticInput = &'static mut MaybeUninit<LED<'static, P>>;
    type Output = &'static LED<'static, P>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        static_init_half!(static_buffer, LED<'static, P>, LED::new(self.pins))
    }
}
