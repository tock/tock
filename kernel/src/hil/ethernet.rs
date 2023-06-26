// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Leon Schuermann <leon@is.currently.online> 2023.
// Copyright Tock Contributors 2023.

//! Ethernet network cards
use crate::ErrorCode;
use core::fmt;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    pub const BROADCAST_MAC_ADDRESS: MacAddress = MacAddress([0xFF; 6]);

    pub const fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    pub fn set(&mut self, bytes: &[u8; 6]) {
        // Can't panic
        self.0.copy_from_slice(bytes);
    }

    const fn get(&self) -> &[u8; 6] {
        &self.0
    }

    pub const fn is_broadcast(&self) -> bool {
        self.0[0] == 0xFF &&
        self.0[1] == 0xFF &&
        self.0[2] == 0xFF &&
        self.0[3] == 0xFF &&
        self.0[4] == 0xFF &&
        self.0[5] == 0xFF
    }

    pub const fn is_multicast(&self) -> bool {
        self.0[0] & 0x1 != 0
    }

    pub const fn is_unicast(&self) -> bool {
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
    fn from(address: MacAddress) -> Self {
        let mut bytes = [0 as u8; 8];
        bytes[2..].copy_from_slice(address.get());
        u64::from_be_bytes(bytes)
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}",
            self.get()[0], self.get()[1], self.get()[2],
            self.get()[3], self.get()[4], self.get()[5]
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

    // TODO: Move this into the Transmit trait
    fn start_transmit(&self) -> Result<(), ErrorCode>;

    // TODO: Move this into the Receive trait
    fn start_receive(&self) -> Result<(), ErrorCode>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mac_address() {
        let mut mac_address = MacAddress::default();
        assert_eq!(&[0; 6], mac_address.get());
        assert_eq!(MacAddress::from(0x0 as u64), mac_address);

        mac_address = MacAddress::from(0x112233445566);
        assert_eq!(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66], mac_address.get());
        assert_eq!(0x112233445566 as u64, mac_address.into());

        mac_address.set(&[0x12, 0x34, 0x56, 0x78, 0x90, 0xAB]);
        assert_eq!(&[0x12, 0x34, 0x56, 0x78, 0x90, 0xAB], mac_address.get());
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

pub trait EthernetAdapterClient {
    fn tx_done(
        &self,
        err: Result<(), ErrorCode>,
        packet_buffer: &'static mut [u8],
        len: u16,
        packet_identifier: usize,
        timestamp: Option<u64>,
    );
    fn rx_packet(&self, packet: &[u8], timestamp: Option<u64>);
}

pub trait EthernetAdapter<'a> {
    fn set_client(&self, client: &'a dyn EthernetAdapterClient);
    fn transmit(
        &self,
        packet: &'static mut [u8],
        len: u16,
        packet_identifier: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;
}
