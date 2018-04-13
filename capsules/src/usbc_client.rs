//! A bare-bones client of the USB hardware interface
//!
//! It responds to standard device requests and can be enumerated.

use core::cell::Cell;
use core::cmp::min;
use kernel::common::VolatileCell;
use kernel::hil;
use kernel::hil::usb::*;
use usb::*;

const VENDOR_ID: u16 = 0x6667;
const PRODUCT_ID: u16 = 0xabcd;

static LANGUAGES: &'static [u16] = &[
    0x0409 // English (United States)
];

static STRINGS: &'static [&'static str] = &[
    "XYZ Corp.",      // Manufacturer
    "The Zorpinator", // Product
    "Serial No. 5",   // Serial number
];

const DESCRIPTOR_BUFLEN: usize = 30;

const N_ENDPOINTS: usize = 1;

pub struct Client<'a, C: 'a> {
    controller: &'a C,
    state: [Cell<State>; N_ENDPOINTS],
    buffers: [[VolatileCell<u8>; 8]; N_ENDPOINTS],
    descriptor_storage: [Cell<u8>; DESCRIPTOR_BUFLEN],
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

impl<'a, C: UsbController> Client<'a, C> {
    pub fn new(controller: &'a C) -> Self {
        Client {
            controller: controller,
            state: Default::default(),
            buffers: Default::default(),
            descriptor_storage: Default::default(),
        }
    }

    #[inline]
    fn descriptor_buf(&'a self) -> &'a [Cell<u8>] {
        &self.descriptor_storage
    }
}

impl<'a, C: UsbController> hil::usb::Client for Client<'a, C> {
    fn enable(&self) {
        // Set up the default control endpoint
        self.controller.endpoint_set_buffer(0, &self.buffers[0]);
        self.controller.enable_as_device(DeviceSpeed::Full); // must be Full for Bulk transfers
        self.controller.endpoint_ctrl_out_enable(0);
    }

    fn attach(&self) {
        self.controller.attach();
    }

    fn bus_reset(&self) {
        // Should the client initiate reconfiguration here?
        // For now, the hardware layer does it.
    }

    /// Handle a Control Setup transaction
    fn ctrl_setup(&self, endpoint: usize) -> CtrlSetupResult {
        SetupData::get(&self.buffers[endpoint]).map_or(CtrlSetupResult::ErrNoParse, |setup_data| {
            setup_data.get_standard_request().map_or_else(
                || {
                    // XX: CtrlSetupResult::ErrNonstandardRequest

                    // For now, promiscuously accept vendor data and even supply
                    // a few debugging bytes when host does a read

                    match setup_data.request_type.transfer_direction() {
                        TransferDirection::HostToDevice => {
                            self.state[endpoint].set(State::CtrlOut);
                            CtrlSetupResult::Ok
                        }
                        TransferDirection::DeviceToHost => {
                            // Arrange to send some crap back
                            let buf = self.descriptor_buf();
                            buf[0].set(0xa);
                            buf[1].set(0xb);
                            buf[2].set(0xc);
                            self.state[endpoint].set(State::CtrlIn(0, 3));
                            CtrlSetupResult::Ok
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
                                        CtrlSetupResult::Ok
                                    }
                                    _ => CtrlSetupResult::ErrInvalidDeviceIndex,
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

                                            let di = InterfaceDescriptor::default();
                                            storage_avail -=
                                                di.write_to(&buf[storage_avail - di.size()..]);

                                            let dc = ConfigurationDescriptor {
                                                num_interfaces: 1,
                                                related_descriptor_length: di.size(),
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
                                            CtrlSetupResult::Ok
                                        }
                                        _ => CtrlSetupResult::ErrInvalidConfigurationIndex,
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
                                        i if i > 0 && (i as usize) <= STRINGS.len()
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
                                        CtrlSetupResult::Ok
                                    } else {
                                        CtrlSetupResult::ErrInvalidStringIndex
                                    }
                                }
                                DescriptorType::DeviceQualifier => {
                                    // We are full-speed only, so we must
                                    // respond with a request error
                                    CtrlSetupResult::ErrNoDeviceQualifier
                                }
                                _ => CtrlSetupResult::ErrUnrecognizedDescriptorType,
                            } // match descriptor_type
                        }
                        StandardDeviceRequest::SetAddress { device_address } => {
                            // Load the address we've been assigned ...
                            self.controller.set_address(device_address);

                            // ... and when this request gets to the Status stage
                            // we will actually enable the address.
                            self.state[endpoint].set(State::SetAddress);
                            CtrlSetupResult::Ok
                        }
                        StandardDeviceRequest::SetConfiguration { .. } => {
                            // We have been assigned a particular configuration: fine!
                            CtrlSetupResult::Ok
                        }
                        _ => CtrlSetupResult::ErrUnrecognizedRequestType,
                    }
                },
            )
        })
    }

    /// Handle a Control In transaction
    fn ctrl_in(&self, endpoint: usize) -> CtrlInResult {
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

                    CtrlInResult::Packet(packet_bytes, transfer_complete)
                } else {
                    CtrlInResult::Packet(0, true)
                }
            }
            _ => CtrlInResult::Error,
        }
    }

    /// Handle a Control Out transaction
    fn ctrl_out(&self, endpoint: usize, packet_bytes: u32) -> CtrlOutResult {
        match self.state[endpoint].get() {
            State::CtrlOut => {
                debug!("Received {} vendor control bytes", packet_bytes);
                // &self.buffer[endpoint][0 .. packet_bytes as usize]
                CtrlOutResult::Ok
            }
            _ => {
                // Bad state
                CtrlOutResult::Halted
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
}
