//! Component for imix board buttons.
//!
//! This provides one Component, ButtonComponent, which implements a
//! userspace syscall interface to the one imix on-board button (pin
//! 24).
//!
//! Usage
//! -----
//! ```rust
//! let button = ButtonComponent::new(board_kernel).finalize();
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules::button;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::gpio;
use kernel::hil::gpio::InterruptPin;
use kernel::hil::gpio::InterruptWithValue;
use kernel::static_init;

pub struct ButtonComponent {
    board_kernel: &'static kernel::Kernel,
}

impl ButtonComponent {
    const NUM_PINS: usize = 1;
    pub fn new(board_kernel: &'static kernel::Kernel) -> ButtonComponent {
        ButtonComponent {
            board_kernel: board_kernel,
        }
    }
}


impl Component for ButtonComponent {
    type Output = &'static button::Button<'static>;
    unsafe fn finalize(&mut self) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let button_pins = static_init!(
            [&'static InterruptPin; ButtonComponent::NUM_PINS],
            [&sam4l::gpio::PC[24]]
        );

        let values = static_init!(
            [kernel::hil::gpio::InterruptValueWrapper; ButtonComponent::NUM_PINS],
            [gpio::InterruptValueWrapper::new()]
        );
 
        // Button expects a configured InterruptValuePin so configure it here.
        for i in 0..ButtonComponent::NUM_PINS {
            let pin = button_pins[i];
            let value = &values[i];
            pin.set_client(value);
            value.set_source(pin);
        }

        let config_values = static_init!(
            [(&'static kernel::hil::gpio::InterruptValuePin, button::GpioMode); ButtonComponent::NUM_PINS],
            [(&values[0], button::GpioMode::LowWhenPressed)]
        );

        let button = static_init!(
            button::Button<'static>,
            button::Button::new(&config_values[..], self.board_kernel.create_grant(&grant_cap))
        );

        for i in 0..ButtonComponent::NUM_PINS {
            values[i].set_client(button);
        }

        button
    }
}
