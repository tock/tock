/// 802.15.4 radio interface

use returncode::ReturnCode;
pub trait TxClient {
    fn send_done(&self, buf: &'static mut [u8], result: ReturnCode);
}

pub trait RxClient {
    fn receive(&self, buf: &'static [u8], len: u8, result: ReturnCode);
}

pub trait Radio {
    fn initialize(&self) -> ReturnCode;
    fn start(&self) -> ReturnCode;
    fn stop(&self) -> ReturnCode;
    fn reset(&self) -> ReturnCode;

    fn set_transmit_client(&self, client: &'static TxClient);
    fn set_receive_client(&self, client: &'static RxClient);

    fn set_address(&self, addr: u16) -> ReturnCode;
    fn set_pan(&self, addr: u16) -> ReturnCode;
    fn payload_offset(&self) -> u8;
    fn header_size(&self) -> u8;

    fn transmit(&self,
                dest: u16,
                tx_data: &'static mut [u8],
                tx_len: u8) -> ReturnCode;
}

pub const HEADER_SIZE: u8 = 10;

#[repr(C, packed)]
pub struct Header {
    len: u8,
    fcf: u16,
    dsn: u8,
    pan: u16,
    src: u16,
    dst: u16,
}
