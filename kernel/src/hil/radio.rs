/// Hardware independent interface for an 802.15.4 radio.
/// Note that configuration commands are asynchronous and
/// must be committed with a call to config_commit. For
/// example, calling set_address will change the source address
/// of packets but does not change the address stored in hardware
/// used for address recognition. This must be committed to hardware
/// with a call to config_commit. Please see the relevant TRD for
/// more details.

use returncode::ReturnCode;
pub trait TxClient {
    fn send_done(&self, buf: &'static mut [u8], result: ReturnCode);
}

pub trait RxClient {
    fn receive(&self, buf: &'static mut [u8], len: u8, result: ReturnCode);
}

pub trait ConfigClient {
    fn config_done(&self, result: ReturnCode);
}

pub const HEADER_SIZE: u8 = 10;
pub const MAX_PACKET_SIZE: u8 = 128;
pub const MAX_BUF_SIZE: usize = 129; // +1 for opcode
pub const MIN_PACKET_SIZE: u8 = HEADER_SIZE + 2; // +2 for CRC

pub trait Radio {
    /// buf must be at least MAX_BUF_SIZE in length, and
    /// reg_read and reg_write must be 2 bytes
    fn initialize(&self,
                  spi_buf: &'static mut [u8],
                  reg_write: &'static mut [u8],
                  reg_read: &'static mut [u8])
                  -> ReturnCode;
    fn start(&self) -> ReturnCode;
    fn stop(&self) -> ReturnCode;
    fn reset(&self) -> ReturnCode;
    fn ready(&self) -> bool;

    fn set_transmit_client(&self, client: &'static TxClient);
    fn set_receive_client(&self, client: &'static RxClient, receive_buffer: &'static mut [u8]);
    fn set_config_client(&self, client: &'static ConfigClient);
    fn set_receive_buffer(&self, receive_buffer: &'static mut [u8]);

    /// The local 16-bit address
    fn config_address(&self) -> u16;
    /// The 16-bit PAN ID
    fn config_pan(&self) -> u16;
    /// The transmit power, in dBm
    fn config_tx_power(&self) -> i8;
    /// The 802.15.4 channel
    fn config_channel(&self) -> u8;

    fn config_set_address(&self, addr: u16);
    fn config_set_pan(&self, addr: u16);
    /// Set the transmit power in dBm. The radio will set
    /// it to the closest available value that is >= the
    /// value specified.
    fn config_set_tx_power(&self, power: i8) -> ReturnCode;
    /// Set the 802.15.4 channel to use. Valid numbers are 11-26
    fn config_set_channel(&self, chan: u8) -> ReturnCode;

    /// Commit the config calls to hardware, changing the address,
    /// PAN ID, TX power, and channel to the specified values.
    fn config_commit(&self) -> ReturnCode;


    fn payload_offset(&self) -> u8;
    fn header_size(&self) -> u8;

    fn transmit(&self, dest: u16, tx_data: &'static mut [u8], tx_len: u8) -> ReturnCode;
}

#[repr(C, packed)]
pub struct Header {
    len: u8,
    fcf: u16,
    dsn: u8,
    pan: u16,
    src: u16,
    dst: u16,
}
