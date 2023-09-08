// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Keyboard USB HID device

use super::descriptors;
use super::descriptors::Buffer64;
use super::descriptors::DescriptorType;
use super::descriptors::EndpointAddress;
use super::descriptors::EndpointDescriptor;
use super::descriptors::HIDCountryCode;
use super::descriptors::HIDDescriptor;
use super::descriptors::HIDSubordinateDescriptor;
use super::descriptors::InterfaceDescriptor;
use super::descriptors::ReportDescriptor;
use super::descriptors::TransferDirection;
use super::usbc_client_ctrl::ClientCtrl;

use kernel::hil;
use kernel::hil::usb::TransferType;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::ErrorCode;

/// Use 1 Interrupt transfer IN/OUT endpoint
const ENDPOINT_NUM: usize = 1;

const IN_BUFFER: usize = 0;

static LANGUAGES: &[u16; 1] = &[
    0x0409, // English (United States)
];
/// Max packet size specified by spec
pub const MAX_CTRL_PACKET_SIZE: u8 = 64;

const N_ENDPOINTS: usize = 1;

/// The HID report descriptor for keyboard from
/// https://www.usb.org/sites/default/files/hid1_11.pdf
static REPORT_DESCRIPTOR: &[u8] = &[
    0x05, 0x01, // Usage Page (Generic Desktop),
    0x09, 0x06, // Usage (Keyboard),
    0xA1, 0x01, // Collection (Application),
    0x75, 0x01, // Report Size (1),
    0x95, 0x08, // Report Count (8),
    0x05, 0x07, // Usage Page (Key Codes),
    0x19, 0xE0, // Usage Minimum (224),
    0x29, 0xE7, // Usage Maximum (231),
    0x15, 0x00, // Logical Minimum (0),
    0x25, 0x01, // Logical Maximum (1),
    0x81, 0x02, // Input (Data, Variable, Absolute),
    // ;Modifier byte
    0x95, 0x01, // Report Count (1),
    0x75, 0x08, // Report Size (8),
    0x81, 0x03, // Input (Constant),
    // ;Reserved byte
    0x95, 0x05, // Report Count (5),
    0x75, 0x01, // Report Size (1),
    0x05, 0x08, // Usage Page (LEDs),
    0x19, 0x01, // Usage Minimum (1),
    0x29, 0x05, // Usage Maximum (5),
    0x91, 0x02, // Output (Data, Variable, Absolute),
    // ;LED report
    0x95, 0x01, // Report Count (1),
    0x75, 0x03, // Report Size (3),
    0x91, 0x03, // Output (Constant), ;LED report
    // padding
    0x95, 0x06, //................// Report Count (6),
    0x75, 0x08, // Report Size (8),
    0x15, 0x00, // Logical Minimum (0),
    0x25, 0x68, // Logical Maximum(104),
    0x05, 0x07, // Usage Page (Key Codes),
    0x19, 0x00, // Usage Minimum (0),
    0x29, 0x68, // Usage Maximum (104),
    0x81, 0x00, // Input (Data, Array),
    0xc0, // End Collection
];

static REPORT: ReportDescriptor<'static> = ReportDescriptor {
    desc: REPORT_DESCRIPTOR,
};

static SUB_HID_DESCRIPTOR: &[HIDSubordinateDescriptor] = &[HIDSubordinateDescriptor {
    typ: DescriptorType::Report,
    len: REPORT_DESCRIPTOR.len() as u16,
}];

static HID_DESCRIPTOR: HIDDescriptor<'static> = HIDDescriptor {
    hid_class: 0x0111,
    country_code: HIDCountryCode::NotSupported,
    sub_descriptors: SUB_HID_DESCRIPTOR,
};

/// Implementation of the CTAP HID (Human Interface Device)
pub struct KeyboardHid<'a, U: 'a> {
    /// Helper USB client library for handling many USB operations.
    client_ctrl: ClientCtrl<'a, 'static, U>,

    /// 64 byte buffers for each endpoint.
    buffers: [Buffer64; N_ENDPOINTS],

    client: OptionalCell<&'a dyn hil::usb_hid::Client<'a, [u8; 64]>>,

    /// A buffer to hold the data we want to send
    send_buffer: TakeCell<'static, [u8; 64]>,
}

impl<'a, U: hil::usb::UsbController<'a>> KeyboardHid<'a, U> {
    pub fn new(
        controller: &'a U,
        vendor_id: u16,
        product_id: u16,
        strings: &'static [&'static str; 3],
    ) -> Self {
        let interfaces: &mut [InterfaceDescriptor] = &mut [InterfaceDescriptor {
            interface_number: 0,
            interface_class: 0x03,    // HID
            interface_subclass: 0x01, // Boot subclass
            interface_protocol: 0x01, // Keyboard
            ..InterfaceDescriptor::default()
        }];

        let endpoints: &[&[EndpointDescriptor]] = &[&[EndpointDescriptor {
            endpoint_address: EndpointAddress::new_const(
                ENDPOINT_NUM,
                TransferDirection::DeviceToHost,
            ),
            transfer_type: TransferType::Interrupt,
            max_packet_size: 8,
            interval: 10,
        }]];

        let (device_descriptor_buffer, other_descriptor_buffer) =
            descriptors::create_descriptor_buffers(
                descriptors::DeviceDescriptor {
                    vendor_id: vendor_id,
                    product_id: product_id,
                    manufacturer_string: 1,
                    product_string: 2,
                    serial_number_string: 3,
                    max_packet_size_ep0: MAX_CTRL_PACKET_SIZE,
                    ..descriptors::DeviceDescriptor::default()
                },
                descriptors::ConfigurationDescriptor {
                    attributes: descriptors::ConfigurationAttributes::new(true, true),
                    max_power: 0x32,
                    ..descriptors::ConfigurationDescriptor::default()
                },
                interfaces,
                endpoints,
                Some(&HID_DESCRIPTOR),
                None,
            );

        KeyboardHid {
            client_ctrl: ClientCtrl::new(
                controller,
                device_descriptor_buffer,
                other_descriptor_buffer,
                Some(&HID_DESCRIPTOR),
                Some(&REPORT),
                LANGUAGES,
                strings,
            ),
            buffers: [Buffer64::default()],
            client: OptionalCell::empty(),
            send_buffer: TakeCell::empty(),
        }
    }

    #[inline]
    fn controller(&self) -> &'a U {
        self.client_ctrl.controller()
    }

    pub fn set_client(&'a self, client: &'a dyn hil::usb_hid::Client<'a, [u8; 64]>) {
        self.client.set(client);
    }
}

impl<'a, U: hil::usb::UsbController<'a>> hil::usb_hid::UsbHid<'a, [u8; 64]> for KeyboardHid<'a, U> {
    fn send_buffer(
        &'a self,
        send: &'static mut [u8; 64],
    ) -> Result<usize, (ErrorCode, &'static mut [u8; 64])> {
        let len = send.len();

        self.send_buffer.replace(send);
        self.controller().endpoint_resume_in(ENDPOINT_NUM);

        Ok(len)
    }

    fn send_cancel(&'a self) -> Result<&'static mut [u8; 64], ErrorCode> {
        match self.send_buffer.take() {
            Some(buf) => Ok(buf),
            None => Err(ErrorCode::BUSY),
        }
    }

    // Keyboard doesn't use receive so this is unimplemented.
    fn receive_buffer(
        &'a self,
        _recv: &'static mut [u8; 64],
    ) -> Result<(), (ErrorCode, &'static mut [u8; 64])> {
        Ok(())
    }

    // Keyboard doesn't use receive so this is unimplemented.
    fn receive_cancel(&'a self) -> Result<&'static mut [u8; 64], ErrorCode> {
        Err(ErrorCode::BUSY)
    }
}

impl<'a, U: hil::usb::UsbController<'a>> hil::usb::Client<'a> for KeyboardHid<'a, U> {
    fn enable(&'a self) {
        // Set up the default control endpoint
        self.client_ctrl.enable();

        // Setup buffers for IN data transfer.
        self.controller()
            .endpoint_set_in_buffer(ENDPOINT_NUM, &self.buffers[IN_BUFFER].buf);
        self.controller()
            .endpoint_in_out_enable(TransferType::Interrupt, ENDPOINT_NUM);
    }

    fn attach(&'a self) {
        self.client_ctrl.attach();
    }

    fn bus_reset(&'a self) {}

    /// Handle a Control Setup transaction.
    fn ctrl_setup(&'a self, endpoint: usize) -> hil::usb::CtrlSetupResult {
        self.client_ctrl.ctrl_setup(endpoint)
    }

    /// Handle a Control In transaction
    fn ctrl_in(&'a self, endpoint: usize) -> hil::usb::CtrlInResult {
        self.client_ctrl.ctrl_in(endpoint)
    }

    /// Handle a Control Out transaction
    fn ctrl_out(&'a self, _endpoint: usize, _packet_bytes: u32) -> hil::usb::CtrlOutResult {
        // self.client_ctrl.ctrl_out(endpoint, packet_bytes)
        hil::usb::CtrlOutResult::Ok
    }

    fn ctrl_status(&'a self, endpoint: usize) {
        self.client_ctrl.ctrl_status(endpoint)
    }

    /// Handle the completion of a Control transfer
    fn ctrl_status_complete(&'a self, endpoint: usize) {
        self.client_ctrl.ctrl_status_complete(endpoint)
    }

    /// Handle a Bulk/Interrupt IN transaction.
    ///
    /// This is called when we can send data to the host. It should get called
    /// when we tell the controller we want to resume the IN endpoint (meaning
    /// we know we have data to send) and afterwards until we return
    /// `hil::usb::InResult::Delay` from this function. That means we can use
    /// this as a callback to mean that the transmission finished by waiting
    /// until this function is called when we don't have anything left to send.
    fn packet_in(&'a self, transfer_type: TransferType, _endpoint: usize) -> hil::usb::InResult {
        match transfer_type {
            TransferType::Interrupt => {
                self.send_buffer
                    .take()
                    .map_or(hil::usb::InResult::Delay, |buf| {
                        // Get packet that we have shared with the underlying
                        // USB stack to copy the tx into.
                        let packet = &self.buffers[IN_BUFFER].buf;

                        // Copy from the TX buffer to the outgoing USB packet.
                        // Our endpoint supports exactly 8 bytes so we use that
                        // length.
                        for i in 0..8 {
                            packet[i].set(buf[i]);
                        }

                        // Put the TX buffer back so we can keep sending from
                        // it.
                        self.send_buffer.replace(buf);

                        // Return that we have data to send.
                        hil::usb::InResult::Packet(8)
                    })
            }
            TransferType::Bulk | TransferType::Control | TransferType::Isochronous => {
                hil::usb::InResult::Error
            }
        }
    }

    /// Handle a Bulk/Interrupt OUT transaction
    ///
    /// Unused for keyboard.
    fn packet_out(
        &'a self,
        transfer_type: TransferType,
        _endpoint: usize,
        _packet_bytes: u32,
    ) -> hil::usb::OutResult {
        match transfer_type {
            TransferType::Interrupt => hil::usb::OutResult::Ok,

            TransferType::Bulk | TransferType::Control | TransferType::Isochronous => {
                hil::usb::OutResult::Error
            }
        }
    }

    fn packet_transmitted(&'a self, endpoint: usize) {
        self.send_buffer.take().map(|buf| {
            self.client.map(move |client| {
                client.packet_transmitted(Ok(()), buf, endpoint);
            });
        });
    }
}
