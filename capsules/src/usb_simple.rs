//! A bare-bones client of the USB hardware interface
//!
//! It responds to standard device requests and can be enumerated.

use usb::*;
use kernel::common::volatile_slice::*;
use kernel::common::copy_slice::*;
use kernel::hil::usb::*;
use core::cell::{RefCell};
use core::ops::DerefMut;
use core::cmp::min;

static LANGUAGES: &'static [u16] = &[
    0x0409, // English (United States)
];

static STRINGS: &'static [&'static str] = &[
    "XYZ Corp.",      // Manufacturer
    "The Zorpinator", // Product
    "Serial No. 5",   // Serial number
];

pub struct SimpleClient<'a, C: 'a> {
    controller: &'a C,
    state: RefCell<State>,
    ep0_buf: VolatileSlice<u8>,
    descriptor_storage: CopySlice<'static, u8>,
}

enum State {
    Init,
    CtrlIn{
        buf: &'static [u8]
    },
    SetAddress,
}

impl<'a, C: UsbController> SimpleClient<'a, C> {
    pub fn new(controller: &'a C) -> Self {
        // XXX It appears we need to declare the storage
        // mutable like this, else later reads will merely
        // return the statically-initialized value.
        let ep0_buf = static_mut_bytes_8();
        let descr_buf = static_mut_bytes_100();

        SimpleClient{
            controller: controller,
            state: RefCell::new(State::Init),
            ep0_buf: VolatileSlice::new_mut(ep0_buf),
            descriptor_storage: CopySlice::new(descr_buf),
        }
    }

    fn map_state<F, R>(&self, f: F) -> R
        where F: FnOnce(&mut State) -> R
    {
        let mut s = self.state.borrow_mut();
        f(s.deref_mut())
    }
}

impl<'a, C: UsbController> Client for SimpleClient<'a, C> {
    fn enable(&self) {
        self.controller.endpoint_set_buffer(0, self.ep0_buf);
        self.controller.enable_device(false);
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
    fn ctrl_setup(&self) -> CtrlSetupResult {
        let buf: &mut [u8] = &mut [0xff; 8];
        copy_from_volatile_slice(buf, self.ep0_buf);

        SetupData::get(buf).map_or(CtrlSetupResult::Error("Couldn't parse setup data"), |setup_data| {
            setup_data.get_standard_request().map_or_else(
                || { CtrlSetupResult::Error(static_fmt!("Nonstandard request: {:?}", setup_data)) },
                |request| {
                match request {
                    StandardDeviceRequest::GetDescriptor{ descriptor_type,
                                                          descriptor_index,
                                                          lang_id,
                                                          requested_length, } => {
                        match descriptor_type {
                            DescriptorType::Device => match descriptor_index {
                                0 => {
                                    self.map_state(|state| {
                                        let d = DeviceDescriptor {
                                                    manufacturer_string: 1,
                                                    product_string: 2,
                                                    serial_number_string: 3,
                                                    .. Default::default()
                                                };
                                        let len = d.write_to(self.descriptor_storage.as_mut());
                                        let end = min(len, requested_length as usize);
                                        *state = State::CtrlIn{ buf: &(self.descriptor_storage.as_slice())[ .. end] };
                                    });
                                    CtrlSetupResult::Ok
                                }
                                _ => CtrlSetupResult::Error("GetDescriptor: invalid Device index"),
                            },
                            DescriptorType::Configuration => match descriptor_index {
                                0 => {
                                    // Place all the descriptors related to this configuration
                                    // into a buffer contiguously, starting with the last

                                    let mut storage_avail = self.descriptor_storage.len();
                                    let s = self.descriptor_storage.as_mut();

                                    let di = InterfaceDescriptor::default();
                                    storage_avail -= di.write_to(&mut s[storage_avail - di.size() ..]);

                                    let dc = ConfigurationDescriptor {
                                                 num_interfaces: 1,
                                                 related_descriptor_length: di.size(),
                                                 .. Default::default() };
                                    storage_avail -= dc.write_to(&mut s[storage_avail - dc.size() ..]);

                                    let request_start = storage_avail;
                                    let request_end = min(request_start + (requested_length as usize),
                                                          self.descriptor_storage.len());
                                    self.map_state(|state| {
                                        *state = State::CtrlIn{
                                            buf: &self.descriptor_storage.as_slice()[request_start ..
                                                                                     request_end]
                                        };
                                    });
                                    CtrlSetupResult::Ok
                                }
                                _ => CtrlSetupResult::Error("GetDescriptor: invalid Configuration index"),
                            },
                            DescriptorType::String => {
                                if let Some(buf) = match descriptor_index {
                                       0 => {
                                            let d = LanguagesDescriptor{ langs: LANGUAGES };
                                            let len = d.write_to(self.descriptor_storage.as_mut());
                                            let end = min(len, requested_length as usize);
                                            Some(&self.descriptor_storage.as_slice()[ .. end])
                                       }
                                       i if i > 0 && (i as usize) <= STRINGS.len() && lang_id == LANGUAGES[0] => {
                                            let d = StringDescriptor{ string: STRINGS[i as usize - 1] };
                                            let len = d.write_to(self.descriptor_storage.as_mut());
                                            let end = min(len, requested_length as usize);
                                            Some(&self.descriptor_storage.as_slice()[ .. end])
                                       },
                                       _ => None,
                                   }
                                {
                                    self.map_state(|state| {
                                        *state = State::CtrlIn{ buf: buf };
                                    });
                                    CtrlSetupResult::Ok
                                }
                                else {
                                    CtrlSetupResult::Error("GetDescriptor: invalid String index")
                                }
                            }
                            DescriptorType::DeviceQualifier => {
                                // We are full-speed only, so we must respond with a request error
                                CtrlSetupResult::Error("GetDescriptor(DeviceQualifier): none")
                            }
                            _ => CtrlSetupResult::Error(static_fmt!("GetDescriptor: unrecognized descriptor type: {:?}", descriptor_type)),
                        } // match descriptor_type
                    }
                    StandardDeviceRequest::SetAddress{device_address} => {
                        // Load the address we've been assigned ...
                        self.controller.set_address(device_address);

                        // ... and when this request gets to the Status stage
                        // we will actually enable the address.
                        self.map_state(|state| {
                            *state = State::SetAddress;
                        });
                        CtrlSetupResult::Ok
                    }
                    StandardDeviceRequest::SetConfiguration{ .. } => {
                        // We have been assigned a particular configuration: fine!
                        CtrlSetupResult::Ok
                    }
                    _ => CtrlSetupResult::Error(static_fmt!("Unrecognized request type: {:?}", request)),
                }
            })
        })
    }

    /// Handle a Control In transaction
    fn ctrl_in(&self) -> CtrlInResult {
        self.map_state(|state| {
            match *state {
                State::CtrlIn{ buf } => {
                    if buf.len() > 0 {
                        let packet_bytes = min(8, buf.len());
                        let packet = &buf[.. packet_bytes];
                        self.ep0_buf.prefix_copy_from_slice(packet);

                        let buf = &buf[packet_bytes ..];
                        let transfer_complete = buf.len() == 0;

                        *state = State::CtrlIn{ buf: buf };

                        CtrlInResult::Packet(packet_bytes, transfer_complete)
                    }
                    else {
                        CtrlInResult::Packet(0, true)
                    }
                }
                _ => CtrlInResult::Error
            }
        })
    }

    /// Handle a Control Out transaction
    ///   (for now, return an error)
    fn ctrl_out(&self, _packet_bytes: u32) -> CtrlOutResult {
        CtrlOutResult::Halted
    }

    fn ctrl_status(&self) {
        // Entered Status stage
    }

    /// Handle the completion of a Control transfer
    fn ctrl_status_complete(&self) {
        // Control Read: IN request acknowledged
        // Control Write: status sent

        self.map_state(|state| {
            match *state {
                State::SetAddress => {
                    self.controller.enable_address();
                },
                _ => {}
            };
            *state = State::Init
        })
    }
}
