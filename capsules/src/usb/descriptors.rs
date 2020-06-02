//! Platform-independent USB 2.0 protocol library.
//!
//! Mostly data types for USB descriptors.

use core::cell::Cell;
use core::cmp::min;
use core::convert::From;
use core::fmt;
use kernel::common::cells::VolatileCell;
use kernel::hil::usb::TransferType;

// On Nordic, USB buffers must be 32-bit aligned, with a power-of-2 size. For
// now we apply these constraints on all platforms.
#[derive(Default)]
#[repr(align(4))]
pub struct Buffer8 {
    pub buf: [VolatileCell<u8>; 8],
}

#[repr(align(4))]
pub struct Buffer64 {
    pub buf: [VolatileCell<u8>; 64],
}

impl Default for Buffer64 {
    fn default() -> Self {
        Self {
            buf: [VolatileCell::default(); 64],
        }
    }
}

/// The data structure sent in a SETUP handshake.
#[derive(Debug, Copy, Clone)]
pub struct SetupData {
    pub request_type: DeviceRequestType,
    pub request_code: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}

impl SetupData {
    /// Create a `SetupData` structure from a packet received from the wire
    pub fn get(p: &[VolatileCell<u8>]) -> Option<Self> {
        if p.len() < 8 {
            return None;
        }
        Some(SetupData {
            request_type: DeviceRequestType(p[0].get()),
            request_code: p[1].get(),
            value: get_u16(p[2].get(), p[3].get()),
            index: get_u16(p[4].get(), p[5].get()),
            length: get_u16(p[6].get(), p[7].get()),
        })
    }

    /// If the `SetupData` represents a standard device request, return it
    pub fn get_standard_request(&self) -> Option<StandardRequest> {
        match self.request_type.request_type() {
            RequestType::Standard => match self.request_code {
                0 => Some(StandardRequest::GetStatus {
                    recipient_index: self.index,
                }),
                1 => Some(StandardRequest::ClearFeature {
                    feature: FeatureSelector::get(self.value),
                    recipient_index: self.index,
                }),
                3 => Some(StandardRequest::SetFeature {
                    feature: FeatureSelector::get(self.value),
                    test_mode: (self.index >> 8) as u8,
                    recipient_index: self.index & 0xff,
                }),
                5 => Some(StandardRequest::SetAddress {
                    device_address: self.value,
                }),
                6 => get_descriptor_type((self.value >> 8) as u8).map_or(None, |dt| {
                    Some(StandardRequest::GetDescriptor {
                        descriptor_type: dt,
                        descriptor_index: (self.value & 0xff) as u8,
                        lang_id: self.index,
                        requested_length: self.length,
                    })
                }),
                7 => get_set_descriptor_type((self.value >> 8) as u8).map_or(None, |dt| {
                    Some(StandardRequest::SetDescriptor {
                        descriptor_type: dt,
                        descriptor_index: (self.value & 0xff) as u8,
                        lang_id: self.index,
                        descriptor_length: self.length,
                    })
                }),
                8 => Some(StandardRequest::GetConfiguration),
                9 => Some(StandardRequest::SetConfiguration {
                    configuration_value: (self.value & 0xff) as u8,
                }),
                10 => Some(StandardRequest::GetInterface {
                    interface: self.index,
                }),
                11 => Some(StandardRequest::SetInterface),
                12 => Some(StandardRequest::SynchFrame),
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum StandardRequest {
    GetStatus {
        recipient_index: u16,
    },
    ClearFeature {
        feature: FeatureSelector,
        recipient_index: u16,
    },
    SetFeature {
        feature: FeatureSelector,
        test_mode: u8,
        recipient_index: u16,
    },
    SetAddress {
        device_address: u16,
    },
    GetDescriptor {
        descriptor_type: DescriptorType,
        descriptor_index: u8,
        lang_id: u16,
        requested_length: u16,
    },
    SetDescriptor {
        descriptor_type: DescriptorType,
        descriptor_index: u8,
        lang_id: u16,
        descriptor_length: u16,
    },
    GetConfiguration,
    SetConfiguration {
        configuration_value: u8,
    },
    GetInterface {
        interface: u16,
    },
    SetInterface,
    SynchFrame,
}

#[derive(Copy, Clone, Debug)]
pub enum DescriptorType {
    Device = 1,
    Configuration,
    String,
    Interface,
    Endpoint,
    DeviceQualifier,
    OtherSpeedConfiguration,
    InterfacePower,
    HID = 0x21,
    Report = 0x22,
    CsInterface = 0x24,
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
        0x21 => Some(DescriptorType::HID),
        0x22 => Some(DescriptorType::Report),
        0x24 => Some(DescriptorType::CsInterface),
        _ => None,
    }
}

/// Get a descriptor type that is legal in a SetDescriptor request
fn get_set_descriptor_type(byte: u8) -> Option<DescriptorType> {
    match get_descriptor_type(byte) {
        dt @ Some(DescriptorType::Device) => dt,
        dt @ Some(DescriptorType::Configuration) => dt,
        dt @ Some(DescriptorType::String) => dt,
        _ => None,
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
        write!(
            f,
            "{{{:?}, {:?}, {:?}}}",
            self.transfer_direction(),
            self.request_type(),
            self.recipient()
        )
    }
}

#[derive(Debug)]
pub enum TransferDirection {
    HostToDevice = 0,
    DeviceToHost = 1,
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
    fn write_to(&self, buf: &[Cell<u8>]) -> usize {
        if self.size() > buf.len() {
            0
        } else {
            self.write_to_unchecked(buf)
        }
    }

    /// Same as `write_to()`, but doesn't check that `buf` is long enough
    /// before indexing into it.  This should be used only if the result
    /// of `size()` is first consulted.
    fn write_to_unchecked(&self, buf: &[Cell<u8>]) -> usize;
}

pub struct DeviceDescriptor {
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

    /// Max packet size for endpoint 0.  Must be 8, 16, 32 or 64
    pub max_packet_size_ep0: u8,

    /// Obtained from USB-IF
    pub vendor_id: u16,

    /// Together with `vendor_id`, this must be unique to the product
    pub product_id: u16,

    /// Device release number in binary coded decimal (BCD)
    pub device_release: u16,

    /// Index of the string descriptor describing manufacturer, or 0 if none
    pub manufacturer_string: u8,

    /// Index of the string descriptor describing product, or 0 if none
    pub product_string: u8,

    /// Index of the string descriptor giving device serial number, or 0 if none
    pub serial_number_string: u8,

    /// Number of configurations the device supports.  Must be at least one
    pub num_configurations: u8,
}

impl Default for DeviceDescriptor {
    fn default() -> Self {
        DeviceDescriptor {
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
    fn size(&self) -> usize {
        18
    }

    fn write_to_unchecked(&self, buf: &[Cell<u8>]) -> usize {
        buf[0].set(18); // Size of descriptor
        buf[1].set(DescriptorType::Device as u8);
        put_u16(&buf[2..4], self.usb_release);
        buf[4].set(self.class);
        buf[5].set(self.subclass);
        buf[6].set(self.protocol);
        buf[7].set(self.max_packet_size_ep0);
        put_u16(&buf[8..10], self.vendor_id);
        put_u16(&buf[10..12], self.product_id);
        put_u16(&buf[12..14], self.device_release);
        buf[14].set(self.manufacturer_string);
        buf[15].set(self.product_string);
        buf[16].set(self.serial_number_string);
        buf[17].set(self.num_configurations);
        18
    }
}

/// Buffer for holding the device descriptor.
// TODO it's dumb that these are Cells, but doing otherwise would require
// rewriting the `write_to` functions
pub struct DeviceBuffer {
    pub buf: [Cell<u8>; 19],
    pub len: usize,
}

impl DeviceBuffer {
    pub fn write_to(&self, buf: &[Cell<u8>]) -> usize {
        for i in 0..self.len {
            buf[i].set(self.buf[i].get());
        }
        self.len
    }
}

/// Buffer for holding the configuration, interface(s), and endpoint(s)
/// descriptors. Also includes class-specific functional descriptors.
pub struct DescriptorBuffer {
    pub buf: [Cell<u8>; 128],
    pub len: usize,
}

impl DescriptorBuffer {
    pub fn write_to(&self, buf: &[Cell<u8>]) -> usize {
        for i in 0..self.len {
            buf[i].set(self.buf[i].get());
        }
        self.len
    }
}

/// Transform descriptor structs into descriptor buffers that can be
/// passed into the control endpoint handler. Each endpoint descriptor list
/// corresponds to the matching index in the interface descriptor list. For
/// example, if the interface descriptor list contains `[ID1, ID2, ID3]`,
/// and the endpoint descriptors list is `[[ED1, ED2], [ED3, ED4, ED5],
/// [ED6]]`, then the third interface descriptor (`ID3`) has one
/// corresponding endpoint descriptor (`ED6`).
pub fn create_descriptor_buffers(
    device_descriptor: DeviceDescriptor,
    mut configuration_descriptor: ConfigurationDescriptor,
    interface_descriptor: &mut [InterfaceDescriptor],
    endpoint_descriptors: &[&[EndpointDescriptor]],
    hid_descriptor: Option<&HIDDescriptor>,
    cdc_descriptor: Option<&[CsInterfaceDescriptor]>,
) -> (DeviceBuffer, DescriptorBuffer) {
    // Create device descriptor buffer and fill.
    // Cell doesn't implement Copy, so here we are.
    let mut dev_buf = DeviceBuffer {
        buf: [
            Cell::default(),
            Cell::default(),
            Cell::default(),
            Cell::default(),
            Cell::default(),
            Cell::default(),
            Cell::default(),
            Cell::default(),
            Cell::default(),
            Cell::default(),
            Cell::default(),
            Cell::default(),
            Cell::default(),
            Cell::default(),
            Cell::default(),
            Cell::default(),
            Cell::default(),
            Cell::default(),
            Cell::default(),
        ],
        len: 0,
    };
    dev_buf.len = device_descriptor.write_to(&dev_buf.buf);

    // Create other descriptors buffer.
    // For the moment, the Default trait is not implemented for arrays
    // of length > 32, and the Cell type is not Copy, so we have to
    // initialize each element manually.
    let mut other_buf = DescriptorBuffer {
        #[rustfmt::skip]
        buf: [
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(), Cell::default(), Cell::default(),
            Cell::default(), Cell::default(), Cell::default(),
        ],
        len: 0,
    };

    // Setup certain descriptor fields since now we know the tree of
    // descriptors.

    // Configuration Descriptor. We assume there is only one configuration
    // descriptor, since this is very common for most USB devices.
    configuration_descriptor.num_interfaces = interface_descriptor.len() as u8;

    // Calculate the length of all dependent descriptors.
    // TODO should we be erroring here if len > 128? Otherwise we'll probably
    // buffer overrun and panic.
    configuration_descriptor.related_descriptor_length =
        interface_descriptor.iter().map(|d| d.size()).sum::<usize>()
            + endpoint_descriptors
                .iter()
                .map(|descs| descs.iter().map(|d| d.size()).sum::<usize>())
                .sum::<usize>()
            + hid_descriptor.map_or(0, |d| d.size())
            + cdc_descriptor.map_or(0, |ds| ds.iter().map(|d| d.size()).sum::<usize>());

    // Set the number of endpoints for each interface descriptor.
    for (i, d) in interface_descriptor.iter_mut().enumerate() {
        d.num_endpoints = endpoint_descriptors[i].len() as u8;
    }

    // Fill a single configuration into the buffer and track length.
    let mut len = 0;
    len += configuration_descriptor.write_to(&other_buf.buf[len..]);

    // Fill in the interface descriptor and its associated endpoints.
    for (i, d) in interface_descriptor.iter().enumerate() {
        // Add the interface descriptor.
        len += d.write_to(&other_buf.buf[len..]);

        // If there is a HID descriptor, we include
        // it with the first interface descriptor.
        if i == 0 {
            // HID descriptor, if any.
            if let Some(dh) = hid_descriptor {
                len += dh.write_to(&other_buf.buf[len..]);
            }
        }

        // If there is a CDC descriptor array, we include
        // it with the first interface descriptor.
        if i == 0 {
            // CDC descriptor, if any.
            if let Some(dcdc) = cdc_descriptor {
                for dcs in dcdc {
                    len += dcs.write_to(&other_buf.buf[len..]);
                }
            }
        }

        // Endpoints for each interface.
        for de in endpoint_descriptors[i] {
            len += de.write_to(&other_buf.buf[len..]);
        }
    }
    other_buf.len = min(len, other_buf.buf.len());

    // return the two buffers
    (dev_buf, other_buf)
}

pub struct ConfigurationDescriptor {
    pub num_interfaces: u8,
    pub configuration_value: u8,
    pub string_index: u8,
    pub attributes: ConfigurationAttributes,
    pub max_power: u8, // in 2mA units
    pub related_descriptor_length: usize,
}

impl Default for ConfigurationDescriptor {
    fn default() -> Self {
        ConfigurationDescriptor {
            num_interfaces: 1,
            configuration_value: 1,
            string_index: 0,
            attributes: ConfigurationAttributes::new(true, false),
            max_power: 0, // in 2mA units
            related_descriptor_length: 0,
        }
    }
}

impl Descriptor for ConfigurationDescriptor {
    fn size(&self) -> usize {
        9
    }

    fn write_to_unchecked(&self, buf: &[Cell<u8>]) -> usize {
        buf[0].set(9); // Size of descriptor
        buf[1].set(DescriptorType::Configuration as u8);
        put_u16(&buf[2..4], (9 + self.related_descriptor_length) as u16);
        buf[4].set(self.num_interfaces);
        buf[5].set(self.configuration_value);
        buf[6].set(self.string_index);
        buf[7].set(From::from(self.attributes));
        buf[8].set(self.max_power);
        9
    }
}

#[derive(Copy, Clone)]
pub struct ConfigurationAttributes(u8);

impl ConfigurationAttributes {
    pub fn new(is_self_powered: bool, supports_remote_wakeup: bool) -> Self {
        ConfigurationAttributes(
            (1 << 7)
                | if is_self_powered { 1 << 6 } else { 0 }
                | if supports_remote_wakeup { 1 << 5 } else { 0 },
        )
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
            num_endpoints: 0,      // (exluding default control endpoint)
            interface_class: 0xff, // vendor_specific
            interface_subclass: 0xab,
            interface_protocol: 0,
            string_index: 0,
        }
    }
}

impl Descriptor for InterfaceDescriptor {
    fn size(&self) -> usize {
        9
    }

    fn write_to_unchecked(&self, buf: &[Cell<u8>]) -> usize {
        buf[0].set(9); // Size of descriptor
        buf[1].set(DescriptorType::Interface as u8);
        buf[2].set(self.interface_number);
        buf[3].set(self.alternate_setting);
        buf[4].set(self.num_endpoints);
        buf[5].set(self.interface_class);
        buf[6].set(self.interface_subclass);
        buf[7].set(self.interface_protocol);
        buf[8].set(self.string_index);
        9
    }
}

pub struct EndpointAddress(u8);

impl EndpointAddress {
    pub fn new(endpoint: usize, direction: TransferDirection) -> Self {
        EndpointAddress(
            endpoint as u8 & 0xf
                | match direction {
                    TransferDirection::HostToDevice => 0,
                    TransferDirection::DeviceToHost => 1,
                } << 7,
        )
    }

    // TODO: Until https://github.com/rust-lang/rust/issues/49146 is resolved, we cannot use `match`
    // in const functions. As we need to initialize static endpoint addresses for the USB client
    // capsule, this function offers a workaround to have a const constructor.
    pub const fn new_const(endpoint: usize, direction: TransferDirection) -> Self {
        EndpointAddress(endpoint as u8 & 0xf | (direction as u8) << 7)
    }
}

pub struct EndpointDescriptor {
    pub endpoint_address: EndpointAddress,
    pub transfer_type: TransferType,
    pub max_packet_size: u16,
    // Poll for device data every `interval` frames
    pub interval: u8,
}

impl Descriptor for EndpointDescriptor {
    fn size(&self) -> usize {
        7
    }

    fn write_to_unchecked(&self, buf: &[Cell<u8>]) -> usize {
        let len = self.size();
        buf[0].set(len as u8);
        buf[1].set(DescriptorType::Endpoint as u8);
        buf[2].set(self.endpoint_address.0);
        // The below implicitly sets Synchronization Type to "No Synchronization" and
        // Usage Type to "Data endpoint"
        buf[3].set(self.transfer_type as u8);
        put_u16(&buf[4..6], self.max_packet_size & 0x7ff as u16);
        buf[6].set(self.interval);
        len
    }
}

#[derive(Copy, Clone)]
pub enum HIDCountryCode {
    NotSupported = 0,
    Arabic,
    Belgian,
    CanadianBilingual,
    CanadianFrench,
    CzechRepublic,
    Danish,
    Finnish,
    French,
    German,
    Greek,
    Hebrew,
    Hungary,
    InternationalISO,
    Italian,
    JapanKatakana,
    Korean,
    LatinAmerican,
    NetherlandsDutch,
    Norwegian,
    PersianFarsi,
    Poland,
    Portuguese,
    Russia,
    Slovakia,
    Spanish,
    Swedish,
    SwissFrench,
    SwissGerman,
    Switzerland,
    Taiwan,
    TurkishQ,
    UK,
    US,
    Yugoslavia,
    TurkishF,
}

pub struct HIDDescriptor<'a> {
    pub hid_class: u16,
    pub country_code: HIDCountryCode,
    pub sub_descriptors: &'a [HIDSubordinateDescriptor],
}

pub struct HIDSubordinateDescriptor {
    pub typ: DescriptorType,
    pub len: u16,
}

impl Descriptor for HIDDescriptor<'_> {
    fn size(&self) -> usize {
        6 + (3 * self.sub_descriptors.len())
    }

    fn write_to_unchecked(&self, buf: &[Cell<u8>]) -> usize {
        let len = self.size();
        buf[0].set(len as u8);
        buf[1].set(DescriptorType::HID as u8);
        put_u16(&buf[2..4], self.hid_class);
        buf[4].set(self.country_code as u8);
        buf[5].set(self.sub_descriptors.len() as u8);
        for (i, desc) in self.sub_descriptors.iter().enumerate() {
            buf[6 + 3 * i].set(desc.typ as u8);
            put_u16(&buf[7 + (3 * i)..9 + (3 * i)], desc.len);
        }
        len
    }
}

pub struct ReportDescriptor<'a> {
    pub desc: &'a [u8],
}

impl Descriptor for ReportDescriptor<'_> {
    fn size(&self) -> usize {
        self.desc.len()
    }

    fn write_to_unchecked(&self, buf: &[Cell<u8>]) -> usize {
        for (i, x) in self.desc.iter().enumerate() {
            buf[i].set(*x);
        }
        self.size()
    }
}

//
// For CDC
//

#[derive(Copy, Clone)]
pub enum CsInterfaceDescriptorSubType {
    Header = 0x00,
    CallManagement = 0x01,
    AbstractControlManagement = 0x02,
    DirectLineManagement = 0x03,
    TelephoneRinger = 0x04,
    TelephoneCallLineStateReportingCapbailities = 0x05,
    Union = 0x06,
    CountrySelection = 0x07,
    TelephoneOperationalModes = 0x08,
    UsbTerminal = 0x09,
    NetworkChannelTerminal = 0x0a,
    ProtocolUnit = 0x0b,
    ExtensionUnity = 0x0c,
    MultiChannelManagement = 0x0d,
    CapiControlManagement = 0x0e,
    EthernetNetworking = 0x0f,
    AtmNetworking = 0x10,
}

pub struct CsInterfaceDescriptor {
    pub subtype: CsInterfaceDescriptorSubType,
    pub field1: u8,
    pub field2: u8,
}

impl Descriptor for CsInterfaceDescriptor {
    fn size(&self) -> usize {
        3 + match self.subtype {
            CsInterfaceDescriptorSubType::Header => 2,
            CsInterfaceDescriptorSubType::CallManagement => 2,
            CsInterfaceDescriptorSubType::AbstractControlManagement => 1,
            CsInterfaceDescriptorSubType::DirectLineManagement => 1,
            CsInterfaceDescriptorSubType::TelephoneRinger => 2,
            CsInterfaceDescriptorSubType::TelephoneCallLineStateReportingCapbailities => 4,
            CsInterfaceDescriptorSubType::Union => 2,
            CsInterfaceDescriptorSubType::CountrySelection => 2,
            CsInterfaceDescriptorSubType::TelephoneOperationalModes => 1,
            CsInterfaceDescriptorSubType::UsbTerminal => 1,
            CsInterfaceDescriptorSubType::NetworkChannelTerminal => 1,
            CsInterfaceDescriptorSubType::ProtocolUnit => 1,
            CsInterfaceDescriptorSubType::ExtensionUnity => 1,
            CsInterfaceDescriptorSubType::MultiChannelManagement => 1,
            CsInterfaceDescriptorSubType::CapiControlManagement => 1,
            CsInterfaceDescriptorSubType::EthernetNetworking => 1,
            CsInterfaceDescriptorSubType::AtmNetworking => 1,
        }
    }

    fn write_to_unchecked(&self, buf: &[Cell<u8>]) -> usize {
        let len = self.size();
        buf[0].set(len as u8);
        buf[1].set(DescriptorType::CsInterface as u8);
        buf[2].set(self.subtype as u8);
        if len >= 4 {
            buf[3].set(self.field1);
        }
        if len >= 5 {
            buf[4].set(self.field2);
        }
        len
    }
}

pub struct LanguagesDescriptor<'a> {
    pub langs: &'a [u16],
}

impl Descriptor for LanguagesDescriptor<'_> {
    fn size(&self) -> usize {
        2 + (2 * self.langs.len())
    }

    fn write_to_unchecked(&self, buf: &[Cell<u8>]) -> usize {
        let len = self.size();
        buf[0].set(len as u8);
        buf[1].set(DescriptorType::String as u8);
        for (i, lang) in self.langs.iter().enumerate() {
            put_u16(&buf[2 + (2 * i)..4 + (2 * i)], *lang);
        }
        len
    }
}

pub struct StringDescriptor<'a> {
    pub string: &'a str,
}

impl Descriptor for StringDescriptor<'_> {
    fn size(&self) -> usize {
        let mut len = 2;
        for ch in self.string.chars() {
            len += 2 * ch.len_utf16();
        }
        len
    }

    // Encode as utf16-le
    fn write_to_unchecked(&self, buf: &[Cell<u8>]) -> usize {
        buf[1].set(DescriptorType::String as u8);
        let mut i = 2;
        for ch in self.string.chars() {
            let mut chbuf = [0; 2];
            for w in ch.encode_utf16(&mut chbuf) {
                put_u16(&buf[i..i + 2], *w);
                i += 2;
            }
        }
        buf[0].set(i as u8);
        i
    }
}

/// Parse a `u16` from two bytes as received on the bus
fn get_u16(b0: u8, b1: u8) -> u16 {
    (b0 as u16) | ((b1 as u16) << 8)
}

/// Write a `u16` to a buffer for transmission on the bus
fn put_u16<'a>(buf: &'a [Cell<u8>], n: u16) {
    buf[0].set((n & 0xff) as u8);
    buf[1].set((n >> 8) as u8);
}
