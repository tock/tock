// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Leon Schuermann <leon@is.currently.online> 2023.
// Copyright Tock Contributors 2023.

//! Ethernet network cards

#![deny(missing_docs)]
#![deny(dead_code)]
#![deny(unused_imports)]

use crate::ErrorCode;
use core::fmt;

#[derive(Copy, Clone, PartialEq, Debug)]
/// MAC Address
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    /// Broadcast address
    pub const BROADCAST_MAC_ADDRESS: MacAddress = MacAddress([0xFF; 6]);

    /// MacAddress constructor
    pub const fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    /// Check whether the address is broadcast
    pub const fn is_broadcast(&self) -> bool {
        self.0[0] == 0xFF
            && self.0[1] == 0xFF
            && self.0[2] == 0xFF
            && self.0[3] == 0xFF
            && self.0[4] == 0xFF
            && self.0[5] == 0xFF
    }

    /// Check whether the address is multicast
    pub const fn is_multicast(&self) -> bool {
        self.0[0] & 0x1 != 0
    }

    /// Check whether the address is unicast
    pub const fn is_unicast(&self) -> bool {
        !self.is_multicast() && !self.is_broadcast()
    }
}

impl Default for MacAddress {
    fn default() -> Self {
        Self { 0: [0; 6] }
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
        bytes[2..].copy_from_slice(&address.0);
        u64::from_be_bytes(bytes)
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

/// Ethernet operation mode
#[derive(PartialEq, Debug)]
pub enum OperationMode {
    /// Half-duplex
    HalfDuplex = 0b0,
    /// Full-duplex
    FullDuplex = 0b1,
}

/// Ethernet speed configuration
#[derive(PartialEq, Debug)]
pub enum EthernetSpeed {
    /// 10 Mb/s
    Speed10Mbs = 0b0,
    /// 100 Mb/s
    Speed100Mbs = 0b1,
}

/// Ethernet configuration
pub trait Configure {
    /// Initialize the peripheral
    fn init(&self) -> Result<(), ErrorCode>;

    /// Set operation mode
    fn set_operation_mode(&self, _operation_mode: OperationMode) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    /// Get the current operation mode
    fn get_operation_mode(&self) -> OperationMode;

    /// Set peripheral speed
    fn set_speed(&self, _speed: EthernetSpeed) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    /// Get the current speed
    fn get_speed(&self) -> EthernetSpeed;

    /// Enable loopback mode
    fn set_loopback_mode(&self, _enable: bool) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    /// Check whether loopback mode is enabled
    fn is_loopback_mode_enabled(&self) -> bool;

    /// Set the peripheral MAC address
    fn set_mac_address(&self, _mac_address: MacAddress) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    /// Get the current MAC address
    fn get_mac_address(&self) -> MacAddress;

    // TODO: Move this into the Transmit trait
    /// Start transmission
    fn start_transmit(&self) -> Result<(), ErrorCode>;

    // TODO: Move this into the Receive trait
    /// Start reception
    fn start_receive(&self) -> Result<(), ErrorCode>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mac_address() {
        let mut mac_address = MacAddress::default();
        assert_eq!(&[0; 6], &mac_address.0);
        assert_eq!(MacAddress::new([0x00; 6]), mac_address);

        mac_address = MacAddress::new([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
        assert_eq!(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66], &mac_address.0);

        mac_address
            .0
            .copy_from_slice(&[0x12, 0x34, 0x56, 0x78, 0x90, 0xAB]);
        assert_eq!(&[0x12, 0x34, 0x56, 0x78, 0x90, 0xAB], &mac_address.0);

        mac_address.0[5] = 0xCD;
        assert_eq!(&[0x12, 0x34, 0x56, 0x78, 0x90, 0xCD], &mac_address.0);

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

/// Ethernet adapter client public interface
pub trait EthernetAdapterClient {
    /// Notify the adapter client when the transmission is done.
    ///
    /// Arguments:
    ///
    // TODO: shouldn't the name of this be transmit_result
    /// 1. err: the result of the transmission
    /// 2. packet_buffer: the raw frame that has been transmitted
    /// 3. len: the length of the raw frame
    /// 4. packet_identifier: the identifier of the packet. This was set by
    ///    [EthernetAdapter::transmit]
    /// 5. timestamp: system timestamp
    fn tx_done(
        &self,
        err: Result<(), ErrorCode>,
        packet_buffer: &'static mut [u8],
        len: u16,
        packet_identifier: usize,
        timestamp: Option<u64>,
    );

    /// Notify the adapter client when a packet has been received
    fn rx_packet(&self, packet: &[u8], timestamp: Option<u64>);
}

/// Ethernet adapter public interface
pub trait EthernetAdapter<'a> {
    /// Set an Ethernet adapter client for the peripheral
    fn set_client(&self, client: &'a dyn EthernetAdapterClient);

    /// Transmit a frame
    ///
    /// Arguments:
    ///
    /// 1. packet: Ethernet raw frame to be transmitted
    /// 2. len: the length of the raw frame
    /// 3. packet_identifier: the identifier of the packet. Used when a transmit callback is issued
    fn transmit(
        &self,
        packet: &'static mut [u8],
        len: u16,
        packet_identifier: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;
}
