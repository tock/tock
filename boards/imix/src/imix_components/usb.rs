//! Component for USB on the imix board.
//!
//! This provides one Component, UsbComponent, which implements
//! a userspace syscall interface to the USB peripheral on the SAM4L.
//!
//! Usage
//! -----
//! ```rust
//! let usb = UsbComponent::new().finalize(());
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

#![allow(dead_code)] // Components are intended to be conditionally included

use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::static_init;

pub struct UsbComponent<'ker> {
    board_kernel: &'ker kernel::Kernel<'ker>,
}

type UsbDevice<'ker> = capsules::usb::usb_user::UsbSyscallDriver<
    'static,
    'ker,
    capsules::usb::usbc_client::Client<'static, sam4l::usbc::Usbc<'static>>,
>;

impl UsbComponent<'ker> {
    pub fn new(board_kernel: &'ker kernel::Kernel<'ker>) -> UsbComponent<'ker> {
        UsbComponent {
            board_kernel: board_kernel,
        }
    }
}

impl Component for UsbComponent<'ker>
where
    'ker: 'static,
{
    type StaticInput = ();
    type Output = &'static UsbDevice<'ker>;

    unsafe fn finalize(&mut self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        // Configure the USB controller
        let usb_client = static_init!(
            capsules::usb::usbc_client::Client<'static, sam4l::usbc::Usbc<'static>>,
            capsules::usb::usbc_client::Client::new(&sam4l::usbc::USBC)
        );
        sam4l::usbc::USBC.set_client(usb_client);

        // Configure the USB userspace driver
        let usb_driver = static_init!(
            capsules::usb::usb_user::UsbSyscallDriver<
                'static,
                '_,
                capsules::usb::usbc_client::Client<'static, sam4l::usbc::Usbc<'static>>,
            >,
            capsules::usb::usb_user::UsbSyscallDriver::new(
                usb_client,
                self.board_kernel.create_grant(&grant_cap)
            )
        );

        usb_driver
    }
}
