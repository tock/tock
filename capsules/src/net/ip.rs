#[repr(C, packed)]
pub struct IP6Header {
    pub version_class_flow: [u8; 4],
    pub payload_len: u16,
    pub next_header: u8,
    pub hop_limit: u8,
    pub src_addr: IPAddr,
    pub dst_addr: IPAddr,
}

pub type MacAddr = u16;

pub type IPAddr = [u8; 8];

pub mod IP6Proto {
    pub const HOP_OPTS: u8 = 0;
    pub const TCP: u8      = 6;
    pub const UDP: u8      = 17;
    pub const IP6: u8      = 41;
    pub const ROUTING: u8  = 43;
    pub const FRAGMENT: u8 = 44;
    pub const DST_OPTS: u8 = 60;
    pub const MOBILITY: u8 = 135;
}
