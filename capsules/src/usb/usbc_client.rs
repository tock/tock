//! A bare-bones client of the USB hardware interface
//!
//! It responds to standard device requests and can be enumerated.

use super::descriptors::{
    self, Buffer8, DeviceDescriptor, EndpointAddress, EndpointDescriptor, TransferDirection,
};
use super::usbc_client_ctrl::ClientCtrl;
use core::cell::Cell;
use kernel::common::cells::VolatileCell;
use kernel::debug;
use kernel::hil;
use kernel::hil::usb::TransferType;

const VENDOR_ID: u16 = 0x6667;
const PRODUCT_ID: u16 = 0xabcd;

static LANGUAGES: &'static [u16; 1] = &[
    0x0409, // English (United States)
];

static STRINGS: &'static [&'static str] = &[
    "XYZ Corp.",      // Manufacturer
    "The Zorpinator", // Product
    "Serial No. 5",   // Serial number
];

const N_ENDPOINTS: usize = 2;

pub struct Client<'a, C: 'a> {
    client_ctrl: ClientCtrl<'a, 'static, C>,

    // An eight-byte buffer for each endpoint
    buffers: [Buffer8; N_ENDPOINTS],

    // State for a debugging feature: A buffer for echoing bulk data
    // from an OUT endpoint back to an IN endpoint
    echo_buf: [Cell<u8>; 8], // Must be no larger than endpoint packet buffer
    echo_len: Cell<usize>,
    delayed_out: Cell<bool>,
}

impl<'a, C: hil::usb::UsbController<'a>> Client<'a, C> {
    pub fn new(controller: &'a C) -> Self {

        let interfaces: &mut [descriptors::InterfaceDescriptor] = &mut [
            descriptors::InterfaceDescriptor {
                interface_number: 0,
                    alternate_setting: 0,
                    num_endpoints: 0,      // (excluding default control endpoint)
                    interface_class: 0xff, // vendor_specific
                    interface_subclass: 0xab,
                    interface_protocol: 0,
                    string_index: 0,
            }
        ];

        let endpoints: &[&[EndpointDescriptor]] = &mut [
            &[
            EndpointDescriptor {
                endpoint_address: EndpointAddress::new_const(1, TransferDirection::DeviceToHost),
                transfer_type: TransferType::Bulk,
                max_packet_size: 8,
                interval: 100,
            },
            EndpointDescriptor {
                endpoint_address: EndpointAddress::new_const(2, TransferDirection::HostToDevice),
                transfer_type: TransferType::Bulk,
                max_packet_size: 8,
                interval: 100,
            },
            ]
        ];

        let (device_descriptor_buffer, other_descriptor_buffer) =
            descriptors::create_descriptor_buffers(
                DeviceDescriptor {
                    vendor_id: VENDOR_ID,
                    product_id: PRODUCT_ID,
                    manufacturer_string: 1,
                    product_string: 2,
                    serial_number_string: 3,
                    ..DeviceDescriptor::default()
                },
                descriptors::ConfigurationDescriptor::default(),
                interfaces,
                endpoints,
                None, // No HID descriptor
                );

        Client {
            client_ctrl: ClientCtrl::new(
                controller,
                device_descriptor_buffer,
                other_descriptor_buffer,
                None, // No HID descriptor
                None, // No report descriptor
                LANGUAGES,
                STRINGS,
            ),
            buffers: Default::default(),
            echo_buf: Default::default(),
            echo_len: Cell::new(0),
            delayed_out: Cell::new(false),
        }
    }

    fn alert_full(&'a self) {
        // Alert the controller that we now have data to send on the Bulk IN endpoint 1
        self.controller().endpoint_resume_in(1);
    }

    fn alert_empty(&'a self) {
        // In case we reported Delay before, alert the controller
        // that we can now receive data on the Bulk OUT endpoint 2
        if self.delayed_out.take() {
            self.controller().endpoint_resume_out(2);
        }
    }

    #[inline]
    fn controller(&'a self) -> &'a C {
        self.client_ctrl.controller()
    }

    #[inline]
    fn buffer(&'a self, i: usize) -> &'a [VolatileCell<u8>; 8] {
        &self.buffers[i - 1].buf
    }
}

impl<'a, C: hil::usb::UsbController<'a>> hil::usb::Client<'a> for Client<'a, C> {
    fn enable(&'a self) {
        // Set up the default control endpoint
        self.client_ctrl.enable();

        // Set up a bulk-in endpoint for debugging
        self.controller().endpoint_set_in_buffer(1, self.buffer(1));
        self.controller().endpoint_in_enable(TransferType::Bulk, 1);

        // Set up a bulk-out endpoint for debugging
        self.controller().endpoint_set_out_buffer(2, self.buffer(2));
        self.controller().endpoint_out_enable(TransferType::Bulk, 2);
    }

    fn attach(&'a self) {
        self.client_ctrl.attach();
    }

    fn bus_reset(&'a self) {
        // Should the client initiate reconfiguration here?
        // For now, the hardware layer does it.

        debug!("Bus reset");

        // Reset the state for our pair of debugging endpoints
        self.echo_len.set(0);
        self.delayed_out.set(false);
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
            TransferType::Interrupt => {
                debug!("interrupt_in({}) not implemented", endpoint);
                hil::usb::InResult::Error
            }
            TransferType::Bulk => {
                // Write a packet into the endpoint buffer
                let packet_bytes = self.echo_len.get();
                if packet_bytes > 0 {
                    // Copy the entire echo buffer into the packet
                    let packet = self.buffer(endpoint);
                    for i in 0..packet_bytes {
                        packet[i].set(self.echo_buf[i].get());
                    }
                    self.echo_len.set(0);

                    // We can receive more now
                    self.alert_empty();

                    hil::usb::InResult::Packet(packet_bytes)
                } else {
                    // Nothing to send
                    hil::usb::InResult::Delay
                }
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
            TransferType::Interrupt => {
                debug!("interrupt_out({}) not implemented", endpoint);
                hil::usb::OutResult::Error
            }
            TransferType::Bulk => {
                // Consume a packet from the endpoint buffer
                let new_len = packet_bytes as usize;
                let current_len = self.echo_len.get();
                let total_len = current_len + new_len as usize;

                if total_len > self.echo_buf.len() {
                    // The packet won't fit in our little buffer.  We'll have
                    // to wait until it is drained
                    self.delayed_out.set(true);
                    hil::usb::OutResult::Delay
                } else if new_len > 0 {
                    // Copy the packet into our echo buffer
                    let packet = self.buffer(endpoint);
                    for i in 0..new_len {
                        self.echo_buf[current_len + i].set(packet[i].get());
                    }
                    self.echo_len.set(total_len);

                    // We can start sending again
                    self.alert_full();
                    hil::usb::OutResult::Ok
                } else {
                    debug!("Ignoring zero-length OUT packet");
                    hil::usb::OutResult::Ok
                }
            }
            TransferType::Control | TransferType::Isochronous => unreachable!(),
        }
    }

    fn packet_transmitted(&'a self, _endpoint: usize) {
        // Nothing to do.
    }
}
