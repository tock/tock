// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Component for USB HID keyboard support.
//!
//! Usage
//! -----
//!
//! ```
//! let strings = static_init!(
//!     [&str; 3],
//!     [
//!         "Nordic Semiconductor", // Manufacturer
//!         "nRF52840dk - TockOS",  // Product
//!         "serial0001",           // Serial number
//!     ]
//! );
//!
//! let (keyboard_hid, keyboard_hid_driver) = components::keyboard_hid::KeyboardHidComponent::new(
//!     board_kernel,
//!     capsules_core::driver::KeyboardHid,
//!     &nrf52840_peripherals.usbd,
//!     0x1915, // Nordic Semiconductor
//!     0x503a,
//!     strings,
//! )
//! .finalize(components::keyboard_hid_component_static!(
//!     nrf52840::usbd::Usbd
//! ));
//!
//! keyboard_hid.enable();
//! keyboard_hid.attach();
//! ```

use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;

// Setup static space for the objects.
#[macro_export]
macro_rules! keyboard_hid_component_static {
    ($U:ty $(,)?) => {{
        let hid = kernel::static_buf!(capsules_extra::usb::keyboard_hid::KeyboardHid<'static, $U>);
        let driver = kernel::static_buf!(
            capsules_extra::usb_hid_driver::UsbHidDriver<
                'static,
                capsules_extra::usb::keyboard_hid::KeyboardHid<'static, $U>,
            >
        );
        let send_buffer = kernel::static_buf!([u8; 64]);
        let recv_buffer = kernel::static_buf!([u8; 64]);

        (hid, driver, send_buffer, recv_buffer)
    };};
}

pub type KeyboardHidComponentType<U> = capsules_extra::usb_hid_driver::UsbHidDriver<
    'static,
    capsules_extra::usb::keyboard_hid::KeyboardHid<'static, U>,
>;

pub struct KeyboardHidComponent<U: 'static + hil::usb::UsbController<'static>> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    usb: &'static U,
    vendor_id: u16,
    product_id: u16,
    strings: &'static [&'static str; 3],
}

impl<U: 'static + hil::usb::UsbController<'static>> KeyboardHidComponent<U> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        usb: &'static U,
        vendor_id: u16,
        product_id: u16,
        strings: &'static [&'static str; 3],
    ) -> KeyboardHidComponent<U> {
        KeyboardHidComponent {
            board_kernel,
            driver_num,
            usb,
            vendor_id,
            product_id,
            strings,
        }
    }
}

impl<U: 'static + hil::usb::UsbController<'static>> Component for KeyboardHidComponent<U> {
    type StaticInput = (
        &'static mut MaybeUninit<capsules_extra::usb::keyboard_hid::KeyboardHid<'static, U>>,
        &'static mut MaybeUninit<
            capsules_extra::usb_hid_driver::UsbHidDriver<
                'static,
                capsules_extra::usb::keyboard_hid::KeyboardHid<'static, U>,
            >,
        >,
        &'static mut MaybeUninit<[u8; 64]>,
        &'static mut MaybeUninit<[u8; 64]>,
    );
    type Output = (
        &'static capsules_extra::usb::keyboard_hid::KeyboardHid<'static, U>,
        &'static capsules_extra::usb_hid_driver::UsbHidDriver<
            'static,
            capsules_extra::usb::keyboard_hid::KeyboardHid<'static, U>,
        >,
    );

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let keyboard_hid =
            s.0.write(capsules_extra::usb::keyboard_hid::KeyboardHid::new(
                self.usb,
                self.vendor_id,
                self.product_id,
                self.strings,
            ));
        self.usb.set_client(keyboard_hid);

        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let send_buffer = s.2.write([0; 64]);
        let recv_buffer = s.3.write([0; 64]);

        let usb_hid_driver = s.1.write(capsules_extra::usb_hid_driver::UsbHidDriver::new(
            Some(keyboard_hid),
            send_buffer,
            recv_buffer,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));

        keyboard_hid.set_client(usb_hid_driver);

        (keyboard_hid, usb_hid_driver)
    }
}
