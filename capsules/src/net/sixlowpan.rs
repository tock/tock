/// Implements the 6LoWPAN specification for sending IPv6 datagrams over
/// 802.15.4 packets efficiently, as detailed in RFC 6282.

use core::mem;
use core::result::Result;

use net::ip;
use net::ip::{IP6Header, MacAddr, IPAddr, ip6_nh};
use net::util;

#[allow(unused_variables,dead_code)]
mod iphc {
    pub const DISPATCH: [u8; 2] = [0x60, 0x00];

    // First byte masks

    pub const TF_MASK: u8 = 0x18;
    pub const TF_TRAFFIC_CLASS: u8 = 0x08;
    pub const TF_FLOW_LABEL: u8 = 0x10;

    pub const NH: u8 = 0x04;

    pub const HLIM_MASK: u8 = 0x03;
    pub const HLIM_INLINE: u8 = 0x00;
    pub const HLIM_1: u8 = 0x01;
    pub const HLIM_64: u8 = 0x02;
    pub const HLIM_255: u8 = 0x03;

    // Second byte masks

    pub const CID: u8 = 0x80;

    pub const SAC: u8 = 0x40;

    pub const SAM_MASK: u8 = 0x30;
    pub const SAM_INLINE: u8 = 0x00;
    pub const SAM_MODE1: u8 = 0x10;
    pub const SAM_MODE2: u8 = 0x20;
    pub const SAM_MODE3: u8 = 0x30;

    pub const MULTICAST: u8 = 0x01;

    pub const DAC: u8 = 0x04;
    pub const DAM_MASK: u8 = 0x03;
    pub const DAM_INLINE: u8 = 0x00;
    pub const DAM_MODE1: u8 = 0x01;
    pub const DAM_MODE2: u8 = 0x02;
    pub const DAM_MODE3: u8 = 0x03;

    // Address compression
    pub const MAC_BASE: [u8; 8] = [0, 0, 0, 0xff, 0xfe, 0, 0, 0];
    pub const MAC_UL: u8 = 0x02;
}

#[allow(unused_variables,dead_code)]
mod nhc {
    pub const DISPATCH_NHC: u8 = 0xe0;
    pub const DISPATCH_UDP: u8 = 0xf8;

    pub const HOP_OPTS: u8     = 0 << 1;
    pub const ROUTING: u8      = 1 << 1;
    pub const FRAGMENT: u8     = 2 << 1;
    pub const DST_OPTS: u8     = 3 << 1;
    pub const MOBILITY: u8     = 4 << 1;
    pub const IP6: u8          = 7 << 1;

    pub const NH: u8           = 0x01;

    pub const UDP_PORT_PREFIX: u16 = 0xf0b0;
    pub const UDP_SHORT_PORT_MASK: u16 = 0xf;
    pub const UDP_PORT_MASK: u16 = 0xff;
    pub const UDP_SRC_PORT_FLAG: u8 = 0b10;
    pub const UDP_DST_PORT_FLAG: u8 = 0b1;
    pub const UDP_CHKSUM_FLAG: u8 = 0b100;
}

#[allow(unused_variables,dead_code)]
pub struct Context<'a> {
    prefix: &'a [u8],
    prefix_len: u8,
    id: u8,
    compress: bool,
}

pub trait ContextStore<'a> {
    fn get_context_from_addr(&self, ip_addr: IPAddr) -> Option<Context<'a>>;
    fn get_context_from_id(&self, ctx_id: u8) -> Option<Context<'a>>;
    fn get_context_from_prefix(&self, prefix: &[u8], prefix_len: u8) -> Option<Context<'a>>;
}

#[allow(unused_variables,dead_code)]
pub struct DummyStore {}

#[allow(unused_variables,dead_code)]
impl<'a> ContextStore<'a> for DummyStore {
    // TODO: Implement these.
    // These methods should also include context 0 (the mesh-local prefix) as
    // one of the possible options

    fn get_context_from_addr(&self, ip_addr: IPAddr) -> Option<Context<'a>> {
        None
    }

    fn get_context_from_id(&self, ctx_id: u8) -> Option<Context<'a>> {
        None
    }

    fn get_context_from_prefix(&self, prefix: &[u8], prefix_len: u8) -> Option<Context<'a>> {
        None
    }
}

#[allow(unused_variables,dead_code)]
pub struct FragInfo {
    dummy: u8,
}

/// Computes the LoWPAN Interface Identifier from either the 16-bit short MAC or
/// the IEEE EUI-64 that is derived from the 48-bit MAC.
fn compute_iid(mac_addr: &MacAddr) -> [u8; 8] {
    match mac_addr {
        &MacAddr::ShortAddr(short_addr) => {
            // IID is 0000:00ff:fe00:XXXX, where XXXX is 16-bit MAC
            let mut iid: [u8; 8] = iphc::MAC_BASE;
            iid[6] = (short_addr >> 1) as u8;
            iid[7] = (short_addr & 0xff) as u8;
            iid
        }
        &MacAddr::LongAddr(long_addr) => {
            // IID is IEEE EUI-64 with universal/local bit inverted
            let mut iid: [u8; 8] = long_addr;
            iid[0] ^= iphc::MAC_UL;
            iid
        }
    }
}

fn is_ip6_nh_compressible(next_header: u8,
                          next_headers: &[u8],
                          nh_offset: usize) -> Result<(bool, u8), ()> {
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
                if next_headers.len() < nh_offset + 2 {
                    return Err(());
                } else {
                    header_len += (next_headers[nh_offset + 1] as u32) * 8;
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
fn nhc_eid_to_ip6_nh(eid: u8) -> Option<u8> {
    match eid {
        nhc::HOP_OPTS => Some(ip6_nh::HOP_OPTS),
        nhc::ROUTING  => Some(ip6_nh::ROUTING),
        nhc::FRAGMENT => Some(ip6_nh::FRAGMENT),
        nhc::DST_OPTS => Some(ip6_nh::DST_OPTS),
        nhc::MOBILITY => Some(ip6_nh::MOBILITY),
        nhc::IP6      => Some(ip6_nh::IP6),
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

    /// Constructs a 6LoWPAN header in `buf` from the given IPv6 header and
    /// 16-bit MAC addresses.  Returns the number of bytes written into `buf`.
    pub fn compress(&self,
                    ip6_header: &IP6Header,
                    next_headers: &[u8],
                    src_mac_addr: MacAddr,
                    dst_mac_addr: MacAddr,
                    mut buf: &mut [u8])
                    -> Result<usize, ()> {
        // The first two bytes are the LOWPAN_IPHC header
        let mut offset: usize = 2;

        // Initialize the LOWPAN_IPHC header
        buf[0..2].copy_from_slice(&iphc::DISPATCH);

        let mut src_ctx: Option<Context> = self.ctx_store
            .get_context_from_addr(ip6_header.src_addr);
        let mut dst_ctx: Option<Context> = if ip::addr_is_multicast(&ip6_header.dst_addr) {
            let prefix_len: u8 = ip6_header.dst_addr[3];
            let prefix: &[u8] = &ip6_header.dst_addr[4..12];
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
            is_ip6_nh_compressible(ip6_header.next_header, next_headers, 0)?;
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
        if ip::addr_is_multicast(&ip6_header.dst_addr) {
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
        let mut nh_offset: usize = 0;
        let mut ip6_nh_type: u8 = ip6_header.next_header;
        while is_nhc {
            match ip6_nh_type {
                ip6_nh::IP6 => {
                    // For IPv6 encapsulation, the NH bit in the NHC ID is 0
                    let nhc_header = nhc::DISPATCH_NHC | nhc::IP6;
                    buf[offset] = nhc_header;
                    offset += 1;

                    // Recursively place IPHC-encoded IPv6 after the NHC ID
                    let ip6_header: &IP6Header = unsafe {
                        mem::transmute(&next_headers[nh_offset..].as_ptr())
                    };
                    offset += self.compress(
                        ip6_header,
                        &next_headers[nh_offset + mem::size_of::<IP6Header>()..],
                        src_mac_addr,
                        dst_mac_addr,
                        &mut buf)?;

                    // The above recursion handles the rest of the packet
                    break;
                },
                ip6_nh::UDP => {
                    let mut nhc_header = nhc::DISPATCH_UDP;
                    // Keep this so we know where the UDP nh is
                    let udp_header_offset = offset;
                    // TODO: Make this a macro/function
                    let udp_packet_len =
                        (next_headers[nh_offset + 5] as u16
                        | ((next_headers[nh_offset + 6] as u16) << 8));
                    // TODO: Check if length is valid, should be minus one?
                    let udp_packet =
                        &next_headers[nh_offset..(udp_packet_len-1) as usize];
                    nhc_header |= self.compress_udp_ports(udp_packet,
                                                          &mut buf,
                                                          &mut offset);
                    // TODO: Checksum elision is currently not supported
                    nhc_header |= self.compress_udp_chksum(udp_packet,
                                                           udp_packet_len,
                                                           &mut buf,
                                                           &mut offset);
                    buf[udp_header_offset] = nhc_header;
                    buf[offset..offset + (udp_packet_len - 1) as usize]
                        .copy_from_slice(udp_packet);
                    offset += udp_packet_len as usize;

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
                    let next_nh_offset = nh_offset + 2 + (nh_len as usize);

                    // Determine if the next header is compressible
                    let (next_is_nhc, next_nh_len) =
                        is_ip6_nh_compressible(next_headers[nh_offset],
                                               next_headers,
                                               next_nh_offset)?;
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
                        buf[offset] = next_headers[nh_offset + 2 + (i as usize)];
                        offset += 1;
                    }

                    ip6_nh_type = next_headers[nh_offset];
                    is_nhc = next_is_nhc;
                    nh_len = next_nh_len;
                    nh_offset = next_nh_offset;
                },
                _ => {
                    // This case should not be reached, since is_nh_compressed
                    // is set by is_ip6_nh_compressible
                    return Err(());
                },
            }
        }

        Ok(offset)
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
        // TODO: All of this needs to be checked for endian-ness and correctness
        // let version = ip6_header.version_class_flow[0] >> 4;
        let class = ((ip6_header.version_class_flow[0] << 4) & 0xf0) |
                    ((ip6_header.version_class_flow[1] >> 4) & 0x0f);
        let ecn = (class >> 6) & 0b11; // Gets leading 2 bits
        let dscp = class & 0b111111; // Gets trailing 6 bits
        let mut flow: [u8; 3] = [0; 3];
        flow[0] = ip6_header.version_class_flow[1] & 0x0f; // Zero upper 4 bits
        flow[1] = ip6_header.version_class_flow[2];
        flow[2] = ip6_header.version_class_flow[3];

        let mut tf_encoding = 0;

        // Flow label is all zeroes and can be elided
        if flow[0] == 0 && flow[1] == 0 && flow[2] == 0 {
            // The 1X cases
            tf_encoding |= iphc::TF_FLOW_LABEL;
        }

        // DSCP can be elided, but ECN elided only if flow also elided
        // X1 cases
        if dscp == 0 {
            // If flow *not* elided, combine with ECN
            // 01 case
            if tf_encoding == 0 {
                buf[*offset] = (ecn << 6) | flow[0];
                buf[*offset + 1] = flow[1];
                buf[*offset + 2] = flow[2];
                *offset += 3;
            }
            tf_encoding |= iphc::TF_TRAFFIC_CLASS;
            // X0 cases
        } else {
            // If DSCP cannot be elided
            buf[*offset] = class;
            *offset += 1;

            // 00 case
            if tf_encoding == 0 {
                buf[*offset] = flow[0];
                buf[*offset + 1] = flow[1];
                buf[*offset + 2] = flow[2];
                *offset += 3;
            }
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
        if ip::addr_is_unspecified(src_ip_addr) {
            // SAC = 1, SAM = 00
            buf[1] |= iphc::SAC;
        } else if ip::addr_is_link_local(src_ip_addr) {
            // SAC = 0, SAM = 01, 10, 11
            self.compress_iid(src_ip_addr, src_mac_addr, true, buf, offset);
        } else if src_ctx.is_some() {
            // SAC = 1, SAM = 01, 10, 11
            buf[1] |= iphc::SAC;
            self.compress_iid(src_ip_addr, src_mac_addr, true, buf, offset);
        } else {
            // SAC = 0, SAM = 00
            buf[*offset..*offset + 16].copy_from_slice(src_ip_addr);
            *offset += 16;
        }
    }

    fn compress_iid(&self,
                    ip_addr: &IPAddr,
                    mac_addr: &MacAddr,
                    is_src: bool,
                    buf: &mut [u8],
                    offset: &mut usize) {
        let iid: [u8; 8] = compute_iid(mac_addr);
        if ip_addr[8..16] == iid {
            // SAM/DAM = 11, 0 bits
            buf[1] |= if is_src {
                iphc::SAM_MODE3
            } else {
                iphc::DAM_MODE3
            };
        } else if ip_addr[8..14] == iphc::MAC_BASE[0..6] {
            // SAM/DAM = 10, 16 bits
            buf[1] |= if is_src {
                iphc::SAM_MODE2
            } else {
                iphc::DAM_MODE2
            };
            buf[*offset..*offset + 2].copy_from_slice(&ip_addr[14..16]);
            *offset += 2;
        } else {
            // SAM/DAM = 01, 64 bits
            buf[1] |= if is_src {
                iphc::SAM_MODE1
            } else {
                iphc::DAM_MODE1
            };
            buf[*offset..*offset + 8].copy_from_slice(&ip_addr[8..16]);
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
        if ip::addr_is_link_local(dst_ip_addr) {
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
            buf[*offset..*offset + 16].copy_from_slice(dst_ip_addr);
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
            buf[*offset..*offset + 2].copy_from_slice(&dst_ip_addr[1..3]);
            buf[*offset + 2..*offset + 6].copy_from_slice(&dst_ip_addr[12..16]);
            *offset += 6;
        } else {
            // M = 1, DAC = 0
            if dst_ip_addr[1] == 0x02 && util::is_zero(&dst_ip_addr[2..15]) {
                // DAM = 11
                buf[1] |= iphc::DAM_MODE3;
                buf[*offset] = dst_ip_addr[15];
                *offset += 1;
            } else {
                if !util::is_zero(&dst_ip_addr[2..11]) {
                    // DAM = 00
                    buf[1] |= iphc::DAM_INLINE;
                    buf[*offset..*offset + 16].copy_from_slice(dst_ip_addr);
                    *offset += 16;
                } else if !util::is_zero(&dst_ip_addr[11..13]) {
                    // DAM = 01, ffXX::00XX:XXXX:XXXX
                    buf[1] |= iphc::DAM_MODE1;
                    buf[*offset] = dst_ip_addr[1];
                    buf[*offset + 1..*offset + 6].copy_from_slice(&dst_ip_addr[11..16]);
                    *offset += 6;
                } else {
                    // DAM = 10, ffXX::00XX:XXXX
                    buf[1] |= iphc::DAM_MODE2;
                    buf[*offset] = dst_ip_addr[1];
                    buf[*offset + 1..*offset + 4].copy_from_slice(&dst_ip_addr[13..16]);
                    *offset += 4;
                }
            }
        }
    }

    fn compress_udp_ports(&self, 
                          udp_header: &[u8],
                          buf: &mut [u8],
                          offset: &mut usize) -> u8 {
        // Little endian conversion
        // TODO: Make macro?
        let src_port: u16 = udp_header[0] as u16 | (udp_header[1] as u16) << 8;
        let dst_port: u16 = udp_header[2] as u16 | (udp_header[3] as u16) << 8;

        let mut udp_port_nhc = 0;
        if (src_port & !nhc::UDP_SHORT_PORT_MASK) == nhc::UDP_PORT_PREFIX
            && (dst_port & !nhc::UDP_SHORT_PORT_MASK) == nhc::UDP_PORT_PREFIX {
            // Both can be compressed to 4 bits
            udp_port_nhc |= (nhc::UDP_SRC_PORT_FLAG | nhc::UDP_DST_PORT_FLAG);
            // This should compress the ports to a single 8-bit value,
            // with the source port before the destination port
            let short_ports: u8 = ((src_port & 0xf) | ((dst_port >> 4) & 0xf0)) as u8;
            buf[*offset] = short_ports;
            *offset += 1;
        } else if (src_port & !nhc::UDP_PORT_MASK) == nhc::UDP_PORT_PREFIX {
            // Source port compressed to 8 bits, destination port uncompressed
            udp_port_nhc |= (nhc::UDP_SRC_PORT_FLAG);
            buf[*offset] = udp_header[0];
            buf[*offset+1] = udp_header[2];
            buf[*offset+2] = udp_header[3];
            *offset += 3;
        } else if (dst_port & !nhc::UDP_PORT_MASK) == nhc::UDP_PORT_PREFIX {
            udp_port_nhc |= (nhc::UDP_DST_PORT_FLAG);
            buf[*offset] = udp_header[0];
            buf[*offset+1] = udp_header[1];
            buf[*offset+2] = udp_header[2];
            *offset += 3;
        } else {
            buf[*offset] = udp_header[0];
            buf[*offset+1] = udp_header[1];
            buf[*offset+2] = udp_header[2];
            buf[*offset+3] = udp_header[3];
            *offset += 4;
        }
        return udp_port_nhc;
    }

    fn compress_udp_chksum(&self,
                           udp_packet: &[u8],
                           packet_len: u16,
                           buf: &mut [u8],
                           offset: &mut usize) -> u8 {
        // TODO: As with the reference implementations, we currently
        // do not support eliding the UDP checksum.
        buf[*offset] = udp_packet[6];
        buf[*offset+1] = udp_packet[7];
        *offset += 2;
        // Since the UDP checksum compression is not implemented, the flag is 0
        return 0;
    }

    /// Decodes the compressed header into a full IPv6 header given the 16-bit
    /// MAC addresses. `buf` is expected to be a slice starting from the
    /// beginning of the IP header.  Returns the number of bytes taken up by the
    /// header, so the remaining bytes are the payload. Also returns an optional
    /// `FragInfo` containing the datagram tag and fragmentation offset if this
    /// packet is part of a set of fragments.
    #[allow(unused_variables,dead_code)]
    pub fn decompress(&self,
                      buf: &mut [u8],
                      src_mac_addr: MacAddr,
                      dst_mac_addr: MacAddr,
                      mesh_local_prefix: &[u8])
                      -> Result<(IP6Header, usize, Option<FragInfo>), ()> {
        Err(())
    }
}
