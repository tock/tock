// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for CDC-EEM over USB support.
//!
//! This provides a component for using the CDC-EEM driver. This allows for
//! serial communication over USB.
//!
//! Usage
//! -----
//! ```rust
//! static STRINGS: &'static [&str; 3] = &[
//!     "XYZ Corp.",      // Manufacturer
//!     "The Zorpinator", // Product
//!     "Serial No. 5",   // Serial number
//! ];
//! let cdc_eem = components::eem::CdcEemComponent::new(
//!     &nrf52::usbd::USBD,
//!     capsules_extra::usb::usbc_client::MAX_CTRL_PACKET_SIZE_NRF52840,
//!     0x2341,
//!     0x005a,
//!     STRINGS)
//! .finalize(components::cdc_eem_component_static!(nrf52::usbd::Usbd));
//! ```

use core::mem::MaybeUninit;

use kernel::component::Component;
use kernel::hil;

// Setup static space for the objects.
#[macro_export]
macro_rules! cdc_eem_component_static {
    ($U:ty $(,)?) => {{
        (kernel::static_buf!(
            capsules_extra::usb::eem::CdcEem<
                'static,
                $U,
            >
        ),)
    };};
}

pub struct CdcEemComponent<
    U: 'static + hil::usb::UsbController<'static>,
> {
    usb: &'static U,
    max_ctrl_packet_size: u8,
    vendor_id: u16,
    product_id: u16,
    strings: &'static [&'static str; 3],
}

impl<U: 'static + hil::usb::UsbController<'static>>
    CdcEemComponent<U>
{
    pub fn new(
        usb: &'static U,
        max_ctrl_packet_size: u8,
        vendor_id: u16,
        product_id: u16,
        strings: &'static [&'static str; 3],
    ) -> Self {
        Self {
            usb,
            max_ctrl_packet_size,
            vendor_id,
            product_id,
            strings,
        }
    }
}

impl<U: 'static + hil::usb::UsbController<'static>> Component
    for CdcEemComponent<U>
{
    type StaticInput = (
        &'static mut MaybeUninit<
            capsules_extra::usb::eem::CdcEem<'static, U>,
        >,
    );
    type Output =
        &'static capsules_extra::usb::eem::CdcEem<'static, U>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let cdc = s.0.write(capsules_extra::usb::eem::CdcEem::new(
            self.usb,
            self.max_ctrl_packet_size,
            self.vendor_id,
            self.product_id,
            self.strings,
        ));
        self.usb.set_client(cdc);

        cdc
    }
}
