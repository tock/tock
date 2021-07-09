//! Component for Crc syscall interface.
//!
//! This provides one Component, `CrcComponent`, which implements a
//! userspace syscall interface to the Crc peripheral.
//!
//! Usage
//! -----
//! ```rust
//! let crc = components::crc::CrcComponent::new(board_kernel, &sam4l::crccu::CrcCU)
//!     .finalize(components::crc_component_helper!(sam4l::crccu::Crccu));
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Author: Leon Schuermann  <leon@is.currently.online>
// Last modified: 6/2/2021

use core::mem::MaybeUninit;

use capsules::crc;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::crc::Crc;
use kernel::{static_init, static_init_half};

// Setup static space for the objects.
#[macro_export]
macro_rules! crc_component_helper {
    ($C:ty $(,)?) => {{
        use capsules::crc;
        use core::mem::MaybeUninit;
        static mut BUF: MaybeUninit<crc::CrcDriver<'static, $C>> = MaybeUninit::uninit();
        &mut BUF
    };};
}

pub struct CrcComponent<C: 'static + Crc<'static>> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    crc: &'static C,
}

impl<C: 'static + Crc<'static>> CrcComponent<C> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        crc: &'static C,
    ) -> CrcComponent<C> {
        CrcComponent {
            board_kernel: board_kernel,
            driver_num: driver_num,
            crc: crc,
        }
    }
}

impl<C: 'static + Crc<'static>> Component for CrcComponent<C> {
    type StaticInput = &'static mut MaybeUninit<crc::CrcDriver<'static, C>>;
    type Output = &'static crc::CrcDriver<'static, C>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let crc_buf = static_init!(
            [u8; crc::DEFAULT_CRC_BUF_LENGTH],
            [0; crc::DEFAULT_CRC_BUF_LENGTH]
        );

        let crc = static_init_half!(
            static_buffer,
            crc::CrcDriver<'static, C>,
            crc::CrcDriver::new(
                self.crc,
                crc_buf,
                self.board_kernel.create_grant(self.driver_num, &grant_cap)
            )
        );

        self.crc.set_client(crc);

        crc
    }
}
