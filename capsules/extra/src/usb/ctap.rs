// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Client to Authenticator Protocol CTAPv2 over USB HID
//!
//! Based on the spec avaliable at: <https://fidoalliance.org/specs/fido-v2.0-id-20180227/fido-client-to-authenticator-protocol-v2.0-id-20180227.html>

use core::cmp;

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

const OUT_BUFFER: usize = 0;
const IN_BUFFER: usize = 1;

static LANGUAGES: &[u16; 1] = &[
    0x0409, // English (United States)
];
/// Max packet size specified by spec
pub const MAX_CTRL_PACKET_SIZE: u8 = 64;

const N_ENDPOINTS: usize = 2;

/// The HID report descriptor for CTAP
/// This is a combination of:
///     - the CTAP spec, example 8
///     - USB HID spec examples
/// Plus it matches: <https://chromium.googlesource.com/chromiumos/platform2/+/master/u2fd/u2fhid.cc>
static REPORT_DESCRIPTOR: &[u8] = &[
    0x06, 0xD0, 0xF1, // HID_UsagePage ( FIDO_USAGE_PAGE ),
    0x09, 0x01, // HID_Usage ( FIDO_USAGE_CTAPHID ),
    0xA1, 0x01, // HID_Collection ( HID_Application ),
    0x09, 0x20, // HID_Usage ( FIDO_USAGE_DATA_IN ),
    0x15, 0x00, // HID_LogicalMin ( 0 ),
    0x26, 0xFF, 0x00, // HID_LogicalMaxS ( 0xff ),
    0x75, 0x08, // HID_ReportSize ( 8 ),
    0x95, 0x40, // HID_ReportCount ( HID_INPUT_REPORT_BYTES ),
    0x81, 0x02, // HID_Input ( HID_Data | HID_Absolute | HID_Variable ),
    0x09, 0x21, // HID_Usage ( FIDO_USAGE_DATA_OUT ),
    0x15, 0x00, // HID_LogicalMin ( 0 ),
    0x26, 0xFF, 0x00, // HID_LogicalMaxS ( 0xff ),
    0x75, 0x08, // HID_ReportSize ( 8 ),
    0x95, 0x40, // HID_ReportCount ( HID_OUTPUT_REPORT_BYTES ),
    0x91, 0x02, // HID_Output ( HID_Data | HID_Absolute | HID_Variable ),
    0xC0, // HID_EndCollection
];

static REPORT: ReportDescriptor<'static> = ReportDescriptor {
    desc: REPORT_DESCRIPTOR,
};

static SUB_HID_DESCRIPTOR: &[HIDSubordinateDescriptor] = &[HIDSubordinateDescriptor {
    typ: DescriptorType::Report,
    len: REPORT_DESCRIPTOR.len() as u16,
}];

static HID_DESCRIPTOR: HIDDescriptor<'static> = HIDDescriptor {
    hid_class: 0x0110,
    country_code: HIDCountryCode::NotSupported,
    sub_descriptors: SUB_HID_DESCRIPTOR,
};

/// Implementation of the CTAP HID (Human Interface Device)
pub struct CtapHid<'a, U: 'a> {
    /// Helper USB client library for handling many USB operations.
    client_ctrl: ClientCtrl<'a, 'static, U>,

    /// 64 byte buffers for each endpoint.
    buffers: [Buffer64; N_ENDPOINTS],

    client: OptionalCell<&'a dyn hil::usb_hid::Client<'a, [u8; 64]>>,

    /// A buffer to hold the data we want to send
    send_buffer: TakeCell<'static, [u8; 64]>,

    /// A holder for the buffer to receive bytes into. We use this as a flag as
    /// well, if we have a buffer then we are actively doing a receive.
    recv_buffer: TakeCell<'static, [u8; 64]>,
}

impl<'a, U: hil::usb::UsbController<'a>> CtapHid<'a, U> {
    pub fn new(
        controller: &'a U,
        vendor_id: u16,
        product_id: u16,
        strings: &'static [&'static str; 3],
    ) -> Self {
        let interfaces: &mut [InterfaceDescriptor] = &mut [InterfaceDescriptor {
            interface_number: 0,
            interface_class: 0x03,    // HID
            interface_subclass: 0x00, // No subcall
            interface_protocol: 0x00, // No protocol
            ..InterfaceDescriptor::default()
        }];

        let endpoints: &[&[EndpointDescriptor]] = &[&[
            EndpointDescriptor {
                endpoint_address: EndpointAddress::new_const(
                    ENDPOINT_NUM,
                    TransferDirection::DeviceToHost,
                ),
                transfer_type: TransferType::Interrupt,
                max_packet_size: 64,
                interval: 5,
            },
            EndpointDescriptor {
                endpoint_address: EndpointAddress::new_const(
                    ENDPOINT_NUM,
                    TransferDirection::HostToDevice,
                ),
                transfer_type: TransferType::Interrupt,
                max_packet_size: 64,
                interval: 5,
            },
        ]];

        let (device_descriptor_buffer, other_descriptor_buffer) =
            descriptors::create_descriptor_buffers(
                descriptors::DeviceDescriptor {
                    vendor_id,
                    product_id,
                    manufacturer_string: 1,
                    product_string: 2,
                    serial_number_string: 3,
                    class: 0x03, // Class: HID
                    max_packet_size_ep0: MAX_CTRL_PACKET_SIZE,
                    ..descriptors::DeviceDescriptor::default()
                },
                descriptors::ConfigurationDescriptor::default(),
                interfaces,
                endpoints,
                Some(&HID_DESCRIPTOR),
                None,
            );

        CtapHid {
            client_ctrl: ClientCtrl::new(
                controller,
                device_descriptor_buffer,
                other_descriptor_buffer,
                Some(&HID_DESCRIPTOR),
                Some(&REPORT),
                LANGUAGES,
                strings,
            ),
            buffers: [Buffer64::default(), Buffer64::default()],
            client: OptionalCell::empty(),
            send_buffer: TakeCell::empty(),
            recv_buffer: TakeCell::empty(),
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

impl<'a, U: hil::usb::UsbController<'a>> hil::usb_hid::UsbHid<'a, [u8; 64]> for CtapHid<'a, U> {
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

    fn receive_buffer(
        &'a self,
        recv: &'static mut [u8; 64],
    ) -> Result<(), (ErrorCode, &'static mut [u8; 64])> {
        self.recv_buffer.replace(recv);
        self.controller().endpoint_resume_out(ENDPOINT_NUM);
        Ok(())
    }

    fn receive_cancel(&'a self) -> Result<&'static mut [u8; 64], ErrorCode> {
        match self.recv_buffer.take() {
            Some(buf) => Ok(buf),
            None => Err(ErrorCode::BUSY),
        }
    }
}

impl<'a, U: hil::usb::UsbController<'a>> hil::usb::Client<'a> for CtapHid<'a, U> {
    fn enable(&'a self) {
        // Set up the default control endpoint
        self.client_ctrl.enable();

        // Setup buffers for IN and OUT data transfer.
        self.controller()
            .endpoint_set_out_buffer(ENDPOINT_NUM, &self.buffers[OUT_BUFFER].buf);
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
    fn ctrl_out(&'a self, endpoint: usize, packet_bytes: u32) -> hil::usb::CtrlOutResult {
        self.client_ctrl.ctrl_out(endpoint, packet_bytes)
    }

    fn ctrl_status(&'a self, endpoint: usize) {
        self.client_ctrl.ctrl_status(endpoint)
    }

    /// Handle the completion of a Control transfer
    fn ctrl_status_complete(&'a self, endpoint: usize) {
        if self.send_buffer.is_some() {
            self.controller().endpoint_resume_in(ENDPOINT_NUM);
        }

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
                        for i in 0..64 {
                            packet[i].set(buf[i]);
                        }

                        // Put the TX buffer back so we can keep sending from it.
                        self.send_buffer.replace(buf);

                        // Return that we have data to send.
                        hil::usb::InResult::Packet(64)
                    })
            }
            TransferType::Bulk | TransferType::Control | TransferType::Isochronous => {
                panic!("Transfer protocol not supported by CTAP v2");
            }
        }
    }

    /// Handle a Bulk/Interrupt OUT transaction
    ///
    /// This is data going from the host to the device (us)
    fn packet_out(
        &'a self,
        transfer_type: TransferType,
        endpoint: usize,
        packet_bytes: u32,
    ) -> hil::usb::OutResult {
        match transfer_type {
            TransferType::Interrupt => {
                // If we have a receive buffer we can copy the incoming data in.
                // If we do not have a buffer, then we apply back pressure by
                // returning `hil::usb::OutResult::Delay` to the USB stack until
                // we get a receive call.
                self.recv_buffer
                    .take()
                    .map_or(hil::usb::OutResult::Delay, |buf| {
                        // How many more bytes can we store in our RX buffer?
                        let copy_length = cmp::min(packet_bytes as usize, buf.len());

                        // Do the copy into the RX buffer.
                        let packet = &self.buffers[OUT_BUFFER].buf;
                        for i in 0..copy_length {
                            buf[i] = packet[i].get();
                        }

                        // Notify the client
                        self.client.map(move |client| {
                            client.packet_received(Ok(()), buf, endpoint);
                        });

                        hil::usb::OutResult::Ok
                    })
            }
            TransferType::Bulk | TransferType::Control | TransferType::Isochronous => {
                panic!("Transfer protocol not supported by CTAP v2");
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
