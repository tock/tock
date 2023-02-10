//! Component for communicating with the nRF51822 (BLE).
//!
//! This provides one Component, Nrf51822Component, which implements
//! a system call interface to the nRF51822 for BLE advertisements.
//!
//! Usage
//! -----
//! ```rust
//! let nrf_serialization = Nrf51822Component::new(&sam4l::usart::USART3,
//!                                                &sam4l::gpio::PA[17])
//!     .finalize(components::nrf51822_component_static!());
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

use core::mem::MaybeUninit;
use extra_capsules::nrf51822_serialization::Nrf51822Serialization;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;

#[macro_export]
macro_rules! nrf51822_component_static {
    () => {{
        let nrf = kernel::static_buf!(
            extra_capsules::nrf51822_serialization::Nrf51822Serialization<'static>
        );
        let write_buffer =
            kernel::static_buf!([u8; extra_capsules::nrf51822_serialization::WRITE_BUF_LEN]);
        let read_buffer =
            kernel::static_buf!([u8; extra_capsules::nrf51822_serialization::READ_BUF_LEN]);

        (nrf, write_buffer, read_buffer)
    };};
}

pub struct Nrf51822Component<
    U: 'static + hil::uart::UartAdvanced<'static>,
    G: 'static + hil::gpio::Pin,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    uart: &'static U,
    reset_pin: &'static G,
}

impl<U: 'static + hil::uart::UartAdvanced<'static>, G: 'static + hil::gpio::Pin>
    Nrf51822Component<U, G>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
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
    type StaticInput = (
        &'static mut MaybeUninit<Nrf51822Serialization<'static>>,
        &'static mut MaybeUninit<[u8; extra_capsules::nrf51822_serialization::WRITE_BUF_LEN]>,
        &'static mut MaybeUninit<[u8; extra_capsules::nrf51822_serialization::READ_BUF_LEN]>,
    );
    type Output = &'static Nrf51822Serialization<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let write_buffer =
            s.1.write([0; extra_capsules::nrf51822_serialization::WRITE_BUF_LEN]);
        let read_buffer =
            s.2.write([0; extra_capsules::nrf51822_serialization::READ_BUF_LEN]);

        let nrf_serialization = s.0.write(Nrf51822Serialization::new(
            self.uart,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
            self.reset_pin,
            write_buffer,
            read_buffer,
        ));
        hil::uart::Transmit::set_transmit_client(self.uart, nrf_serialization);
        hil::uart::Receive::set_receive_client(self.uart, nrf_serialization);
        nrf_serialization
    }
}
