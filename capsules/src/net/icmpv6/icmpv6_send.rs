//! This file contains the definition and implementation of a simple ICMPv6
//! sending interface. The [ICMP6Sender](trait.ICMP6Sender.html) trait provides
//! an interface for an upper layer to send an ICMPv6 packet, and the
//! [ICMP6SendClient](trait.ICMP6SendClient.html) trait is implemented by the
//! upper layer to allow them to receive the `send_done` callback once
//! transmission has completed.
//!
//! - Author: Conor McAvity <cmcavity@stanford.edu>

use crate::net::icmpv6::icmpv6::ICMP6Header;
use crate::net::ipv6::ip_utils::IPAddr;
use crate::net::ipv6::ipv6::TransportHeader;
use crate::net::ipv6::ipv6_send::{IP6SendClient, IP6Sender};
use kernel::common::buffer::LeasableBuffer;
use kernel::common::cells::OptionalCell;
use kernel::ReturnCode;

/// A trait for a client of an `ICMP6Sender`.
pub trait ICMP6SendClient {
    /// A client callback invoked after an ICMP6Sender has completed sending
    /// a requested packet.
    fn send_done(&self, result: ReturnCode);
}

/// A trait that defines an interface for sending ICMPv6 packets.
pub trait ICMP6Sender<'a> {
    /// Sets the client for the `ICMP6Sender` instance.
    ///
    /// # Arguments
    ///
    /// `client` - The `ICMP6SendClient` instance to be set as the client
    /// of the `ICMP6Sender` instance
    fn set_client(&self, client: &'a dyn ICMP6SendClient);

    /// Constructs and sends an IP packet from provided ICMPv6 header
    /// and payload.
    ///
    /// # Arguments
    ///
    /// `dest` - The destination IP address
    /// `icmp_header` - The ICMPv6 header to be sent
    /// `buf` - The byte array containing the ICMPv6 payload
    ///
    /// # Return Value
    ///
    /// This function returns a code reporting either success or any
    /// synchronous errors. Note that any asynchronous errors are returned
    /// via the callback.
    fn send(&self, dest: IPAddr, icmp_header: ICMP6Header, buf: &'static mut [u8]) -> ReturnCode;
}

/// A struct that implements the `ICMP6Sender` trait.
pub struct ICMP6SendStruct<'a, T: IP6Sender<'a>> {
    ip_send_struct: &'a T,
    client: OptionalCell<&'a dyn ICMP6SendClient>,
}

impl<T: IP6Sender<'a>> ICMP6SendStruct<'a, T> {
    pub fn new(ip_send_struct: &'a T) -> ICMP6SendStruct<'a, T> {
        ICMP6SendStruct {
            ip_send_struct: ip_send_struct,
            client: OptionalCell::empty(),
        }
    }
}

impl<T: IP6Sender<'a>> ICMP6Sender<'a> for ICMP6SendStruct<'a, T> {
    fn set_client(&self, client: &'a dyn ICMP6SendClient) {
        self.client.set(client);
    }

    fn send(
        &self,
        dest: IPAddr,
        mut icmp_header: ICMP6Header,
        buf: &'static mut [u8],
    ) -> ReturnCode {
        let total_len = buf.len() + icmp_header.get_hdr_size();
        icmp_header.set_len(total_len as u16);
        let transport_header = TransportHeader::ICMP(icmp_header);
        self.ip_send_struct
            .send_to(dest, transport_header, &LeasableBuffer::new(buf))
    }
}

impl<T: IP6Sender<'a>> IP6SendClient for ICMP6SendStruct<'a, T> {
    /// Forwards callback received from the `IP6Sender` to the
    /// `ICMP6SendClient`.
    fn send_done(&self, result: ReturnCode) {
        self.client.map(|client| client.send_done(result));
    }
}
