//! Component for USB
//!
//! This provides one Component, UsbComponent, which implements
//! a userspace syscall interface to the USB peripheral on the EarlGrey SoC.
//!
//! Usage
//! -----
//! ```rust
//! let usb = UsbComponent::new().finalize(());
//! ```

#![allow(dead_code)] // Components are intended to be conditionally included

use earlgrey::usbdev::Usb;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::static_init;

pub struct UsbComponent {
    board_kernel: &'static kernel::Kernel,
    usb: &'static Usb<'static>,
}

impl UsbComponent {
    pub fn new(usb: &'static Usb, board_kernel: &'static kernel::Kernel) -> Self {
        Self { usb, board_kernel }
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
            capsules::usb::usbc_client::Client::new(
                self.usb,
                capsules::usb::usbc_client::MAX_CTRL_PACKET_SIZE_EARLGREY
            )
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
