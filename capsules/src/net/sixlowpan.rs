/// Implements the 6LoWPAN specification for sending IPv6 datagrams over
/// 802.15.4 packets efficiently, as detailed in RFC 6282.

pub struct Context {
    prefix: &[u8],
    prefix_len: u8,
    ctx_id: u8,
    compress: bool,
}

pub trait ContextStore {
    fn get_context(ip_addr: IP6Address) -> Option<Context>;
    fn get_context(ctx_id: u8) -> Option<Context>;
}

pub struct DummyStore {
}

pub impl ContextStore for DummyStore {
    fn get_context(ip_addr: IP6Address) -> Option<Context> {
        None
    }

    fn get_context(ctx_id: u8) -> Option<Context> {
        None
    }
}

pub struct LoWPAN<'a, C: ContextStore> {
    ctx_store: 'a &C,
}

impl<'a, C: ContextStore> LoWPAN {
    pub fn new(ctx_store: &'a C) -> LoWPAN<'a, C> {
        LoWPAN {
            ctx_store: ctx_store,
        }
    }

    /// Constructs a 6LoWPAN header in `buf` from the given IPv6 header and
    /// 16-bit MAC addresses.  Returns the number of bytes written into `buf`.
    pub fn compress(&self,
                    ip6_header: &IP6Header,
                    src_mac_addr: MacAddr,
                    dest_mac_addr: MacAddr,
                    buf: &'static mut [u8]) -> u8 {
        // The first two bytes are the LOWPAN_IPHC header
        let mut offset: u8 = 2;

        // Initialize the LOWPAN_IPHC header
        buf[0] = 0b01100000;
        buf[1] = 0b00000000;

        let mut src_ctx: Option<Context> = self.ctx_store.get_context(ip6_header.src_addr);
        let mut dst_ctx: Option<Context> = self.ctx_store.get_context(ip6_header.dst_addr);

        // Do not use these contexts if they are not to be used for compression
        src_ctx = src_ctx.and_then(|ctx| { if ctx.compress { Some(ctx) } else { None } });
        dst_ctx = dst_ctx.and_then(|ctx| { if ctx.compress { Some(ctx) } else { None } });

        // Context Identifier Extension
        self.compress_cie(&src_ctx, &dst_ctx, buf, &mut offset);

        // Traffic Class & Flow Label
        self.compress_tf(ip6_header, buf, &mut offset);
    }

    /// Decodes the compressed header into a full IPv6 header given the 16-bit
    /// MAC addresses. `buf` is expected to be a slice starting from the
    /// beginning of the IP header.  Returns the number of bytes taken up by the
    /// header, so the remaining bytes are the payload. Also returns an optional
    /// `FragInfo` containing the datagram tag and fragmentation offset if this
    /// packet is part of a set of fragments.
    pub fn decompress(&self,
                      buf: &'static mut [u8],
                      src_mac_addr: MacAddr,
                      dest_mac_addr: MacAddr,
                      mesh_local_prefix: &[u8]) -> Ok<(IP6Header, u8, Option<FragInfo>)> {
    }
}
