/// This capsule exposes 6LoWPAN packet transmission capabilities to
/// userland.

use kernel::ReturnCode;
use kernel::hil::sixlowpan;

pub struct SixLowPan {
    transmit_client: Option<TxClient>;
    receive_client: Option<RxClient>;
    config_client: Option<ConfigClient>;
}

impl SixLowPan {
    fn new() -> SixLowPan {}
}

impl kernel::hil::sixlowpan::SixLowPan for SixLowPan {
    fn set_config_client(&self, client: &'static ConfigClient);

    fn config_commit(&self) -> ReturnCode;
    fn config_set_address(&self, addr: LinkAddress);
    fn config_set_pan(&self, id: u16);

    fn set_transmit_client(&self, client: &'static TxClient);
    fn set_receive_client(&self, client: &'static RxClient, receive_buffer: &'static mut [u8]);
    fn set_receive_buffer(&self, receive_buffer: &'static mut [u8]);

    fn transmit(&self,
                dest: LinkAddress,
                header_desc: PacketDesc,
                payload: &'static mut [u8],
                payload_len: u16,
                source_long: bool)
                -> ReturnCode;
}

impl kernel::hil::radio::TxClient for SixLowPan {
    fn send_done(&self, buf: &'static mut [u8], acked: bool, result: ReturnCode) {
        // if fragmentation is done then
        // self.transmit_client.map(|client| client.send_done(/*  */));
    }
}

impl kernel::hil::radio::RxClient for SixLowPan {
    fn receive(&self, buf: &'static mut [u8], len: u8, result: ReturnCode) {
        // receive 802.15.4 packets and reassemble
        // when done reassembling:
        // self.receive_client.map(|client| client.receive(/* */));
    }
}

impl kernel::hil::radio::ConfigClient for SixLowPan {
    fn config_done(&self, result: ReturnCode) {
        self.config_client.map(|client| client.config_done(result));
    }
}
