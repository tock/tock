// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

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

use capsules_extra::nrf51822_serialization::Nrf51822Serialization;
use core::mem::MaybeUninit;
use kernel::capabilities::MemoryAllocationCapability;
use kernel::component::Component;
use kernel::hil;

#[macro_export]
macro_rules! nrf51822_component_static {
    () => {{
        let nrf = kernel::static_buf!(
            capsules_extra::nrf51822_serialization::Nrf51822Serialization<'static>
        );
        let write_buffer =
            kernel::static_buf!([u8; capsules_extra::nrf51822_serialization::WRITE_BUF_LEN]);
        let read_buffer =
            kernel::static_buf!([u8; capsules_extra::nrf51822_serialization::READ_BUF_LEN]);

        (nrf, write_buffer, read_buffer)
    };};
}

pub struct Nrf51822Component<
    U: 'static + hil::uart::UartAdvanced<'static>,
    G: 'static + hil::gpio::Pin,
    CAP: MemoryAllocationCapability + 'static,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    uart: &'static U,
    reset_pin: &'static G,
    mem_cap: CAP,
}

impl<
    U: 'static + hil::uart::UartAdvanced<'static>,
    G: 'static + hil::gpio::Pin,
    CAP: MemoryAllocationCapability + 'static,
> Nrf51822Component<U, G, CAP>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        uart: &'static U,
        reset_pin: &'static G,
        mem_cap: CAP,
    ) -> Nrf51822Component<U, G, CAP> {
        Nrf51822Component {
            board_kernel,
            driver_num,
            uart,
            reset_pin,
            mem_cap,
        }
    }
}

impl<
    U: 'static + hil::uart::UartAdvanced<'static>,
    G: 'static + hil::gpio::Pin,
    CAP: MemoryAllocationCapability + 'static,
> Component for Nrf51822Component<U, G, CAP>
{
    type StaticInput = (
        &'static mut MaybeUninit<Nrf51822Serialization<'static>>,
        &'static mut MaybeUninit<[u8; capsules_extra::nrf51822_serialization::WRITE_BUF_LEN]>,
        &'static mut MaybeUninit<[u8; capsules_extra::nrf51822_serialization::READ_BUF_LEN]>,
    );
    type Output = &'static Nrf51822Serialization<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let write_buffer =
            s.1.write([0; capsules_extra::nrf51822_serialization::WRITE_BUF_LEN]);
        let read_buffer =
            s.2.write([0; capsules_extra::nrf51822_serialization::READ_BUF_LEN]);

        let nrf_serialization = s.0.write(Nrf51822Serialization::new(
            self.uart,
            self.board_kernel
                .create_grant(self.driver_num, &self.mem_cap),
            self.reset_pin,
            write_buffer,
            read_buffer,
        ));
        hil::uart::Transmit::set_transmit_client(self.uart, nrf_serialization);
        hil::uart::Receive::set_receive_client(self.uart, nrf_serialization);
        nrf_serialization
    }
}
