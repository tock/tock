// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for CDC-ACM over USB support.
//!
//! This provides a component for using the CDC-ACM driver. This allows for
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
//! let cdc_acm = components::cdc::CdcAcmComponent::new(
//!     &nrf52::usbd::USBD,
//!     capsules_extra::usb::usbc_client::MAX_CTRL_PACKET_SIZE_NRF52840,
//!     0x2341,
//!     0x005a,
//!     STRINGS)
//! .finalize(components::cdc_acm_component_static!(nrf52::usbd::Usbd));
//! ```

use core::mem::MaybeUninit;

use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::component::Component;
use kernel::hil;
use kernel::hil::time::Alarm;

// Setup static space for the objects.
#[macro_export]
macro_rules! cdc_acm_component_static {
    ($U:ty, $A:ty $(,)?) => {{
        let alarm = kernel::static_buf!(
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>
        );
        let cdc = kernel::static_buf!(
            capsules_extra::usb::cdc::CdcAcm<
                'static,
                $U,
                capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>,
            >
        );

        (alarm, cdc)
    };};
}

pub struct CdcAcmComponent<
    U: 'static + hil::usb::UsbController<'static>,
    A: 'static + Alarm<'static>,
> {
    usb: &'static U,
    max_ctrl_packet_size: u8,
    vendor_id: u16,
    product_id: u16,
    strings: &'static [&'static str; 3],
    alarm_mux: &'static MuxAlarm<'static, A>,
    host_initiated_function: Option<&'static (dyn Fn() + 'static)>,
}

impl<U: 'static + hil::usb::UsbController<'static>, A: 'static + Alarm<'static>>
    CdcAcmComponent<U, A>
{
    pub fn new(
        usb: &'static U,
        max_ctrl_packet_size: u8,
        vendor_id: u16,
        product_id: u16,
        strings: &'static [&'static str; 3],
        alarm_mux: &'static MuxAlarm<'static, A>,
        host_initiated_function: Option<&'static (dyn Fn() + 'static)>,
    ) -> Self {
        Self {
            usb,
            max_ctrl_packet_size,
            vendor_id,
            product_id,
            strings,
            alarm_mux,
            host_initiated_function,
        }
    }
}

impl<U: 'static + hil::usb::UsbController<'static>, A: 'static + Alarm<'static>> Component
    for CdcAcmComponent<U, A>
{
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<
            capsules_extra::usb::cdc::CdcAcm<'static, U, VirtualMuxAlarm<'static, A>>,
        >,
    );
    type Output =
        &'static capsules_extra::usb::cdc::CdcAcm<'static, U, VirtualMuxAlarm<'static, A>>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let cdc_alarm = s.0.write(VirtualMuxAlarm::new(self.alarm_mux));
        cdc_alarm.setup();

        let cdc = s.1.write(capsules_extra::usb::cdc::CdcAcm::new(
            self.usb,
            self.max_ctrl_packet_size,
            self.vendor_id,
            self.product_id,
            self.strings,
            cdc_alarm,
            self.host_initiated_function,
        ));
        kernel::deferred_call::DeferredCallClient::register(cdc);
        self.usb.set_client(cdc);
        cdc_alarm.set_alarm_client(cdc);

        cdc
    }
}
