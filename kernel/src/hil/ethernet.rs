// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Leon Schuermann <leon@is.currently.online> 2023.
// Copyright Tock Contributors 2023.

//! Ethernet network cards

use crate::ErrorCode;

/// Ethernet adapter client public interface
pub trait EthernetAdapterClient {
    /// Transmit callback
    ///
    /// Arguments:
    ///
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

    /// Receive callback
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
