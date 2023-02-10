// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022
// Copyright OxidOS Automotive SRL 2022
//
// Author: Teona Severin <teona.severin@oxidos.io>

//! Component for CAN syscall interface.
//!
//! This provides one Component, `CanComponent`, which implements a
//! userspace syscall interface to the Can peripheral.
//!
//! Usage
//! -----
//! ```rust
//! let can = components::can::CanComponent::new(
//!     board_kernel,
//!     extra_capsules::can::DRIVER_NUM,
//!     &peripherals.can1
//! ).finalize(components::can_component_static!(
//!     stm32f429zi::can::Can<'static>
//! ));
//! ```
//!

use core::mem::MaybeUninit;
use extra_capsules::can::CanCapsule;
use kernel::component::Component;
use kernel::hil::can;
use kernel::{capabilities, create_capability};

#[macro_export]
macro_rules! can_component_static {
    ($C:ty $(,)?) => {{
        use core::mem::MaybeUninit;
        use extra_capsules::can::CanCapsule;
        use kernel::hil::can;
        use kernel::static_buf;

        let CAN_TX_BUF = static_buf!([u8; can::STANDARD_CAN_PACKET_SIZE]);
        let CAN_RX_BUF = static_buf!([u8; can::STANDARD_CAN_PACKET_SIZE]);
        let can = static_buf!(extra_capsules::can::CanCapsule<'static, $C>);
        (can, CAN_TX_BUF, CAN_RX_BUF)
    };};
}

pub struct CanComponent<A: 'static + can::Can> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    can: &'static A,
}

impl<A: 'static + can::Can> CanComponent<A> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        can: &'static A,
    ) -> CanComponent<A> {
        CanComponent {
            board_kernel,
            driver_num,
            can,
        }
    }
}

impl<A: 'static + can::Can> Component for CanComponent<A> {
    type StaticInput = (
        &'static mut MaybeUninit<CanCapsule<'static, A>>,
        &'static mut MaybeUninit<[u8; can::STANDARD_CAN_PACKET_SIZE]>,
        &'static mut MaybeUninit<[u8; can::STANDARD_CAN_PACKET_SIZE]>,
    );
    type Output = &'static CanCapsule<'static, A>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_can = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let can = static_buffer.0.write(extra_capsules::can::CanCapsule::new(
            self.can,
            grant_can,
            static_buffer.1.write([0; can::STANDARD_CAN_PACKET_SIZE]),
            static_buffer.2.write([0; can::STANDARD_CAN_PACKET_SIZE]),
        ));
        can::Controller::set_client(self.can, Some(can));
        can::Transmit::set_client(self.can, Some(can));
        can::Receive::set_client(self.can, Some(can));

        can
    }
}
