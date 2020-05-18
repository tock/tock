//! Component for USB
//!
//! This provides one Component, UsbComponent, which implements
//! a userspace syscall interface to the USB peripheral on a lowRISC SoC.
//!
//! Usage
//! -----
//! ```rust
//! let usb = UsbComponent::new().finalize(());
//! ```

#![allow(dead_code)] // Components are intended to be conditionally included

use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::static_init;

pub struct UsbComponent {
    board_kernel: &'static kernel::Kernel,
}

impl UsbComponent {
    pub fn new(board_kernel: &'static kernel::Kernel) -> UsbComponent {
        UsbComponent { board_kernel }
    }
}

impl Component for UsbComponent {
    type StaticInput = ();
    type Output = &'static capsules::usb::usb_user::UsbSyscallDriver<
        'static,
        capsules::usb::usbc_client::Client<'static, lowrisc::usbdev::Usb<'static>>,
    >;

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        // Configure the USB controller
        let usb_client = static_init!(
            capsules::usb::usbc_client::Client<'static, lowrisc::usbdev::Usb<'static>>,
            capsules::usb::usbc_client::Client::new(&ibex::usbdev::USB)
        );

        // Configure the USB userspace driver
        let usb_driver = static_init!(
            capsules::usb::usb_user::UsbSyscallDriver<
                'static,
                capsules::usb::usbc_client::Client<'static, lowrisc::usbdev::Usb<'static>>,
            >,
            capsules::usb::usb_user::UsbSyscallDriver::new(
                usb_client,
                self.board_kernel.create_grant(&grant_cap)
            )
        );

        usb_driver
    }
}
