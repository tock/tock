/// Implements the 6LoWPAN specification for sending IPv6 datagrams over
/// 802.15.4 packets efficiently, as detailed in RFC 6282.

use core::mem;
use core::cmp::min;
use core::result::Result;

use net::ip;
use net::ip::{IP6Header, MacAddr, IPAddr, ip6_nh};
use net::ip::{ntohs, htons, slice_to_u16, u16_to_slice};
use net::util;

#[allow(unused_variables,dead_code)]
mod iphc {
    pub const DISPATCH: [u8; 2]    = [0x60, 0x00];

    // First byte masks

    pub const TF_MASK: u8          = 0x18;
    pub const TF_TRAFFIC_CLASS: u8 = 0x08;
    pub const TF_FLOW_LABEL: u8    = 0x10;

    pub const NH: u8               = 0x04;

    pub const HLIM_MASK: u8        = 0x03;
    pub const HLIM_INLINE: u8      = 0x00;
    pub const HLIM_1: u8           = 0x01;
    pub const HLIM_64: u8          = 0x02;
    pub const HLIM_255: u8         = 0x03;

    // Second byte masks

    pub const CID: u8              = 0x80;

    pub const SAC: u8              = 0x40;

    pub const SAM_MASK: u8         = 0x30;
    pub const SAM_INLINE: u8       = 0x00;
    pub const SAM_MODE1: u8        = 0x10;
    pub const SAM_MODE2: u8        = 0x20;
    pub const SAM_MODE3: u8        = 0x30;

    pub const MULTICAST: u8        = 0x01;

    pub const DAC: u8              = 0x04;
    pub const DAM_MASK: u8         = 0x03;
    pub const DAM_INLINE: u8       = 0x00;
    pub const DAM_MODE1: u8        = 0x01;
    pub const DAM_MODE2: u8        = 0x02;
    pub const DAM_MODE3: u8        = 0x03;

    // Address compression
    pub const MAC_BASE: [u8; 8]    = [0, 0, 0, 0xff, 0xfe, 0, 0, 0];
    pub const MAC_UL: u8           = 0x02;
}

#[allow(unused_variables,dead_code)]
mod nhc {
    pub const DISPATCH_NHC: u8           = 0xe0;
    pub const DISPATCH_UDP: u8           = 0xf0;
    pub const DISPATCH_MASK: u8          = 0xf0;

    pub const EID_MASK: u8               = 0x0e;
    pub const HOP_OPTS: u8               = 0 << 1;
    pub const ROUTING: u8                = 1 << 1;
    pub const FRAGMENT: u8               = 2 << 1;
    pub const DST_OPTS: u8               = 3 << 1;
    pub const MOBILITY: u8               = 4 << 1;
    pub const IP6: u8                    = 7 << 1;

    pub const NH: u8                     = 0x01;

    pub const UDP_4BIT_PORT: u16         = 0xf0b0;
    pub const UDP_4BIT_PORT_MASK: u16    = 0xfff0;
    pub const UDP_8BIT_PORT: u16         = 0xf000;
    pub const UDP_8BIT_PORT_MASK: u16    = 0xff00;

    pub const UDP_CHECKSUM_FLAG: u8      = 0b100;
    pub const UDP_SRC_PORT_FLAG: u8      = 0b010;
    pub const UDP_DST_PORT_FLAG: u8      = 0b001;
}

#[allow(unused_variables,dead_code)]
#[derive(Copy,Clone,Debug)]
pub struct Context<'a> {
    pub prefix: &'a [u8],
    pub prefix_len: u8,
    pub id: u8,
    pub compress: bool,
}

pub trait ContextStore<'a> {
    fn get_context_from_addr(&self, ip_addr: IPAddr) -> Option<Context<'a>>;
    fn get_context_from_id(&self, ctx_id: u8) -> Option<Context<'a>>;
    fn get_context_from_prefix(&self, prefix: &[u8], prefix_len: u8) -> Option<Context<'a>>;
}

#[allow(unused_variables,dead_code)]
pub struct FragInfo {
    dummy: u8,
}

/// Computes the LoWPAN Interface Identifier from either the 16-bit short MAC or
/// the IEEE EUI-64 that is derived from the 48-bit MAC.
pub fn compute_iid(mac_addr: &MacAddr) -> [u8; 8] {
    match mac_addr {
        &MacAddr::ShortAddr(short_addr) => {
            // IID is 0000:00ff:fe00:XXXX, where XXXX is 16-bit MAC
            let mut iid: [u8; 8] = iphc::MAC_BASE;
            iid[6] = (short_addr >> 1) as u8;
            iid[7] = (short_addr & 0xff) as u8;
            iid
        },
        &MacAddr::LongAddr(long_addr) => {
            // IID is IEEE EUI-64 with universal/local bit inverted
            let mut iid: [u8; 8] = long_addr;
            iid[0] ^= iphc::MAC_UL;
            iid
        }
    }
}

/// This function writes the context bits into an IP address. Note that this
/// function must be called after the remaining bits of the IP address have
/// been set.
// NOTE: This function must always be called *after* the remaining bits
// of the address have been set, since its length is in terms of bits,
// and it "merges" the (nbits%8) context bits with the other bits.
fn set_ctx_bits_in_addr(ip_addr: &mut IPAddr, ctx: &Context) {
    ip_addr.set_prefix(&ctx.prefix, ctx.prefix_len);
}

fn is_ip6_nh_compressible(next_header: u8,
                          next_headers: &[u8]) -> Result<(bool, u8), ()> {
    match next_header {
        // IP6 encapsulated headers are always compressed
        ip6_nh::IP6 => Ok((true, 0)),
        // UDP headers are always compresed
        ip6_nh::UDP => Ok((true, 0)),
        ip6_nh::FRAGMENT
        | ip6_nh::HOP_OPTS
        | ip6_nh::ROUTING
        | ip6_nh::DST_OPTS
        | ip6_nh::MOBILITY => {
            let mut header_len: u32 = 6;
            if next_header != ip6_nh::FRAGMENT {
                if next_headers.len() < 2 {
                    return Err(());
                } else {
                    header_len += (next_headers[1] as u32) * 8;
                }
            }
            if header_len <= 255 {
                Ok((true, header_len as u8))
            } else {
                Ok((false, 0))
            }
        },
        _ => Ok((false, 0)),
    }
}

fn ones_complement_addition(val_1: u16, val_2: u16) -> u16 {
    let (sum, overflow) = val_1.overflowing_add(val_2);
    if overflow {
        sum + 1
    } else {
        sum
    }
}

// Note that this function returns a checksum in network-byte order
// TODO: Set checksum field to zero
// Note that there might be a possibility of the u32 overflowing
fn compute_udp_checksum(ip6_header: &IP6Header, udp_header: &[u8], payload: &[u8]) -> u16 {
    let len = payload.len();
    let mut checksum: u16 = 0;
    // IPv6 addresses
    for i in 0..8 {
        let val_1 = slice_to_u16(&ip6_header.src_addr.0[i*2..(i+1)*2]);
        checksum = ones_complement_addition(checksum, val_1);
        let val_2 = slice_to_u16(&ip6_header.dst_addr.0[i*2..(i+1)*2]);
        checksum = ones_complement_addition(checksum, val_2);
    }
    // Length of payload + UDP header
    let udp_len: u32 = (len + 8) as u32;
    checksum = ones_complement_addition(checksum, ((udp_len >> 16) & 0xffff) as u16);
    checksum = ones_complement_addition(checksum, (udp_len & 0xffff) as u16);
    checksum = ones_complement_addition(checksum, 0x11); // UDP next header value
    // UDP header
    for i in 0..4 {
        let val = slice_to_u16(&udp_header[i*2..(i+1)*2]);
        checksum = ones_complement_addition(checksum, val);
    }
    // Payload
    for i in 0..len/2 {
        let val = slice_to_u16(&payload[i*2..(i+1)*2]);
        checksum = ones_complement_addition(checksum, val);
    }
    // Remaining payload byte
    if len % 2 != 0 {
        let val = (payload[len] as u16) << 8;
        checksum = ones_complement_addition(checksum, val);
    }
    // Take the complement
    !checksum
}

/// Maps values of a IPv6 next header field to a corresponding LoWPAN
/// NHC-encoding extension ID
fn ip6_nh_to_nhc_eid(next_header: u8) -> Option<u8> {
    match next_header {
        ip6_nh::HOP_OPTS => Some(nhc::HOP_OPTS),
        ip6_nh::ROUTING  => Some(nhc::ROUTING),
        ip6_nh::FRAGMENT => Some(nhc::FRAGMENT),
        ip6_nh::DST_OPTS => Some(nhc::DST_OPTS),
        ip6_nh::MOBILITY => Some(nhc::MOBILITY),
        ip6_nh::IP6      => Some(nhc::IP6),
        _ => None,
    }
}

/// Maps LoWPAN NHC-encoded EIDs to the corresponding IPv6 next header
/// field value
#[allow(dead_code)]
fn nhc_to_ip6_nh(nhc: u8) -> Option<u8> {
    match nhc & nhc::DISPATCH_MASK {
        nhc::DISPATCH_NHC => match nhc & nhc::EID_MASK {
            nhc::HOP_OPTS => Some(ip6_nh::HOP_OPTS),
            nhc::ROUTING  => Some(ip6_nh::ROUTING),
            nhc::FRAGMENT => Some(ip6_nh::FRAGMENT),
            nhc::DST_OPTS => Some(ip6_nh::DST_OPTS),
            nhc::MOBILITY => Some(ip6_nh::MOBILITY),
            nhc::IP6      => Some(ip6_nh::IP6),
            _ => None,
        },
        nhc::DISPATCH_UDP => Some(ip6_nh::UDP),
        _ => None,
    }
}

pub struct LoWPAN<'a, C: ContextStore<'a> + 'a> {
    ctx_store: &'a C,
}

impl<'a, C: ContextStore<'a> + 'a> LoWPAN<'a, C> {
    pub fn new(ctx_store: &'a C) -> LoWPAN<'a, C> {
        LoWPAN { ctx_store: ctx_store }
    }

    /// Constructs a 6LoWPAN header in `buf` from the given IPv6 datagram and
    /// 16-bit MAC addresses.  Returns the number of bytes consumed from the
    /// IPv6 datagram and the number of bytes written into `buf`.
    pub fn compress(&self,
                    ip6_datagram: &[u8],
                    src_mac_addr: MacAddr,
                    dst_mac_addr: MacAddr,
                    mut buf: &mut [u8])
                    -> Result<(usize, usize), ()> {
        let ip6_header: &IP6Header = unsafe {
            mem::transmute(ip6_datagram.as_ptr())
        };
        let mut consumed: usize = mem::size_of::<IP6Header>();
        let mut next_headers: &[u8] = &ip6_datagram[consumed..];

        // The first two bytes are the LOWPAN_IPHC header
        let mut offset: usize = 2;

        // Initialize the LOWPAN_IPHC header
        buf[0..2].copy_from_slice(&iphc::DISPATCH);

        let mut src_ctx: Option<Context> = self.ctx_store
            .get_context_from_addr(ip6_header.src_addr);
        let mut dst_ctx: Option<Context> = if ip6_header.dst_addr.is_multicast() {
            let prefix_len: u8 = ip6_header.dst_addr.0[3];
            let prefix: &[u8] = &ip6_header.dst_addr.0[4..12];
            if util::verify_prefix_len(prefix, prefix_len) {
                self.ctx_store.get_context_from_prefix(prefix, prefix_len)
            } else {
                None
            }
        } else {
            self.ctx_store.get_context_from_addr(ip6_header.dst_addr)
        };

        // Do not use these contexts if they are not to be used for compression
        src_ctx = src_ctx.and_then(|ctx| if ctx.compress { Some(ctx) } else { None });
        dst_ctx = dst_ctx.and_then(|ctx| if ctx.compress { Some(ctx) } else { None });

        // Context Identifier Extension
        self.compress_cie(&src_ctx, &dst_ctx, &mut buf, &mut offset);

        // Traffic Class & Flow Label
        self.compress_tf(ip6_header, &mut buf, &mut offset);

        // Next Header
        let (mut is_nhc, mut nh_len): (bool, u8) =
            is_ip6_nh_compressible(ip6_header.next_header, next_headers)?;
        self.compress_nh(ip6_header, is_nhc, &mut buf, &mut offset);


        // Hop Limit
        self.compress_hl(ip6_header, &mut buf, &mut offset);

        // Source Address
        self.compress_src(&ip6_header.src_addr,
                          &src_mac_addr,
                          &src_ctx,
                          &mut buf,
                          &mut offset);

        // Destination Address
        if ip6_header.dst_addr.is_multicast() {
            self.compress_multicast(&ip6_header.dst_addr,
                                    &dst_ctx,
                                    &mut buf,
                                    &mut offset);
        } else {
            self.compress_dst(&ip6_header.dst_addr,
                              &dst_mac_addr,
                              &dst_ctx,
                              &mut buf,
                              &mut offset);
        }

        // Next Headers
        let mut ip6_nh_type: u8 = ip6_header.next_header;
        while is_nhc {
            match ip6_nh_type {
                ip6_nh::IP6 => {
                    // For IPv6 encapsulation, the NH bit in the NHC ID is 0
                    let nhc_header = nhc::DISPATCH_NHC | nhc::IP6;
                    buf[offset] = nhc_header;
                    offset += 1;

                    // Recursively place IPHC-encoded IPv6 after the NHC ID
                    let (encap_consumed, encap_offset) =
                        self.compress(next_headers,
                                      src_mac_addr,
                                      dst_mac_addr,
                                      &mut buf[offset..])?;
                    consumed += encap_consumed;
                    offset += encap_offset;

                    // The above recursion handles the rest of the packet
                    // headers, so we are done
                    break;
                },
                ip6_nh::UDP => {
                    let mut nhc_header = nhc::DISPATCH_UDP;

                    // Leave a space for the UDP LoWPAN_NHC byte
                    let udp_nh_offset = offset;
                    offset += 1;

                    // Compress ports and checksum
                    let udp_header = &next_headers[0..8];
                    nhc_header |= self.compress_udp_ports(udp_header,
                                                          &mut buf,
                                                          &mut offset);
                    nhc_header |= self.compress_udp_checksum(udp_header,
                                                             &mut buf,
                                                             &mut offset);

                    // Write the UDP LoWPAN_NHC byte
                    buf[udp_nh_offset] = nhc_header;
                    consumed += 8;

                    // There cannot be any more next headers after UDP
                    break;
                },
                ip6_nh::FRAGMENT
                | ip6_nh::HOP_OPTS
                | ip6_nh::ROUTING
                | ip6_nh::DST_OPTS
                | ip6_nh::MOBILITY => {
                    // The NHC EID is guaranteed not to be 0 here.
                    let mut nhc_header = nhc::DISPATCH_NHC
                        | ip6_nh_to_nhc_eid(ip6_nh_type).unwrap_or(0);
                    // next_nh_offset includes the next header field and the
                    // length byte, while nh_len does not
                    let next_nh_offset = 2 + (nh_len as usize);

                    // Determine if the next header is compressible
                    let (next_is_nhc, next_nh_len) =
                        is_ip6_nh_compressible(next_headers[0],
                                               &next_headers[next_nh_offset..])?;
                    if next_is_nhc {
                        nhc_header |= nhc::NH;
                    }

                    // Place NHC ID in buffer
                    buf[offset] = nhc_header;
                    if ip6_nh_type != ip6_nh::FRAGMENT {
                        // Fragment extension does not have a length field
                        buf[offset + 1] = nh_len;
                    }
                    offset += 2;

                    // Copy over the remaining packet data
                    for i in 0..nh_len {
                        buf[offset] = next_headers[2 + (i as usize)];
                        offset += 1;
                    }

                    ip6_nh_type = next_headers[0];
                    is_nhc = next_is_nhc;
                    nh_len = next_nh_len;
                    next_headers = &next_headers[next_nh_offset..];
                    consumed += next_nh_offset;
                },
                _ => {
                    // This case should not be reached, since is_nh_compressed
                    // is set by is_ip6_nh_compressible
                    return Err(());
                },
            }
        }

        Ok((consumed, offset))
    }

    fn compress_cie(&self,
                    src_ctx: &Option<Context>,
                    dst_ctx: &Option<Context>,
                    buf: &mut [u8],
                    offset: &mut usize) {
        let mut cie: u8 = 0;

        src_ctx.as_ref().map(|ctx| if ctx.id != 0 {
            cie |= ctx.id << 4;
        });
        dst_ctx.as_ref().map(|ctx| if ctx.id != 0 {
            cie |= ctx.id;
        });

        if cie != 0 {
            buf[1] |= iphc::CID;
            buf[*offset] = cie;
            *offset += 1;
        }
    }

    fn compress_tf(&self, ip6_header: &IP6Header, buf: &mut [u8], offset: &mut usize) {
        let ecn = ip6_header.get_ecn();
        let dscp = ip6_header.get_dscp();
        let flow = ip6_header.get_flow_label();

        let mut tf_encoding = 0;
        let old_offset = *offset;

        if dscp == 0 {
            tf_encoding |= iphc::TF_TRAFFIC_CLASS;
        } else {
            buf[*offset] = dscp;
            *offset += 1;
        }

        if flow == 0 {
            tf_encoding |= iphc::TF_FLOW_LABEL;
        } else {
            buf[*offset]     = ((flow >> 16) & 0x0f) as u8;
            buf[*offset + 1] = (flow >> 8) as u8;
            buf[*offset + 2] = flow as u8;
            *offset += 3;
        }

        if *offset != old_offset {
            buf[old_offset] |= ecn << 6;
        }
        buf[0] |= tf_encoding;
    }

    fn compress_nh(&self,
                   ip6_header: &IP6Header,
                   is_nhc: bool,
                   buf: &mut [u8],
                   offset: &mut usize) {
        if is_nhc {
            buf[0] |= iphc::NH;
        } else {
            buf[*offset] = ip6_header.next_header;
            *offset += 1;
        }
    }

    fn compress_hl(&self, ip6_header: &IP6Header, buf: &mut [u8], offset: &mut usize) {
        let hop_limit_flag = {
            match ip6_header.hop_limit {
                // Compressed
                1 => iphc::HLIM_1,
                64 => iphc::HLIM_64,
                255 => iphc::HLIM_255,
                // Uncompressed
                _ => {
                    buf[*offset] = ip6_header.hop_limit;
                    *offset += 1;
                    iphc::HLIM_INLINE
                }
            }
        };
        buf[0] |= hop_limit_flag;
    }

    // TODO: We should check to see whether context or link local compression
    // schemes gives the better compression; currently, we will always match
    // on link local even if we could get better compression through context.
    fn compress_src(&self,
                    src_ip_addr: &IPAddr,
                    src_mac_addr: &MacAddr,
                    src_ctx: &Option<Context>,
                    buf: &mut [u8],
                    offset: &mut usize) {
        if src_ip_addr.is_unspecified() {
            // SAC = 1, SAM = 00
            buf[1] |= iphc::SAC;
        } else if src_ip_addr.is_unicast_link_local() {
            // SAC = 0, SAM = 01, 10, 11
            self.compress_iid(src_ip_addr, src_mac_addr, true, buf, offset);
        } else if src_ctx.is_some() {
            // SAC = 1, SAM = 01, 10, 11
            buf[1] |= iphc::SAC;
            self.compress_iid(src_ip_addr, src_mac_addr, true, buf, offset);
        } else {
            // SAC = 0, SAM = 00
            buf[*offset..*offset + 16].copy_from_slice(&src_ip_addr.0);
            *offset += 16;
        }
    }

    // TODO: For the SAC=0, SAM=11 case, we must also consider computing the
    // address from an encapsulating IPv6 packet (e.g. when we recurse), not
    // just from a 802.15.4 frame.
    fn compress_iid(&self,
                    ip_addr: &IPAddr,
                    mac_addr: &MacAddr,
                    is_src: bool,
                    buf: &mut [u8],
                    offset: &mut usize) {
        let iid: [u8; 8] = compute_iid(mac_addr);
        if ip_addr.0[8..16] == iid {
            // SAM/DAM = 11, 0 bits
            buf[1] |= if is_src {
                iphc::SAM_MODE3
            } else {
                iphc::DAM_MODE3
            };
        } else if ip_addr.0[8..14] == iphc::MAC_BASE[0..6] {
            // SAM/DAM = 10, 16 bits
            buf[1] |= if is_src {
                iphc::SAM_MODE2
            } else {
                iphc::DAM_MODE2
            };
            buf[*offset..*offset + 2].copy_from_slice(&ip_addr.0[14..16]);
            *offset += 2;
        } else {
            // SAM/DAM = 01, 64 bits
            buf[1] |= if is_src {
                iphc::SAM_MODE1
            } else {
                iphc::DAM_MODE1
            };
            buf[*offset..*offset + 8].copy_from_slice(&ip_addr.0[8..16]);
            *offset += 8;
        }
    }

    // Compresses non-multicast destination address
    // TODO: We should check to see whether context or link local compression
    // schemes gives the better compression; currently, we will always match
    // on link local even if we could get better compression through context.
    fn compress_dst(&self,
                    dst_ip_addr: &IPAddr,
                    dst_mac_addr: &MacAddr,
                    dst_ctx: &Option<Context>,
                    buf: &mut [u8],
                    offset: &mut usize) {
        // Assumes dst_ip_addr is not a multicast address (prefix ffXX)
        if dst_ip_addr.is_unicast_link_local() {
            // Link local compression
            // M = 0, DAC = 0, DAM = 01, 10, 11
            self.compress_iid(dst_ip_addr, dst_mac_addr, false, buf, offset);
        } else if dst_ctx.is_some() {
            // Context compression
            // DAC = 1, DAM = 01, 10, 11
            buf[1] |= iphc::DAC;
            self.compress_iid(dst_ip_addr, dst_mac_addr, false, buf, offset);
        } else {
            // Full address inline
            // DAC = 0, DAM = 00
            buf[*offset..*offset + 16].copy_from_slice(&dst_ip_addr.0);
            *offset += 16;
        }
    }

    // Compresses multicast destination addresses
    fn compress_multicast(&self,
                          dst_ip_addr: &IPAddr,
                          dst_ctx: &Option<Context>,
                          buf: &mut [u8],
                          offset: &mut usize) {
        // Assumes dst_ip_addr is indeed a multicast address (prefix ffXX)
        buf[1] |= iphc::MULTICAST;
        if dst_ctx.is_some() {
            // M = 1, DAC = 1, DAM = 00
            buf[1] |= iphc::DAC;
            buf[*offset..*offset + 2].copy_from_slice(&dst_ip_addr.0[1..3]);
            buf[*offset + 2..*offset + 6].copy_from_slice(&dst_ip_addr.0[12..16]);
            *offset += 6;
        } else {
            // M = 1, DAC = 0
            if dst_ip_addr.0[1] == 0x02 && util::is_zero(&dst_ip_addr.0[2..15]) {
                // DAM = 11
                buf[1] |= iphc::DAM_MODE3;
                buf[*offset] = dst_ip_addr.0[15];
                *offset += 1;
            } else {
                if !util::is_zero(&dst_ip_addr.0[2..11]) {
                    // DAM = 00
                    buf[1] |= iphc::DAM_INLINE;
                    buf[*offset..*offset + 16].copy_from_slice(&dst_ip_addr.0);
                    *offset += 16;
                } else if !util::is_zero(&dst_ip_addr.0[11..13]) {
                    // DAM = 01, ffXX::00XX:XXXX:XXXX
                    buf[1] |= iphc::DAM_MODE1;
                    buf[*offset] = dst_ip_addr.0[1];
                    buf[*offset + 1..*offset + 6].copy_from_slice(&dst_ip_addr.0[11..16]);
                    *offset += 6;
                } else {
                    // DAM = 10, ffXX::00XX:XXXX
                    buf[1] |= iphc::DAM_MODE2;
                    buf[*offset] = dst_ip_addr.0[1];
                    buf[*offset + 1..*offset + 4].copy_from_slice(&dst_ip_addr.0[13..16]);
                    *offset += 4;
                }
            }
        }
    }

    fn compress_udp_ports(&self,
                          udp_header: &[u8],
                          buf: &mut [u8],
                          offset: &mut usize) -> u8 {
        let src_port: u16 = ntohs(slice_to_u16(&udp_header[0..2]));
        let dst_port: u16 = ntohs(slice_to_u16(&udp_header[2..4]));

        let mut udp_port_nhc = 0;
        if (src_port & nhc::UDP_4BIT_PORT_MASK) == nhc::UDP_4BIT_PORT
            && (dst_port & nhc::UDP_4BIT_PORT_MASK) == nhc::UDP_4BIT_PORT {
            // Both can be compressed to 4 bits
            udp_port_nhc |= nhc::UDP_SRC_PORT_FLAG | nhc::UDP_DST_PORT_FLAG;
            // This should compress the ports to a single 8-bit value,
            // with the source port before the destination port
            buf[*offset] = (((src_port & !nhc::UDP_4BIT_PORT_MASK) << 4)
                           | (dst_port & !nhc::UDP_4BIT_PORT_MASK)) as u8;
            *offset += 1;
        } else if (src_port & nhc::UDP_8BIT_PORT_MASK) == nhc::UDP_8BIT_PORT {
            // Source port compressed to 8 bits, destination port uncompressed
            udp_port_nhc |= nhc::UDP_SRC_PORT_FLAG;
            buf[*offset] = (src_port & !nhc::UDP_8BIT_PORT_MASK) as u8;
            u16_to_slice(htons(dst_port), &mut buf[*offset + 1..*offset + 3]);
            *offset += 3;
        } else if (dst_port & nhc::UDP_8BIT_PORT_MASK) == nhc::UDP_8BIT_PORT {
            udp_port_nhc |= nhc::UDP_DST_PORT_FLAG;
            u16_to_slice(htons(src_port), &mut buf[*offset..*offset + 2]);
            buf[*offset + 3] = (dst_port & !nhc::UDP_8BIT_PORT_MASK) as u8;
            *offset += 3;
        } else {
            buf[*offset..*offset + 4].copy_from_slice(&udp_header[0..4]);
            *offset += 4;
        }
        return udp_port_nhc;
    }

    fn compress_udp_checksum(&self,
                             udp_header: &[u8],
                             buf: &mut [u8],
                             offset: &mut usize) -> u8 {
        // TODO: Checksum is always inline, elision is currently not supported
        buf[*offset] = udp_header[6];
        buf[*offset + 1] = udp_header[7];
        *offset += 2;
        // Inline checksum corresponds to the 0 flag
        0
    }

    /// Decodes the compressed header into a full IPv6 header given the 16-bit
    /// MAC addresses. `buf` is expected to be a slice starting from the
    /// beginning of the IP header.  Returns the number of bytes taken up by the
    /// header, so the remaining bytes are the payload. Also returns an optional
    /// `FragInfo` containing the datagram tag and fragmentation offset if this
    /// packet is part of a set of fragments.
    #[allow(unused_variables,dead_code)]
    pub fn decompress(&self,
                      buf: &[u8],
                      src_mac_addr: MacAddr,
                      dst_mac_addr: MacAddr,
                      mut out_buf: &mut [u8])
                      -> Result<(usize, usize), ()> {
        // Get the LOWPAN_IPHC header (the first two bytes are the header)
        let iphc_header_1: u8 = buf[0];
        let iphc_header_2: u8 = buf[1];
        let mut offset: usize = 2;

        let mut ip6_header: &mut IP6Header = unsafe {
            mem::transmute(out_buf.as_mut_ptr())
        };
        let mut bytes_written: usize = mem::size_of::<IP6Header>();
        *ip6_header = IP6Header::new();

        // Decompress CID and CIE fields if they exist
        let (src_ctx, dst_ctx) =
            self.decompress_cie(iphc_header_1, &buf, &mut offset)?;

        // Traffic Class & Flow Label
        self.decompress_tf(&mut ip6_header, iphc_header_1, &buf, &mut offset);

        // Next Header
        let (mut is_nhc, mut next_header) = self.decompress_nh(iphc_header_1,
                                                               &buf,
                                                               &mut offset);

        // Decompress hop limit field
        self.decompress_hl(&mut ip6_header, iphc_header_1, &buf, &mut offset)?;

        // Decompress source address
        self.decompress_src(&mut ip6_header, iphc_header_2,
                            &src_mac_addr, &src_ctx, &buf, &mut offset)?;

        // Decompress destination address
        if (iphc_header_2 & iphc::MULTICAST) != 0 {
            self.decompress_multicast(&mut ip6_header, iphc_header_2, &dst_ctx,
                                      &buf, &mut offset)?;
        } else {
            self.decompress_dst(&mut ip6_header, iphc_header_2,
                                &dst_mac_addr, &dst_ctx, &buf, &mut offset)?;
        }

        // Note that next_header is already set only if is_nhc is false
        if is_nhc {
            next_header = nhc_to_ip6_nh(buf[offset]).ok_or(())?;
        }
        ip6_header.set_next_header(next_header);
        // While the next header is still compressed
        // Note that at each iteration, offset points to the NHC header field
        // and next_header refers to the type of this field.
        while is_nhc {
            // Advance past the LoWPAN NHC byte
            let nhc_header = buf[offset];
            offset += 1;

            // Scoped mutable borrow of out_buf
            let mut next_headers: &mut [u8] = &mut out_buf[bytes_written..];

            match next_header {
                ip6_nh::IP6 => {
                    let (encap_written, encap_processed) =
                        self.decompress(&buf[offset..],
                                        src_mac_addr,
                                        dst_mac_addr,
                                        &mut next_headers[bytes_written..])?;
                    bytes_written += encap_written;
                    offset += encap_processed;
                    break;
                },
                ip6_nh::UDP => {
                    // UDP length includes UDP header and data in bytes
                    let udp_length = (8 + (buf.len() - offset)) as u16;
                    // Decompress UDP header fields
                    let (src_port, dst_port) =
                        self.decompress_udp_ports(nhc_header,
                                                  &buf,
                                                  &mut offset);
                    // Fill in uncompressed UDP header
                    u16_to_slice(htons(src_port), &mut next_headers[0..2]);
                    u16_to_slice(htons(dst_port), &mut next_headers[2..4]);
                    u16_to_slice(htons(udp_length), &mut next_headers[4..6]);
                    u16_to_slice(0, &mut next_headers[6..8]); // Zero out checksum
                    // Need to fill in header values before computing the checksum
                    let udp_checksum =
                        self.decompress_udp_checksum(nhc_header,
                                                     &next_headers[0..8],
                                                     &ip6_header,
                                                     &buf,
                                                     &mut offset);
                    u16_to_slice(htons(udp_checksum), &mut next_headers[6..8]);

                    bytes_written += 8;
                    break;
                },
                ip6_nh::FRAGMENT
                | ip6_nh::HOP_OPTS
                | ip6_nh::ROUTING
                | ip6_nh::DST_OPTS
                | ip6_nh::MOBILITY => {
                    // True if the next header is also compressed
                    is_nhc = (nhc_header & nhc::NH) != 0;

                    // len is the number of octets following the length field
                    let len = buf[offset] as usize;
                    offset += 1;

                    // Check that there is a next header in the buffer,
                    // which must be the case if the last next header specifies
                    // NH = 1
                    if offset + len >= buf.len() {
                        return Err(());
                    }

                    // Length in 8-octet units after the first 8 octets
                    // (per the IPv6 ext hdr spec)
                    let mut hdr_len_field = (len - 6) / 8;
                    if (len - 6) % 8 != 0 {
                        hdr_len_field += 1;
                    }

                    // Gets the type of the subsequent next header.  If is_nhc
                    // is true, there must be a LoWPAN NHC header byte,
                    // otherwise there is either an uncompressed next header.
                    next_header = if is_nhc {
                        // The next header is LoWPAN NHC-compressed
                        nhc_to_ip6_nh(buf[offset + len]).ok_or(())?
                    } else {
                        // The next header is uncompressed
                        buf[offset + len]
                    };

                    // Fill in the extended header in uncompressed IPv6 format
                    next_headers[0] = next_header;
                    next_headers[1] = hdr_len_field as u8;
                    // Copies over the remaining options.
                    next_headers[2..2 + len]
                        .copy_from_slice(&buf[offset..offset + len]);

                    // Fill in padding
                    let pad_bytes = hdr_len_field * 8 - len + 6;
                    if pad_bytes == 1 {
                        // Pad1
                        next_headers[2 + len] = 0;
                    } else {
                        // PadN, 2 <= pad_bytes <= 7
                        next_headers[2 + len] = 1;
                        next_headers[2 + len + 1] = pad_bytes as u8 - 2;
                        for i in 2..pad_bytes {
                            next_headers[2 + len + i] = 0;
                        }
                    }

                    bytes_written += 8 + hdr_len_field * 8;
                    offset += len;
                },
                _ => {
                    // This should be unreachable
                    return Err(());
                },
            }
        }

        // IPv6 header length field is the size of the IPv6 payload, including
        // extension headers. This will be the number of bytes not consumed from
        // the raw LoWPAN-compresed buffer + the number of bytes written that
        // were not the IPv6 header
        let total_len = (buf.len() - offset)
                        + (bytes_written - mem::size_of::<IP6Header>());
        ip6_header.payload_len = ip::htons(total_len as u16);
        Ok((bytes_written, offset))
    }

    fn decompress_cie(&self,
                      iphc_header: u8,
                      buf: &[u8],
                      offset: &mut usize) -> Result<(Context, Context), ()> {
        let mut sci = 0;
        let mut dci = 0;
        if iphc_header & iphc::CID != 0 {
            sci = buf[*offset] >> 4;
            dci = buf[*offset] & 0xf;
            *offset += 1;
        }
        // Since context with id 0 must *always* exist, we can unwrap it
        // directly.
        return Ok((self.ctx_store.get_context_from_id(sci).ok_or(())?,
                   self.ctx_store.get_context_from_id(dci).ok_or(())?));
    }

    // TODO: Check
    fn decompress_tf(&self,
                     ip6_header: &mut IP6Header,
                     iphc_header: u8,
                     buf: &[u8],
                     offset: &mut usize) {
        let fl_compressed = (iphc_header & iphc::TF_FLOW_LABEL) != 0;
        let tc_compressed = (iphc_header & iphc::TF_TRAFFIC_CLASS) != 0;

        let mut ecn: u8 = 0;
        let mut dscp: u8 = 0;

        // Determine ECN and DSCP separately, they are in a different order.
        if fl_compressed || tc_compressed {
            ecn = buf[*offset] >> 6;
        }
        if !tc_compressed {
            dscp = buf[*offset] & 0b111111;
            *offset += 1;
        }
        ip6_header.set_ecn(ecn);
        ip6_header.set_dscp(dscp);

        // Flow label is always in the same bit position relative to the last
        // three bytes in the inline fields
        if fl_compressed {
            ip6_header.set_flow_label(0);
        } else {
            let flow = (((buf[*offset] & 0x0f) as u32) << 16)
                       | ((buf[*offset+1] as u32) << 8)
                       | (buf[*offset+2] as u32);
            *offset += 3;
            ip6_header.set_flow_label(flow);
        }
    }

    fn decompress_nh(&self,
                     iphc_header: u8,
                     buf: &[u8],
                     offset: &mut usize) -> (bool, u8) {
        let is_nhc = (iphc_header & iphc::NH) != 0;
        let mut next_header: u8 = 0;
        if !is_nhc {
            next_header = buf[*offset];
            *offset += 1;
        }
        return (is_nhc, next_header);
    }

    fn decompress_hl(&self, 
                     ip6_header: &mut IP6Header,
                     iphc_header: u8,
                     buf: &[u8],
                     offset: &mut usize) -> Result<(), ()> {
        // TODO: Does this match work?
        let hop_limit = match iphc_header & iphc::HLIM_MASK {
            iphc::HLIM_1      => 1,
            iphc::HLIM_64     => 64,
            iphc::HLIM_255    => 255,
            iphc::HLIM_INLINE => {
                *offset +=1;
                buf[*offset-1]
            },
            // This case is unreachable
            _                 => {
                return Err(());
            },
        };
        ip6_header.set_hop_limit(hop_limit);
        Ok(())
    }

    fn decompress_src(&self,
                      ip6_header: &mut IP6Header,
                      iphc_header: u8,
                      mac_addr: &MacAddr,
                      ctx: &Context, // Must be non-null
                      buf: &[u8],
                      offset: &mut usize) -> Result<(), ()> {
        let uses_context = (iphc_header & iphc::SAC) != 0;
        let sam_mode = iphc_header & iphc::SAM_MASK;
        // The UNSPECIFIED address ::
        if uses_context && sam_mode == iphc::SAM_INLINE {
            // The default src_addr is already unspecified
        } else if uses_context {
            self.decompress_iid_context(sam_mode,
                                        &mut ip6_header.src_addr,
                                        mac_addr,
                                        ctx,
                                        buf,
                                        offset)?;
        } else {
            self.decompress_iid_no_context(sam_mode,
                                           &mut ip6_header.src_addr,
                                           mac_addr,
                                           buf,
                                           offset)?;
        }
        Ok(())
    }

    fn decompress_dst(&self,
                      ip6_header: &mut IP6Header,
                      iphc_header: u8,
                      mac_addr: &MacAddr,
                      ctx: &Context, // Must be non-null
                      buf: &[u8],
                      offset: &mut usize) -> Result<(), ()> {
        let uses_context = (iphc_header & iphc::DAC) != 0;
        let dam_mode = iphc_header & iphc::DAM_MASK;
        if uses_context && dam_mode == iphc::DAM_INLINE {
            // This is a reserved address
            return Err(());
        } else if uses_context {
            self.decompress_iid_context(dam_mode,
                                        &mut ip6_header.dst_addr,
                                        mac_addr,
                                        ctx,
                                        buf,
                                        offset)?;
        } else {
            self.decompress_iid_no_context(dam_mode,
                                           &mut ip6_header.dst_addr,
                                           mac_addr,
                                           buf,
                                           offset)?;
        }
        Ok(())
    }

    fn decompress_multicast(&self,
                            ip6_header: &mut IP6Header,
                            iphc_header: u8,
                            ctx: &Context,
                            buf: &[u8],
                            offset: &mut usize) -> Result<(), ()> {
        let uses_context = (iphc_header & iphc::DAC) != 0;
        let dam_mode = iphc_header & iphc::DAM_MASK;
        let mut ip_addr: &mut IPAddr = &mut ip6_header.dst_addr;
        if uses_context {
            match dam_mode {
                iphc::DAM_INLINE => {
                    // ffXX:XX + plen + pfx64 + XXXX:XXXX
                    // We want to copy over at most 8 bytes
                    let prefix_bytes = min(((ctx.prefix_len + 7) / 8) as usize, 8);
                    ip_addr.0[0] = 0xff;
                    ip_addr.0[1] = buf[*offset];
                    ip_addr.0[2] = buf[*offset+1];
                    ip_addr.0[3] = ctx.prefix_len;
                    // Zero out memory so that if prefix_bytes < 8, the prefix
                    // is zero-padded
                    ip_addr.0[4..12].copy_from_slice(&[0; 8]);
                    ip_addr.0[4..4 + prefix_bytes]
                        .copy_from_slice(&ctx.prefix[0..prefix_bytes]);
                    ip_addr.0[12..16].copy_from_slice(&buf[*offset + 2..*offset + 4]);
                    *offset += 6;
                },
                _ => {
                    // No other options supported
                    return Err(());
                },
            }
        } else {
            match dam_mode {
                // Full multicast address carried inline
                iphc::DAM_INLINE => {
                    ip_addr.0.copy_from_slice(&buf[*offset..*offset + 16]);
                    *offset += 16;
                },
                // ffXX::00XX:XXXX:XXXX
                iphc::DAM_MODE1  => {
                    ip_addr.0[0] = 0xff;
                    ip_addr.0[1] = buf[*offset];
                    *offset += 1;
                    ip_addr.0[11..16].copy_from_slice(&buf[*offset..*offset + 5]);
                    *offset += 5;
                },
                // ffXX::00XX:XXXX
                iphc::DAM_MODE2  => {
                    ip_addr.0[0] = 0xff;
                    ip_addr.0[1] = buf[*offset];
                    *offset += 1;
                    ip_addr.0[13..16].copy_from_slice(&buf[*offset..*offset + 3]);
                    *offset += 3;
                },
                // ff02::00XX
                iphc::DAM_MODE3  => {
                    ip_addr.0[0] = 0xff;
                    ip_addr.0[1] = 0x02;
                    ip_addr.0[15] = buf[*offset];
                    *offset += 1;
                },
                _ => {
                    // Unreachable error case
                    return Err(());
                },
            }
        }
        Ok(())
    }

    fn decompress_iid_no_context(&self,
                                 addr_mode: u8,
                                 ip_addr: &mut IPAddr,
                                 mac_addr: &MacAddr,
                                 buf: &[u8],
                                 offset: &mut usize) -> Result<(), ()> {
        let mode = addr_mode & (iphc::SAM_MASK | iphc::DAM_MASK);
        match mode {
            // SAM = 00, DAM = 00; address carried inline
            iphc::SAM_INLINE /* | iphc::DAM_INLINE */ => {
                ip_addr.0.copy_from_slice(&buf);
                *offset += 16;
            },
            // First 64-bits link local prefix, remaining 64 bits carried inline
            iphc::SAM_MODE1 | iphc::DAM_MODE1 => {
                ip_addr.set_unicast_link_local();
                ip_addr.0[8..16].copy_from_slice(&buf[*offset..*offset + 8]);
                *offset += 8;
            },
            // First 112 bits elided; First 64 bits link-local prefix, remaining
            // 64 bits are 0000:00ff:fe00:XXXX
            iphc::SAM_MODE2 | iphc::DAM_MODE2 => {
                ip_addr.set_unicast_link_local();
                ip_addr.0[8..16].copy_from_slice(&iphc::MAC_BASE);
                ip_addr.0[14..16].copy_from_slice(&buf[*offset..*offset + 2]);
                *offset += 2;
            },
            // Address fully elided. First 64 bits link-local prefix, remaining
            // 64 bits computed from encapsulating header.
            iphc::SAM_MODE3 | iphc::DAM_MODE3 => {
                ip_addr.set_unicast_link_local();
                ip_addr.0[8..16].copy_from_slice(&compute_iid(mac_addr));
            },
            // Unreachable error case
            _ => { 
                return Err(());
            },
        }
        Ok(())
    }

    fn decompress_iid_context(&self,
                              addr_mode: u8,
                              ip_addr: &mut IPAddr,
                              mac_addr: &MacAddr,
                              ctx: &Context, // Must be non-null
                              buf: &[u8],
                              offset: &mut usize) -> Result<(), ()> {
        let mode = addr_mode & (iphc::SAM_MASK | iphc::DAM_MASK);
        match mode {
            // SAM = 00, DAM = 00; address equals :: or reserved
            iphc::SAM_INLINE /* | iphc::DAM_INLINE */ => {
                // This case should be handled separately by the callers,
                // as the behavior differs between source and destination
                // addresses
                return Err(());
            },
            // 64 bits; context information always used
            iphc::SAM_MODE1 | iphc::DAM_MODE1 => {
                ip_addr.0[8..16].copy_from_slice(&buf[*offset..*offset+8]);
                *offset += 8;
            },
            // 16 bits inline; any IID bits not covered by context are taken
            // from IID mapping
            iphc::SAM_MODE2 | iphc::DAM_MODE2 => {
                ip_addr.0[8..16].copy_from_slice(&iphc::MAC_BASE);
                ip_addr.0[14..16].copy_from_slice(&buf[*offset..*offset+2]);
                *offset += 2;
            },
            // Address elided, derived using context + encapsulating header
            iphc::SAM_MODE3 | iphc::DAM_MODE3 => {
                let iid = compute_iid(mac_addr);
                ip_addr.0[8..16].copy_from_slice(&iid[0..8]);
            },
            // Unrechable; error case
            _ => {
                return Err(());
            },
        }
        // Note that we copy the non-context bits into the ip_addr first, as
        // we must always use the context bits
        set_ctx_bits_in_addr(ip_addr, ctx);
        Ok(())
    }

    fn decompress_udp_ports(&self,
                            udp_nhc: u8,
                            buf: &[u8],
                            offset: &mut usize) -> (u16, u16) {
        let src_compressed = (udp_nhc & nhc::UDP_SRC_PORT_FLAG) != 0;
        let dst_compressed = (udp_nhc & nhc::UDP_DST_PORT_FLAG) != 0;

        let src_port: u16;
        let dst_port: u16;
        if src_compressed && dst_compressed {
            // Both src and dst are compressed to 4 bits
            let src_short = ((buf[*offset] >> 4) & 0xf) as u16;
            let dst_short = (buf[*offset] & 0xf) as u16;
            src_port = nhc::UDP_4BIT_PORT | src_short;
            dst_port = nhc::UDP_4BIT_PORT | dst_short;
            *offset += 1;
        } else if src_compressed {
            // Source port is compressed to 8 bits
            src_port = nhc::UDP_8BIT_PORT | (buf[*offset] as u16);
            // Destination port is uncompressed
            dst_port = ntohs(slice_to_u16(&buf[*offset + 1..*offset + 3]));
            *offset += 3;
        } else if dst_compressed {
            // Source port is uncompressed
            src_port = ntohs(slice_to_u16(&buf[*offset..*offset + 2]));
            // Destination port is compressed to 8 bits
            dst_port = nhc::UDP_8BIT_PORT | (buf[*offset + 2] as u16);
            *offset += 3;
        } else {
            // Both ports are uncompressed
            src_port = ntohs(slice_to_u16(&buf[*offset..*offset + 2]));
            dst_port = ntohs(slice_to_u16(&buf[*offset + 2..*offset + 4]));
            *offset += 4;
        }
        (src_port, dst_port)
    }

    // Returns the checksum in network-byte order
    fn decompress_udp_checksum(&self,
                               udp_nhc: u8,
                               udp_header: &[u8],
                               ip6_header: &IP6Header,
                               buf: &[u8],
                               offset: &mut usize) -> u16{
        if (udp_nhc & nhc::UDP_CHECKSUM_FLAG) != 0 {
            // TODO: Need to verify that the packet was sent with *some* kind
            // of integrity check at a lower level (otherwise, we need to drop
            // the packet)
            compute_udp_checksum(ip6_header, udp_header, &buf[*offset..])
        } else {
            let checksum = slice_to_u16(&buf[*offset..*offset + 2]);
            *offset += 2;
            checksum
        }
    }
}
