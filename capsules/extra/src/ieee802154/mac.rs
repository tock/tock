// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Specifies the interface for IEEE 802.15.4 MAC protocol layers. MAC protocols
//! expose similar configuration (address, PAN, transmission power) options
//! as ieee802154::device::MacDevice layers above it, but retain control over
//! radio power management and channel selection. All frame processing should
//! be completed above this layer such that Mac implementations receive fully
//! formatted 802.15.4 MAC frames for transmission.
//!
//! AwakeMac provides a default implementation of such a layer, maintaining
//! the underlying kernel::hil::radio::Radio powered at all times and passing
//! through each frame for transmission.

use crate::net::ieee802154::{Header, MacAddress};
use kernel::hil::radio::{self, MAX_FRAME_SIZE, PSDU_OFFSET};
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

pub trait Mac<'a> {
    /// Initializes the layer.
    fn initialize(&self) -> Result<(), ErrorCode>;

    /// Sets the notified client for configuration changes
    fn set_config_client(&self, client: &'a dyn radio::ConfigClient);
    /// Sets the notified client for transmission completions
    fn set_transmit_client(&self, client: &'a dyn radio::TxClient);
    /// Sets the notified client for frame receptions
    fn set_receive_client(&self, client: &'a dyn radio::RxClient);
    /// Sets the buffer for packet reception
    fn set_receive_buffer(&self, buffer: &'static mut [u8]);

    /// The short 16-bit address of the radio
    fn get_address(&self) -> u16;
    /// The long 64-bit address of the radio
    fn get_address_long(&self) -> [u8; 8];
    /// The 16-bit PAN id of the radio
    fn get_pan(&self) -> u16;

    /// Sets the short 16-bit address of the radio
    fn set_address(&self, addr: u16);
    /// Sets the long 64-bit address of the radio
    fn set_address_long(&self, addr: [u8; 8]);
    /// Sets the 16-bit PAN id of the radio
    fn set_pan(&self, id: u16);

    /// Must be called after one or more calls to `set_*`. If
    /// `set_*` is called without calling `config_commit`, there is no guarantee
    /// that the underlying hardware configuration (addresses, pan ID) is in
    /// line with this MAC protocol implementation. The specified config_client is
    /// notified on completed reconfiguration.
    fn config_commit(&self);

    /// Indicates whether or not the MAC protocol is active and can send frames
    fn is_on(&self) -> bool;

    /// Transmits complete MAC frames, which must be prepared by an ieee802154::device::MacDevice
    /// before being passed to the Mac layer. Returns the frame buffer in case of an error.
    fn transmit(
        &self,
        full_mac_frame: &'static mut [u8],
        frame_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;
}

///
/// Default implementation of a Mac layer. Acts as a pass-through between a MacDevice
/// implementation and the underlying radio::Radio device. Does not change the power
/// state of the radio during operation.
///
pub struct AwakeMac<'a, R: radio::Radio<'a>> {
    radio: &'a R,

    tx_client: OptionalCell<&'a dyn radio::TxClient>,
    rx_client: OptionalCell<&'a dyn radio::RxClient>,
}

impl<'a, R: radio::Radio<'a>> AwakeMac<'a, R> {
    pub fn new(radio: &'a R) -> AwakeMac<'a, R> {
        AwakeMac {
            radio: radio,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
        }
    }
}

impl<'a, R: radio::Radio<'a>> Mac<'a> for AwakeMac<'a, R> {
    fn initialize(&self) -> Result<(), ErrorCode> {
        // do nothing, extra buffer unnecessary
        Ok(())
    }

    fn is_on(&self) -> bool {
        self.radio.is_on()
    }

    fn set_config_client(&self, client: &'a dyn radio::ConfigClient) {
        self.radio.set_config_client(client)
    }

    fn set_address(&self, addr: u16) {
        self.radio.set_address(addr)
    }

    fn set_address_long(&self, addr: [u8; 8]) {
        self.radio.set_address_long(addr)
    }

    fn set_pan(&self, id: u16) {
        self.radio.set_pan(id)
    }

    fn get_address(&self) -> u16 {
        self.radio.get_address()
    }

    fn get_address_long(&self) -> [u8; 8] {
        self.radio.get_address_long()
    }

    fn get_pan(&self) -> u16 {
        self.radio.get_pan()
    }

    fn config_commit(&self) {
        self.radio.config_commit()
    }

    fn set_transmit_client(&self, client: &'a dyn radio::TxClient) {
        self.tx_client.set(client);
    }

    fn set_receive_client(&self, client: &'a dyn radio::RxClient) {
        self.rx_client.set(client);
    }

    fn set_receive_buffer(&self, buffer: &'static mut [u8]) {
        self.radio.set_receive_buffer(buffer);
    }

    fn transmit(
        &self,
        full_mac_frame: &'static mut [u8],
        frame_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // We must add the PSDU_OFFSET required for the radio
        // hardware. We first error check the provided arguments
        // and then shift the 15.4 frame by the `PSDU_OFFSET`.

        if full_mac_frame.len() < frame_len + PSDU_OFFSET {
            return Err((ErrorCode::NOMEM, full_mac_frame));
        }

        if frame_len > MAX_FRAME_SIZE {
            return Err((ErrorCode::INVAL, full_mac_frame));
        }

        full_mac_frame.copy_within(0..frame_len, PSDU_OFFSET);
        self.radio.transmit(full_mac_frame, frame_len)
    }
}

impl<'a, R: radio::Radio<'a>> radio::TxClient for AwakeMac<'a, R> {
    fn send_done(&self, buf: &'static mut [u8], acked: bool, result: Result<(), ErrorCode>) {
        self.tx_client.map(move |c| {
            c.send_done(buf, acked, result);
        });
    }
}

impl<'a, R: radio::Radio<'a>> radio::RxClient for AwakeMac<'a, R> {
    fn receive(
        &self,
        buf: &'static mut [u8],
        frame_len: usize,
        lqi: u8,
        crc_valid: bool,
        result: Result<(), ErrorCode>,
    ) {
        // Filter packets by destination because radio is in promiscuous mode
        let mut addr_match = false;
        if let Some((_, (header, _))) = Header::decode(&buf[radio::PSDU_OFFSET..], false).done() {
            if let Some(dst_addr) = header.dst_addr {
                addr_match = match dst_addr {
                    MacAddress::Short(addr) => {
                        // Check if address matches radio or is set to multicast short addr 0xFFFF
                        (addr == self.radio.get_address()) || (addr == 0xFFFF)
                    }
                    MacAddress::Long(long_addr) => long_addr == self.radio.get_address_long(),
                };
            }
        }
        if addr_match {
            // debug!("[AwakeMAC] Rcvd a 15.4 frame addressed to this device");
            self.rx_client.map(move |c| {
                c.receive(buf, frame_len, lqi, crc_valid, result);
            });
        } else {
            // debug!("[AwakeMAC] Received a packet, but not addressed to us");
            self.radio.set_receive_buffer(buf);
        }
    }
}
