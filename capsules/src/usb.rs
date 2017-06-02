//! Platform-independent USB 2.0 protocol library

use core::fmt;
use core::convert::From;

/// The datastructure sent in a SETUP handshake
#[derive(Debug, Copy, Clone)]
pub struct SetupData {
    pub request_type: DeviceRequestType,
    pub request_code: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}

impl SetupData {
    /// Create a `SetupData` structure from a buffer as received from the wire
    pub fn get(buf: &[u8]) -> Option<Self> {
        if buf.len() != 8 {
            return None;
        }
        Some(SetupData {
            request_type: DeviceRequestType(buf[0]),
            request_code: buf[1],
            value: get_u16(&buf[2..4]).unwrap(),
            index: get_u16(&buf[4..6]).unwrap(),
            length: get_u16(&buf[6..8]).unwrap(),
        })
    }

    /// If the `SetupData` represents a standard device request, return it
    pub fn get_standard_request(&self) -> Option<StandardDeviceRequest> {
        match self.request_type.request_type() {
            RequestType::Standard =>
                match self.request_code {
                    0 => Some(StandardDeviceRequest::GetStatus{
                             recipient_index: self.index
                         }),
                    1 => Some(StandardDeviceRequest::ClearFeature{
                            feature: FeatureSelector::get(self.value),
                            recipient_index: self.index,
                         }),
                    3 => Some(StandardDeviceRequest::SetFeature{
                            feature: FeatureSelector::get(self.value),
                            test_mode: (self.index >> 8) as u8,
                            recipient_index: self.index & 0xff,
                         }),
                    5 => Some(StandardDeviceRequest::SetAddress{
                            device_address: self.value
                         }),
                    6 => {
                        get_descriptor_type((self.value >> 8) as u8).map_or(None, |dt| {
                            Some(StandardDeviceRequest::GetDescriptor{
                                    descriptor_type: dt,
                                    descriptor_index: (self.value & 0xff) as u8,
                                    lang_id: self.index,
                                    requested_length: self.length,
                            })
                        })
                    }
                    7 => {
                        get_set_descriptor_type((self.value >> 8) as u8).map_or(None, |dt| {
                            Some(StandardDeviceRequest::SetDescriptor{
                                descriptor_type: dt,
                                descriptor_index: (self.value & 0xff) as u8,
                                lang_id: self.index,
                                descriptor_length: self.length,
                            })
                        })
                    }
                    8 => Some(StandardDeviceRequest::GetConfiguration),
                    9 => Some(StandardDeviceRequest::SetConfiguration{
                            configuration_value: (self.value & 0xff) as u8
                         }),
                    10 => Some(StandardDeviceRequest::GetInterface{
                              interface: self.index
                          }),
                    11 => Some(StandardDeviceRequest::SetInterface),
                    12 => Some(StandardDeviceRequest::SynchFrame),
                    _ => None,
                },
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum StandardDeviceRequest {
    GetStatus{
        recipient_index: u16,
    },
    ClearFeature{
        feature: FeatureSelector,
        recipient_index: u16,
    },
    SetFeature{
        feature: FeatureSelector,
        test_mode: u8,
        recipient_index: u16,
    },
    SetAddress{
        device_address: u16,
    },
    GetDescriptor{
        descriptor_type: DescriptorType,
        descriptor_index: u8,
        lang_id: u16,
        requested_length: u16,
    },
    SetDescriptor{
        descriptor_type: DescriptorType,
        descriptor_index: u8,
        lang_id: u16,
        descriptor_length: u16,
    },
    GetConfiguration,
    SetConfiguration{
        configuration_value: u8,
    },
    GetInterface{
        interface: u16,
    },
    SetInterface,
    SynchFrame,
}

#[derive(Debug)]
pub enum DescriptorType {
    Device = 1,
    Configuration,
    String,
    Interface,
    Endpoint,
    DeviceQualifier,
    OtherSpeedConfiguration,
    InterfacePower,
}

fn get_descriptor_type(byte: u8) -> Option<DescriptorType> {
    match byte {
        1 => Some(DescriptorType::Device),
        2 => Some(DescriptorType::Configuration),
        3 => Some(DescriptorType::String),
        4 => Some(DescriptorType::Interface),
        5 => Some(DescriptorType::Endpoint),
        6 => Some(DescriptorType::DeviceQualifier),
        7 => Some(DescriptorType::OtherSpeedConfiguration),
        8 => Some(DescriptorType::InterfacePower),
        _ => None,
    }
}

/// Get a descriptor type that is legal in a SetDescriptor request
fn get_set_descriptor_type(byte: u8) -> Option<DescriptorType> {
    match get_descriptor_type(byte) {
        dt @ Some(DescriptorType::Device) => dt,
        dt @ Some(DescriptorType::Configuration) => dt,
        dt @ Some(DescriptorType::String) => dt,
        _ => None
    }
}

#[derive(Copy, Clone)]
pub struct DeviceRequestType(u8);

impl DeviceRequestType {
    pub fn transfer_direction(self) -> TransferDirection {
        match self.0 & (1 << 7) {
            0 => TransferDirection::HostToDevice,
            _ => TransferDirection::DeviceToHost,
        }
    }

    pub fn request_type(self) -> RequestType {
        match (self.0 & (0b11 << 5)) >> 5 {
            0 => RequestType::Standard,
            1 => RequestType::Class,
            2 => RequestType::Vendor,
            _ => RequestType::Reserved,
        }
    }

    pub fn recipient(self) -> Recipient {
        match self.0 & 0b11111 {
            0 => Recipient::Device,
            1 => Recipient::Interface,
            2 => Recipient::Endpoint,
            3 => Recipient::Other,
            _ => Recipient::Reserved,
        }
    }
}

impl fmt::Debug for DeviceRequestType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{{:?}, {:?}, {:?}}}",
               self.transfer_direction(), self.request_type(), self.recipient())
    }
}

#[derive(Debug)]
pub enum TransferDirection {
    DeviceToHost,
    HostToDevice,
}

#[derive(Debug)]
pub enum RequestType {
    Standard,
    Class,
    Vendor,
    Reserved,
}

#[derive(Debug)]
pub enum Recipient {
    Device,
    Interface,
    Endpoint,
    Other,
    Reserved,
}

#[derive(Debug)]
pub enum FeatureSelector {
    DeviceRemoteWakeup,
    EndpointHalt,
    TestMode,
    Unknown,
}

impl FeatureSelector {
    fn get(value: u16) -> Self {
        match value {
            1 => FeatureSelector::DeviceRemoteWakeup,
            0 => FeatureSelector::EndpointHalt,
            2 => FeatureSelector::TestMode,
            _ => FeatureSelector::Unknown,
        }
    }
}

pub trait Descriptor {
    /// Serialized size of Descriptor
    fn size(&self) -> usize;

    /// Serialize the descriptor to a buffer for transmission on the bus
    fn write_to(&self, buf: &mut [u8]) -> usize;
}

pub struct DeviceDescriptor {
    pub usb_release: u16,
    pub class: u8,
    pub subclass: u8,
    pub protocol: u8,
    pub max_packet_size_ep0: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_release: u16,
    pub manufacturer_string: u8,
    pub product_string: u8,
    pub serial_number_string: u8,
    pub num_configurations: u8,
}

impl Default for DeviceDescriptor {
    fn default() -> Self {
        DeviceDescriptor{
            usb_release: 0x0200,
            class: 0,
            subclass: 0,
            protocol: 0,
            max_packet_size_ep0: 8,
            vendor_id: 0x6667,
            product_id: 0xabcd,
            device_release: 0x0001,
            manufacturer_string: 0,
            product_string: 0,
            serial_number_string: 0,
            num_configurations: 1,
        }
    }
}

impl Descriptor for DeviceDescriptor {
    fn size(&self) -> usize { 18 }

    fn write_to(&self, b: &mut [u8]) -> usize {
        b[0] = 18;  // Size of descriptor
        b[1] = DescriptorType::Device as u8;
        put_u16(&mut b[2..4], self.usb_release);
        b[4] = self.class;
        b[5] = self.subclass;
        b[6] = self.protocol;
        b[7] = self.max_packet_size_ep0;
        put_u16(&mut b[8..10], self.vendor_id);
        put_u16(&mut b[10..12], self.product_id);
        put_u16(&mut b[12..14], self.device_release);
        b[14] = self.manufacturer_string;
        b[15] = self.product_string;
        b[16] = self.serial_number_string;
        b[17] = self.num_configurations;
        18
    }
}

pub struct ConfigurationDescriptor {
     pub num_interfaces: u8,
     pub configuration_value: u8,
     pub string_index: u8,
     pub attributes: ConfigurationAttributes,
     pub max_power: u8,   // in 2mA units
     pub related_descriptor_length: usize,
}

impl Default for ConfigurationDescriptor {
    fn default() -> Self {
        ConfigurationDescriptor {
            num_interfaces: 1,
            configuration_value: 0,
            string_index: 0,
            attributes: ConfigurationAttributes::new(true, false),
            max_power: 0,   // in 2mA units
            related_descriptor_length: 0
        }
    }
}

impl Descriptor for ConfigurationDescriptor {
    fn size(&self) -> usize { 9 }

    fn write_to(&self, buf: &mut [u8]) -> usize {
        buf[0] = 9; // Size of descriptor
        buf[1] = DescriptorType::Configuration as u8;
        put_u16(&mut buf[2..4], (9 + self.related_descriptor_length) as u16);
        buf[4] = self.num_interfaces;
        buf[5] = self.configuration_value;
        buf[6] = self.string_index;
        buf[7] = From::from(self.attributes);
        buf[8] = self.max_power;
        9
    }
}

#[derive(Copy, Clone)]
pub struct ConfigurationAttributes(u8);

impl ConfigurationAttributes {
    pub fn new(is_self_powered: bool, supports_remote_wakeup: bool) -> Self {
        ConfigurationAttributes((1 << 7)
                                | if is_self_powered { 1 << 6 } else { 0 }
                                | if supports_remote_wakeup { 1 << 5 } else { 0 })
    }
}

impl From<ConfigurationAttributes> for u8 {
    fn from(ca: ConfigurationAttributes) -> u8 {
        ca.0
    }
}

pub struct InterfaceDescriptor {
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub num_endpoints: u8,
    pub interface_class: u8,
    pub interface_subclass: u8,
    pub interface_protocol: u8,
    pub string_index: u8,
}

impl Default for InterfaceDescriptor {
    fn default() -> Self {
        InterfaceDescriptor {
            interface_number: 0,
            alternate_setting: 0,
            num_endpoints: 0, // (exluding default control endpoint)
            interface_class: 0xff, // vendor_specific
            interface_subclass: 0xab,
            interface_protocol: 0,
            string_index: 0,
        }
    }
}

impl Descriptor for InterfaceDescriptor {
    fn size(&self) -> usize { 9 }

    fn write_to(&self, buf: &mut [u8]) -> usize {
        buf[0] = 9; // Size of descriptor
        buf[1] = DescriptorType::Interface as u8;
        buf[2] = self.interface_number;
        buf[3] = self.alternate_setting;
        buf[4] = self.num_endpoints;
        buf[5] = self.interface_class;
        buf[6] = self.interface_subclass;
        buf[7] = self.interface_protocol;
        buf[8] = self.string_index;
        9
    }
}

pub struct LanguagesDescriptor<'a> {
    pub langs: &'a [u16],
}

impl<'a> Descriptor for LanguagesDescriptor<'a> {
    fn size(&self) -> usize {
        2 + (2 * self.langs.len())
    }

    fn write_to(&self, buf: &mut [u8]) -> usize {
        let len = self.size();
        buf[0] = len as u8;
        buf[1] = DescriptorType::String as u8;
        for (i, lang) in self.langs.iter().enumerate() {
            put_u16(&mut buf[2 + (2 * i) .. 4 + (2 * i)], *lang);
        }
        len
    }
}

pub struct StringDescriptor<'a> {
    pub string: &'a str,
}

impl<'a> Descriptor for StringDescriptor<'a> {
    fn size(&self) -> usize {
        let mut len = 2;
        for ch in self.string.chars() {
            len += 2 * ch.len_utf16();
        }
        len
    }

    // Encode as utf16-le
    fn write_to(&self, buf: &mut [u8]) -> usize {
        buf[1] = DescriptorType::String as u8;
        let mut i = 2;
        for ch in self.string.chars() {
            let mut chbuf = [0; 2];
            for w in ch.encode_utf16(&mut chbuf) {
                put_u16(&mut buf[i .. i+2], *w);
                i += 2;
            }
        }
        buf[0] = i as u8;
        i
    }
}

/// Parse a `u16` from a two-byte buffer as received on the bus
fn get_u16(buf: &[u8]) -> Option<u16> {
    if buf.len() != 2 {
        return None;
    }
    Some ((buf[0] as u16) | ((buf[1] as u16) << 8))
}

/// Write a `u16` to a buffer for transmission on the bus
fn put_u16<'a>(buf: &'a mut [u8], n: u16) {
    if buf.len() != 2 {
        panic!("Wrong length");
    }
    buf[0] = (n & 0xff) as u8;
    buf[1] = (n >> 8) as u8;
}
