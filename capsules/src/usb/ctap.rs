//! Client to Authenticator Protocol CTAPv2 over USB HID
//!
//! Based on the spec avaliable at: https://fidoalliance.org/specs/fido-v2.0-id-20180227/fido-client-to-authenticator-protocol-v2.0-id-20180227.html

use core::cell::Cell;
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

use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::common::cells::VolatileCell;
use kernel::hil;
use kernel::hil::usb::TransferType;
use kernel::ReturnCode;

/// The spec defines 1 Interrupt transfer in endpoint
const ENDPOINT_IN_NUM: usize = 2;
/// The spec defines 1 Interrupt transfer out endpoint
const ENDPOINT_OUT_NUM: usize = 1;

static LANGUAGES: &'static [u16; 1] = &[
    0x0409, // English (United States)
];
/// Max packet size specified by spec
pub const MAX_CTRL_PACKET_SIZE: u8 = 64;

const N_ENDPOINTS: usize = 2;

/// The HID report descriptor for CTAP
/// This is a combinfrom of:
///     - the CTAP spec, example 8
///     - USB HID spec examples
/// Plus it matches: https://chromium.googlesource.com/chromiumos/platform2/+/master/u2fd/u2fhid.cc
static REPORT_DESCRIPTOR: &'static [u8] = &[
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

static SUB_HID_DESCRIPTOR: &'static [HIDSubordinateDescriptor] = &[HIDSubordinateDescriptor {
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

    client: OptionalCell<&'a dyn hil::usb_hid::Client<'a>>,

    /// A buffer to hold the data we want to send
    send_buffer: TakeCell<'static, [u8; 64]>,

    /// A holder for the buffer to receive bytes into. We use this as a flag as
    /// well, if we have a buffer then we are actively doing a receive.
    recv_buffer: TakeCell<'static, [u8; 64]>,
    /// How many bytes the client wants us to receive.
    recv_len: Cell<usize>,
    /// How many bytes we have received so far.
    recv_offset: Cell<usize>,

    saved_endpoint: OptionalCell<usize>,
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
                endpoint_address: EndpointAddress::new_const(0x02, TransferDirection::DeviceToHost),
                transfer_type: TransferType::Interrupt,
                max_packet_size: 64,
                interval: 5,
            },
            EndpointDescriptor {
                endpoint_address: EndpointAddress::new_const(0x01, TransferDirection::HostToDevice),
                transfer_type: TransferType::Interrupt,
                max_packet_size: 64,
                interval: 5,
            },
        ]];

        let (device_descriptor_buffer, other_descriptor_buffer) =
            descriptors::create_descriptor_buffers(
                descriptors::DeviceDescriptor {
                    vendor_id: vendor_id,
                    product_id: product_id,
                    manufacturer_string: 1,
                    product_string: 2,
                    serial_number_string: 3,
                    class: 0x03, // Class: HID
                    max_packet_size_ep0: MAX_CTRL_PACKET_SIZE,
                    ..descriptors::DeviceDescriptor::default()
                },
                descriptors::ConfigurationDescriptor {
                    ..descriptors::ConfigurationDescriptor::default()
                },
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
            recv_len: Cell::new(0),
            recv_offset: Cell::new(0),
            saved_endpoint: OptionalCell::empty(),
        }
    }

    #[inline]
    fn controller(&self) -> &'a U {
        self.client_ctrl.controller()
    }

    #[inline]
    fn buffer(&'a self, i: usize) -> &'a [VolatileCell<u8>; 64] {
        &self.buffers[i - 1].buf
    }

    pub fn set_client(&'a self, client: &'a dyn hil::usb_hid::Client<'a>) {
        self.client.set(client);
    }

    fn can_receive(&'a self) -> bool {
        self.client
            .map(move |client| client.can_receive())
            .unwrap_or(false)
    }
}

impl<'a, U: hil::usb::UsbController<'a>> hil::usb_hid::UsbHid<'a> for CtapHid<'a, U> {
    fn set_recv_buffer(&'a self, recv: &'static mut [u8; 64]) {
        self.recv_buffer.replace(recv);
    }

    fn send_buffer(
        &'a self,
        send: &'static mut [u8; 64],
    ) -> Result<usize, (ReturnCode, &'static mut [u8; 64])> {
        let len = send.len();

        self.send_buffer.replace(send);
        self.controller().endpoint_resume_in(ENDPOINT_IN_NUM);

        Ok(len)
    }

    fn allow_receive(&'a self) {
        if self.saved_endpoint.is_some() {
            // We have saved data from before, let's send it.
            if self.can_receive() {
                self.recv_buffer.take().map(|buf| {
                    self.client.map(move |client| {
                        client.packet_received(
                            buf,
                            self.recv_offset.get(),
                            self.saved_endpoint.take().unwrap(),
                        );
                    });
                });
                // Reset the offset
                self.recv_offset.set(0);
            }
        } else {
            // If we have nothing to process, accept more data
            self.controller().endpoint_resume_out(ENDPOINT_OUT_NUM);
        }
    }
}

impl<'a, U: hil::usb::UsbController<'a>> hil::usb::Client<'a> for CtapHid<'a, U> {
    fn enable(&'a self) {
        // Set up the default control endpoint
        self.client_ctrl.enable();

        // Setup buffers for IN and OUT data transfer.
        self.controller()
            .endpoint_set_in_buffer(ENDPOINT_IN_NUM, self.buffer(ENDPOINT_IN_NUM));
        self.controller()
            .endpoint_in_enable(TransferType::Interrupt, ENDPOINT_IN_NUM);

        self.controller()
            .endpoint_set_out_buffer(ENDPOINT_OUT_NUM, self.buffer(ENDPOINT_OUT_NUM));
        self.controller()
            .endpoint_out_enable(TransferType::Interrupt, ENDPOINT_OUT_NUM);
    }

    fn attach(&'a self) {
        self.client_ctrl.attach();
    }

    fn bus_reset(&'a self) {}

    /// Handle a Control Setup transaction.
    fn ctrl_setup(&'a self, endpoint: usize) -> hil::usb::CtrlSetupResult {
        descriptors::SetupData::get(&self.client_ctrl.ctrl_buffer.buf).map(|setup_data| {
            let _b_request = setup_data.request_code;
            let _value = setup_data.value;
        });

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
            self.controller().endpoint_resume_in(ENDPOINT_IN_NUM);
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
    fn packet_in(&'a self, transfer_type: TransferType, endpoint: usize) -> hil::usb::InResult {
        match transfer_type {
            TransferType::Interrupt => {
                self.send_buffer
                    .take()
                    .map_or(hil::usb::InResult::Delay, |buf| {
                        // Get packet that we have shared with the underlying
                        // USB stack to copy the tx into.
                        let packet = self.buffer(endpoint);

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
                self.recv_buffer
                    .take()
                    .map_or(hil::usb::OutResult::Error, |buf| {
                        let recv_offset = self.recv_offset.get();

                        // How many more bytes can we store in our RX buffer?
                        let available_bytes = buf.len() - recv_offset;
                        let copy_length = cmp::min(packet_bytes as usize, available_bytes);

                        // Do the copy into the RX buffer.
                        let packet = self.buffer(endpoint);
                        for i in 0..copy_length {
                            buf[recv_offset + i] = packet[i].get();
                        }

                        // Keep track of how many bytes we have received so far.
                        let total_received_bytes = recv_offset + copy_length;

                        // Update how many bytes we have gotten.
                        self.recv_offset.set(total_received_bytes);

                        // Check if we have received at least as many bytes as the
                        // client asked for.
                        if total_received_bytes >= self.recv_len.get() {
                            if self.can_receive() {
                                self.client.map(move |client| {
                                    client.packet_received(buf, total_received_bytes, endpoint);
                                });
                                // Reset the offset
                                self.recv_offset.set(0);
                                hil::usb::OutResult::Ok
                            } else {
                                // We can't receive data. Record that we have data to send later
                                // and apply back pressure to USB
                                self.saved_endpoint.set(endpoint);
                                self.recv_buffer.replace(buf);
                                hil::usb::OutResult::Delay
                            }
                        } else {
                            // Make sure to put the RX buffer back.
                            self.recv_buffer.replace(buf);
                            hil::usb::OutResult::Ok
                        }
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
                client.packet_transmitted(Ok(()), buf, 64, endpoint);
            });
        });
    }
}
