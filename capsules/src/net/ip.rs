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
    LongAddr([u8; 8])
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
    pub const DST_OPTS: u8 = 60;
    pub const MOBILITY: u8 = 135;
}

// TODO: Implement
#[allow(unused_variables,dead_code)]
pub fn addr_is_unspecified(ip_addr: &IPAddr) -> bool {
    false
}

#[allow(unused_variables,dead_code)]
pub fn addr_is_link_local(ip_addr: &IPAddr) -> bool {
    false
}

#[allow(unused_variables,dead_code)]
pub fn addr_is_multicast(ip_addr: &IPAddr) -> bool {
    false
}

