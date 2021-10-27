//! A generic USB client layer managing control requests
//!
//! This layer responds to control requests and handles the state machine for
//! implementing them.

use super::descriptors::Buffer64;
use super::descriptors::Descriptor;
use super::descriptors::DescriptorType;
use super::descriptors::LanguagesDescriptor;
use super::descriptors::Recipient;
use super::descriptors::SetupData;
use super::descriptors::StandardRequest;
use super::descriptors::StringDescriptor;
use super::descriptors::TransferDirection;

use core::cell::Cell;
use core::cmp::min;

use kernel::hil;
use kernel::hil::usb::TransferType;

const DESCRIPTOR_BUFLEN: usize = 128;

pub struct DevDescriptor {
    /// Valid values include 0x0100 (USB1.0), 0x0110 (USB1.1) and 0x0200 (USB2.0)
    pub usb_release: u16,

    /// 0x00 means each interface defines its own class.
    /// 0xFF means the class behavior is defined by the vendor.
    /// All other values have meaning assigned by USB-IF
    pub class: u8,

    /// Assigned by USB-IF if `class` is
    pub subclass: u8,

    /// Assigned by USB-IF if `class` is
    pub protocol: u8,

    /// Obtained from USB-IF
    pub vendor_id: u16,

    /// Together with `vendor_id`, this must be unique to the product
    pub product_id: u16,

    /// Device release number in binary coded decimal (BCD)
    pub device_release: u16,

    /// Index of the string descriptor describing manufacturer, or 0 if none
    pub manufacturer_string: &'static str,

    /// Index of the string descriptor describing product, or 0 if none
    pub product_string: &'static str,

    /// Index of the string descriptor giving device serial number, or 0 if none
    pub serial_number_string: &'static str,
}


/// Handler for USB control endpoint requests.
pub struct Device<'a, U: 'a, C: super::configuration::Configuration> {
    /// The USB hardware controller.
    controller: &'a U,

    /// State of control endpoint (endpoint 0).
    state: Cell<State>,

    /// A 64-byte buffer for the control endpoint to be passed to the USB
    /// driver.
    pub ctrl_buffer: Buffer64,

    descriptor: DevDescriptor,

    configuration: C,

    /// Storage for composing responses to device descriptor requests.
    descriptor_storage: [Cell<u8>; DESCRIPTOR_BUFLEN],

    /// Supported language (only one for now).
    language: u16,
}

/// States for the individual endpoints.
#[derive(Copy, Clone)]
enum State {
    Init,

    /// We are doing a Control In transfer of some data in
    /// self.descriptor_storage, with the given extent remaining to send.
    CtrlIn(usize, usize),

    /// We will accept data from the host.
    CtrlOutInterface(u16),

    SetAddress,
}

impl Default for State {
    fn default() -> Self {
        State::Init
    }
}

impl<'a, U: hil::usb::UsbController<'a>, C: super::configuration::Configuration> Device<'a, U, C> {
    pub fn new(
        controller: &'a U,
        descriptor: DevDescriptor,
        configuration: C,
        language: u16,
    ) -> Self {
        Device {
            descriptor,
            configuration,
            controller: controller,
            state: Default::default(),
            // For the moment, the Default trait is not implemented for arrays
            // of length > 32, and the Cell type is not Copy, so we have to
            // initialize each element manually.
            #[rustfmt::skip]
            descriptor_storage: [
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
                Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            ],
            ctrl_buffer: Buffer64::default(),
            language,
        }
    }

    fn write_device_descriptor(&self, buf: &[Cell<u8>]) -> usize {
        use super::configuration::put_u16;
        buf[0].set(18); // Size of descriptor
        buf[1].set(DescriptorType::Device as u8);
        put_u16(&buf[2..4], self.descriptor.usb_release);
        buf[4].set(self.descriptor.class);
        buf[5].set(self.descriptor.subclass);
        buf[6].set(self.descriptor.protocol);
        buf[7].set(U::MAX_CTRL_PACKET_SIZE);
        put_u16(&buf[8..10], self.descriptor.vendor_id);
        put_u16(&buf[10..12], self.descriptor.product_id);
        put_u16(&buf[12..14], self.descriptor.device_release);
        buf[14].set(1); // Manufacturer String
        buf[15].set(2); // Product String
        buf[16].set(3); // Serial Number String
        buf[17].set(1); // NUM CONFIGURATIONS, change if we start supporting more than 1
        18
    }

    #[inline]
    pub fn controller(&self) -> &'a U {
        self.controller
    }

    #[inline]
    fn descriptor_buf(&'a self) -> &'a [Cell<u8>] {
        &self.descriptor_storage
    }

    fn handle_standard_device_request(
        &'a self,
        endpoint: usize,
        request: StandardRequest,
    ) -> hil::usb::CtrlSetupResult {
        match request {
            StandardRequest::GetDescriptor {
                descriptor_type,
                descriptor_index,
                lang_id,
                requested_length,
            } => {
                match descriptor_type {
                    DescriptorType::Device => match descriptor_index {
                        0 => {
                            let buf = self.descriptor_buf();
                            let len = self.write_device_descriptor(buf);

                            let end = min(len, requested_length as usize);

                            self.state.set(State::CtrlIn(0, end));
                            hil::usb::CtrlSetupResult::Ok
                        }
                        _ => hil::usb::CtrlSetupResult::ErrInvalidDeviceIndex,
                    },
                    DescriptorType::Configuration => {
                        if descriptor_index == 0 {
                            let buf = self.descriptor_buf();
                            // 0-value configuration means deconfigure, so start from 1
                            let mut len = self.configuration.write_to(descriptor_index + 1, buf);
                            // endpoint 0 is the control endpoint, start from 1
                            let mut endpoint_num = 1;
                            for (interface_num, interface) in self.configuration.interfaces().enumerate() {
                                len += interface.write_to(interface_num as u8, &buf[len..]);
                                let mut cd_i = 0;
                                while let Some(class_descriptor) = interface.class_descriptor(cd_i) {
                                    len += class_descriptor.write_to(&buf[len..]);
                                    cd_i += 1;
                                }
                                let mut j = 0;
                                while let Some(endpoint) = interface.endpoint(j) {
                                    len += endpoint.write_to(endpoint_num, &buf[len..]);
                                    j += 1;
                                    endpoint_num += 1;
                                }
                            }

                            let end = min(len, requested_length as usize);
                            self.state.set(State::CtrlIn(0, end));
                            hil::usb::CtrlSetupResult::Ok
                        } else {
                            hil::usb::CtrlSetupResult::ErrInvalidConfigurationIndex
                        }
                    },
                    DescriptorType::String => {
                        if let Some(len) = match descriptor_index {
                            0 => {
                                let buf = self.descriptor_buf();
                                let d = LanguagesDescriptor {
                                    langs: &[self.language],
                                };
                                let len = d.write_to(buf);
                                Some(len)
                            }
                            i@(1 ..= 3) => {
                                if lang_id == self.language {
                                    (match i {
                                        1 => Some(self.descriptor.manufacturer_string),
                                        2 => Some(self.descriptor.product_string),
                                        3 => Some(self.descriptor.serial_number_string),
                                        _ => None,
                                    }).and_then(|s| {
                                        let d = StringDescriptor {
                                            string: s
                                        };
                                        let buf = self.descriptor_buf();
                                        let len = d.write_to(buf);
                                        Some(len)
                                    })
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        } {
                            let end = min(len, requested_length as usize);
                            self.state.set(State::CtrlIn(0, end));
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
            StandardRequest::SetAddress { device_address } => {
                // Load the address we've been assigned ...
                self.controller.set_address(device_address);

                // ... and when this request gets to the Status stage we will actually enable the
                // address.
                self.state.set(State::SetAddress);
                hil::usb::CtrlSetupResult::OkSetAddress
            }
            StandardRequest::SetConfiguration { configuration_value } => {
                if configuration_value == 1 {
                    for interface in self.configuration.interfaces() {
                        interface.bus_reset();
                    }
                } else {
                    // TODO(alevy): Deconfigure (0) or no such configuration? What does deconfigure
                    // mean though?
                }

                hil::usb::CtrlSetupResult::Ok
            }
            _ => hil::usb::CtrlSetupResult::ErrUnrecognizedRequestType,
        }
    }

}

impl<'a, U: hil::usb::UsbController<'a>, C: super::configuration::Configuration> hil::usb::Client<'a> for Device<'a, U, C> {
    fn enable(&'a self) {
        // Set up the default control endpoint
        self.controller
            .endpoint_set_ctrl_buffer(&self.ctrl_buffer.buf);
        self.controller
            .enable_as_device(hil::usb::DeviceSpeed::Full); // must be Full for Bulk transfers
        self.controller
            .endpoint_out_enable(TransferType::Control, 0);

        let mut endpoint_num = 1;
        for interface in self.configuration.interfaces() {
            let mut j = 0;
            while let Some(endpoint) = interface.endpoint(j) {
                match endpoint.direction {
                    TransferDirection::DeviceToHost => {
                        let buffer = endpoint.buffer();
                        if buffer.len() > 0 {
                            self.controller.endpoint_set_in_buffer(endpoint_num, buffer);
                        }
                        self.controller().endpoint_in_enable(endpoint.transfer_type, endpoint_num);
                    },
                    TransferDirection::HostToDevice => {
                        let buffer = endpoint.buffer();
                        if buffer.len() > 0 {
                            self.controller.endpoint_set_out_buffer(endpoint_num, buffer);
                        }
                        self.controller().endpoint_out_enable(endpoint.transfer_type, endpoint_num);
                    },
                }
                j += 1;
                endpoint_num += 1;
            }
        }
    }

    fn attach(&'a self) {
        self.controller.attach();
    }

    fn bus_reset(&'a self) {

    }

    fn ctrl_setup(&'a self, endpoint: usize) -> hil::usb::CtrlSetupResult {
        if endpoint != 0 {
            // For now we only support the default Control endpoint
            return hil::usb::CtrlSetupResult::ErrInvalidDeviceIndex;
        }
        SetupData::get(&self.ctrl_buffer.buf).map_or(hil::usb::CtrlSetupResult::ErrNoParse, |setup_data| {
                let transfer_direction = setup_data.request_type.transfer_direction();
                let recipient = setup_data.request_type.recipient();
                match recipient {
                    Recipient::Device => {
                        setup_data.get_standard_request().map_or(hil::usb::CtrlSetupResult::ErrGeneric, |request|
                            self.handle_standard_device_request(endpoint, request)
                        )
                    }
                    Recipient::Interface => {
                        //self.handle_standard_interface_request(endpoint, request),
                        match transfer_direction {
                            TransferDirection::HostToDevice => {
                                self.state.set(State::CtrlOutInterface(setup_data.index));
                            },
                            TransferDirection::DeviceToHost => {
                                self.state.set(State::CtrlOutInterface(setup_data.index));
                            },
                        }
                        self.configuration.interface(setup_data.index as usize).map(|interface|
                            interface.ctrl_setup(setup_data)
                        ).unwrap_or(hil::usb::CtrlSetupResult::ErrGeneric)
                    },
                    _ => hil::usb::CtrlSetupResult::ErrGeneric,
                }
            })
    }

    /// Handle a Control In transaction
    fn ctrl_in(&'a self, endpoint: usize) -> hil::usb::CtrlInResult {
        match self.state.get() {
            State::CtrlIn(start, end) => {
                let len = end.saturating_sub(start);
                if len > 0 {
                    let packet_bytes = min(self.ctrl_buffer.buf.len(), len);
                    let packet = &self.descriptor_storage[start..start + packet_bytes];

                    // Copy a packet into the endpoint buffer
                    for (b, p) in self.ctrl_buffer.buf.iter().zip(packet.iter()) {
                        b.set(p.get());
                    }

                    let start = start + packet_bytes;
                    let len = end.saturating_sub(start);
                    let transfer_complete = len == 0;

                    self.state.set(State::CtrlIn(start, end));

                    hil::usb::CtrlInResult::Packet(packet_bytes, transfer_complete)
                } else {
                    hil::usb::CtrlInResult::Packet(0, true)
                }
            }
            _ => hil::usb::CtrlInResult::Error,
        }
    }

    fn ctrl_out(&'a self, endpoint: usize, packet_bytes: u32) -> hil::usb::CtrlOutResult {
        match self.state.get() {
            State::CtrlOutInterface(interface) => {
                self.configuration.interface(interface as usize).map(|interface|
                    interface.ctrl_out(self.ctrl_buffer.buf, packet_bytes)
                ).unwrap_or(hil::usb::CtrlOutResult::Halted)
            },
            _ => hil::usb::CtrlOutResult::Halted
        }
    }

    fn ctrl_status(&'a self, _endpoint: usize) {
        // Entered Status stage
    }

    fn ctrl_status_complete(&'a self, endpoint: usize) {
        // Control Read: IN request acknowledged
        // Control Write: status sent

        match self.state.get() {
            State::SetAddress => {
                self.controller.enable_address();
            },
            State::CtrlOutInterface(interface) => {
                self.configuration.interface(interface as usize).map(|int| {
                    int.ctrl_status_complete(endpoint);
                });
            },
            _ => {}
        };
        self.state.set(State::Init);
    }

    fn packet_in(&'a self, transfer_type: TransferType, endpoint: usize) -> hil::usb::InResult {
        let mut max_prev_endpoint = 1;
        for interface in self.configuration.interfaces() {
            let largest_endpoint = interface.details().num_endpoints + max_prev_endpoint;
            if largest_endpoint > endpoint {
                return interface.packet_in(transfer_type, endpoint - max_prev_endpoint);
            }
            max_prev_endpoint = largest_endpoint;
        }
        hil::usb::InResult::Error
    }

    fn packet_out(
        &'a self,
        transfer_type: TransferType,
        endpoint: usize,
        packet_bytes: u32,
    ) -> hil::usb::OutResult {
        let mut max_prev_endpoint = 1;
        for interface in self.configuration.interfaces() {
            let largest_endpoint = interface.details().num_endpoints + max_prev_endpoint;
            if largest_endpoint > endpoint {
                return interface.packet_out(transfer_type, endpoint - max_prev_endpoint, packet_bytes);
            }
            max_prev_endpoint = largest_endpoint;
        }
        hil::usb::OutResult::Error
    }

    fn packet_transmitted(&'a self, endpoint: usize) {
        let mut max_prev_endpoint = 1;
        for interface in self.configuration.interfaces() {
            let largest_endpoint = interface.details().num_endpoints + max_prev_endpoint;
            if largest_endpoint > endpoint {
                return interface.packet_transmitted(endpoint);
            }
            max_prev_endpoint = largest_endpoint;
        }
    }
}
