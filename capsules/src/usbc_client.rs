//! A bare-bones client of the USB hardware interface
//!
//! It responds to standard device requests and can be enumerated.

use core::cell::Cell;
use core::cmp::min;
use kernel::common::cells::VolatileCell;
use kernel::hil;
use usb::ConfigurationDescriptor;
use usb::Descriptor;
use usb::DescriptorType;
use usb::DeviceDescriptor;
use usb::EndpointAddress;
use usb::EndpointDescriptor;
use usb::InterfaceDescriptor;
use usb::LanguagesDescriptor;
use usb::SetupData;
use usb::StandardDeviceRequest;
use usb::StringDescriptor;
use usb::TransferDirection;
use usb::TransferType;

const VENDOR_ID: u16 = 0x6667;
const PRODUCT_ID: u16 = 0xabcd;

static LANGUAGES: &'static [u16] = &[
    0x0409, // English (United States)
];

static STRINGS: &'static [&'static str] = &[
    "XYZ Corp.",      // Manufacturer
    "The Zorpinator", // Product
    "Serial No. 5",   // Serial number
];

// Currently, our descriptors fit exactly into this buffer size.  An
// inconvenience with anything bigger: there is no derived Default
// implementation for [Cell<u8>; DESCRIPTOR_BUFLEN]
const DESCRIPTOR_BUFLEN: usize = 32;

const N_ENDPOINTS: usize = 3;

pub struct Client<'a, C: 'a> {
    // The hardware controller
    controller: &'a C,

    // State for tracking each endpoint
    state: [Cell<State>; N_ENDPOINTS],

    // An eight-byte buffer for each endpoint
    buffers: [[VolatileCell<u8>; 8]; N_ENDPOINTS],

    // Storage for composing responses to device-descriptor requests
    descriptor_storage: [Cell<u8>; DESCRIPTOR_BUFLEN],

    // State for a debugging feature: A buffer for echoing bulk data
    // from an OUT endpoint back to an IN endpoint
    echo_buf: [Cell<u8>; 8], // Must be no larger than endpoint packet buffer
    echo_len: Cell<usize>,
    delayed_in: Cell<bool>,
    delayed_out: Cell<bool>,
}

#[derive(Copy, Clone)]
enum State {
    Init,

    /// We are doing a Control In transfer of some data
    /// in self.descriptor_storage, with the given extent
    /// remaining to send
    CtrlIn(usize, usize),

    /// We will accept data from the host
    CtrlOut,

    SetAddress,
}

impl Default for State {
    fn default() -> Self {
        State::Init
    }
}

impl<C: hil::usb::UsbController> Client<'a, C> {
    pub fn new(controller: &'a C) -> Self {
        Client {
            controller: controller,
            state: Default::default(),
            buffers: Default::default(),
            descriptor_storage: Default::default(),

            echo_buf: Default::default(),
            echo_len: Cell::new(0),
            delayed_in: Cell::new(false),
            delayed_out: Cell::new(false),
        }
    }

    #[inline]
    fn descriptor_buf(&'a self) -> &'a [Cell<u8>] {
        &self.descriptor_storage
    }

    fn alert_full(&self) {
        // In case we reported Delay before, alert the controller
        // that we now have data to send on the Bulk IN endpoint 1
        if self.delayed_in.take() {
            self.controller.endpoint_bulk_resume(1);
        }
    }

    fn alert_empty(&self) {
        // In case we reported Delay before, alert the controller
        // that we can now receive data on the Bulk OUT endpoint 2
        if self.delayed_out.take() {
            self.controller.endpoint_bulk_resume(2);
        }
    }
}

impl<C: hil::usb::UsbController> hil::usb::Client for Client<'a, C> {
    fn enable(&self) {
        // Set up the default control endpoint
        self.controller.endpoint_set_buffer(0, &self.buffers[0]);
        self.controller
            .enable_as_device(hil::usb::DeviceSpeed::Full); // must be Full for Bulk transfers
        self.controller.endpoint_ctrl_out_enable(0);

        // Set up a bulk-in endpoint for debugging
        self.controller.endpoint_set_buffer(1, &self.buffers[1]);
        self.controller.endpoint_bulk_in_enable(1);

        // Set up a bulk-out endpoint for debugging
        self.controller.endpoint_set_buffer(2, &self.buffers[2]);
        self.controller.endpoint_bulk_out_enable(2);
    }

    fn attach(&self) {
        self.controller.attach();
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
        if endpoint != 0 {
            // For now we only support the default Control endpoint
            return hil::usb::CtrlSetupResult::ErrInvalidDeviceIndex;
        }
        SetupData::get(&self.buffers[endpoint]).map_or(
            hil::usb::CtrlSetupResult::ErrNoParse,
            |setup_data| {
                setup_data.get_standard_request().map_or_else(
                    || {
                        // XX: CtrlSetupResult::ErrNonstandardRequest

                        // For now, promiscuously accept vendor data and even supply
                        // a few debugging bytes when host does a read

                        match setup_data.request_type.transfer_direction() {
                            TransferDirection::HostToDevice => {
                                self.state[endpoint].set(State::CtrlOut);
                                hil::usb::CtrlSetupResult::Ok
                            }
                            TransferDirection::DeviceToHost => {
                                // Arrange to send some crap back
                                let buf = self.descriptor_buf();
                                buf[0].set(0xa);
                                buf[1].set(0xb);
                                buf[2].set(0xc);
                                self.state[endpoint].set(State::CtrlIn(0, 3));
                                hil::usb::CtrlSetupResult::Ok
                            }
                        }
                    },
                    |request| {
                        match request {
                            StandardDeviceRequest::GetDescriptor {
                                descriptor_type,
                                descriptor_index,
                                lang_id,
                                requested_length,
                            } => {
                                match descriptor_type {
                                    DescriptorType::Device => match descriptor_index {
                                        0 => {
                                            let buf = self.descriptor_buf();
                                            let d = DeviceDescriptor {
                                                vendor_id: VENDOR_ID,
                                                product_id: PRODUCT_ID,
                                                manufacturer_string: 1,
                                                product_string: 2,
                                                serial_number_string: 3,
                                                ..Default::default()
                                            };
                                            let len = d.write_to(buf);
                                            let end = min(len, requested_length as usize);
                                            self.state[endpoint].set(State::CtrlIn(0, end));
                                            hil::usb::CtrlSetupResult::Ok
                                        }
                                        _ => hil::usb::CtrlSetupResult::ErrInvalidDeviceIndex,
                                    },
                                    DescriptorType::Configuration => {
                                        match descriptor_index {
                                        0 => {
                                            // Place all the descriptors
                                            // related to this configuration
                                            // into a buffer contiguously,
                                            // starting with the last

                                            let buf = self.descriptor_buf();
                                            let mut storage_avail = buf.len();
                                            let mut related_descriptor_length = 0;
                                            let mut num_endpoints = 0;

                                            // endpoint 1: a Bulk-In endpoint
                                            let e1 = EndpointDescriptor {
                                                endpoint_address: EndpointAddress::new(
                                                    1,
                                                    TransferDirection::DeviceToHost,
                                                ),
                                                transfer_type: TransferType::Bulk,
                                                max_packet_size: 8,
                                                interval: 100,
                                            };
                                            storage_avail -=
                                                e1.write_to(&buf[storage_avail - e1.size()..]);
                                            related_descriptor_length += e1.size();
                                            num_endpoints += 1;

                                            // endpoint 2: a Bulk-Out endpoint
                                            let e2 = EndpointDescriptor {
                                                endpoint_address: EndpointAddress::new(
                                                    2,
                                                    TransferDirection::HostToDevice,
                                                ),
                                                transfer_type: TransferType::Bulk,
                                                max_packet_size: 8,
                                                interval: 100,
                                            };
                                            storage_avail -=
                                                e2.write_to(&buf[storage_avail - e2.size()..]);
                                            related_descriptor_length += e2.size();
                                            num_endpoints += 1;

                                            // A single interface, with the above endpoints
                                            let di = InterfaceDescriptor {
                                                num_endpoints: num_endpoints,
                                                ..Default::default()
                                            };
                                            storage_avail -=
                                                di.write_to(&buf[storage_avail - di.size()..]);
                                            related_descriptor_length += di.size();

                                            // A single configuration, with the above interface
                                            let dc = ConfigurationDescriptor {
                                                num_interfaces: 1,
                                                related_descriptor_length:
                                                    related_descriptor_length,
                                                ..Default::default()
                                            };
                                            storage_avail -=
                                                dc.write_to(&buf[storage_avail - dc.size()..]);

                                            let request_start = storage_avail;
                                            let request_end = min(
                                                request_start + (requested_length as usize),
                                                buf.len(),
                                            );
                                            self.state[endpoint]
                                                .set(State::CtrlIn(request_start, request_end));
                                            hil::usb::CtrlSetupResult::Ok
                                        }
                                        _ => hil::usb::CtrlSetupResult::ErrInvalidConfigurationIndex,
                                    }
                                    }
                                    DescriptorType::String => {
                                        if let Some(buf) = match descriptor_index {
                                            0 => {
                                                let buf = self.descriptor_buf();
                                                let d = LanguagesDescriptor { langs: LANGUAGES };
                                                let len = d.write_to(buf);
                                                let end = min(len, requested_length as usize);
                                                Some(&buf[..end])
                                            }
                                            i if i > 0
                                                && (i as usize) <= STRINGS.len()
                                                && lang_id == LANGUAGES[0] =>
                                            {
                                                let buf = self.descriptor_buf();
                                                let d = StringDescriptor {
                                                    string: STRINGS[i as usize - 1],
                                                };
                                                let len = d.write_to(buf);
                                                let end = min(len, requested_length as usize);
                                                Some(&buf[..end])
                                            }
                                            _ => None,
                                        } {
                                            self.state[endpoint].set(State::CtrlIn(0, buf.len()));
                                            hil::usb::CtrlSetupResult::Ok
                                        } else {
                                            hil::usb::CtrlSetupResult::ErrInvalidStringIndex
                                        }
                                    }
                                    DescriptorType::DeviceQualifier => {
                                        // We are full-speed only, so we must
                                        // respond with a request error
                                        hil::usb::CtrlSetupResult::ErrNoDeviceQualifier
                                    }
                                    _ => hil::usb::CtrlSetupResult::ErrUnrecognizedDescriptorType,
                                } // match descriptor_type
                            }
                            StandardDeviceRequest::SetAddress { device_address } => {
                                // Load the address we've been assigned ...
                                self.controller.set_address(device_address);

                                // ... and when this request gets to the Status stage
                                // we will actually enable the address.
                                self.state[endpoint].set(State::SetAddress);
                                hil::usb::CtrlSetupResult::Ok
                            }
                            StandardDeviceRequest::SetConfiguration { .. } => {
                                // We have been assigned a particular configuration: fine!
                                hil::usb::CtrlSetupResult::Ok
                            }
                            _ => hil::usb::CtrlSetupResult::ErrUnrecognizedRequestType,
                        }
                    },
                )
            },
        )
    }

    /// Handle a Control In transaction
    fn ctrl_in(&self, endpoint: usize) -> hil::usb::CtrlInResult {
        match self.state[endpoint].get() {
            State::CtrlIn(start, end) => {
                let len = end.saturating_sub(start);
                if len > 0 {
                    let packet_bytes = min(8, len);
                    let packet = &self.descriptor_storage[start..start + packet_bytes];
                    let buf = &self.buffers[endpoint];

                    // Copy a packet into the endpoint buffer
                    for (i, b) in packet.iter().enumerate() {
                        buf[i].set(b.get());
                    }

                    let start = start + packet_bytes;
                    let len = end.saturating_sub(start);
                    let transfer_complete = len == 0;

                    self.state[endpoint].set(State::CtrlIn(start, end));

                    hil::usb::CtrlInResult::Packet(packet_bytes, transfer_complete)
                } else {
                    hil::usb::CtrlInResult::Packet(0, true)
                }
            }
            _ => hil::usb::CtrlInResult::Error,
        }
    }

    /// Handle a Control Out transaction
    fn ctrl_out(&self, endpoint: usize, _packet_bytes: u32) -> hil::usb::CtrlOutResult {
        match self.state[endpoint].get() {
            State::CtrlOut => {
                // Gamely accept the data
                hil::usb::CtrlOutResult::Ok
            }
            _ => {
                // Bad state
                hil::usb::CtrlOutResult::Halted
            }
        }
    }

    fn ctrl_status(&self, _endpoint: usize) {
        // Entered Status stage
    }

    /// Handle the completion of a Control transfer
    fn ctrl_status_complete(&self, endpoint: usize) {
        // Control Read: IN request acknowledged
        // Control Write: status sent

        match self.state[endpoint].get() {
            State::SetAddress => {
                self.controller.enable_address();
            }
            _ => {}
        };
        self.state[endpoint].set(State::Init);
    }

    /// Handle a Bulk IN transaction
    fn bulk_in(&self, endpoint: usize) -> hil::usb::BulkInResult {
        // Write a packet into the endpoint buffer

        let packet_bytes = self.echo_len.get();
        if packet_bytes > 0 {
            // Copy the entire echo buffer into the packet
            let packet = &self.buffers[endpoint];
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
            let packet = &self.buffers[endpoint];
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
