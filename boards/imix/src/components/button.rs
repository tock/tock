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
            [(&'static InterruptPin, button::GpioMode); ButtonComponent::NUM_PINS],
            [(&sam4l::gpio::PC[24], button::GpioMode::LowWhenPressed)]
        );

        let button = static_init!(
            button::Button<'static>,
            button::Button::new(&button_pins[..], self.board_kernel.create_grant(&grant_cap))
        );

        let values = static_init!(
            [kernel::hil::gpio::InterruptWithValue; ButtonComponent::NUM_PINS],
            [gpio::InterruptWithValue::new()]
        );

        for i in 0..ButtonComponent::NUM_PINS {
            let (pin, _config) = button_pins[i];
            pin.set_client(&values[i]);
            values[i].set_client(button);
            values[i].set_value(i as u32);
        }

        button
    }
}
