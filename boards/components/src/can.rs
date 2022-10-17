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
//!     capsules::can::DRIVER_NUM,
//!     &peripherals.can1
//! ).finalize(components::can_component_static!(
//!     stm32f429zi::can::Can<'static>
//! ));
//! ```
//!
//! Author: Teona Severin <teona.severin@oxidos.io>

use capsules::can::CanCapsule;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::can;
use kernel::{capabilities, create_capability};

#[macro_export]
macro_rules! can_component_static {
    ($C:ty $(,)?) => {{
        use capsules::can::CanCapsule;
        use core::mem::MaybeUninit;
        use kernel::hil::can;

        static mut CAN_TX_BUF: [u8; can::STANDARD_CAN_PACKET_SIZE] =
            [0; can::STANDARD_CAN_PACKET_SIZE];
        static mut CAN_RX_BUF: [u8; can::STANDARD_CAN_PACKET_SIZE] =
            [0; can::STANDARD_CAN_PACKET_SIZE];
        let can = kernel::static_buf!(capsules::can::CanCapsule<'static, $C>);
        (can, &mut CAN_TX_BUF, &mut CAN_RX_BUF)
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
        &'static mut [u8; can::STANDARD_CAN_PACKET_SIZE],
        &'static mut [u8; can::STANDARD_CAN_PACKET_SIZE],
    );
    type Output = &'static CanCapsule<'static, A>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_can = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let can = static_buffer.0.write(capsules::can::CanCapsule::new(
            self.can,
            grant_can,
            static_buffer.1,
            static_buffer.2,
        ));
        can::Controller::set_client(self.can, Some(can));
        can::Transmit::set_client(self.can, Some(can));
        can::Receive::set_client(self.can, Some(can));

        can
    }
}
