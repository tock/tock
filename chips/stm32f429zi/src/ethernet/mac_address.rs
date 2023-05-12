use kernel::debug;
use kernel::hil::ethernet::MacAddress;

pub mod tests {
    use super::*;

    pub fn test_mac_address() {
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing Ethernet MAC address struct...");

        let mut mac_address = MacAddress::new();
        assert_eq!([0; 6], mac_address.get_address());
        mac_address = MacAddress::from(0x112233445566);
        assert_eq!([0x11, 0x22, 0x33, 0x44, 0x55, 0x66], mac_address.get_address());
        assert_eq!(0x112233445566 as u64, mac_address.into());
        mac_address.set_address(0x1234567890AB);
        assert_eq!([0x12, 0x34, 0x56, 0x78, 0x90, 0xAB], mac_address.get_address());
        assert_eq!(0x1234567890AB as u64, mac_address.into());

        debug!("Finished testing Ethernet MAC address struct");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    }
}
