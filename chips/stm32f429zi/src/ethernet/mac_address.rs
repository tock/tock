use kernel::debug;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct MacAddress {
    address: [u8; 6],
}

impl MacAddress {
    pub fn new() -> Self {
        Self {
            address: [0; 6],
        }
    }

    pub fn set_address(&mut self, address: u64)  {
        let mask: u64 = 0xFF0000000000;
        for index in 0..6 {
            self.address[index] = ((address & (mask >> (index * 8))) >> (40 - 8 * index)) as u8;
        }
    }

    pub fn get_address(&self) -> [u8; 6] {
        // Never panics because address is never assigned to none
        self.address
    }
}

impl From<u64> for MacAddress {
    fn from(value: u64) -> Self {
        let mut mac_address = MacAddress::new();
        mac_address.set_address(value);
        mac_address
    }
}

impl From<MacAddress> for u64 {
    fn from(mac_address: MacAddress) -> Self {
        let mut result: u64 = 0;
        for byte in mac_address.get_address() {
            result += byte as u64;
            result <<= 8;
        }

        result >> 8
    }
}

pub mod tests {
    use super::*;
    use crate::ethernet::DEFAULT_MAC_ADDRESS;

    pub fn test_mac_address() {
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing Ethernet MAC address struct...");

        let mut mac_address = MacAddress::new();
        assert_eq!([0; 6], mac_address.get_address());
        mac_address.set_address(DEFAULT_MAC_ADDRESS);
        assert_eq!([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC], mac_address.get_address());
        let mac_address = MacAddress::from(0x112233445566);
        assert_eq!([0x11, 0x22, 0x33, 0x44, 0x55, 0x66], mac_address.get_address());
        assert_eq!(0x112233445566 as u64, mac_address.into());

        debug!("Finished testing Ethernet MAC address struct");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    }
}
