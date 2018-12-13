//! Component for communicating with the nRF51822 (BLE) on imix boards.
//!
//! This provides one Component, Nrf51822Component, which implements
//! a system call interface to the nRF51822 for BLE advertisements.
//!
//! Usage
//! -----
//! ```rust
//! let nrf_serialization = Nrf51822Component::new(&sam4l::usart::USART2).finalize();
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules::nrf51822_serialization;
use hil;
use kernel::component::Component;

pub struct Nrf51822Component {
    uart: &'static sam4l::usart::USART<'static>,
    reset_pin: &'static sam4l::gpio::GPIOPin,
}

impl Nrf51822Component {
    pub fn new(
        uart: &'static sam4l::usart::USART<'static>,
        reset_pin: &'static sam4l::gpio::GPIOPin,
    ) -> Nrf51822Component {
        Nrf51822Component {
            uart: uart,
            reset_pin: reset_pin,
        }
    }
}

impl Component for Nrf51822Component {
    type Output =
        &'static nrf51822_serialization::Nrf51822Serialization<'static>;

    unsafe fn finalize(&mut self) -> Self::Output {
        sam4l::usart::USART2.set_mode(sam4l::usart::UsartMode::Uart);
        let nrf_serialization = static_init!(
            nrf51822_serialization::Nrf51822Serialization<'static>,
            nrf51822_serialization::Nrf51822Serialization::new(
                self.uart,
                self.reset_pin,
                &mut nrf51822_serialization::WRITE_BUF,
                &mut nrf51822_serialization::READ_BUF
            )
        );
        hil::uart::Transmit::set_transmit_client(self.uart, nrf_serialization);
        hil::uart::Receive::set_receive_client(self.uart, nrf_serialization);
        nrf_serialization
    }
}
