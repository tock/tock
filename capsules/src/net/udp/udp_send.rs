//! This file contains the definition and implementation for a simple UDP
//! sending interface. The [UDPSender](trait.UDPSender.html) trait provides
//! an interface for upper layer to send a UDP packet, and the
//! [UDPSendClient](trait.UDPSendClient.html) trait is implemented by the
//! upper layer to allow them to receive the `send_done` callback once
//! transmission has completed.

use core::cell::Cell;
use kernel::ReturnCode;
use net::ipv6::ip_utils::IPAddr;
use net::ipv6::ipv6::TransportHeader;
use net::ipv6::ipv6_send::{IP6Client, IP6Sender};
use net::udp::udp::UDPHeader;

/// The `send_done` function in this trait is invoked after the UDPSender
/// has completed sending the requested packet. Note that the
/// `UDPSender::set_client` method must be called to set the client.
pub trait UDPSendClient {
    fn send_done(&self, result: ReturnCode);
}

/// This trait represents the bulk of the UDP functionality. The two
/// variants of sending a packet (either via the `send_to` or `send` methods)
/// represent whether the caller wants to construct a custom `UDPHeader` or
/// not. Calling `send_to` tells the UDP layer to construct a default
/// `UDPHeader` and forward the payload to the respective destination and port.
pub trait UDPSender<'a> {
    /// This function sets the client for the `UDPSender` instance
    ///
    /// # Arguments
    /// `client` - Implementation of `UDPSendClient` to be set as the client
    /// for the `UDPSender` instance
    fn set_client(&self, client: &'a UDPSendClient);

    /// This function constructs a `UDPHeader` and sends the payload to the
    /// provided destination IP address over the provided source and
    /// destination ports.
    ///
    /// # Arguments
    /// `dest` - IPv6 address to send the UDP packet to
    /// `dst_port` - Destination port to send the packet to
    /// `src_port` - Port to send the packet from
    /// `buf` - UDP payload
    ///
    /// # Return Value
    /// Any synchronous errors are returned via the returned `ReturnCode`
    /// value; asynchronous errors are delivered via the callback.
    fn send_to(&self, dest: IPAddr, dst_port: u16, src_port: u16, buf: &'a [u8]) -> ReturnCode;

    /// This function constructs an IP packet from the completed `UDPHeader`
    /// and buffer, and sends it to the provided IP address
    ///
    /// # Arguments
    /// `dest` - IP address to send the UDP packet to
    /// `udp_header` - Completed UDP header to be sent to the destination
    /// `buf` - A byte array containing the UDP payload
    ///
    /// # Return Value
    /// Returns any synchronous errors or success. Note that any asynchrounous
    /// errors are returned via the callback.
    fn send(&self, dest: IPAddr, udp_header: UDPHeader, buf: &'a [u8]) -> ReturnCode;
}

/// This is a specific instantiation of the `UDPSender` trait. Note
/// that this struct contains a reference to an `IP6Sender` which it
/// forwards packets to (and receives callbacks from).
pub struct UDPSendStruct<'a, T: IP6Sender<'a> + 'a> {
    ip_send_struct: &'a T,
    client: Cell<Option<&'a UDPSendClient>>,
}

/// Below is the implementation of the `UDPSender` traits for the
/// `UDPSendStruct`.
impl<'a, T: IP6Sender<'a>> UDPSender<'a> for UDPSendStruct<'a, T> {
    fn set_client(&self, client: &'a UDPSendClient) {
        self.client.set(Some(client));
    }

    fn send_to(&self, dest: IPAddr, dst_port: u16, src_port: u16, buf: &'a [u8]) -> ReturnCode {
        let mut udp_header = UDPHeader::new();
        udp_header.set_dst_port(dst_port);
        udp_header.set_src_port(src_port);
        self.send(dest, udp_header, buf)
    }

    fn send(&self, dest: IPAddr, mut udp_header: UDPHeader, buf: &'a [u8]) -> ReturnCode {
        let total_length = buf.len() + udp_header.get_hdr_size();
        udp_header.set_len(total_length as u16);
        let transport_header = TransportHeader::UDP(udp_header);
        self.ip_send_struct.send_to(dest, transport_header, buf)
    }
}

impl<'a, T: IP6Sender<'a>> UDPSendStruct<'a, T> {
    pub fn new(ip_send_struct: &'a T) -> UDPSendStruct<'a, T> {
        UDPSendStruct {
            ip_send_struct: ip_send_struct,
            client: Cell::new(None),
        }
    }
}

/// This function implements the `IP6Client` trait for the `UDPSendStruct`,
/// and is necessary to receive callbacks from the lower (IP) layer. When
/// the UDP layer receives this callback, it forwards it to the `UDPSendClient`.
impl<'a, T: IP6Sender<'a>> IP6Client for UDPSendStruct<'a, T> {
    fn send_done(&self, result: ReturnCode) {
        self.client.get().map(|client| client.send_done(result));
    }
}
