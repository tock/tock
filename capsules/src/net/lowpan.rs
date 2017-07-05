/// Implements the 6LoWPAN specification for sending IPv6 datagrams over
/// 802.15.4 packets efficiently, as detailed in RFC 6282.

use core::mem;
use core::result::Result;

use net::ip;
use net::ip::{IP6Header, MacAddr, IPAddr, ip6_nh};
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
    pub const DISPATCH_UDP: u8           = 0xf8;

    pub const HOP_OPTS: u8               = 0 << 1;
    pub const ROUTING: u8                = 1 << 1;
    pub const FRAGMENT: u8               = 2 << 1;
    pub const DST_OPTS: u8               = 3 << 1;
    pub const MOBILITY: u8               = 4 << 1;
    pub const IP6: u8                    = 7 << 1;

    pub const NH: u8                     = 0x01;

    pub const UDP_SHORT_PORT_PREFIX: u16 = 0xf0b0;
    pub const UDP_SHORT_PORT_MASK: u16   = 0xf;
    pub const UDP_PORT_PREFIX: u16       = 0xf000;
    pub const UDP_PORT_MASK: u16         = 0xff;
    pub const UDP_SRC_PORT_FLAG: u8      = 0b10;
    pub const UDP_DST_PORT_FLAG: u8      = 0b1;
    pub const UDP_CHKSUM_FLAG: u8        = 0b100;
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
        // Little endian conversion
        // TODO: Make macro? Also, think this is wrong order
        let src_port: u16 = udp_header[0] as u16 | (udp_header[1] as u16) << 8;
        let dst_port: u16 = udp_header[2] as u16 | (udp_header[3] as u16) << 8;

        let mut udp_port_nhc = 0;
        if (src_port & !nhc::UDP_SHORT_PORT_MASK) == nhc::UDP_SHORT_PORT_PREFIX
            && (dst_port & !nhc::UDP_SHORT_PORT_MASK) == nhc::UDP_SHORT_PORT_PREFIX {
            // Both can be compressed to 4 bits
            udp_port_nhc |= nhc::UDP_SRC_PORT_FLAG | nhc::UDP_DST_PORT_FLAG;
            // This should compress the ports to a single 8-bit value,
            // with the source port before the destination port
            let short_ports: u8 = ((src_port & 0xf) | ((dst_port >> 4) & 0xf0)) as u8;
            buf[*offset] = short_ports;
            *offset += 1;
        } else if (src_port & !nhc::UDP_PORT_MASK) == nhc::UDP_PORT_PREFIX {
            // Source port compressed to 8 bits, destination port uncompressed
            udp_port_nhc |= nhc::UDP_SRC_PORT_FLAG;
            buf[*offset..*offset + 3].copy_from_slice(&udp_header[0..3]);
            *offset += 3;
        } else if (dst_port & !nhc::UDP_PORT_MASK) == nhc::UDP_PORT_PREFIX {
            udp_port_nhc |= nhc::UDP_DST_PORT_FLAG;
            buf[*offset..*offset + 3].copy_from_slice(&udp_header[0..3]);
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
                      out_buf: &mut [u8])
                      -> Result<usize, ()> {
        // Get the LOWPAN_IPHC header (the first two bytes are the header)
        let iphc_header_1: u8 = buf[0];
        let iphc_header_2: u8 = buf[1];
        let mut offset: usize = 2;

        let mut ip6_header: &mut IP6Header = unsafe {
            mem::transmute(out_buf.as_mut_ptr())
        };
        let mut out_offset: usize = mem::size_of::<IP6Header>();
        let mut next_headers: &[u8] = &out_buf[out_offset..];
        *ip6_header = IP6Header::new();

        // Decompress CIE and get context
        let (sci,dci) = self.decompress_cie(iphc_header_1, &buf, &mut offset);

        // TODO: Proper error messages
        // Note that, since context with id 0 must *always* exist, we can unwrap
        // it directly.
        let src_context = self.ctx_store.get_context_from_id(sci).ok_or(())?;
        let dst_context = self.ctx_store.get_context_from_id(dci).ok_or(())?;

        // Traffic Class & Flow Label
        self.decompress_tf(&mut ip6_header, iphc_header_1, &buf, &mut offset);

        // Next header
        self.decompress_nh(iphc_header_1);

        // Decompress hop limit field
        self.decompress_hl(&mut ip6_header, iphc_header_1, &buf, &mut offset);

        // Decompress source address
        self.decompress_src(&mut ip6_header, iphc_header_2,
                            &src_mac_addr, &src_context, &buf, &mut offset);

        // Decompress destination address
        if (iphc_header_2 & iphc::MULTICAST) != 0 {
            self.decompress_multicast(&mut ip6_header, iphc_header_2, &dst_context,
                                      &buf, &mut offset);
        } else {
            self.decompress_dst(&mut ip6_header, iphc_header_2,
                                &dst_mac_addr, &dst_context, &buf, &mut offset);
        }

        // TODO: Next headers

        Err(())
    }

    fn decompress_cie(&self, 
                      iphc_header: u8, 
                      buf: &[u8], 
                      offset: &mut usize) -> (u8, u8) {
        let mut sci = 0;
        let mut dci = 0;
        if iphc_header & iphc::CID != 0 {
            let sci = buf[*offset] >> 4;
            let dci = buf[*offset] & 0xf;
            *offset += 1;
        }
        return (sci, dci);
    }

    // TODO: Check
    fn decompress_tf(&self,
                     ip6_header: &mut IP6Header,
                     iphc_header: u8,
                     buf: &[u8],
                     offset: &mut usize) {
        let fl_compressed = (iphc_header & iphc::TF_FLOW_LABEL) != 0;
        let tc_compressed = (iphc_header & iphc::TF_TRAFFIC_CLASS) != 0;

        // Both traffic class and flow label elided, must be zero
        if fl_compressed && tc_compressed {
            ip6_header.set_traffic_class(0);
            ip6_header.set_flow_label(0);
        // Only flow label compressed (10 case)
        } else if fl_compressed {
            ip6_header.set_flow_label(0);
            // Traffic Class = ECN+DSCP
            let traffic_class = buf[*offset];
            ip6_header.set_traffic_class(traffic_class);
            *offset += 1;
        // Only traffic class compressed (01 case)
        } else if tc_compressed {
            // ECN is the lower two bits of the first byte
            let ecn = buf[*offset] & 0b11;
            // TODO: Here (and everywhere) ensure masking off unneeded bits
            // TODO: Confirm correct
            let fl_unshifted: u32 = (((buf[*offset] & 0xf0) as u32) << 8)
                | ((buf[*offset+1] as u32) << 16)
                | ((buf[*offset+2] as u32) << 24);
            *offset += 3;
            ip6_header.set_ecn(ecn);
            ip6_header.set_flow_label(fl_unshifted);
        // Neither compressed (00 case)
        } else {
            let traffic_class = buf[*offset];
            let fl_unshifted: u32 = (((buf[*offset] & 0xf0) as u32) << 8)
                | ((buf[*offset+1] as u32) << 16)
                | ((buf[*offset+2] as u32) << 24);
            *offset += 4;
            ip6_header.set_traffic_class(traffic_class);
            ip6_header.set_flow_label(fl_unshifted);
        }
    }

    // TODO: impl
    fn decompress_nh(&self, iphc_header: u8) {
        if iphc_header & iphc::NH != 0 {
            // TODO: Impl
        }
    }

    fn decompress_hl(&self, 
                     ip6_header: &mut IP6Header,
                     iphc_header: u8,
                     buf: &[u8],
                     offset: &mut usize) {
        // TODO: Does this match work?
        let hop_limit = match (iphc_header & iphc::HLIM_MASK) {
            iphc::HLIM_1      => 1,
            iphc::HLIM_64     => 64,
            iphc::HLIM_255    => 255,
            iphc::HLIM_INLINE => {
                *offset +=1;
                buf[*offset-1]
            },
            // TODO: Unreachable
            _                 => 0,
        };
        ip6_header.set_hop_limit(hop_limit);
    }

    fn decompress_src(&self,
                      ip6_header: &mut IP6Header,
                      iphc_header: u8,
                      mac_addr: &MacAddr,
                      ctx: &Context, // Must be non-null
                      buf: &[u8],
                      offset: &mut usize) {
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
                                        offset);
        } else {
            self.decompress_iid_no_context(sam_mode,
                                           &mut ip6_header.src_addr,
                                           mac_addr,
                                           buf,
                                           offset);
        }
    }

    fn decompress_dst(&self,
                      ip6_header: &mut IP6Header,
                      iphc_header: u8,
                      mac_addr: &MacAddr,
                      ctx: &Context, // Must be non-null
                      buf: &[u8],
                      offset: &mut usize) {
        let uses_context = (iphc_header & iphc::DAC) != 0;
        let dam_mode = iphc_header & iphc::DAM_MASK;
        if uses_context && dam_mode == iphc::DAM_INLINE {
            // TODO: Reserved address: error
        } else if uses_context {
            self.decompress_iid_context(dam_mode,
                                        &mut ip6_header.dst_addr,
                                        mac_addr,
                                        ctx,
                                        buf,
                                        offset);
        } else {
            self.decompress_iid_no_context(dam_mode,
                                           &mut ip6_header.dst_addr,
                                           mac_addr,
                                           buf,
                                           offset);
        }
    }

    fn decompress_multicast(&self,
                            ip6_header: &mut IP6Header,
                            iphc_header: u8,
                            ctx: &Context,
                            buf: &[u8],
                            offset: &mut usize) {
        let uses_context = (iphc_header & iphc::DAC) != 0;
        let dam_mode = iphc_header & iphc::DAM_MASK;
        let mut ip_addr: &mut IPAddr = &mut ip6_header.dst_addr;
        if uses_context {
            match dam_mode {
                iphc::DAM_INLINE => {
                    // ffXX:XX + plen + pfx64 + XXXX:XXXX
                    // TODO: Check prefix_len <= 64
                    let prefix_bytes = ((ctx.prefix_len + 7) / 8) as usize;
                    ip_addr.0[0] = 0xff;
                    ip_addr.0[1] = buf[*offset];
                    ip_addr.0[2] = buf[*offset+1];
                    ip_addr.0[3] = ctx.prefix_len;
                    ip_addr.0[4..4 + prefix_bytes]
                        .copy_from_slice(&ctx.prefix[0..prefix_bytes]);
                    ip_addr.0[12..16].copy_from_slice(&buf[*offset + 2..*offset + 4]);
                    *offset += 6;
                },
                _ => {
                    // TODO: Reserved/unsupported
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
                    // TODO: Error case
                },
            }
        }
    }

    fn decompress_iid_no_context(&self,
                                 addr_mode: u8,
                                 ip_addr: &mut IPAddr,
                                 mac_addr: &MacAddr,
                                 buf: &[u8],
                                 offset: &mut usize) {
        let mode = addr_mode & (iphc::SAM_MASK | iphc::DAM_MASK);
        match mode {
            // SAM = 00, DAM = 00; address carried inline
            iphc::SAM_INLINE | iphc::DAM_INLINE => {
                ip_addr.0.copy_from_slice(&buf);
                *offset += 16;
            },
            // First 64-bits link local prefix, remaining 64 bits carried inline
            iphc::SAM_MODE1 | iphc::DAM_MODE1 => {
                ip_addr.set_unicast_link_local();
                ip_addr.0[8..16].copy_from_slice(&buf[*offset..*offset+8]);
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
            // TODO: Unreachable error case
            _ => { 
            },
        }
    }

    fn decompress_iid_context(&self,
                              addr_mode: u8,
                              ip_addr: &mut IPAddr,
                              mac_addr: &MacAddr,
                              ctx: &Context, // Must be non-null
                              buf: &[u8],
                              offset: &mut usize) {
        let mode = addr_mode & (iphc::SAM_MASK | iphc::DAM_MASK);
        match mode {
            // SAM = 00, DAM = 00; address equals :: or reserved
            iphc::SAM_INLINE | iphc::DAM_INLINE => {
                // TODO: Error case; should be handled elsewhere(?)
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
            // TODO: Unrechable; error case
            _ => {
            },
        }
        // Note that we copy the non-context bits into the ip_addr first, as
        // we must always use the context bits
        set_ctx_bits_in_addr(ip_addr, ctx);
    }
}
