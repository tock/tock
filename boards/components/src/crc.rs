//! Component for CRC syscall interface.
//!
//! This provides one Component, `CrcComponent`, which implements a
//! userspace syscall interface to the CRC peripheral.
//!
//! Usage
//! -----
//! ```rust
//! let crc = components::crc::CrcComponent::new(board_kernel, &sam4l::crccu::CRCCU)
//!     .finalize(components::crc_component_helper!(sam4l::crccu::Crccu));
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

#![allow(dead_code)] // Components are intended to be conditionally included

use core::mem::MaybeUninit;

use capsules::crc;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! crc_component_helper {
    ($C:ty) => {{
        use capsules::crc;
        use core::mem::MaybeUninit;
        static mut BUF: MaybeUninit<crc::Crc<'static, $C>> = MaybeUninit::uninit();
        &mut BUF
    };};
}

pub struct CrcComponent<C: 'static + hil::crc::CRC> {
    board_kernel: &'static kernel::Kernel,
    crc: &'static C,
}

impl<C: 'static + hil::crc::CRC> CrcComponent<C> {
    pub fn new(board_kernel: &'static kernel::Kernel, crc: &'static C) -> CrcComponent<C> {
        CrcComponent {
            board_kernel: board_kernel,
            crc: crc,
        }
    }
}

impl<C: 'static + hil::crc::CRC> Component for CrcComponent<C> {
    type StaticInput = &'static mut MaybeUninit<crc::Crc<'static, C>>;
    type Output = &'static crc::Crc<'static, C>;

    unsafe fn finalize(&mut self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = capabilities::MemoryAllocationCapability::new();

        let crc = static_init_half!(
            static_buffer,
            crc::Crc<'static, C>,
            crc::Crc::new(self.crc, self.board_kernel.create_grant(&grant_cap))
        );

        crc
    }
}
