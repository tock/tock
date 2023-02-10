//! Generic component for initializing a USB device given a USBController.
//!
//! This provides one Component, UsbComponent, which implements
//! A userspace syscall interface to a USB peripheral.
//!
//! Usage
//! -----
//! ```rust
//! let usb_driver = components::usb::UsbComponent::new(
//!     board_kernel,
//!     extra_capsules::usb::usb_user::DRIVER_NUM,
//!     &peripherals.usbc,
//! )
//! .finalize(components::usb_component_static!(sam4l::usbc::Usbc));
//! ```

use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::usb::UsbController;

#[macro_export]
macro_rules! usb_component_static {
    ($U:ty $(,)?) => {{
        let usb_client = kernel::static_buf!(extra_capsules::usb::usbc_client::Client<'static, $U>);
        let usb_driver = kernel::static_buf!(
            extra_capsules::usb::usb_user::UsbSyscallDriver<
                'static,
                extra_capsules::usb::usbc_client::Client<'static, $U>,
            >
        );
        (usb_client, usb_driver)
    }};
}

pub struct UsbComponent<U: UsbController<'static> + 'static> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    usbc: &'static U,
}

impl<U: UsbController<'static> + 'static> UsbComponent<U> {
    pub fn new(board_kernel: &'static kernel::Kernel, driver_num: usize, usbc: &'static U) -> Self {
        Self {
            board_kernel,
            driver_num,
            usbc,
        }
    }
}

impl<U: UsbController<'static> + 'static> Component for UsbComponent<U> {
    type StaticInput = (
        &'static mut MaybeUninit<extra_capsules::usb::usbc_client::Client<'static, U>>,
        &'static mut MaybeUninit<
            extra_capsules::usb::usb_user::UsbSyscallDriver<
                'static,
                extra_capsules::usb::usbc_client::Client<'static, U>,
            >,
        >,
    );
    type Output = &'static extra_capsules::usb::usb_user::UsbSyscallDriver<
        'static,
        extra_capsules::usb::usbc_client::Client<'static, U>,
    >;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        // Configure the USB controller
        let usb_client = s.0.write(extra_capsules::usb::usbc_client::Client::new(
            &self.usbc,
            extra_capsules::usb::usbc_client::MAX_CTRL_PACKET_SIZE_SAM4L,
        ));
        self.usbc.set_client(usb_client);

        // Configure the USB userspace driver
        let usb_driver =
            s.1.write(extra_capsules::usb::usb_user::UsbSyscallDriver::new(
                usb_client,
                self.board_kernel.create_grant(self.driver_num, &grant_cap),
            ));

        usb_driver
    }
}
