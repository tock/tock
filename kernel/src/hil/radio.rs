/// 802.15.4 radio interface

use returncode::ReturnCode;
pub trait TxClient {
    fn send_done(&self, buf: &'static mut [u8], result: ReturnCode);
}

pub trait RxClient {
    fn receive(&self, buf: &'static [u8], len: u8, result: ReturnCode);
}

pub const HEADER_SIZE: u8        = 10;
pub const MAX_PACKET_SIZE: u8    = 128;
pub const MAX_BUF_SIZE: usize    = 129;    // +1 for opcode
pub const MIN_PACKET_SIZE: u8    = HEADER_SIZE + 2; // +2 for CRC


pub trait Radio {

    /// buf must be at least MAX_BUF_SIZE in length
    /// reg_read and reg_write must be 2 bytes
    fn initialize(&self,
                  spi_buf: &'static mut [u8],
                  reg_write: &'static mut [u8],
                  reg_read: &'static mut [u8]) -> ReturnCode;
    fn start(&self) -> ReturnCode;
    fn stop(&self) -> ReturnCode;
    fn reset(&self) -> ReturnCode;
    fn ready(&self) -> bool;

    fn set_transmit_client(&self, client: &'static TxClient);
    fn set_receive_client(&self, client:
                          &'static RxClient,
                          receive_buffer: &'static mut [u8]);

    fn set_address(&self, addr: u16) -> ReturnCode;
    fn set_pan(&self, addr: u16) -> ReturnCode;
    fn payload_offset(&self) -> u8;
    fn header_size(&self) -> u8;

    fn transmit(&self,
                dest: u16,
                tx_data: &'static mut [u8],
                tx_len: u8) -> ReturnCode;
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
