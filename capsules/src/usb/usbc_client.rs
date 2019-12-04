//! A bare-bones client of the USB hardware interface
//!
//! It responds to standard device requests and can be enumerated.

use super::descriptors::Buffer8;
use super::descriptors::DeviceDescriptor;
use super::descriptors::EndpointAddress;
use super::descriptors::EndpointDescriptor;
use super::descriptors::TransferDirection;
use super::descriptors::TransferType;
use super::usbc_client_ctrl::ClientCtrl;
use core::cell::Cell;
use kernel::common::cells::VolatileCell;
use kernel::debug;
use kernel::hil;

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

static ENDPOINTS: &'static [EndpointDescriptor; N_ENDPOINTS] = &[
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
];

pub struct Client<'a, C: 'a> {
    client_ctrl: ClientCtrl<'a, 'static, C>,

    // An eight-byte buffer for each endpoint
    buffers: [Buffer8; N_ENDPOINTS],

    // State for a debugging feature: A buffer for echoing bulk data
    // from an OUT endpoint back to an IN endpoint
    echo_buf: [Cell<u8>; 8], // Must be no larger than endpoint packet buffer
    echo_len: Cell<usize>,
    delayed_in: Cell<bool>,
    delayed_out: Cell<bool>,
}

impl<C: hil::usb::UsbController> Client<'a, C> {
    pub fn new(controller: &'a C) -> Self {
        Client {
            client_ctrl: ClientCtrl::new(
                controller,
                DeviceDescriptor {
                    vendor_id: VENDOR_ID,
                    product_id: PRODUCT_ID,
                    manufacturer_string: 1,
                    product_string: 2,
                    serial_number_string: 3,
                    ..Default::default()
                },
                Default::default(),
                Default::default(),
                ENDPOINTS,
                None, // No interface class descriptor
                None, // No report descriptor
                LANGUAGES,
                STRINGS,
            ),
            buffers: Default::default(),
            echo_buf: Default::default(),
            echo_len: Cell::new(0),
            delayed_in: Cell::new(false),
            delayed_out: Cell::new(false),
        }
    }

    fn alert_full(&self) {
        // In case we reported Delay before, alert the controller
        // that we now have data to send on the Bulk IN endpoint 1
        if self.delayed_in.take() {
            self.controller().endpoint_bulk_resume(1);
        }
    }

    fn alert_empty(&self) {
        // In case we reported Delay before, alert the controller
        // that we can now receive data on the Bulk OUT endpoint 2
        if self.delayed_out.take() {
            self.controller().endpoint_bulk_resume(2);
        }
    }

    #[inline]
    fn controller(&self) -> &'a C {
        self.client_ctrl.controller()
    }

    #[inline]
    fn buffer(&'a self, i: usize) -> &'a [VolatileCell<u8>; 8] {
        &self.buffers[i - 1].buf
    }
}

impl<C: hil::usb::UsbController> hil::usb::Client for Client<'a, C> {
    fn enable(&self) {
        // Set up the default control endpoint
        self.client_ctrl.enable();

        // Set up a bulk-in endpoint for debugging
        self.controller().endpoint_set_buffer(1, self.buffer(1));
        self.controller().endpoint_bulk_in_enable(1);

        // Set up a bulk-out endpoint for debugging
        self.controller().endpoint_set_buffer(2, self.buffer(2));
        self.controller().endpoint_bulk_out_enable(2);
    }

    fn attach(&self) {
        self.client_ctrl.attach();
    }

    fn bus_reset(&self) {
        // Should the client initiate reconfiguration here?
        // For now, the hardware layer does it.

        debug!("Bus reset");

        // Reset the state for our pair of debugging endpoints
        self.echo_len.set(0);
        self.delayed_in.set(false);
        self.delayed_out.set(false);
    }

    /// Handle a Control Setup transaction
    fn ctrl_setup(&self, endpoint: usize) -> hil::usb::CtrlSetupResult {
        self.client_ctrl.ctrl_setup(endpoint)
    }

    /// Handle a Control In transaction
    fn ctrl_in(&self, endpoint: usize) -> hil::usb::CtrlInResult {
        self.client_ctrl.ctrl_in(endpoint)
    }

    /// Handle a Control Out transaction
    fn ctrl_out(&self, endpoint: usize, packet_bytes: u32) -> hil::usb::CtrlOutResult {
        self.client_ctrl.ctrl_out(endpoint, packet_bytes)
    }

    fn ctrl_status(&self, endpoint: usize) {
        self.client_ctrl.ctrl_status(endpoint)
    }

    /// Handle the completion of a Control transfer
    fn ctrl_status_complete(&self, endpoint: usize) {
        self.client_ctrl.ctrl_status_complete(endpoint)
    }

    /// Handle a Bulk IN transaction
    fn bulk_in(&self, endpoint: usize) -> hil::usb::BulkInResult {
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

            hil::usb::BulkInResult::Packet(packet_bytes)
        } else {
            // Nothing to send
            self.delayed_in.set(true);
            hil::usb::BulkInResult::Delay
        }
    }

    /// Handle a Bulk OUT transaction
    fn bulk_out(&self, endpoint: usize, packet_bytes: u32) -> hil::usb::BulkOutResult {
        // Consume a packet from the endpoint buffer

        let new_len = packet_bytes as usize;
        let current_len = self.echo_len.get();
        let total_len = current_len + new_len as usize;

        if total_len > self.echo_buf.len() {
            // The packet won't fit in our little buffer.  We'll have
            // to wait until it is drained
            self.delayed_out.set(true);
            hil::usb::BulkOutResult::Delay
        } else if new_len > 0 {
            // Copy the packet into our echo buffer
            let packet = self.buffer(endpoint);
            for i in 0..new_len {
                self.echo_buf[current_len + i].set(packet[i].get());
            }
            self.echo_len.set(total_len);

            // We can start sending again
            self.alert_full();

            hil::usb::BulkOutResult::Ok
        } else {
            debug!("Ignoring zero-length OUT packet");
            hil::usb::BulkOutResult::Ok
        }
    }
}
