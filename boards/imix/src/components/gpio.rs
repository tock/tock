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
use kernel::hil::gpio::InterruptValueWrapper;
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
            [&'static kernel::hil::gpio::InterruptValuePin; 7],
            [
                static_init!(
                    InterruptValueWrapper,
                    InterruptValueWrapper::new(&sam4l::gpio::PC[31])
                )
                .finalize(),
                static_init!(
                    InterruptValueWrapper,
                    InterruptValueWrapper::new(&sam4l::gpio::PC[30])
                )
                .finalize(),
                static_init!(
                    InterruptValueWrapper,
                    InterruptValueWrapper::new(&sam4l::gpio::PC[29])
                )
                .finalize(),
                static_init!(
                    InterruptValueWrapper,
                    InterruptValueWrapper::new(&sam4l::gpio::PC[28])
                )
                .finalize(),
                static_init!(
                    InterruptValueWrapper,
                    InterruptValueWrapper::new(&sam4l::gpio::PC[27])
                )
                .finalize(),
                static_init!(
                    InterruptValueWrapper,
                    InterruptValueWrapper::new(&sam4l::gpio::PC[26])
                )
                .finalize(),
                static_init!(
                    InterruptValueWrapper,
                    InterruptValueWrapper::new(&sam4l::gpio::PC[20])
                )
                .finalize(),
            ]
        );

        let gpio = static_init!(
            gpio::GPIO<'static>,
            gpio::GPIO::new(gpio_pins, self.board_kernel.create_grant(&grant_cap))
        );

        for pin in gpio_pins.iter() {
            pin.set_client(gpio);
        }
        gpio
    }
}
