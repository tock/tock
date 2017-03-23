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
    fn send_done(&self, buf: &'static mut [u8], acked: bool, result: ReturnCode);
}

pub trait RxClient {
    fn receive(&self, buf: &'static mut [u8], len: u8, result: ReturnCode);
}

pub trait ConfigClient {
    fn config_done(&self, result: ReturnCode);
}

pub trait PowerClient {
    fn changed(&self, on: bool);
}

pub const HEADER_SIZE: u8 = 10;
pub const MAX_PACKET_SIZE: u8 = 128;
pub const MAX_BUF_SIZE: usize = 129; // +1 for opcode
pub const MIN_PACKET_SIZE: u8 = HEADER_SIZE + 2; // +2 for CRC

pub trait Radio: RadioConfig + RadioData {}

pub trait RadioConfig {
    /// buf must be at least MAX_BUF_SIZE in length, and
    /// reg_read and reg_write must be 2 bytes
    fn initialize(&self,
                  spi_buf: &'static mut [u8],
                  reg_write: &'static mut [u8],
                  reg_read: &'static mut [u8])
                  -> ReturnCode;
    fn reset(&self) -> ReturnCode;
    fn start(&self) -> ReturnCode;
    fn stop(&self) -> ReturnCode;
    fn is_on(&self) -> bool;
    fn busy(&self) -> bool;

    fn set_power_client(&self, client: &'static PowerClient);

    /// Commit the config calls to hardware, changing the address,
    /// PAN ID, TX power, and channel to the specified values, issues
    /// a callback to the config client when done.
    fn config_commit(&self) -> ReturnCode;
    fn set_config_client(&self, client: &'static ConfigClient);

    fn config_address(&self) -> u16; //....... The local 16-bit address
    fn config_address_long(&self) -> [u8; 8]; // 64-bit address
    fn config_pan(&self) -> u16; //........... The 16-bit PAN ID
    fn config_tx_power(&self) -> i8; //....... The transmit power, in dBm
    fn config_channel(&self) -> u8; // ....... The 802.15.4 channel

    fn config_set_address(&self, addr: u16);
    fn config_set_address_long(&self, addr: [u8; 8]);
    fn config_set_pan(&self, addr: u16);
    fn config_set_tx_power(&self, power: i8) -> ReturnCode;
    fn config_set_channel(&self, chan: u8) -> ReturnCode;
}

pub trait RadioData {
    fn payload_offset(&self, long_src: bool, long_dest: bool) -> u8;
    fn header_size(&self, long_src: bool, long_dest: bool) -> u8;
    fn packet_header_size(&self, packet: &'static [u8]) -> u8;
    fn packet_get_src(&self, packet: &'static [u8]) -> u16;
    fn packet_get_dest(&self, packet: &'static [u8]) -> u16;
    fn packet_get_src_long(&self, packet: &'static [u8]) -> [u8; 8];
    fn packet_get_dest_long(&self, packet: &'static [u8]) -> [u8; 8];
    fn packet_get_length(&self, packet: &'static [u8]) -> u16;
    fn packet_get_pan(&self, packet: &'static [u8]) -> u16;
    fn packet_has_src_long(&self, packet: &'static [u8]) -> bool;
    fn packet_has_dest_long(&self, packet: &'static [u8]) -> bool;

    fn set_transmit_client(&self, client: &'static TxClient);
    fn set_receive_client(&self, client: &'static RxClient, receive_buffer: &'static mut [u8]);
    fn set_receive_buffer(&self, receive_buffer: &'static mut [u8]);

    fn transmit(&self,
                dest: u16,
                tx_data: &'static mut [u8],
                tx_len: u8,
                source_long: bool)
                -> ReturnCode;
    fn transmit_long(&self,
                     dest: [u8; 8],
                     tx_data: &'static mut [u8],
                     tx_len: u8,
                     source_long: bool)
                     -> ReturnCode;
}

pub enum RadioMacLen {
    ZERO = 0,
    FOUR = 4,
    EIGHT = 8,
    SIXTEEN = 16,
}

// C is the confidentiality key type, I is the integrity key type
pub trait RadioCrypto<C, I> {
    fn set_encrypt_key(&self, key: C);
    fn set_decrypt_key(&self, key: C);
    fn set_mac_key(&self, key: I);
    fn set_mac_check_key(&self, key: I);
    fn set_encrypt(&self, on: bool);
    fn set_mac(&self, len: RadioMacLen);
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
