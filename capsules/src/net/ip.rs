#[repr(C, packed)]
pub struct IP6Header {
    version_class_flow: [u8, 4],
    payload_len: u16,
    next_header: u8,
    hop_limit: u8,
    src_addr: IPAddr,
    dst_addr: IPAddr,
}

type MacAddr = u16;

type IPAddr = [u8, 8];
