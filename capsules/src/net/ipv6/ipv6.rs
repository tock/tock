//! This file contains structs, traits, and methods associated with the IP layer
//! of the networking stack. This includes the declaration and methods for the
//! IP6Header, IP6Packet, and IP6Payload structs. These methods implement the
//! bulk of the functionality required for manipulating the fields of the
//! IPv6 header. Additionally, the IP6Packet struct allows for multiple types
//! of transport-level structures to be encapsulated within.

use net::icmpv6::icmpv6::ICMP6Header;
use net::ipv6::ip_utils::{compute_icmp_checksum, compute_udp_checksum, IPAddr, ip6_nh};
use net::stream::{decode_bytes, decode_u16, decode_u8};
use net::stream::{encode_bytes, encode_u16, encode_u8};
use net::stream::SResult;
use net::tcp::TCPHeader;
use net::udp::udp::UDPHeader;

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
    pub fn new() -> IP6Header {
        IP6Header::default()
    }

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

    // Returns the offset wrapped in an SResult
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
}

// TODO: Note that this design decision means that we cannot have recursive
// IP6 packets directly - we must have/use RawIPPackets instead. This makes
// it difficult to recursively compress IP6 packets as required by 6lowpan
pub enum TransportHeader {
    UDP(UDPHeader),
    TCP(TCPHeader),
    ICMP(ICMP6Header),
    // TODO: Need a length in RawIPPacket for the buffer in TransportHeader
    /* Raw(RawIPPacket<'a>), */
}

pub struct IPPayload<'a> {
    pub header: TransportHeader,
    pub payload: &'a mut [u8],
}

impl<'a> IPPayload<'a> {
    pub fn new(header: TransportHeader, payload: &'a mut [u8]) -> IPPayload<'a> {
        IPPayload {
            header: header,
            payload: payload,
        }
    }

    pub fn set_payload(&mut self, transport_header: TransportHeader, payload: &[u8]) -> (u8, u16) {
        if self.payload.len() < payload.len() {
            // TODO: Error
        }
        self.payload.copy_from_slice(&payload);
        match transport_header {
            TransportHeader::UDP(mut udp_header) => {
                let length = (payload.len() + udp_header.get_hdr_size()) as u16;
                udp_header.set_len(length);
                (ip6_nh::UDP, length)
            }
            TransportHeader::ICMP(mut icmp_header) => {
                let length = (payload.len() + icmp_header.get_hdr_size()) as u16;
                icmp_header.set_len(length);
                (ip6_nh::ICMP, length)
            }
            _ => (ip6_nh::NO_NEXT, payload.len() as u16),
        }
    }

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

pub struct IP6Packet<'a> {
    pub header: IP6Header,
    pub payload: IPPayload<'a>,
}

// Note: We want to have the IP6Header struct implement these methods,
// as there are cases where we want to allocate/modify the IP6Header without
// allocating/modifying the entire IP6Packet
impl<'a> IP6Packet<'a> {
    // Sets fields to appropriate defaults

    pub fn new(pyld: IPPayload<'a>) -> IP6Packet<'a> {
        IP6Packet {
            header: IP6Header::default(),
            payload: pyld,
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
        //Looks at internal buffer assuming
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

    pub fn set_payload(&mut self, transport_header: TransportHeader, payload: &[u8]) {
        let (next_header, payload_len) = self.payload.set_payload(transport_header, payload);
        self.header.set_next_header(next_header);
        self.header.set_payload_len(payload_len);
    }

    // TODO: Currently, the receive path is unimplemented, and this function
    // should *not* be called
    pub fn decode(buf: &[u8], ip6_packet: &mut IP6Packet) -> Result<usize, ()> {
        let (_offset, header) = IP6Header::decode(buf).done().ok_or(())?;
        ip6_packet.header = header;
        // TODO: Not sure how to convert an IP6Packet with a UDP payload to
        // an IP6Packet with a TCP payload.
        unimplemented!();
    }

    pub fn encode(&self, buf: &mut [u8]) -> SResult<usize> {
        let ip6_header = self.header;

        // TODO: Handle unwrap safely
        let (off, _) = ip6_header.encode(buf).done().unwrap();
        self.payload.encode(buf, off)
    }
}
