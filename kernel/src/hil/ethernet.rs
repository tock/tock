//! Ethernet network cards

use crate::errorcode::ErrorCode;

pub trait EthernetAdapterClient {
    fn tx_done(
        &self,
        err: Result<(), ErrorCode>,
        packet_buffer: &'static mut [u8],
        len: u16,
        packet_identifier: usize,
        timestamp: Option<u64>,
    );
    fn rx_packet(&self, packet: &[u8], timestamp: Option<u64>);
}

pub trait EthernetAdapter<'a> {
    fn set_client(&self, client: &'a dyn EthernetAdapterClient);
    fn transmit(
        &self,
        packet: &'static mut [u8],
        len: u16,
        packet_identifier: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;
}
