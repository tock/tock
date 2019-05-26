//! This file contains structs, traits, and methods associated with the IP layer
//! of the networking stack. This includes the declaration and methods for the
//! IP6Header, IP6Packet, and IP6Payload structs. These methods implement the
//! bulk of the functionality required for manipulating the fields of the
//! IPv6 header. Additionally, the IP6Packet struct allows for multiple types
//! of transport-level structures to be encapsulated within.
//!
//! An implementation for the structure of an IPv6 packet is provided by this
//! file, and a rough outline is given below:
//!
//!            ----------------------------------------------
//!            |                 IP6Packet                  |
//!            |--------------------------------------------|
//!            |                 |         IPPayload        |
//!            |    IP6Header    |--------------------------|
//!            |                 |TransportHeader | Payload |
//!            ----------------------------------------------
//!
//! The [IP6Packet](struct.IP6Packet.html) struct contains an
//! [IP6Header](struct.IP6Header.html) struct and an
//! [IPPayload](struct.IPPayload.html) struct, with the `IPPayload` struct
//! also containing a [TransportHeader](enum.TransportHeader.html) enum and
//! a `Payload` buffer. Note that transport-level headers are contained inside
//! the `TransportHeader`.
//!
//! For a client interested in using this interface, they first statically
//! allocate an `IP6Packet` struct, then set the appropriate headers and
//! payload using the functions defined for the different structs. These
//! methods are described in greater detail below.

// Discussion of Design Decisions
// ------------------------------
// Although still a work-in-progress, the IPv6 layer is quite complicated, and
// this initial interface represents some of the compromises made in trying
// to design a memory efficient, modular IP layer. The primary decision made
// for the IPv6 layer was the design of the `IP6Packet` struct. We noticed
// that the mutable payload buffer should always be associated with some type
// of headers; that is, we noticed that the payload for an IP packet should
// be perminantly owned by an instance of an IPv6 packet. This avoids runtime
// checks, as Rust can guarantee that the payload for an IPv6 packet is always
// there, as it cannot be moved. In order to facilitate this design while still
// allowing for (somewhat) arbitrary transport-level headers, we needed to
// separate out the `TransportHeader` enum from the payload itself. Since
// we did not want the IP layer to always have to have knowledge of/deal with
// the transport-level header, we decided to add an intermediate `IPPayload`
// struct, which encapsulated the `TransportHeader` and associated payload.
//
// Known Problems and Remaining Work
// ---------------------------------
// This layer is still in the early stages of implementation, and both the
// interfaces and underlying code will change substantially. There are two main
// areas of focus for additional work: 1) ensuring that the IP6Packet/IP6Header/
// IPPayload design makes sense and is properly layered, and 2) figuring out
// and implementing a receive path that uses this encapsulation.
//
// One of the primary problems with the current encapsulation design is that
// it is impossible to encode recursive headers - any subsequent headers (IPv6
// or transport) must be serialized and carried in the raw payload. This may
// be avoided with references and allocation, but since we do not have
// a memory allocator we could not allocate all possible headers at compile
// time. Additionally, we couldn't just allocate headers "as-needed" on the
// stack, as the network send interface is asynchronous, so anything allocated
// on the stack would eventually be popped/disappear. Although this is not
// a major problem in general, it makes handling encapsulated IPv6 packets
// (as required by 6LoWPAN) difficult.

use crate::net::buffer::Buffer;
use crate::net::icmpv6::icmpv6::ICMP6Header;
use crate::net::ipv6::ip_utils::{compute_icmp_checksum, compute_udp_checksum, ip6_nh, IPAddr};
use crate::net::stream::SResult;
use crate::net::stream::{decode_bytes, decode_u16, decode_u8};
use crate::net::stream::{encode_bytes, encode_u16, encode_u8};
use crate::net::tcp::TCPHeader;
use crate::net::udp::udp::UDPHeader;
use kernel::ReturnCode;

pub const UDP_HDR_LEN: usize = 8;
pub const ICMP_HDR_LEN: usize = 8;

/// This is the struct definition for an IPv6 header. It contains (in order)
/// the same fields as a normal IPv6 header.
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct IP6Header {
    pub version_class_flow: [u8; 4],
    pub payload_len: u16,
    pub next_header: u8,
    pub hop_limit: u8,
    pub src_addr: IPAddr,
    pub dst_addr: IPAddr,
}

impl Default for IP6Header {
    fn default() -> IP6Header {
        let version = 0x60;
        let hop_limit = 255;
        IP6Header {
            version_class_flow: [version, 0, 0, 0],
            payload_len: 0,
            next_header: ip6_nh::NO_NEXT,
            hop_limit: hop_limit,
            src_addr: IPAddr::new(),
            dst_addr: IPAddr::new(),
        }
    }
}

impl IP6Header {
    /// This function returns an IP6Header struct initialized to the default
    /// values.
    pub fn new() -> IP6Header {
        IP6Header::default()
    }

    /// This function is used to transform a raw buffer into an IP6Header
    /// struct. This is useful for deserializing a header upon reception.
    ///
    /// # Arguments
    ///
    /// `buf` - The serialized version of an IPv6 header
    ///
    /// # Return Value
    ///
    /// `SResult<IP6Header>` - The resulting decoded IP6Header struct wrapped
    /// in an SResult
    pub fn decode(buf: &[u8]) -> SResult<IP6Header> {
        // TODO: Let size of header be a constant
        stream_len_cond!(buf, 40);

        let mut ip6_header = Self::new();
        // Note that `dec_consume!` uses the length of the output buffer to
        // determine how many bytes are to be read.
        let off = dec_consume!(buf, 0; decode_bytes, &mut ip6_header.version_class_flow);
        let (off, payload_len_be) = dec_try!(buf, off; decode_u16);
        ip6_header.payload_len = u16::from_be(payload_len_be);
        let (off, next_header) = dec_try!(buf, off; decode_u8);
        ip6_header.next_header = next_header;
        let (off, hop_limit) = dec_try!(buf, off; decode_u8);
        ip6_header.hop_limit = hop_limit;
        let off = dec_consume!(buf, off; decode_bytes, &mut ip6_header.src_addr.0);
        let off = dec_consume!(buf, off; decode_bytes, &mut ip6_header.dst_addr.0);
        stream_done!(off, ip6_header);
    }

    /// This function transforms the `self` instance of an IP6Header into a
    /// byte array
    ///
    /// # Arguments
    ///
    /// `buf` - A mutable array where the serialized version of the IP6Header
    /// struct is written to
    ///
    /// # Return Value
    ///
    /// `SResult<usize>` - The offset wrapped in an SResult
    pub fn encode(&self, buf: &mut [u8]) -> SResult<usize> {
        stream_len_cond!(buf, 40);

        let mut off = enc_consume!(buf, 0; encode_bytes, &self.version_class_flow);
        off = enc_consume!(buf, off; encode_u16, self.payload_len.to_be());
        off = enc_consume!(buf, off; encode_u8, self.next_header);
        off = enc_consume!(buf, off; encode_u8, self.hop_limit);
        off = enc_consume!(buf, off; encode_bytes, &self.src_addr.0);
        off = enc_consume!(buf, off; encode_bytes, &self.dst_addr.0);
        stream_done!(off, off);
    }

    pub fn get_src_addr(&self) -> IPAddr {
        self.src_addr
    }

    pub fn get_dst_addr(&self) -> IPAddr {
        self.dst_addr
    }

    // Version should always be 6
    pub fn get_version(&self) -> u8 {
        (self.version_class_flow[0] & 0xf0) >> 4
    }

    pub fn get_traffic_class(&self) -> u8 {
        (self.version_class_flow[0] & 0x0f) << 4 | (self.version_class_flow[1] & 0xf0) >> 4
    }

    pub fn set_traffic_class(&mut self, new_tc: u8) {
        self.version_class_flow[0] &= 0xf0;
        self.version_class_flow[0] |= (new_tc & 0xf0) >> 4;
        self.version_class_flow[1] &= 0x0f;
        self.version_class_flow[1] |= (new_tc & 0x0f) << 4;
    }

    fn get_dscp_unshifted(&self) -> u8 {
        self.get_traffic_class() & 0b11111100
    }

    pub fn get_dscp(&self) -> u8 {
        self.get_dscp_unshifted() >> 2
    }

    pub fn set_dscp(&mut self, new_dscp: u8) {
        let ecn = self.get_ecn();
        self.set_traffic_class(ecn | ((new_dscp << 2) & 0b11111100));
    }

    pub fn get_ecn(&self) -> u8 {
        self.get_traffic_class() & 0b11
    }

    pub fn set_ecn(&mut self, new_ecn: u8) {
        let dscp_unshifted = self.get_dscp_unshifted();
        self.set_traffic_class(dscp_unshifted | (new_ecn & 0b11));
    }

    // This returns the flow label as the lower 20 bits of a u32
    pub fn get_flow_label(&self) -> u32 {
        let mut flow_label: u32 = 0;
        flow_label |= ((self.version_class_flow[1] & 0x0f) as u32) << 16;
        flow_label |= (self.version_class_flow[2] as u32) << 8;
        flow_label |= self.version_class_flow[3] as u32;
        flow_label
    }

    pub fn set_flow_label(&mut self, new_fl_val: u32) {
        self.version_class_flow[1] &= 0xf0;
        self.version_class_flow[1] |= ((new_fl_val >> 16) & 0x0f) as u8;
        self.version_class_flow[2] = (new_fl_val >> 8) as u8;
        self.version_class_flow[3] = new_fl_val as u8;
    }

    pub fn get_payload_len(&self) -> u16 {
        u16::from_be(self.payload_len)
    }

    // TODO: 40 = size of IP6header - find idiomatic way to compute
    pub fn get_total_len(&self) -> u16 {
        40 + self.get_payload_len()
    }

    pub fn set_payload_len(&mut self, new_len: u16) {
        self.payload_len = new_len.to_be();
    }

    pub fn get_next_header(&self) -> u8 {
        self.next_header
    }

    pub fn set_next_header(&mut self, new_nh: u8) {
        self.next_header = new_nh;
    }

    pub fn get_hop_limit(&self) -> u8 {
        self.hop_limit
    }

    pub fn set_hop_limit(&mut self, new_hl: u8) {
        self.hop_limit = new_hl;
    }

    /// Utility function for verifying whether a transport layer checksum of a received
    /// packet is correct. Is called on the assocaite IPv6 Header, and passed the buffer
    /// containing the remainder of the packet.
    pub fn check_transport_checksum(&self, buf: &[u8]) -> ReturnCode {
        match self.next_header {
            ip6_nh::UDP => {
                let mut udp_header: [u8; UDP_HDR_LEN] = [0; UDP_HDR_LEN];
                udp_header.copy_from_slice(&buf[..UDP_HDR_LEN]);
                let checksum = match UDPHeader::decode(&udp_header).done() {
                    Some((_offset, hdr)) => u16::from_be(compute_udp_checksum(
                        &self,
                        &hdr,
                        buf.len() as u16,
                        &buf[UDP_HDR_LEN..],
                    )),
                    None => 0xffff, //Will be dropped, as ones comp -0 checksum is invalid
                };
                if checksum != 0 {
                    return ReturnCode::FAIL; //Incorrect cksum
                }
                ReturnCode::SUCCESS
            }
            ip6_nh::ICMP => {
                // Untested (10/5/18)
                let mut icmp_header: [u8; ICMP_HDR_LEN] = [0; ICMP_HDR_LEN];
                icmp_header.copy_from_slice(&buf[..ICMP_HDR_LEN]);
                let checksum = match ICMP6Header::decode(&icmp_header).done() {
                    Some((_offset, mut hdr)) => {
                        hdr.set_len(buf.len() as u16);
                        u16::from_be(compute_icmp_checksum(&self, &hdr, &buf[ICMP_HDR_LEN..]))
                    }
                    None => 0xffff, //Will be dropped, as ones comp -0 checksum is invalid
                };
                if checksum != 0 {
                    return ReturnCode::FAIL; //Incorrect cksum
                }
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

/// This defines the currently supported `TransportHeader` types. The contents
/// of each header is encapsulated by the enum type. Note that this definition
/// of `TransportHeader`s means that recursive headers are not supported.
#[derive(Copy, Clone)] // TODO: is this ok?
pub enum TransportHeader {
    UDP(UDPHeader),
    TCP(TCPHeader),
    ICMP(ICMP6Header),
    // TODO: Need a length in RawIPPacket for the buffer in TransportHeader
    /* Raw(RawIPPacket<'a>), */
}

/// The `IPPayload` struct contains a `TransportHeader` and a mutable buffer
/// (the payload).
pub struct IPPayload<'a> {
    pub header: TransportHeader,
    pub payload: &'a mut [u8],
}

impl IPPayload<'a> {
    /// This function constructs a new `IPPayload` struct
    ///
    /// # Arguments
    ///
    /// `header` - A `TransportHeader` for the `IPPayload`
    /// `payload` - A reference to a mutable buffer for the raw payload
    pub fn new(header: TransportHeader, payload: &'a mut [u8]) -> IPPayload<'a> {
        IPPayload {
            header: header,
            payload: payload,
        }
    }

    /// This function sets the payload for the `IPPayload`, and sets both the
    /// TransportHeader and copies the provided payload buffer.
    ///
    /// # Arguments
    ///
    /// `transport_header` - The new `TransportHeader` header for the payload
    /// `payload` - The payload to be copied into the `IPPayload`
    ///
    /// # Return Value
    ///
    /// `(u8, u16)` - Returns a tuple of the `ip6_nh` type of the
    /// `transport_header` and the total length of the `IPPayload`
    /// (when serialized)
    pub fn set_payload(
        &mut self,
        transport_header: TransportHeader,
        payload: &mut Buffer<'static, u8>,
    ) -> (u8, u16) {
        if self.payload.len() < payload.len() {
            // TODO: Error
        }
        for i in 0..payload.len() {
            self.payload[i] = payload[i];
        }
        //self.payload[..payload.len()].copy_from_slice(payload.as_ptr());
        match transport_header {
            TransportHeader::UDP(mut udp_header) => {
                let length = (payload.len() + udp_header.get_hdr_size()) as u16;
                udp_header.set_len(length);
                self.header = transport_header;
                (ip6_nh::UDP, length)
            }
            TransportHeader::ICMP(mut icmp_header) => {
                let length = (payload.len() + icmp_header.get_hdr_size()) as u16;
                icmp_header.set_len(length);
                self.header = transport_header;
                (ip6_nh::ICMP, length)
            }
            _ => (ip6_nh::NO_NEXT, payload.len() as u16),
        }
    }

    /// This function encodes the `IPPayload` as a byte array
    ///
    /// # Arguments
    ///
    /// `buf` - Buffer to write the serialized `IPPayload` to
    /// `offset` - Current offset into the buffer
    ///
    /// # Return Value
    ///
    /// `SResult<usize>` - The final offset into the buffer `buf` is returned
    /// wrapped in an SResult
    pub fn encode(&self, buf: &mut [u8], offset: usize) -> SResult<usize> {
        let (offset, _) = match self.header {
            TransportHeader::UDP(udp_header) => udp_header.encode(buf, offset).done().unwrap(),
            TransportHeader::ICMP(icmp_header) => icmp_header.encode(buf, offset).done().unwrap(),
            _ => {
                unimplemented!();
            }
        };
        let payload_length = self.get_payload_length();
        let offset = enc_consume!(buf, offset; encode_bytes, &self.payload[..payload_length]);
        stream_done!(offset, offset)
    }

    fn get_payload_length(&self) -> usize {
        match self.header {
            TransportHeader::UDP(udp_header) => {
                udp_header.get_len() as usize - udp_header.get_hdr_size()
            }
            TransportHeader::ICMP(icmp_header) => {
                icmp_header.get_len() as usize - icmp_header.get_hdr_size()
            }
            _ => {
                unimplemented!();
            }
        }
    }
}

/// This struct defines the `IP6Packet` format, and contains an `IP6Header`
/// and an `IPPayload`.
pub struct IP6Packet<'a> {
    pub header: IP6Header,
    pub payload: IPPayload<'a>,
}

// Note: We want to have the IP6Header struct implement these methods,
// as there are cases where we want to allocate/modify the IP6Header without
// allocating/modifying the entire IP6Packet
impl IP6Packet<'a> {
    // Sets fields to appropriate defaults

    /// This function returns a new `IP6Packet` struct. Note that the
    /// `IP6Packet.header` field is set to `IP6Header::default()`
    ///
    /// # Arguments
    ///
    /// `payload` - The `IPPayload` struct for the `IP6Packet`
    pub fn new(payload: IPPayload<'a>) -> IP6Packet<'a> {
        IP6Packet {
            header: IP6Header::default(),
            payload: payload,
        }
    }

    pub fn reset(&mut self) {
        self.header = IP6Header::default();
    }

    pub fn get_total_len(&self) -> u16 {
        40 + self.header.get_payload_len()
    }

    pub fn get_payload(&self) -> &[u8] {
        self.payload.payload
    }

    pub fn get_total_hdr_size(&self) -> usize {
        let transport_hdr_size = match self.payload.header {
            TransportHeader::UDP(udp_hdr) => udp_hdr.get_hdr_size(),
            TransportHeader::ICMP(icmp_header) => icmp_header.get_hdr_size(),
            _ => unimplemented!(),
        };
        40 + transport_hdr_size
    }

    pub fn set_transport_checksum(&mut self) {
        // Looks at internal buffer assuming
        // it contains a valid IP packet, checks the payload type. If the payload
        // type requires a cksum calculation, this function calculates the
        // psuedoheader cksum and calls the appropriate transport packet function
        // using this pseudoheader cksum to set the transport packet cksum

        match self.payload.header {
            TransportHeader::UDP(ref mut udp_header) => {
                let cksum = compute_udp_checksum(
                    &self.header,
                    &udp_header,
                    udp_header.get_len(),
                    self.payload.payload,
                );
                udp_header.set_cksum(cksum);
            }
            TransportHeader::ICMP(ref mut icmp_header) => {
                let cksum = compute_icmp_checksum(&self.header, &icmp_header, self.payload.payload);
                icmp_header.set_cksum(cksum);
            }
            _ => {
                unimplemented!();
            }
        }
    }

    /// This function should be the function used to set the payload for a
    /// given `IP6Packet` object. It first calls the `IPPayload::set_payload`
    /// method to set the transport header and transport payload, which then
    /// returns the `ip6_nh` value for the `TransportHeader` and the length of
    /// the serialized `IPPayload` region. This function then sets the
    /// `IP6Header` next header field correctly. **Without using this function,
    /// the `IP6Header.next_header` field may not agree with the actual
    /// next header (`IP6Header.payload.header`)**
    ///
    /// # Arguments
    ///
    /// `transport_header` - The `TransportHeader` to be set as the next header
    /// `payload` - The transport payload to be copied into the `IPPayload`
    /// transport payload
    pub fn set_payload(
        &mut self,
        transport_header: TransportHeader,
        payload: &mut Buffer<'static, u8>,
    ) {
        let (next_header, payload_len) = self.payload.set_payload(transport_header, payload);
        self.header.set_next_header(next_header);
        self.header.set_payload_len(payload_len);
    }

    // TODO: Do we need a decode equivalent? I don't think so, but we might

    pub fn encode(&self, buf: &mut [u8]) -> SResult<usize> {
        let ip6_header = self.header;

        // TODO: Handle unwrap safely
        let (off, _) = ip6_header.encode(buf).done().unwrap();
        self.payload.encode(buf, off)
    }
}
