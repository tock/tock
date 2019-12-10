//! Components for collections of LEDs.
//!
//! Usage
//! -----
//! ```rust
//! let led = components::led::LedsThree::new(
//!     (&nrf52840::gpio::PORT[LED_RED_PIN], capsules::led::ActivationMode::ActiveLow),
//!     (&nrf52840::gpio::PORT[LED_GREEN_PIN], capsules::led::ActivationMode::ActiveLow),
//!     (&nrf52840::gpio::PORT[LED_BLUE_PIN], capsules::led::ActivationMode::ActiveLow
//! )).finalize(());
//! ```


#![allow(dead_code)] // Components are intended to be conditionally included

use capsules;
use kernel::component::Component;
use kernel::hil::gpio::Pin;
use kernel::static_init;

pub struct LedsThree {
    led0: (&'static dyn Pin, capsules::led::ActivationMode),
    led1: (&'static dyn Pin, capsules::led::ActivationMode),
    led2: (&'static dyn Pin, capsules::led::ActivationMode),
}

impl LedsThree {
    pub fn new(
        led0: (&'static dyn Pin, capsules::led::ActivationMode),
        led1: (&'static dyn Pin, capsules::led::ActivationMode),
        led2: (&'static dyn Pin, capsules::led::ActivationMode),
    ) -> Self {
        LedsThree { led0, led1, led2 }
    }
}

impl Component for LedsThree {
    type StaticInput = ();
    type Output = &'static capsules::led::LED<'static>;

    unsafe fn finalize(&mut self, _static_buffer: Self::StaticInput) -> Self::Output {
        let led_pins = static_init!(
            [(
                &'static dyn kernel::hil::gpio::Pin,
                capsules::led::ActivationMode
            ); 3],
            [self.led0, self.led1, self.led2,]
        );

        static_init!(
            capsules::led::LED<'static>,
            capsules::led::LED::new(led_pins)
        )
    }
}
