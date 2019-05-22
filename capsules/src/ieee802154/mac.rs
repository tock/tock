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
use kernel::common::cells::OptionalCell;
use kernel::debug;
use kernel::hil::radio;
use kernel::ReturnCode;

pub trait Mac {
    /// Initializes the layer; may require a buffer to temporarily retaining frames to be
    /// transmitted
    fn initialize(&self, mac_buf: &'static mut [u8]) -> ReturnCode;

    /// Sets the notified client for configuration changes
    fn set_config_client(&self, client: &'static radio::ConfigClient);
    /// Sets the notified client for transmission completions
    fn set_transmit_client(&self, client: &'static radio::TxClient);
    /// Sets the notified client for frame receptions
    fn set_receive_client(&self, client: &'static radio::RxClient);
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
    /// line with this MAC protocol implementation. The specificed config_client is
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
    ) -> (ReturnCode, Option<&'static mut [u8]>);
}

///
/// Default implementation of a Mac layer. Acts as a pass-through between a MacDevice
/// implementation and the underlying radio::Radio device. Does not change the power
/// state of the radio during operation.
///
pub struct AwakeMac<'a, R: radio::Radio> {
    radio: &'a R,

    tx_client: OptionalCell<&'static radio::TxClient>,
    rx_client: OptionalCell<&'static radio::RxClient>,
}

impl<R: radio::Radio> AwakeMac<'a, R> {
    pub fn new(radio: &'a R) -> AwakeMac<'a, R> {
        AwakeMac {
            radio: radio,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
        }
    }
}

impl<R: radio::Radio> Mac for AwakeMac<'a, R> {
    fn initialize(&self, _mac_buf: &'static mut [u8]) -> ReturnCode {
        // do nothing, extra buffer unnecessary
        ReturnCode::SUCCESS
    }

    fn is_on(&self) -> bool {
        self.radio.is_on()
    }

    fn set_config_client(&self, client: &'static radio::ConfigClient) {
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

    fn set_transmit_client(&self, client: &'static radio::TxClient) {
        self.tx_client.set(client);
    }

    fn set_receive_client(&self, client: &'static radio::RxClient) {
        self.rx_client.set(client);
    }

    fn set_receive_buffer(&self, buffer: &'static mut [u8]) {
        self.radio.set_receive_buffer(buffer);
    }

    fn transmit(
        &self,
        full_mac_frame: &'static mut [u8],
        frame_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        self.radio.transmit(full_mac_frame, frame_len)
    }
}

impl<R: radio::Radio> radio::TxClient for AwakeMac<'a, R> {
    fn send_done(&self, buf: &'static mut [u8], acked: bool, result: ReturnCode) {
        self.tx_client.map(move |c| {
            c.send_done(buf, acked, result);
        });
    }
}

impl<R: radio::Radio> radio::RxClient for AwakeMac<'a, R> {
    fn receive(
        &self,
        buf: &'static mut [u8],
        frame_len: usize,
        crc_valid: bool,
        result: ReturnCode,
    ) {
        // Filter packets by destination because radio is in promiscuous mode
        let mut addr_match = false;
        if let Some((_, (header, _))) = Header::decode(&buf[radio::PSDU_OFFSET..], false).done() {
            if let Some(dst_addr) = header.dst_addr {
                addr_match = match dst_addr {
                    MacAddress::Short(addr) => addr == self.radio.get_address(),
                    MacAddress::Long(long_addr) => long_addr == self.radio.get_address_long(),
                };
            }
        }

        if addr_match {
            self.rx_client.map(move |c| {
                c.receive(buf, frame_len, crc_valid, result);
            });
        } else {
            debug!("[AwakeMAC] Received a packet, but not addressed to us");
            debug!("radio addr is: {:x}", self.radio.get_address());
            self.radio.set_receive_buffer(buf);
        }
    }
}
