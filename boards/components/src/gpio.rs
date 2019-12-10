//! Components for GPIO pins.
//!
//! Usage
//! -----
//! ```rust
//! let gpio = components::gpio::GpioPinsNine::new(
//!     &nrf52840::gpio::PORT[GPIO_D2],
//!     &nrf52840::gpio::PORT[GPIO_D3],
//!     &nrf52840::gpio::PORT[GPIO_D4],
//!     &nrf52840::gpio::PORT[GPIO_D5],
//!     &nrf52840::gpio::PORT[GPIO_D6],
//!     &nrf52840::gpio::PORT[GPIO_D7],
//!     &nrf52840::gpio::PORT[GPIO_D8],
//!     &nrf52840::gpio::PORT[GPIO_D9],
//!     &nrf52840::gpio::PORT[GPIO_D10],
//!     board_kernel,
//! )
//! .finalize(());
//! ```


#![allow(dead_code)] // Components are intended to be conditionally included

use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::gpio::{InterruptPin, InterruptValueWrapper};
use kernel::static_init;

pub struct GpioPinsNine {
    pin0: &'static dyn InterruptPin,
    pin1: &'static dyn InterruptPin,
    pin2: &'static dyn InterruptPin,
    pin3: &'static dyn InterruptPin,
    pin4: &'static dyn InterruptPin,
    pin5: &'static dyn InterruptPin,
    pin6: &'static dyn InterruptPin,
    pin7: &'static dyn InterruptPin,
    pin8: &'static dyn InterruptPin,
    board_kernel: &'static kernel::Kernel,
}

impl GpioPinsNine {
    pub fn new(
        pin0: &'static dyn InterruptPin,
        pin1: &'static dyn InterruptPin,
        pin2: &'static dyn InterruptPin,
        pin3: &'static dyn InterruptPin,
        pin4: &'static dyn InterruptPin,
        pin5: &'static dyn InterruptPin,
        pin6: &'static dyn InterruptPin,
        pin7: &'static dyn InterruptPin,
        pin8: &'static dyn InterruptPin,
        board_kernel: &'static kernel::Kernel,
    ) -> Self {
        GpioPinsNine {
            pin0,
            pin1,
            pin2,
            pin3,
            pin4,
            pin5,
            pin6,
            pin7,
            pin8,
            board_kernel,
        }
    }
}

impl Component for GpioPinsNine {
    type StaticInput = ();
    type Output = &'static capsules::gpio::GPIO<'static>;

    unsafe fn finalize(&mut self, _static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let gpio_pins = static_init!(
            [&'static dyn kernel::hil::gpio::InterruptValuePin; 9],
            [
                static_init!(InterruptValueWrapper, InterruptValueWrapper::new(self.pin0))
                    .finalize(),
                static_init!(InterruptValueWrapper, InterruptValueWrapper::new(self.pin1))
                    .finalize(),
                static_init!(InterruptValueWrapper, InterruptValueWrapper::new(self.pin2))
                    .finalize(),
                static_init!(InterruptValueWrapper, InterruptValueWrapper::new(self.pin3))
                    .finalize(),
                static_init!(InterruptValueWrapper, InterruptValueWrapper::new(self.pin4))
                    .finalize(),
                static_init!(InterruptValueWrapper, InterruptValueWrapper::new(self.pin5))
                    .finalize(),
                static_init!(InterruptValueWrapper, InterruptValueWrapper::new(self.pin6))
                    .finalize(),
                static_init!(InterruptValueWrapper, InterruptValueWrapper::new(self.pin7))
                    .finalize(),
                static_init!(InterruptValueWrapper, InterruptValueWrapper::new(self.pin8))
                    .finalize(),
            ]
        );

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
