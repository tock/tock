//! Interface for sending and receiving IEEE 802.15.4 packets.
//!
//! Hardware independent interface for an 802.15.4 radio. Note that
//! configuration commands are asynchronous and must be committed with a call to
//! config_commit. For example, calling set_address will change the source
//! address of packets but does not change the address stored in hardware used
//! for address recognition. This must be committed to hardware with a call to
//! config_commit. Please see the relevant TRD for more details.

use returncode::ReturnCode;
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
/// - The source PAN ID is omitted
/// - There is no auxiliary security header
/// - There are no IEs
///
/// +---------+-----+-----+-------------+-----+
/// | SPI com | PHR | MHR | MAC payload | MFR |
/// +---------+-----+-----+-------------+-----+
/// \______ Static buffer rx/txed to SPI _____/
///                 \__ PSDU / frame length __/
/// \___ 2 bytes ___/

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

    fn set_power_client(&self, client: &'static PowerClient);

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
    fn set_receive_client(&self, client: &'static RxClient, receive_buffer: &'static mut [u8]);
    fn set_receive_buffer(&self, receive_buffer: &'static mut [u8]);

    fn transmit(
        &self,
        spi_buf: &'static mut [u8],
        frame_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>);
}
