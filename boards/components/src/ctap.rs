//! Component for CTAP HID over USB support.
//!
//! This provides a component for using the CTAP driver. This allows for
//! Client to Authenticator Protocol Authentication.
//!
//! Usage
//! -----
//! ```rust
//! static STRINGS: &'static [&str; 3] = &[
//!     "XYZ Corp.",     // Manufacturer
//!     "FIDO Key",      // Product
//!     "Serial No. 5",  // Serial number
//! ];
//!
//!     let (ctap, ctap_driver) = components::ctap::CtapComponent::new(
//!         &earlgrey::usbdev::USB,
//!         0x1337, // My important company
//!         0x0DEC, // My device name
//!         strings,
//!         board_kernel,
//!         ctap_send_buffer,
//!         ctap_recv_buffer,
//!     )
//!     .finalize(components::ctap_component_static!(lowrisc::usbdev::Usb));
//!
//!     ctap.enable();
//!     ctap.attach();
//! ```

use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;

// Setup static space for the objects.
#[macro_export]
macro_rules! ctap_component_static {
    ($U:ty $(,)?) => {{
        let hid = kernel::static_buf!(extra_capsules::usb::ctap::CtapHid<'static, $U>);
        let driver = kernel::static_buf!(
            extra_capsules::ctap::CtapDriver<
                'static,
                extra_capsules::usb::ctap::CtapHid<'static, $U>,
            >
        );
        let send_buffer = kernel::static_buf!([u8; 64]);
        let recv_buffer = kernel::static_buf!([u8; 64]);

        (hid, driver, send_buffer, recv_buffer)
    };};
}

pub struct CtapComponent<U: 'static + hil::usb::UsbController<'static>> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    usb: &'static U,
    vendor_id: u16,
    product_id: u16,
    strings: &'static [&'static str; 3],
}

impl<U: 'static + hil::usb::UsbController<'static>> CtapComponent<U> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        usb: &'static U,
        vendor_id: u16,
        product_id: u16,
        strings: &'static [&'static str; 3],
    ) -> CtapComponent<U> {
        CtapComponent {
            board_kernel,
            driver_num,
            usb,
            vendor_id,
            product_id,
            strings,
        }
    }
}

impl<U: 'static + hil::usb::UsbController<'static>> Component for CtapComponent<U> {
    type StaticInput = (
        &'static mut MaybeUninit<extra_capsules::usb::ctap::CtapHid<'static, U>>,
        &'static mut MaybeUninit<
            extra_capsules::ctap::CtapDriver<
                'static,
                extra_capsules::usb::ctap::CtapHid<'static, U>,
            >,
        >,
        &'static mut MaybeUninit<[u8; 64]>,
        &'static mut MaybeUninit<[u8; 64]>,
    );
    type Output = (
        &'static extra_capsules::usb::ctap::CtapHid<'static, U>,
        &'static extra_capsules::ctap::CtapDriver<
            'static,
            extra_capsules::usb::ctap::CtapHid<'static, U>,
        >,
    );

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let ctap = s.0.write(extra_capsules::usb::ctap::CtapHid::new(
            self.usb,
            self.vendor_id,
            self.product_id,
            self.strings,
        ));
        self.usb.set_client(ctap);

        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let send_buffer = s.2.write([0; 64]);
        let recv_buffer = s.3.write([0; 64]);

        let ctap_driver = s.1.write(extra_capsules::ctap::CtapDriver::new(
            Some(ctap),
            send_buffer,
            recv_buffer,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));

        ctap.set_client(ctap_driver);

        (ctap, ctap_driver)
    }
}
