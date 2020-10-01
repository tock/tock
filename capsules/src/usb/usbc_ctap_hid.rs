//! A USB HID client of the USB hardware interface

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
use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::usb::TransferType;
use kernel::ReturnCode;
use kernel::{debug, hil};

static LANGUAGES: &'static [u16; 1] = &[
    0x0409, // English (United States)
];

const ENDPOINT_NUM: usize = 1;

static CTAP_REPORT_DESCRIPTOR: &'static [u8] = &[
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

static CTAP_REPORT: ReportDescriptor<'static> = ReportDescriptor {
    desc: CTAP_REPORT_DESCRIPTOR,
};

static HID_SUB_DESCRIPTORS: &'static [HIDSubordinateDescriptor] = &[HIDSubordinateDescriptor {
    typ: DescriptorType::Report,
    len: CTAP_REPORT_DESCRIPTOR.len() as u16,
}];

static HID: HIDDescriptor<'static> = HIDDescriptor {
    hid_class: 0x0110,
    country_code: HIDCountryCode::NotSupported,
    sub_descriptors: HID_SUB_DESCRIPTORS,
};

pub struct ClientCtapHID<'a, 'b, C: 'a> {
    client_ctrl: ClientCtrl<'a, 'static, C>,

    // 64-byte buffers for the endpoint
    in_buffer: Buffer64,
    out_buffer: Buffer64,

    // Interaction with the client
    client: OptionalCell<&'b dyn hil::usb_hid::Client<'b>>,
    tx_buffer: TakeCell<'static, [u8; 64]>,
    rx_buffer: TakeCell<'static, [u8; 64]>,
    delayed_out: Cell<bool>,
    pending_in: Cell<bool>,
}

impl<'a, 'b, C: hil::usb::UsbController<'a>> ClientCtapHID<'a, 'b, C> {
    pub fn new(
        controller: &'a C,
        max_ctrl_packet_size: u8,
        vendor_id: u16,
        product_id: u16,
        strings: &'static [&'static str],
    ) -> Self {
        let interfaces: &mut [InterfaceDescriptor] = &mut [
            // Interface declared in the FIDO2 specification, section 8.1.8.1
            InterfaceDescriptor {
                interface_class: 0x03, // HID
                interface_subclass: 0x00,
                interface_protocol: 0x00,
                ..InterfaceDescriptor::default()
            },
        ];

        let endpoints: &[&[EndpointDescriptor]] = &[&[
            EndpointDescriptor {
                endpoint_address: EndpointAddress::new_const(
                    ENDPOINT_NUM,
                    TransferDirection::HostToDevice,
                ),
                transfer_type: TransferType::Interrupt,
                max_packet_size: 64,
                interval: 5,
            },
            EndpointDescriptor {
                endpoint_address: EndpointAddress::new_const(
                    ENDPOINT_NUM,
                    TransferDirection::DeviceToHost,
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
                    max_packet_size_ep0: max_ctrl_packet_size,
                    ..descriptors::DeviceDescriptor::default()
                },
                descriptors::ConfigurationDescriptor {
                    configuration_value: 1,
                    ..descriptors::ConfigurationDescriptor::default()
                },
                interfaces,
                endpoints,
                Some(&HID),
                None, // No CDC descriptor array
            );

        ClientCtapHID {
            client_ctrl: ClientCtrl::new(
                controller,
                device_descriptor_buffer,
                other_descriptor_buffer,
                Some(&HID),
                Some(&CTAP_REPORT),
                LANGUAGES,
                strings,
            ),
            in_buffer: Buffer64::default(),
            out_buffer: Buffer64::default(),
            client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            rx_buffer: TakeCell::empty(),
            delayed_out: Cell::new(false),
            pending_in: Cell::new(false),
        }
    }

    pub fn set_client(&'a self, client: &'b dyn hil::usb_hid::Client<'b>) {
        self.client.set(client);
    }

    // Send an OUT packet available in the controller back to the client.
    // This returns false if the client is not ready to receive a packet, and true if the client
    // successfully accepted the packet.
    fn send_packet_to_client(&'a self) -> bool {
        assert!(!self.delayed_out.get());

        // Notify the client
        if self.client.map_or(false, |client| client.can_receive()) {
            // Clear any pending packet on the transmitting side.
            // It's up to the client to handle the received packet and decide if this packet
            // should be re-transmitted or not.
            if let Some(buf) = self.tx_buffer.take() {
                self.client.map(move |client| {
                    client.packet_transmitted(ReturnCode::ECANCEL, buf, 64, ENDPOINT_NUM)
                });
            }

            // Copy the packet into a buffer to send to the client.
            let buf = self.rx_buffer.take().unwrap();
            for (i, x) in self.out_buffer.buf.iter().enumerate() {
                buf[i] = x.get();
            }

            self.client.map(move |client| {
                client.packet_received(ReturnCode::SUCCESS, buf, 64, ENDPOINT_NUM)
            });
            true
        } else {
            // Cannot receive now, indicate a delay to the controller.
            self.delayed_out.set(true);
            false
        }
    }

    #[inline]
    fn controller(&'a self) -> &'a C {
        self.client_ctrl.controller()
    }
}

impl<'a, 'b, C: hil::usb::UsbController<'a>> hil::usb_hid::UsbHid<'a> for ClientCtapHID<'a, 'b, C> {
    fn send_buffer(
        &'a self,
        buf: &'static mut [u8; 64],
    ) -> Result<usize, (ReturnCode, &'static mut [u8; 64])> {
        if self.tx_buffer.is_some() {
            // The previous packet has not yet been transmitted, reject the new one.
            Err((ReturnCode::EBUSY, buf))
        } else {
            self.tx_buffer.replace(buf);
            // Alert the controller that we now have data to send on the Interrupt IN endpoint.
            self.controller().endpoint_resume_in(ENDPOINT_NUM);
            Ok(64)
        }
    }

    fn send_cancel(&'a self) -> Result<&'static mut [u8; 64], ReturnCode> {
        if self.pending_in.get() {
            // The packet is being sent at the moment. Cannot return the buffer yet.
            Err(ReturnCode::EBUSY)
        } else {
            // Cancel the send if there was one. Return EALREADY if it was already cancelled.
            self.tx_buffer.take().ok_or(ReturnCode::EALREADY)
        }
    }

    fn receive_buffer(
        &'a self,
        buf: &'static mut [u8; 64],
    ) -> Result<(), (ReturnCode, &'static mut [u8; 64])> {
        if self.rx_buffer.is_some() {
            // The previous packet has not yet been received, reject the new one.
            Err((ReturnCode::EBUSY, buf))
        } else {
            self.rx_buffer.replace(buf);
            // In case we reported Delay before, send the pending packet back to the client.
            // Otherwise, there's nothing to do, the controller will send us a packet_out when a
            // packet arrives.
            if self.delayed_out.take() {
                if self.send_packet_to_client() {
                    // If that succeeds, alert the controller that we can now
                    // receive data on the Interrupt OUT endpoint.
                    self.controller().endpoint_resume_out(ENDPOINT_NUM);
                }
            }
            Ok(())
        }
    }

    fn receive_cancel(&'a self) -> Result<&'static mut [u8; 64], ReturnCode> {
        // Cancel the receive if there was one. Return EALREADY if it was already cancelled.
        self.rx_buffer.take().ok_or(ReturnCode::EALREADY)
    }
}

impl<'a, 'b, C: hil::usb::UsbController<'a>> hil::usb::Client<'a> for ClientCtapHID<'a, 'b, C> {
    fn enable(&'a self) {
        // Set up the default control endpoint
        self.client_ctrl.enable();

        // Set up the interrupt in-out endpoint
        self.controller()
            .endpoint_set_in_buffer(ENDPOINT_NUM, &self.in_buffer.buf);
        self.controller()
            .endpoint_set_out_buffer(ENDPOINT_NUM, &self.out_buffer.buf);
        self.controller()
            .endpoint_in_out_enable(TransferType::Interrupt, ENDPOINT_NUM);
    }

    fn attach(&'a self) {
        self.client_ctrl.attach();
    }

    fn bus_reset(&'a self) {
        // Should the client initiate reconfiguration here?
        // For now, the hardware layer does it.

        debug!("Bus reset");
    }

    /// Handle a Control Setup transaction
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
        self.client_ctrl.ctrl_status_complete(endpoint)
    }

    /// Handle a Bulk/Interrupt IN transaction
    fn packet_in(&'a self, transfer_type: TransferType, endpoint: usize) -> hil::usb::InResult {
        match transfer_type {
            TransferType::Bulk => hil::usb::InResult::Error,
            TransferType::Interrupt => {
                if endpoint != ENDPOINT_NUM {
                    return hil::usb::InResult::Error;
                }

                self.tx_buffer.map_or(
                    // Nothing to send
                    hil::usb::InResult::Delay,
                    |packet| {
                        self.pending_in.set(true);

                        let buf = &self.in_buffer.buf;
                        for i in 0..64 {
                            buf[i].set(packet[i]);
                        }

                        hil::usb::InResult::Packet(64)
                    },
                )
            }
            TransferType::Control | TransferType::Isochronous => unreachable!(),
        }
    }

    /// Handle a Bulk/Interrupt OUT transaction
    fn packet_out(
        &'a self,
        transfer_type: TransferType,
        endpoint: usize,
        packet_bytes: u32,
    ) -> hil::usb::OutResult {
        match transfer_type {
            TransferType::Bulk => hil::usb::OutResult::Error,
            TransferType::Interrupt => {
                if endpoint != ENDPOINT_NUM {
                    return hil::usb::OutResult::Error;
                }

                if packet_bytes != 64 {
                    // Cannot process this packet
                    hil::usb::OutResult::Error
                } else {
                    if self.send_packet_to_client() {
                        hil::usb::OutResult::Ok
                    } else {
                        hil::usb::OutResult::Delay
                    }
                }
            }
            TransferType::Control | TransferType::Isochronous => unreachable!(),
        }
    }

    fn packet_transmitted(&'a self, endpoint: usize) {
        if endpoint != ENDPOINT_NUM {
            panic!("Unexpected transmission on ep {}", endpoint);
        }

        // Clear any pending packet on the receiving side.
        // It's up to the client to handle the transmitted packet and decide if they want to
        // receive another packet.
        if let Some(buf) = self.rx_buffer.take() {
            self.client.map(move |client| {
                client.packet_received(ReturnCode::ECANCEL, buf, 64, ENDPOINT_NUM)
            });
        }

        // Notify the client
        assert!(self.pending_in.take());
        let buf = self.tx_buffer.take().unwrap();
        self.client.map(move |client| {
            client.packet_transmitted(ReturnCode::SUCCESS, buf, 64, ENDPOINT_NUM)
        });
    }
}
