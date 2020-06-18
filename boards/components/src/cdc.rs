//! Component for CDC-ACM over USB support.
//!
//! This provides a component for using the CDC-ACM driver. This allows for
//! serial communication over USB.
//!
//! Usage
//! -----
//! ```rust
//! let cdc_acm = components::cdc::CdcAcmComponent::new(
//!     &nrf52::usbd::USBD,
//!     capsules::usb::usbc_client::MAX_CTRL_PACKET_SIZE_NRF52840)
//! .finalize(components::usb_cdc_acm_component_helper!(nrf52::usbd::Usbd));
//! ```

use core::mem::MaybeUninit;

use kernel::component::Component;
use kernel::hil;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! usb_cdc_acm_component_helper {
    ($U:ty) => {{
        use core::mem::MaybeUninit;
        static mut BUF: MaybeUninit<capsules::usb::cdc::CdcAcm<'static, $U>> =
            MaybeUninit::uninit();
        &mut BUF
    };};
}

pub struct CdcAcmComponent<U: 'static + hil::usb::UsbController<'static>> {
    usb: &'static U,
    max_ctrl_packet_size: u8,
}

impl<U: 'static + hil::usb::UsbController<'static>> CdcAcmComponent<U> {
    pub fn new(usb: &'static U, max_ctrl_packet_size: u8) -> CdcAcmComponent<U> {
        CdcAcmComponent {
            usb,
            max_ctrl_packet_size,
        }
    }
}

impl<U: 'static + hil::usb::UsbController<'static>> Component for CdcAcmComponent<U> {
    type StaticInput = &'static mut MaybeUninit<capsules::usb::cdc::CdcAcm<'static, U>>;
    type Output = &'static capsules::usb::cdc::CdcAcm<'static, U>;

    unsafe fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let cdc = static_init_half!(
            s,
            capsules::usb::cdc::CdcAcm<'static, U>,
            capsules::usb::cdc::CdcAcm::new(self.usb, self.max_ctrl_packet_size)
        );
        self.usb.set_client(cdc);

        cdc
    }
}
