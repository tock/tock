#[repr(C, packed)]
pub struct IPv6Header {
    version_class_flow: [u8, 4],
    payload_len: u16,
    next_header: u8,
    hop_limit: u8,
    src_addr: [u8, 8],
    dst_addr: [u8, 8],
}

type Addr16 = u16;
