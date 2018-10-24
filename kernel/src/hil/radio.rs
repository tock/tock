//! Interface for sending and receiving IEEE 802.15.4 packets.
//!
//! Hardware independent interface for an 802.15.4 radio. Note that
//! configuration commands are asynchronous and must be committed with a call to
//! config_commit. For example, calling set_address will change the source
//! address of packets but does not change the address stored in hardware used
//! for address recognition. This must be committed to hardware with a call to
//! config_commit. Please see the relevant TRD for more details.

use returncode::ReturnCode;
use core::convert::TryFrom;

pub trait TxClient {
    fn send_done(&self, buf: &'static mut [u8], acked: bool, result: ReturnCode);
}

pub trait RxClient {
    fn receive(
        &self,
        buf: &'static mut [u8],
        frame_len: usize,
        crc_valid: bool,
        result: ReturnCode,
    );
}

pub trait ConfigClient {
    fn config_done(&self, result: ReturnCode);
}

pub trait PowerClient {
    fn changed(&self, on: bool);
}

/// These constants are used for interacting with the SPI buffer, which contains
/// a 1-byte SPI command, a 1-byte PHY header, and then the 802.15.4 frame. In
/// theory, the number of extra bytes in front of the frame can depend on the
/// particular method used to communicate with the radio, but we leave this as a
/// constant in this generic trait for now.
///
/// Furthermore, the minimum MHR size assumes that
///
/// - The source PAN ID is omitted
/// - There is no auxiliary security header
/// - There are no IEs
///
/// ```text
/// +---------+-----+-----+-------------+-----+
/// | SPI com | PHR | MHR | MAC payload | MFR |
/// +---------+-----+-----+-------------+-----+
/// \______ Static buffer rx/txed to SPI _____/
///                 \__ PSDU / frame length __/
/// \___ 2 bytes ___/
/// ```

pub const MIN_MHR_SIZE: usize = 9;
pub const MFR_SIZE: usize = 2;
pub const MAX_MTU: usize = 127;
pub const MIN_FRAME_SIZE: usize = MIN_MHR_SIZE + MFR_SIZE;
pub const MAX_FRAME_SIZE: usize = MAX_MTU;

pub const PSDU_OFFSET: usize = 2;
pub const MAX_BUF_SIZE: usize = PSDU_OFFSET + MAX_MTU;
pub const MIN_PAYLOAD_OFFSET: usize = PSDU_OFFSET + MIN_MHR_SIZE;

pub trait Radio: RadioConfig + RadioData {}

/// Configure the 802.15.4 radio.
pub trait RadioConfig {
    /// buf must be at least MAX_BUF_SIZE in length, and
    /// reg_read and reg_write must be 2 bytes.
    fn initialize(
        &self,
        spi_buf: &'static mut [u8],
        reg_write: &'static mut [u8],
        reg_read: &'static mut [u8],
    ) -> ReturnCode;
    fn reset(&self) -> ReturnCode;
    fn start(&self) -> ReturnCode;
    fn stop(&self) -> ReturnCode;
    fn is_on(&self) -> bool;
    fn busy(&self) -> bool;

    //fn set_power_client(&self, client: &'static PowerClient);

    /// Commit the config calls to hardware, changing the address,
    /// PAN ID, TX power, and channel to the specified values, issues
    /// a callback to the config client when done.
    fn config_commit(&self);
    fn set_config_client(&self, client: &'static ConfigClient);

    fn get_address(&self) -> u16; //....... The local 16-bit address
    fn get_address_long(&self) -> [u8; 8]; // 64-bit address
    fn get_pan(&self) -> u16; //........... The 16-bit PAN ID
    fn get_tx_power(&self) -> i8; //....... The transmit power, in dBm
    fn get_channel(&self) -> u8; // ....... The 802.15.4 channel

    fn set_address(&self, addr: u16);
    fn set_address_long(&self, addr: [u8; 8]);
    fn set_pan(&self, id: u16);
    fn set_tx_power(&self, power: i8) -> ReturnCode;
    fn set_channel(&self, chan: u8) -> ReturnCode;
}

pub trait RadioData {
    fn set_transmit_client(&self, client: &'static TxClient);
    fn set_receive_client(&self, client: &'static RxClient);
    //fn set_receive_buffer(&self, receive_buffer: &'static mut [u8]);

    fn transmit(
        &self,
        buf: &'static mut [u8],
        len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>);

    fn receive(
        &self
    ) -> ReturnCode ;
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum RadioChannel {
    DataChannel11 =  5,
    DataChannel12 = 10,
    DataChannel13 = 15,
    DataChannel14 = 20,
    DataChannel15 = 25,
    DataChannel16 = 30,
    DataChannel17 = 35,
    DataChannel18 = 40,
    DataChannel19 = 45,
    DataChannel20 = 50,
    DataChannel21 = 55,
    DataChannel22 = 60,
    DataChannel23 = 65,
    DataChannel24 = 70,
    DataChannel25 = 75,
    DataChannel26 = 80
}

impl RadioChannel {
    pub fn get_channel_index(&self) -> u8 {
        match *self {
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
            RadioChannel::DataChannel26 => 26
        }
    }
}

impl TryFrom<u8> for RadioChannel {
    type Error = ();

    fn try_from(val: u8) -> Result<RadioChannel, ()> {
        match val {
            11 => Ok(RadioChannel::DataChannel11),
            12 => Ok(RadioChannel::DataChannel12),
            13 => Ok(RadioChannel::DataChannel13),
            14 => Ok(RadioChannel::DataChannel14),
            15 => Ok(RadioChannel::DataChannel15),
            16 => Ok(RadioChannel::DataChannel16),
            17 => Ok(RadioChannel::DataChannel17),
            18 => Ok(RadioChannel::DataChannel18),
            19 => Ok(RadioChannel::DataChannel19),
            20 => Ok(RadioChannel::DataChannel20),
            21 => Ok(RadioChannel::DataChannel21),
            22 => Ok(RadioChannel::DataChannel22),
            23 => Ok(RadioChannel::DataChannel23),
            24 => Ok(RadioChannel::DataChannel24),
            25 => Ok(RadioChannel::DataChannel25),
            26 => Ok(RadioChannel::DataChannel26),
            _ => Err(()),
        }
    }
}


