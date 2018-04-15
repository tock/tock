//! Bluetooth Low Energy HIL
//!
//! ```text
//! Application
//!
//!           +------------------------------------------------+
//!           | Applications                                   |
//!           +------------------------------------------------+
//!
//! ```
//!
//! ```text
//! Host
//!
//!           +------------------------------------------------+
//!           | Generic Access Profile                         |
//!           +------------------------------------------------+
//!
//!           +------------------------------------------------+
//!           | Generic Attribute Profile                      |
//!           +------------------------------------------------+
//!
//!           +--------------------+      +-------------------+
//!           | Attribute Protocol |      | Security Manager  |
//!           +--------------------+      +-------------------+
//!
//!           +-----------------------------------------------+
//!           | Logical Link and Adaptation Protocol          |
//!           +-----------------------------------------------+
//!
//! ```
//!
//! ```text
//! Controller
//!
//!           +--------------------------------------------+
//!           | Host Controller Interface                  |
//!           +--------------------------------------------+
//!
//!           +------------------+      +------------------+
//!           | Link Layer       |      | Direct Test Mode |
//!           +------------------+      +------------------+
//!
//!           +--------------------------------------------+
//!           | Physical Layer                             |
//!           +--------------------------------------------+
//!
//! ```

use core::convert::TryFrom;
use returncode::ReturnCode;

#[allow(unused)]
#[repr(u8)]
enum BLEAdvertisementType {
    ConnectUndirected = 0x00,
    ConnectDirected = 0x01,
    NonConnectUndirected = 0x02,
    ScanUndirected = 0x06,
}

/// BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B]
/// 2.3 Advertising Channel PDU
bitfield!{
    #[derive(Copy, Clone)]
    pub struct Header(u16);
    impl Debug;
    /// pdu 4 bits
    pub pdu,        _:    3, 0;
    /// rfu 2 bits
    pub _rfu1,      _:          5, 4;
    /// txAdd 1 bit
    pub tx_add,     _: 6, 6;
    /// rxAdd 1 bit
    pub rx_add,     _: 7, 7;
    /// length 6 bits
    pub length,     _: 13, 8;
    /// rfu 2 bits
    pub _rfu2,      _:          15, 14;
}

pub trait Advertisement {
    fn header(&self) -> u16;
    fn address(&self) -> &'static [u8];
    fn data(&self) -> &'static [u8];
}

#[derive(Debug)]
pub struct AdvertisingNonConnectUndirected {
    header: Header,
    adv_a: &'static [u8],
    adv_data: &'static [u8],
}

impl Advertisement for AdvertisingNonConnectUndirected {
    fn header(&self) -> u16 {
        self.header.0
    }

    fn address(&self) -> &'static [u8] {
        self.adv_a
    }

    fn data(&self) -> &'static [u8] {
        self.adv_data
    }
}

#[derive(Debug)]
pub struct AdvertisingConnectDirected {
    header: Header,
    adv_a: &'static [u8],
    init_a: &'static [u8],
}

impl Advertisement for AdvertisingConnectDirected {
    fn header(&self) -> u16 {
        self.header.0
    }

    fn address(&self) -> &'static [u8] {
        self.adv_a
    }

    fn data(&self) -> &'static [u8] {
        self.init_a
    }
}

impl TryFrom<&'static mut [u8]> for AdvertisingNonConnectUndirected {
    type Error = ();

    fn try_from(buf: &'static mut [u8]) -> Result<Self, Self::Error> {
        if buf.len() == 39 && buf[0] & 0xf == 2 {
            let header = Header((buf[0]) as u16 | (buf[1] as u16) << 8);
            Ok(AdvertisingNonConnectUndirected {
                header: header,
                adv_a: &buf[2..8],
                adv_data: &buf[8..],
            })
        } else {
            Err(())
        }
    }
}

impl TryFrom<&'static mut [u8]> for AdvertisingConnectDirected {
    type Error = ();

    fn try_from(buf: &'static mut [u8]) -> Result<Self, Self::Error> {
        if buf.len() == 12 && buf[0] & 0xf == 1 {
            let header = Header((buf[0]) as u16 | (buf[1] as u16) << 8);
            Ok(AdvertisingConnectDirected {
                header: header,
                adv_a: &buf[2..8],
                init_a: &buf[8..],
            })
        } else {
            Err(())
        }
    }
}

pub trait BleAdvertisementDriver {
    fn transmit_advertisement<A: Advertisement>(&self, buf: &A, channel: RadioChannel);
    fn receive_advertisement(&self, channel: RadioChannel);
    fn set_receive_client(&self, client: &'static RxClient);
    fn set_transmit_client(&self, client: &'static TxClient);
}

pub trait BleConfig {
    fn set_tx_power(&self, power: u8) -> ReturnCode;
}

pub trait RxClient {
    fn receive_event(&self, buf: &'static mut [u8], len: u8, result: ReturnCode);
}

pub trait TxClient {
    fn transmit_event(&self, result: ReturnCode);
}

// Bluetooth Core Specification:Vol. 6. Part B, section 1.4.1 Advertising and Data Channel Indices
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum RadioChannel {
    DataChannel0 = 4,
    DataChannel1 = 6,
    DataChannel2 = 8,
    DataChannel3 = 10,
    DataChannel4 = 12,
    DataChannel5 = 14,
    DataChannel6 = 16,
    DataChannel7 = 18,
    DataChannel8 = 20,
    DataChannel9 = 22,
    DataChannel10 = 24,
    DataChannel11 = 28,
    DataChannel12 = 30,
    DataChannel13 = 32,
    DataChannel14 = 34,
    DataChannel15 = 36,
    DataChannel16 = 38,
    DataChannel17 = 40,
    DataChannel18 = 42,
    DataChannel19 = 44,
    DataChannel20 = 46,
    DataChannel21 = 48,
    DataChannel22 = 50,
    DataChannel23 = 52,
    DataChannel24 = 54,
    DataChannel25 = 56,
    DataChannel26 = 58,
    DataChannel27 = 60,
    DataChannel28 = 62,
    DataChannel29 = 64,
    DataChannel30 = 66,
    DataChannel31 = 68,
    DataChannel32 = 70,
    DataChannel33 = 72,
    DataChannel34 = 74,
    DataChannel35 = 76,
    DataChannel36 = 78,
    AdvertisingChannel37 = 2,
    AdvertisingChannel38 = 26,
    AdvertisingChannel39 = 80,
}

impl RadioChannel {
    pub fn get_channel_index(&self) -> u32 {
        match *self {
            RadioChannel::DataChannel0 => 0,
            RadioChannel::DataChannel1 => 1,
            RadioChannel::DataChannel2 => 2,
            RadioChannel::DataChannel3 => 3,
            RadioChannel::DataChannel4 => 4,
            RadioChannel::DataChannel5 => 5,
            RadioChannel::DataChannel6 => 6,
            RadioChannel::DataChannel7 => 7,
            RadioChannel::DataChannel8 => 8,
            RadioChannel::DataChannel9 => 9,
            RadioChannel::DataChannel10 => 10,
            RadioChannel::DataChannel11 => 11,
            RadioChannel::DataChannel12 => 12,
            RadioChannel::DataChannel13 => 13,
            RadioChannel::DataChannel14 => 14,
            RadioChannel::DataChannel15 => 15,
            RadioChannel::DataChannel16 => 16,
            RadioChannel::DataChannel17 => 17,
            RadioChannel::DataChannel18 => 18,
            RadioChannel::DataChannel19 => 19,
            RadioChannel::DataChannel20 => 20,
            RadioChannel::DataChannel21 => 21,
            RadioChannel::DataChannel22 => 22,
            RadioChannel::DataChannel23 => 23,
            RadioChannel::DataChannel24 => 24,
            RadioChannel::DataChannel25 => 25,
            RadioChannel::DataChannel26 => 26,
            RadioChannel::DataChannel27 => 27,
            RadioChannel::DataChannel28 => 28,
            RadioChannel::DataChannel29 => 29,
            RadioChannel::DataChannel30 => 30,
            RadioChannel::DataChannel31 => 31,
            RadioChannel::DataChannel32 => 32,
            RadioChannel::DataChannel33 => 33,
            RadioChannel::DataChannel34 => 34,
            RadioChannel::DataChannel35 => 35,
            RadioChannel::DataChannel36 => 36,
            RadioChannel::AdvertisingChannel37 => 37,
            RadioChannel::AdvertisingChannel38 => 38,
            RadioChannel::AdvertisingChannel39 => 39,
        }
    }
}
