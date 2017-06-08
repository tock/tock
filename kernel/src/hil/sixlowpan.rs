/// Software implementation of the 6LoWPAN specification for
/// encoding IPv6 packets over 802.15.4

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

#[derive(Copy, Clone)]
pub enum LinkAddress {
    ShortAddress(u16),
    LongAddress([u8; 8]),
}

pub trait SixLowPan {
    fn set_config_client(&self, client: &'static ConfigClient);

    fn config_commit(&self) -> ReturnCode;
    fn config_set_address(&self, addr: LinkAddress);
    fn config_set_pan(&self, id: u16);

    fn set_transmit_client(&self, client: &'static TxClient);
    fn set_receive_client(&self, client: &'static RxClient, receive_buffer: &'static mut [u8]);
    fn set_receive_buffer(&self, receive_buffer: &'static mut [u8]);

    // Does IPv6 Header (within the 6LoWPAN standard) contain
    // the IPv6 Link-local addresses?

    fn transmit(&self,
                dest: LinkAddress,
                header: PacketDescription,
                payload: &'static mut [u8],
                payload_len: u16,
                source_long: bool)
                -> ReturnCode;
}
