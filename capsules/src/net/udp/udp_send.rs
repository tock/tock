//! This file contains the definition and implementation for a simple UDP
//! sending interface. The [UDPSender](trait.UDPSender.html) trait provides
//! an interface for upper layer to send a UDP packet, and the
//! [UDPSendClient](trait.UDPSendClient.html) trait is implemented by the
//! upper layer to allow them to receive the `send_done` callback once
//! transmission has completed.

use crate::net::buffer::Buffer;
use crate::net::ipv6::ip_utils::IPAddr;
use crate::net::ipv6::ipv6::TransportHeader;
use crate::net::ipv6::ipv6_send::{IP6SendClient, IP6Sender};
use crate::net::udp::udp::UDPHeader;
use core::cell::Cell;
use kernel::capabilities::UdpDriverSendCapability;
use kernel::common::cells::{MapCell, OptionalCell};
use kernel::common::{List, ListLink, ListNode};
use kernel::debug;
use kernel::udp_port_table::UdpSenderBinding;
use kernel::ReturnCode;

pub struct MuxUdpSender<'a, T: IP6Sender<'a>> {
    sender_list: List<'a, UDPSendStruct<'a, T>>,
    ip_sender: &'a IP6Sender<'a>,
}

impl<T: IP6Sender<'a>> MuxUdpSender<'a, T> {
    pub fn new(ip6_sender: &'a IP6Sender<'a>) -> MuxUdpSender<'a, T> {
        // similar to UdpSendStruct new()
        MuxUdpSender {
            sender_list: List::new(),
            ip_sender: ip6_sender,
        }
    }

    fn send_to(
        &self,
        dest: IPAddr,
        transport_header: TransportHeader,
        caller: &'a UDPSendStruct<'a, T>,
    ) -> ReturnCode {
        // TO DO: Enforce port binding here ()
        // Add this sender to the tail of the sender_list
        let list_empty = self.sender_list.head().is_none();
        self.add_client(caller);
        let mut ret = ReturnCode::SUCCESS;
        if list_empty {
            ret = match caller.tx_buffer.take() {
                Some(buf) => {
                    let ret = self.ip_sender.send_to(dest, transport_header, &buf);
                    debug!("replacing buffer after calling ip_sender send_to");
                    caller.tx_buffer.replace(buf); //Replace buffer as soon as sent.
                    ret
                }
                None => {
                    debug!("No buffer available to take.");
                    ReturnCode::FAIL
                }
            }
        }
        ret
    }

    fn add_client(&self, sender: &'a UDPSendStruct<'a, T>) {
        self.sender_list.push_tail(sender);
    }
}

/// This function implements the `IP6SendClient` trait for the `UDPSendStruct`,
/// and is necessary to receive callbacks from the lower (IP) layer. When
/// the UDP layer receives this callback, it forwards it to the `UDPSendClient`.
impl<T: IP6Sender<'a>> IP6SendClient for MuxUdpSender<'a, T> {
    fn send_done(&self, result: ReturnCode) {
        let last_sender = self.sender_list.pop_head();
        last_sender.map(|last_sender| {
            last_sender
                .client
                .map(|client| match last_sender.tx_buffer.take() {
                    Some(buf) => {
                        debug!("Successful call to send done.");
                        let bind = last_sender.get_binding().expect("hi");
                        debug!("Bound port: {:?}", bind.get_port());
                        last_sender.set_binding(bind);
                        client.send_done(result, buf);
                    }
                    None => {
                        debug!("ERROR: Missing buffer in send done.");
                        let bind = last_sender.get_binding().expect("hi");
                        debug!("Bound port: {:?}", bind.get_port());
                        last_sender.set_binding(bind);
                    }
                })
        });

        let success = match self.sender_list.head() {
            Some(next_sender) => {
                //send next packet in queue
                match next_sender.tx_buffer.take() {
                    Some(buf) => match next_sender.next_th.take() {
                        Some(th) => {
                            let ret = self
                                .ip_sender
                                .send_to(next_sender.next_dest.get(), th, &buf);
                            next_sender.tx_buffer.replace(buf);
                            ret
                        }

                        None => {
                            debug!("Missing transport header.");
                            ReturnCode::FAIL
                        }
                    },
                    None => {
                        debug!("No buffer available to take.");
                        ReturnCode::FAIL
                    }
                }
            }
            None => ReturnCode::SUCCESS, //No more packets queued.
        };
        if success != ReturnCode::SUCCESS {
            debug!("Error in udp_send send_done() callback.");
        }
    }
}

/// The `send_done` function in this trait is invoked after the UDPSender
/// has completed sending the requested packet. Note that the
/// `UDPSender::set_client` method must be called to set the client.
pub trait UDPSendClient {
    fn send_done(&self, result: ReturnCode, dgram: Buffer<'static, u8>);
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
    /// provided destination IP address and
    /// destination port from the src port contained in the UdpSenderBinding.
    ///
    /// # Arguments
    /// `dest` - IPv6 address to send the UDP packet to
    /// `dst_port` - Destination port to send the packet to
    /// `buf` - UDP payload
    /// `binding` - type that specifies what port the sender is bound to.
    ///
    /// # Return Value
    /// Any synchronous errors are returned via the returned `ReturnCode`
    /// value; asynchronous errors are delivered via the callback.
    fn send_to(
        &'a self,
        dest: IPAddr,
        dst_port: u16,
        //src_port: u16,
        buf: Buffer<'static, u8>,
    ) -> ReturnCode;

    /// This function is identical to `send_to()` except that it takes in
    /// an explicit src_port instead of a binding. This allows it to be used
    /// by the userspace driver, above which apps are bound to multiple ports
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
    fn driver_send_to(
        &'a self,
        dest: IPAddr,
        dst_port: u16,
        src_port: u16,
        buf: Buffer<'static, u8>,
        driver_send_cap: &UdpDriverSendCapability,
    ) -> ReturnCode;
    // TODO: Require capability to call above function

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
    fn send(&'a self, dest: IPAddr, udp_header: UDPHeader, buf: Buffer<'static, u8>) -> ReturnCode;

    fn get_binding(&self) -> Option<UdpSenderBinding>;

    fn is_bound(&self) -> bool;

    fn set_binding(&self, binding: UdpSenderBinding) -> Option<UdpSenderBinding>;
}

/// This is a specific instantiation of the `UDPSender` trait. Note
/// that this struct contains a reference to an `IP6Sender` which it
/// forwards packets to (and receives callbacks from).
pub struct UDPSendStruct<'a, T: IP6Sender<'a>> {
    udp_mux_sender: &'a MuxUdpSender<'a, T>,
    client: OptionalCell<&'a UDPSendClient>,
    next: ListLink<'a, UDPSendStruct<'a, T>>,
    tx_buffer: MapCell<Buffer<'static, u8>>,
    next_dest: Cell<IPAddr>,
    next_th: OptionalCell<TransportHeader>,
    binding: MapCell<UdpSenderBinding>,
}

impl<'a, T: IP6Sender<'a>> ListNode<'a, UDPSendStruct<'a, T>> for UDPSendStruct<'a, T> {
    fn next(&'a self) -> &'a ListLink<'a, UDPSendStruct<'a, T>> {
        &self.next
    }
}

/// Below is the implementation of the `UDPSender` traits for the
/// `UDPSendStruct`.
impl<T: IP6Sender<'a>> UDPSender<'a> for UDPSendStruct<'a, T> {
    fn set_client(&self, client: &'a UDPSendClient) {
        self.client.set(client);
    }

    fn send_to(&'a self, dest: IPAddr, dst_port: u16, buf: Buffer<'static, u8>) -> ReturnCode {
        let mut udp_header = UDPHeader::new();
        udp_header.set_dst_port(dst_port);
        match self.binding.take() {
            Some(binding) => {
                if binding.get_port() == 0 {
                    ReturnCode::EINVAL
                } else {
                    udp_header.set_src_port(binding.get_port());
                    self.binding.replace(binding);
                    self.send(dest, udp_header, buf)
                }
            }
            None => ReturnCode::EINVAL,
        }
    }

    fn driver_send_to(
        &'a self,
        dest: IPAddr,
        dst_port: u16,
        src_port: u16,
        buf: Buffer<'static, u8>,
        _driver_send_cap: &UdpDriverSendCapability,
    ) -> ReturnCode {
        let mut udp_header = UDPHeader::new();
        udp_header.set_dst_port(dst_port);
        udp_header.set_src_port(src_port);
        self.send(dest, udp_header, buf)
    }
    // TODO: Require capability to call above function

    fn send(
        &'a self,
        dest: IPAddr,
        mut udp_header: UDPHeader,
        buf: Buffer<'static, u8>,
    ) -> ReturnCode {
        // TODO: need to enforce port binding here? Up to what point do we
        // enforce it? IP layer?
        udp_header.set_len((buf.len() + udp_header.get_hdr_size()) as u16);
        let transport_header = TransportHeader::UDP(udp_header);
        self.tx_buffer.replace(buf);
        self.next_dest.replace(dest);
        self.next_th.replace(transport_header); // th = transport header
        self.udp_mux_sender.send_to(dest, transport_header, &self)
    }

    fn get_binding(&self) -> Option<UdpSenderBinding> {
        self.binding.take()
    }

    fn is_bound(&self) -> bool {
        self.binding.is_some()
    }

    fn set_binding(&self, binding: UdpSenderBinding) -> Option<UdpSenderBinding> {
        self.binding.replace(binding)
    }
}

impl<T: IP6Sender<'a>> UDPSendStruct<'a, T> {
    pub fn new(
        udp_mux_sender: &'a MuxUdpSender<'a, T> /*binding: UdpSenderBinding*/
    ) -> UDPSendStruct<'a, T> {
        UDPSendStruct {
            udp_mux_sender: udp_mux_sender,
            client: OptionalCell::empty(),
            next: ListLink::empty(),
            tx_buffer: MapCell::empty(),
            next_dest: Cell::new(IPAddr::new()),
            next_th: OptionalCell::empty(),
            binding: MapCell::empty(),
        }
    }
}
