//! Component for CTAP HID over USB support.
//!
//! This provides a component for using the CTAP driver. This allows for
//! Client to Authenticator Protool Authentication
//!
//! Usage
//! -----
//! ```rust
//! static STRINGS: &'static [&str; 3] = &[
//!     "XYZ Corp.",     // Manufacturer
//!     "FIDO Key",      // Product
//!     "Serial No. 5",  // Serial number
//! ];
//!     let ctap_send_buffer = static_init!([u8; 64], [0; 64]);
//!     let ctap_recv_buffer = static_init!([u8; 64], [0; 64]);
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
//!     .finalize(components::usb_ctap_component_helper!(lowrisc::usbdev::Usb));
//!
//!     ctap.enable();
//!     ctap.attach();
//! ```

use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! usb_ctap_component_helper {
    ($U:ty $(,)?) => {{
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<capsules::usb::ctap::CtapHid<'static, $U>> =
            MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<
            capsules::ctap::CtapDriver<'static, capsules::usb::ctap::CtapHid<'static, $U>>,
        > = MaybeUninit::uninit();
        (&mut BUF1, &mut BUF2)
    };};
}

pub struct CtapComponent<U: 'static + hil::usb::UsbController<'static>> {
    board_kernel: &'static kernel::Kernel,
    driver_num: u32,
    usb: &'static U,
    vendor_id: u16,
    product_id: u16,
    strings: &'static [&'static str; 3],
    send_buffer: &'static mut [u8; 64],
    recv_buffer: &'static mut [u8; 64],
}

impl<U: 'static + hil::usb::UsbController<'static>> CtapComponent<U> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: u32,
        usb: &'static U,
        vendor_id: u16,
        product_id: u16,
        strings: &'static [&'static str; 3],
        send_buffer: &'static mut [u8; 64],
        recv_buffer: &'static mut [u8; 64],
    ) -> CtapComponent<U> {
        CtapComponent {
            board_kernel,
            driver_num,
            usb,
            vendor_id,
            product_id,
            strings,
            send_buffer,
            recv_buffer,
        }
    }
}

impl<U: 'static + hil::usb::UsbController<'static>> Component for CtapComponent<U> {
    type StaticInput = (
        &'static mut MaybeUninit<capsules::usb::ctap::CtapHid<'static, U>>,
        &'static mut MaybeUninit<
            capsules::ctap::CtapDriver<'static, capsules::usb::ctap::CtapHid<'static, U>>,
        >,
    );
    type Output = (
        &'static capsules::usb::ctap::CtapHid<'static, U>,
        &'static capsules::ctap::CtapDriver<'static, capsules::usb::ctap::CtapHid<'static, U>>,
    );

    unsafe fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let ctap = static_init_half!(
            s.0,
            capsules::usb::ctap::CtapHid<'static, U>,
            capsules::usb::ctap::CtapHid::new(
                self.usb,
                self.vendor_id,
                self.product_id,
                self.strings
            )
        );
        self.usb.set_client(ctap);

        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let ctap_driver = static_init_half!(
            s.1,
            capsules::ctap::CtapDriver<'static, capsules::usb::ctap::CtapHid<'static, U>>,
            capsules::ctap::CtapDriver::new(
                Some(ctap),
                self.send_buffer,
                self.recv_buffer,
                self.board_kernel.create_grant(self.driver_num, &grant_cap),
            )
        );

        ctap.set_client(ctap_driver);

        (ctap, ctap_driver)
    }
}
