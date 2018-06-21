//! Component for imix board buttons.
//!
//! This provides one Component, ButtonComponent, which implements a
//! userspace syscall interface to the one imix on-board button (pin
//! 24).
//!
//! Usage
//! -----
//! ```rust
//! let button = ButtonComponent::new().finalize();
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules::button;
use kernel;
use kernel::component::Component;
use sam4l;

pub struct ButtonComponent {}

impl ButtonComponent {
    pub fn new() -> ButtonComponent {
        ButtonComponent {}
    }
}

impl Component for ButtonComponent {
    type Output = &'static button::Button<'static, sam4l::gpio::GPIOPin>;

    unsafe fn finalize(&mut self) -> Self::Output {
        let button_pins = static_init!(
            [(&'static sam4l::gpio::GPIOPin, button::GpioMode); 1],
            [(&sam4l::gpio::PC[24], button::GpioMode::LowWhenPressed)]
        );

        let button = static_init!(
            button::Button<'static, sam4l::gpio::GPIOPin>,
            button::Button::new(button_pins, kernel::Grant::create())
        );
        for &(btn, _) in button_pins.iter() {
            btn.set_client(button);
        }

        button
    }
}
