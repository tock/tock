use core::cell::Cell;
use kernel::ReturnCode;
use net::ipv6::ipv6::IP6Header;
use net::sixlowpan::sixlowpan_state::SixlowpanRxClient;

pub trait IP6RecvClient {
    // TODO: What should the upper layers receive?
    fn receive(&self, header: IP6Header, payload: &[u8]);
}

pub trait IP6Receiver<'a> {
    fn set_client(&self, client: &'a IP6RecvClient);
}

pub struct IP6RecvStruct<'a> {
    client: Cell<Option<&'a IP6RecvClient>>,
}

impl<'a> IP6Receiver<'a> for IP6RecvStruct<'a> {
    fn set_client(&self, client: &'a IP6RecvClient) {
        self.client.set(Some(client));
    }
}

impl<'a> IP6RecvStruct<'a> {
    pub fn new() -> IP6RecvStruct<'a> {
        IP6RecvStruct {
            client: Cell::new(None),
        }
    }
}

impl<'a> SixlowpanRxClient for IP6RecvStruct<'a> {
    fn receive(&self, buf: &[u8], len: usize, result: ReturnCode) {
        // TODO: Drop here?
        if len > buf.len() || result != ReturnCode::SUCCESS {
            return;
        }
        match IP6Header::decode(buf).done() {
            Some((offset, header)) => {
                // TODO: Probably do some sanity checking, check for checksum
                // correctness, length, etc.
                self.client
                    .get()
                    .map(|client| client.receive(header, &buf[offset..]));
            }
            None => {
                // TODO: Report the error somewhere...
            }
        }
    }
}
