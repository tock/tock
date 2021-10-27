use core::cell::Cell;
use super::descriptors::{Descriptor, ConfigurationAttributes, DescriptorType, TransferDirection, SetupData};
use kernel::utilities::cells::{OptionalCell, VolatileCell};
use kernel::hil::usb::{CtrlOutResult, CtrlSetupResult, TransferType};

pub struct InterfaceIterator<'a, 'b, C: ?Sized> {
    inner: &'a C,
    idx: usize,
    _marker: core::marker::PhantomData<&'b C>,
}

impl<'a: 'b, 'b, C: Configuration> Iterator for InterfaceIterator<'a, 'b, C> {
    type Item = &'b dyn Interface;
    
    fn next(&mut self) -> Option<Self::Item> {
        let prev = self.idx;
        self.idx += 1;
        self.inner.interface(prev)
    }
}

pub struct ConfigurationDetails {
    pub string_index: u8,
    pub attributes: ConfigurationAttributes,
    pub max_power: u8,
    pub num_interfaces: usize,
}

pub trait Configuration {
    fn details(&self) -> ConfigurationDetails;

    // when generic_const_exprs is more stable, we can use the NUM_INTERFACES constant to return a
    // fixed-sized array instead
    fn interface(&self, i: usize) -> Option<&dyn Interface>;

    fn interfaces<'a, 'b>(&'a self) -> InterfaceIterator<'a, 'b, Self> {
        InterfaceIterator {
            inner: self,
            idx: 0,
            _marker: core::marker::PhantomData,
        }
    }

    fn size(&self) -> usize {
        let mut total_size = 9;
        let mut i = 0;
        while let Some(interface) = self.interface(i) {
            total_size += interface.size();
            i += 1;
        }
        total_size
    }

    fn write_to(&self, configuration_num: u8, buf: &[Cell<u8>]) -> usize {
        let details = self.details();
        buf[0].set(9); // Size of descriptor
        buf[1].set(DescriptorType::Configuration as u8);
        put_u16(&buf[2..4], self.size() as u16);
        buf[4].set(details.num_interfaces as u8);
        buf[5].set(configuration_num);
        buf[6].set(details.string_index);
        buf[7].set(From::from(details.attributes));
        buf[8].set(details.max_power);
        9
    }
}

pub struct SimpleConfiguration<'a, I: AsRef<[&'a dyn Interface]>> {
    pub interfaces: I,
    pub is_self_powered: bool,
    pub supports_remote_wakeup: bool,
    pub max_power: u8,
    pub _f: core::marker::PhantomData<&'a ()>,
}

impl<'a, I: AsRef<[&'a dyn Interface]>> Configuration for SimpleConfiguration<'a, I> {
    fn details(&self) -> ConfigurationDetails {
        ConfigurationDetails {
            string_index: 0,
            attributes: ConfigurationAttributes::new(self.is_self_powered, self.supports_remote_wakeup),
            max_power: self.max_power, // in 2ma units
            num_interfaces: self.interfaces.as_ref().len(),
        }
    }

    fn interface(&self, i: usize) -> Option<&dyn Interface> {
        self.interfaces.as_ref().get(i).map(|x| *x)
    }
}

pub struct InterfaceDetails {
    pub alternate_setting: u8,
    pub interface_class: u8,
    pub interface_subclass: u8,
    pub interface_protocol: u8,
    pub string_index: u8,
    pub num_endpoints: usize,
}

pub trait Interface {
    fn details(&self) -> InterfaceDetails;
    fn class_descriptor(&self, i: usize) -> Option<&dyn Descriptor>;
    fn endpoint(&self, i: usize) -> Option<&Endpoint>;

    fn size(&self) -> usize {
        let mut total_size = 9;
        let mut i = 0;
        while let Some(cd) = self.class_descriptor(i) {
            total_size += cd.size();
            i += 1;
        }
        let mut i = 0;
        while let Some(ep) = self.endpoint(i) {
            total_size += ep.size();
            i += 1;
        }
        total_size
    }

    fn write_to(&self, interface_num: u8, buf: &[Cell<u8>]) -> usize {
        let details = self.details();
        buf[0].set(9); // Size of descriptor
        buf[1].set(DescriptorType::Interface as u8);
        buf[2].set(interface_num);
        buf[3].set(details.alternate_setting);
        buf[4].set(details.num_endpoints as u8);
        buf[5].set(details.interface_class);
        buf[6].set(details.interface_subclass);
        buf[7].set(details.interface_protocol);
        buf[8].set(details.string_index);
        9
    }

    fn ctrl_setup(&self, _setup_data: SetupData) -> CtrlSetupResult {
        CtrlSetupResult::ErrGeneric
    }

    fn ctrl_out(&self, _buf: [VolatileCell<u8>; 64], _packet_bytes: u32) -> CtrlOutResult {
        CtrlOutResult::Halted
    }

    fn ctrl_status_complete(&self, _endpoint: usize) {
    }

    fn packet_in(&self, _transfer_type: TransferType, _endpoint: usize) -> kernel::hil::usb::InResult {
        kernel::hil::usb::InResult::Delay
    }

    fn packet_out(
        &self,
        _transfer_type: TransferType,
        _endpoint: usize,
        _packet_bytes: u32,
    ) -> kernel::hil::usb::OutResult {
        kernel::hil::usb::OutResult::Ok
    }

    fn packet_transmitted(&self, _endpoint: usize) {}

    fn bus_reset(&self) {}
}

pub struct Endpoint {
    pub direction: TransferDirection,
    pub transfer_type: TransferType,
    // Poll for device data every `interval` frames
    pub interval: u8,
    pub buffer: Option<super::descriptors::Buffer64>,
    pub endpoint_number: OptionalCell<u8>,
}

impl Endpoint {
    fn size(&self) -> usize {
        7
    }

    pub fn buffer<'a>(&'a self) -> &'a [kernel::utilities::cells::VolatileCell<u8>] {
        if let Some(ref buf) = self.buffer {
            &buf.buf
        } else {
            &[]
        }
    }

    pub fn write_to(&self, endpoint_number: u8, buf: &[Cell<u8>]) -> usize {
        self.endpoint_number.set(endpoint_number);
        let len = self.size();
        buf[0].set(len as u8);
        buf[1].set(DescriptorType::Endpoint as u8);
        let address = endpoint_number & 0xf | match self.direction {
            TransferDirection::HostToDevice => 0,
            TransferDirection::DeviceToHost => 1,
        } << 7;
        buf[2].set(address);
        // The below implicitly sets Synchronization Type to "No Synchronization" and
        // Usage Type to "Data endpoint"
        buf[3].set(self.transfer_type as u8);
        put_u16(&buf[4..6], self.buffer.as_ref().map_or(0, |b| b.buf.len() as u16) & 0x7ff);
        buf[6].set(self.interval);
        len
    }
}


/// Write a `u16` to a buffer for transmission on the bus
pub(crate) fn put_u16<'a>(buf: &'a [Cell<u8>], n: u16) {
    buf[0].set((n & 0xff) as u8);
    buf[1].set((n >> 8) as u8);
}
