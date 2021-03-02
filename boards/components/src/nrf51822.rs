//! Component for communicating with the nRF51822 (BLE).
//!
//! This provides one Component, Nrf51822Component, which implements
//! a system call interface to the nRF51822 for BLE advertisements.
//!
//! Usage
//! -----
//! ```rust
//! let nrf_serialization = Nrf51822Component::new(&sam4l::usart::USART3,
//!                                                &sam4l::gpio::PA[17]).finalize(());
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

use capsules::nrf51822_serialization;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::static_init;

pub struct Nrf51822Component<
    U: 'static + hil::uart::UartAdvanced<'static>,
    G: 'static + hil::gpio::Pin,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: u32,
    uart: &'static U,
    reset_pin: &'static G,
}

impl<U: 'static + hil::uart::UartAdvanced<'static>, G: 'static + hil::gpio::Pin>
    Nrf51822Component<U, G>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: u32,
        uart: &'static U,
        reset_pin: &'static G,
    ) -> Nrf51822Component<U, G> {
        Nrf51822Component {
            board_kernel: board_kernel,
            driver_num: driver_num,
            uart: uart,
            reset_pin: reset_pin,
        }
    }
}

impl<U: 'static + hil::uart::UartAdvanced<'static>, G: 'static + hil::gpio::Pin> Component
    for Nrf51822Component<U, G>
{
    type StaticInput = ();
    type Output = &'static nrf51822_serialization::Nrf51822Serialization<'static>;

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let nrf_serialization = static_init!(
            nrf51822_serialization::Nrf51822Serialization<'static>,
            nrf51822_serialization::Nrf51822Serialization::new(
                self.uart,
                self.board_kernel.create_grant(self.driver_num, &grant_cap),
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
