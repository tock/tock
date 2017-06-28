use net::util;

#[repr(C, packed)]
#[allow(unused_variables)]
pub struct IP6Header {
    pub version_class_flow: [u8; 4],
    pub payload_len: u16,
    pub next_header: u8,
    pub hop_limit: u8,
    pub src_addr: IPAddr,
    pub dst_addr: IPAddr,
}

pub enum MacAddr {
    ShortAddr(u16),
    LongAddr([u8; 8]),
}

pub type IPAddr = [u8; 16];

#[allow(unused_variables)]
pub mod ip6_nh {
    pub const HOP_OPTS: u8 = 0;
    pub const TCP: u8      = 6;
    pub const UDP: u8      = 17;
    pub const IP6: u8      = 41;
    pub const ROUTING: u8  = 43;
    pub const FRAGMENT: u8 = 44;
    pub const ICMP: u8     = 58;
    pub const NO_NEXT: u8  = 59;
    pub const DST_OPTS: u8 = 60;
    pub const MOBILITY: u8 = 135;
}

#[allow(unused_variables,dead_code)]
pub fn addr_is_unspecified(ip_addr: &IPAddr) -> bool {
    util::is_zero(ip_addr)
}

#[allow(unused_variables,dead_code)]
pub fn addr_is_link_local(ip_addr: &IPAddr) -> bool {
    // First 64 bits match fe80:: with mask ffc0::
    ip_addr[0] == 0xfe
    && (ip_addr[1] & 0xc0) == 0x80
    // Remaining bits are 0
    && (ip_addr[1] & 0x3f) == 0
    && util::is_zero(&ip_addr[2..8])
}

#[allow(unused_variables,dead_code)]
pub fn addr_is_multicast(ip_addr: &IPAddr) -> bool {
    // Address is prefixed by ffxx::
    ip_addr[0] == 0xff
}
