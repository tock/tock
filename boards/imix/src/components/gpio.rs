//! Component for GPIO on the imix board.
//!
//! This provides one Component, GpioComponent, which implements
//! a userspace syscall interface to a subset of the SAM4L GPIO
//! pins provided on the board headers. It provides 5 pins:
//! 31 (P2), 30 (P3), 29 (P4), 28 (P5), 27  (P6), 26 (P7),
//! and 20 (P8).
//!
//! Usage
//! -----
//! ```rust
//! let gpio = GpioComponent::new(board_kernel).finalize();
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules::gpio;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::hil::gpio::InterruptWithValue;
use kernel::static_init;

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
    type Output = &'static gpio::GPIO<'static>;

    unsafe fn finalize(&mut self) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let gpio_pins = static_init!(
            [&'static hil::gpio::InterruptPin; 7],
            [
                &sam4l::gpio::PC[31], // P2
                &sam4l::gpio::PC[30], // P3
                &sam4l::gpio::PC[29], // P4
                &sam4l::gpio::PC[28], // P5
                &sam4l::gpio::PC[27], // P6
                &sam4l::gpio::PC[26], // P7
                &sam4l::gpio::PA[20], // P8
            ]
        );

        let gpio_values = static_init!(
            [hil::gpio::InterruptValueWrapper; 7],
            [
                hil::gpio::InterruptValueWrapper::new(),
                hil::gpio::InterruptValueWrapper::new(),
                hil::gpio::InterruptValueWrapper::new(),
                hil::gpio::InterruptValueWrapper::new(),
                hil::gpio::InterruptValueWrapper::new(),
                hil::gpio::InterruptValueWrapper::new(),
                hil::gpio::InterruptValueWrapper::new(),
            ]
        );

        for i in 0..7 {
            gpio_pins[i].set_client(&gpio_values[i]);
            gpio_values[i].set_source(gpio_pins[i]);
            gpio_values[i].set_value(i as u32);
        }

        let gpio_refs = static_init!(
            [&'static hil::gpio::InterruptValuePin; 7],
            [
                &gpio_values[0],
                &gpio_values[1],
                &gpio_values[2],
                &gpio_values[3],
                &gpio_values[4],
                &gpio_values[5],
                &gpio_values[6],
            ]
        );

        let gpio = static_init!(
            gpio::GPIO<'static>,
            gpio::GPIO::new(&gpio_refs[..], self.board_kernel.create_grant(&grant_cap))
        );

        for i in 0..7 {
            gpio_values[i].set_client(gpio);
        }
        gpio
    }
}
