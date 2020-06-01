//! CDC

use super::descriptors::{
    self, Buffer8, EndpointAddress, EndpointDescriptor, InterfaceDescriptor, TransferDirection,
};
use super::usbc_client_ctrl::ClientCtrl;
use core::cell::Cell;
use kernel::common::cells::VolatileCell;
use kernel::debug;
use kernel::hil;
use kernel::hil::usb::TransferType;

const VENDOR_ID: u16 = 0x6668;
const PRODUCT_ID: u16 = 0xabce;

static LANGUAGES: &'static [u16; 1] = &[
    0x0409, // English (United States)
];

static STRINGS: &'static [&'static str] = &[
    "aXYZ Corp.",      // Manufacturer
    "aThe Zorpinator", // Product
    "aSerial No. 5",   // Serial number
];

const N_ENDPOINTS: usize = 3;

// // Communication interface descriptor

// 0x09,            // Descriptor size in bytes
// 0x04,            // INTERFACE descriptor type
// 0x00,            // Interface number
// 0x00,            // Alternate setting number
// 0x01,            // Number of endpoints
// 0x02,            // Class: CDC communication
// 0x02,            // Subclass: abstract control model
// 0x02,            // Protocol: V.25ter (AT commands)
// 0x00,            // Interface string index

// // Data interface descriptor

// 0x09,            // Descriptor size in bytes
// 0x04,            // INTERFACE descriptor type
// 0x01,            // Interface number
// 0x00,            // Alternate setting number
// 0x02,            // Number of endpoints
// 0x0a,            // Class: CDC data
// 0x00,            // Subclass: none
// 0x00,            // Protocol: none
// 0x00,            // Interface string index


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

// device
// {
// 0x12,            // Descriptor size in bytes
// 0x01,            // DEVICE descriptor type
// 0x0200,          // USB version, BCD (2.0)
// 0x02             // Class: CDC
// 0x00,            // Subclass: none
// 0x00,            // Protocol: none
// 0x08,            // Max. packet size, Endpoint 0
// 0x0925,          // USB Vendor ID
// 0x9060,          // USB Product ID
// 0x0100,          // Device release, BCD (1.0)
// 0x00,            // Manufacturer string index
// 0x00,            // Product string index
// 0x01,            // Serial number string index
// 0x01             // Number of configurations
// };





impl<'a, C: hil::usb::UsbController<'a>> Client<'a, C> {
    pub fn new(controller: &'a C) -> Self {

        let interfaces: &mut [InterfaceDescriptor] = &mut [
            InterfaceDescriptor {
                interface_class: 0x02,    // CDC communication
                interface_subclass: 0x02, // abstract control model (ACM)
                interface_protocol: 0x02, // V.25ter (AT commands)
                ..InterfaceDescriptor::default()
            },

            InterfaceDescriptor {
                interface_class: 0x0a,    // CDC data
                interface_subclass: 0x00, // none
                interface_protocol: 0x00, // none
                ..InterfaceDescriptor::default()
            }
        ];

        let endpoints: &[&[EndpointDescriptor]] = &[
            &[
                EndpointDescriptor {
                    endpoint_address: EndpointAddress::new_const(4, TransferDirection::DeviceToHost),
                    transfer_type: TransferType::Interrupt,
                    max_packet_size: 8,
                    interval: 100,
                },
            ], &[
                EndpointDescriptor {
                    endpoint_address: EndpointAddress::new_const(2, TransferDirection::DeviceToHost),
                    transfer_type: TransferType::Bulk,
                    max_packet_size: 16,
                    // max_packet_size: 8,
                    interval: 100,
                },
                EndpointDescriptor {
                    endpoint_address: EndpointAddress::new_const(3, TransferDirection::HostToDevice),
                    transfer_type: TransferType::Bulk,
                    max_packet_size: 16,
                    // max_packet_size: 8,
                    interval: 100,
                },
            ]
        ];

        let (device_descriptor_buffer, other_descriptor_buffer) =
            descriptors::create_descriptor_buffers(
                descriptors::DeviceDescriptor {
                    vendor_id: VENDOR_ID,
                    product_id: PRODUCT_ID,
                    manufacturer_string: 1,
                    product_string: 2,
                    serial_number_string: 3,
                    class: 0x2, // Class: CDC
                    ..descriptors::DeviceDescriptor::default()
                },
                descriptors::ConfigurationDescriptor {
                    ..descriptors::ConfigurationDescriptor::default()
                },
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
