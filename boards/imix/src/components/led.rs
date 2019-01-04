//! Component for imix board LEDs.
//!
//! This provides one Component, LedComponent, which implements
//! a userspace syscall interface to the two imix on-board LEDs.
//!
//! Usage
//! -----
//! ```rust
//! let led = LedComponent::new().finalize();
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules::led;
use kernel::component::Component;
use kernel::static_init;

pub struct LedComponent {}

impl LedComponent {
    pub fn new() -> LedComponent {
        LedComponent {}
    }
}

impl Component for LedComponent {
    type Output = &'static led::LED<'static, sam4l::gpio::GPIOPin>;

    unsafe fn finalize(&mut self) -> Self::Output {
        let led_pins = static_init!(
            [(&'static sam4l::gpio::GPIOPin, led::ActivationMode); 1],
            [(&sam4l::gpio::PC[10], led::ActivationMode::ActiveHigh),]
        );
        let led = static_init!(
            led::LED<'static, sam4l::gpio::GPIOPin>,
            led::LED::new(led_pins)
        );
        led
    }
}
