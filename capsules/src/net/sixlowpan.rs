/// Implements the 6LoWPAN specification for sending IPv6 datagrams over
/// 802.15.4 packets efficiently, as detailed in RFC 6282.

pub mod sixlowpan {

    /// Constructs a 6LoWPAN header in `buf` from the given IPv6 header and
    /// 16-bit MAC addresses.  Returns the number of bytes written into `buf`.
    fn compress(ip6header: &IP6Header,
                src_addr: Addr16,
                dest_addr: Addr16,
                buf: &'static mut [u8]) -> Ok<u8> {
    }

    /// Decodes the compressed header into a full IPv6 header given the 16-bit
    /// MAC addresses. `buf` is expected to be a slice starting from the
    /// beginning of the IP header.  Returns the number of bytes taken up by the
    /// header, so the remaining bytes are the payload. Also returns an optional
    /// `FragInfo` containing the datagram tag and fragmentation offset if this
    /// packet is part of a set of fragments.
    fn decompress(buf: &'static mut [u8],
                  src_addr: Addr16,
                  dest_addr: Addr16) -> Ok<(IP6Header, u8, Option<FragInfo>)> {
    }

}
