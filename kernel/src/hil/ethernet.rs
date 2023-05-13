use crate::ErrorCode;
use core::fmt;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    pub const BROADCAST_MAC_ADDRESS: MacAddress = MacAddress([0xFF; 6]);

    pub const fn new(bytes: [u8; 6]) -> Self {
        Self {
            0: bytes
        }
    }

    pub fn set_address(&mut self, bytes: &[u8; 6]) {
        // Can't panic
        self.0.copy_from_slice(bytes);
    }

    pub const fn get_address(&self) -> [u8; 6] {
        self.0
    }

    pub fn is_broadcast(&self) -> bool {
        *self == Self::BROADCAST_MAC_ADDRESS
    }

    pub const fn is_multicast(&self) -> bool {
        self.get_address()[0] & 0x1 != 0
    }

    pub fn is_unicast(&self) -> bool {
        !self.is_multicast() && !self.is_broadcast()
    }
}

impl Default for MacAddress {
    fn default() -> Self {
        Self {
            0: [0; 6]
        }
    }
}

impl From<u64> for MacAddress {
    fn from(value: u64) -> Self {
        // Can't panic
        MacAddress(value.to_be_bytes()[2..8].try_into().unwrap())
    }
}

impl From<MacAddress> for u64 {
    fn from(mac_address: MacAddress) -> Self {
        // Can't panic
        let high: u16 = u16::from_be_bytes(mac_address.get_address()[0..2].try_into().unwrap());
        let low: u32 = u32::from_be_bytes(mac_address.get_address()[2..6].try_into().unwrap());

        ((high as u64) << 32) + (low as u64)
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}",
            self.get_address()[0], self.get_address()[1], self.get_address()[2],
            self.get_address()[3], self.get_address()[4], self.get_address()[5]
        )
    }
}

#[derive(PartialEq, Debug)]
pub enum OperationMode {
    HalfDuplex = 0b0,
    FullDuplex = 0b1,
}

#[derive(PartialEq, Debug)]
pub enum EthernetSpeed {
    Speed10Mbs = 0b0,
    Speed100Mbs = 0b1,
}

pub trait Configure {
    fn init(&self) -> Result<(), ErrorCode>;

    fn set_operation_mode(&self, _operation_mode: OperationMode) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn get_operation_mode(&self) -> OperationMode;

    fn set_speed(&self, _speed: EthernetSpeed) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn get_speed(&self) -> EthernetSpeed;

    fn set_loopback_mode(&self, _enable: bool) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn is_loopback_mode_enabled(&self) -> bool;

    fn set_mac_address(&self, _mac_address: MacAddress) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn get_mac_address(&self) -> MacAddress;
    
    fn start_transmitter(&self) -> Result<(), ErrorCode>;

    fn stop_transmitter(&self) -> Result<(), ErrorCode>;

    fn is_transmitter_up(&self) -> bool;

    fn start_receiver(&self) -> Result<(), ErrorCode>;

    fn stop_receiver(&self) -> Result<(), ErrorCode>;

    fn is_receiver_up(&self) -> bool;

    fn start(&self) -> Result<(), ErrorCode> {
        self.start_transmitter()?;
        self.start_receiver()
    }

    fn stop(&self) -> Result<(), ErrorCode> {
        self.stop_transmitter()?;
        self.stop_receiver()
    }

    fn is_up(&self) -> bool {
        self.is_transmitter_up() && self.is_receiver_up()
    }
}

pub trait Transmit<'a> {
    fn set_transmit_client(&self, transmit_client: &'a dyn TransmitClient);
    fn transmit_data(&self, destination_address: MacAddress, data: &[u8]) -> Result<(), ErrorCode>;
}

pub trait Receive<'a> {
    fn set_receive_client(&self, receive_client: &'a dyn ReceiveClient);
    fn receive_frame(&self, buffer: &mut [u8]) -> Result<(), ErrorCode>;
}

pub trait TransmitClient {
    fn transmitted_frame(&self, transmit_status: Result<(), ErrorCode>);
}

pub trait ReceiveClient {
    fn received_frame(&self,
        receive_status: Result<(), ErrorCode>,
        received_frame_length: usize
    );
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_mac_address() {
        let mut mac_address = MacAddress::default();
        assert_eq!([0; 6], mac_address.get_address());
        assert_eq!(0x0 as u64, mac_address.into());

        mac_address = MacAddress::from(0x112233445566);
        assert_eq!([0x11, 0x22, 0x33, 0x44, 0x55, 0x66], mac_address.get_address());
        assert_eq!(0x112233445566 as u64, mac_address.into());

        mac_address.set_address(&[0x12, 0x34, 0x56, 0x78, 0x90, 0xAB]);
        assert_eq!([0x12, 0x34, 0x56, 0x78, 0x90, 0xAB], mac_address.get_address());
        assert_eq!(0x1234567890AB as u64, mac_address.into());

        assert_eq!(false, mac_address.is_broadcast());
        assert_eq!(false, mac_address.is_multicast());
        assert_eq!(true, mac_address.is_unicast());

        mac_address = MacAddress([0xFF; 6]);
        assert_eq!(true, mac_address.is_broadcast());
        assert_eq!(true, mac_address.is_multicast());
        assert_eq!(false, mac_address.is_unicast());

        mac_address = MacAddress::new([0x13, 0x34, 0x56, 0x78, 0x90, 0xAB]);
        assert_eq!(false, mac_address.is_broadcast());
        assert_eq!(true, mac_address.is_multicast());
        assert_eq!(false, mac_address.is_unicast());
    }
}
