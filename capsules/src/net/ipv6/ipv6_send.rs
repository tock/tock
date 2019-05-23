//! This file contains the interface definition for sending an IPv6 packet.
//! The [IP6Sender](trait.IP6Sender.html) trait provides an interface
//! for sending IPv6 packets, while the [IP6SendClient](trait.IP6SendClient) trait
//! must be implemented by upper layers to receive the `send_done` callback
//! when a transmission has completed.
//!
//! This file also includes an implementation of the `IP6Sender` trait, which
//! sends an IPv6 packet using 6LoWPAN.

// Additional Work and Known Problems
// ----------------------------------
// The main areas for additional work is with regards to the interface provided
// by `IP6Sender`. The current interface differs from the one provided in
// the networking stack overview document, and should be changed to better
// reflect that document. Additionally, the specific implementation is
// over 6LoWPAN, and should be separated from the generic IPv6 sending
// interface.

use crate::ieee802154::device::{MacDevice, TxClient};
use crate::net::ieee802154::MacAddress;
use crate::net::ipv6::ip_utils::IPAddr;
use crate::net::ipv6::ipv6::{IP6Header, IP6Packet, TransportHeader};
use crate::net::sixlowpan::sixlowpan_state::TxState;
use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::debug;
use kernel::hil::time::{self, Frequency};
use kernel::ReturnCode;

/// This trait must be implemented by upper layers in order to receive
/// the `send_done` callback when a transmission has completed. The upper
/// layer must then call `IP6Sender.set_client` in order to receive this
/// callback.
pub trait IP6SendClient {
    fn send_done(&self, result: ReturnCode);
}

/// This trait provides a basic IPv6 sending interface. It exposes basic
/// configuration information for the IPv6 layer (setting the source address,
/// setting the gateway MAC address), as well as a way to send an IPv6
/// packet.
pub trait IP6Sender<'a> {
    /// This method sets the `IP6SendClient` for the `IP6Sender` instance, which
    /// receives the `send_done` callback when transmission has finished.
    ///
    /// # Arguments
    /// `client` - Client that implements the `IP6SendClient` trait to receive the
    /// `send_done` callback
    fn set_client(&self, client: &'a IP6SendClient);

    /// This method sets the source address for packets sent from the
    /// `IP6Sender` instance.
    ///
    /// # Arguments
    /// `src_addr` - `IPAddr` to set as the source address for packets sent
    /// from this instance of `IP6Sender`
    fn set_addr(&self, src_addr: IPAddr);

    /// This method sets the gateway/next hop MAC address for this `IP6Sender`
    /// instance.
    ///
    /// # Arguments
    /// `gateway` - MAC address to send the constructed packet to
    fn set_gateway(&self, gateway: MacAddress);

    /// This method sets the `IP6Header` for the `IP6Sender` instance
    ///
    /// # Arguments
    /// `ip6_header` - New `IP6Header` that subsequent packets sent via this
    /// `IP6Sender` instance will use
    fn set_header(&mut self, ip6_header: IP6Header);

    /// This method sends the provided transport header and payload to the
    /// given destination IP address
    ///
    /// # Arguments
    /// `dst` - IPv6 address to send the packet to
    /// `transport_header` - The `TransportHeader` for the packet being sent
    /// `payload` - The transport payload for the packet being sent
    fn send_to(&self, dst: IPAddr, transport_header: TransportHeader, payload: &[u8])
        -> ReturnCode;
}

/// This struct is a specific implementation of the `IP6Sender` trait. This
/// struct sends the packet using 6LoWPAN over a generic `MacDevice` object.
pub struct IP6SendStruct<'a, A: time::Alarm> {
    // We want the ip6_packet field to be a TakeCell so that it is easy to mutate
    ip6_packet: TakeCell<'static, IP6Packet<'static>>,
    alarm: &'a A, // Alarm so we can introduce a small delay between fragments to ensure
    // successful reception on receivers with slow copies out of the radio buffer
    // (imix)
    src_addr: Cell<IPAddr>,
    gateway: Cell<MacAddress>,
    tx_buf: TakeCell<'static, [u8]>,
    sixlowpan: TxState<'a>,
    radio: &'a MacDevice<'a>,
    dst_mac_addr: MacAddress,
    src_mac_addr: MacAddress,
    client: OptionalCell<&'a IP6SendClient>,
}

impl<A: time::Alarm> IP6Sender<'a> for IP6SendStruct<'a, A> {
    fn set_client(&self, client: &'a IP6SendClient) {
        self.client.set(client);
    }

    fn set_addr(&self, src_addr: IPAddr) {
        self.src_addr.set(src_addr);
    }

    fn set_gateway(&self, gateway: MacAddress) {
        self.gateway.set(gateway);
    }

    fn set_header(&mut self, ip6_header: IP6Header) {
        self.ip6_packet
            .map(|ip6_packet| ip6_packet.header = ip6_header);
    }

    fn send_to(
        &self,
        dst: IPAddr,
        transport_header: TransportHeader,
        payload: &[u8],
    ) -> ReturnCode {
        self.sixlowpan.init(
            self.src_mac_addr,
            self.dst_mac_addr,
            self.radio.get_pan(),
            None,
        );
        self.init_packet(dst, transport_header, payload);
        let ret = self.send_next_fragment();
        debug!("ip6_send ret: {:?}", ret);
        ret
    }
}

impl<A: time::Alarm> IP6SendStruct<'a, A> {
    pub fn new(
        ip6_packet: &'static mut IP6Packet<'static>,
        alarm: &'a A,
        tx_buf: &'static mut [u8],
        sixlowpan: TxState<'a>,
        radio: &'a MacDevice<'a>,
        dst_mac_addr: MacAddress,
        src_mac_addr: MacAddress,
    ) -> IP6SendStruct<'a, A> {
        IP6SendStruct {
            ip6_packet: TakeCell::new(ip6_packet),
            alarm: alarm,
            src_addr: Cell::new(IPAddr::new()),
            gateway: Cell::new(dst_mac_addr),
            tx_buf: TakeCell::new(tx_buf),
            sixlowpan: sixlowpan,
            radio: radio,
            dst_mac_addr: dst_mac_addr,
            src_mac_addr: src_mac_addr,
            client: OptionalCell::empty(),
        }
    }

    fn init_packet(&self, dst_addr: IPAddr, transport_header: TransportHeader, payload: &[u8]) {
        self.ip6_packet.map(|ip6_packet| {
            ip6_packet.header = IP6Header::default();
            ip6_packet.header.src_addr = self.src_addr.get();
            ip6_packet.header.dst_addr = dst_addr;
            ip6_packet.set_payload(transport_header, payload);
            ip6_packet.set_transport_checksum();
        });
    }

    // Returns EBUSY if the tx_buf is not there
    fn send_next_fragment(&self) -> ReturnCode {
        // Originally send_complete() was called within the below closure.
        // However, this led to a race condition where when multiple apps transmitted
        // simultaneously, it was possible for send_complete to trigger another
        // transmission before the below closure would exit, leading to this function
        // being called again by another app before ip6_packet is replaced.
        // To fix this, we pass a bool out of the closure to indicate whether send_completed()
        // should be called once the closure exits
        let (ret, call_send_complete) = self
            .ip6_packet
            .map(move |ip6_packet| match self.tx_buf.take() {
                Some(tx_buf) => {
                    let mut send = false;
                    let mut send_complete_return = ReturnCode::SUCCESS;
                    let next_frame = self.sixlowpan.next_fragment(ip6_packet, tx_buf, self.radio);
                    match next_frame {
                        Ok((is_done, frame)) => {
                            if is_done {
                                self.tx_buf.replace(frame.into_buf());
                                //self.send_completed(ReturnCode::SUCCESS);
                                send = true;
                                return (send_complete_return, send);
                            } else {
                                let (err, _frame_option) = self.radio.transmit(frame);
                                return (err, send);
                            }
                        }
                        Err((retcode, buf)) => {
                            self.tx_buf.replace(buf);
                            //self.send_completed(retcode);
                            send = true;
                            send_complete_return = retcode;
                        }
                    }
                    (send_complete_return, send)
                }
                None => (ReturnCode::EBUSY, false),
            })
            .unwrap_or((ReturnCode::ENOMEM, false));
        debug!("send_complete?: {:?}", call_send_complete);
        if call_send_complete {
            self.send_completed(ret);
            return ReturnCode::SUCCESS;
        }
        ret
    }

    fn send_completed(&self, result: ReturnCode) {
        self.client.map(move |client| {
            client.send_done(result);
        });
    }
}

impl<A: time::Alarm> time::Client for IP6SendStruct<'a, A> {
    fn fired(&self) {
        let result = self.send_next_fragment();
        if result != ReturnCode::SUCCESS {
            self.send_completed(result);
        }
    }
}

impl<A: time::Alarm> TxClient for IP6SendStruct<'a, A> {
    fn send_done(&self, tx_buf: &'static mut [u8], acked: bool, result: ReturnCode) {
        self.tx_buf.replace(tx_buf);
        if result != ReturnCode::SUCCESS {
            debug!("Send Failed: {:?}, acked: {}", result, acked);
        }
        // Below code adds delay between fragments. Despite some efforts
        // to fix this bug, I find that without it the receiving imix cannot
        // receive more than 2 fragments in a single packet without hanging
        // waiting for the third fragments.
        // Specifically, here we set a timer, which fires and sends the next fragment
        // One flaw with this is that we also introduce a delay after sending the last
        // fragment, before passing the send_done callback back to the client. This
        // could be optimized by checking if it is the last fragment before setting the timer.
        let interval = (100000 as u32) * <A::Frequency>::frequency() / 1000000;
        let tics = self.alarm.now().wrapping_add(interval);
        self.alarm.set_alarm(tics);
    }
}
