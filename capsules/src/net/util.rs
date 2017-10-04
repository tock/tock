/// Utility functions used in the 6LoWPAN implementation

/// Verifies that a prefix given in the form of a byte array slice is valid with
/// respect to its length in bits (prefix_len):
/// - The byte array slice must contain enough bytes to cover the prefix length
/// (no implicit zero-padding)
/// - The rest of the prefix array slice is zero-padded
pub fn verify_prefix_len(prefix: &[u8], prefix_len: u8) -> bool {
    let full_bytes = (prefix_len / 8) as usize;
    let remaining_bits = prefix_len % 8;
    let bytes = full_bytes + if remaining_bits != 0 { 1 } else { 0 };

    if bytes > prefix.len() {
        return false;
    }

    // The bits between the prefix's end and the next byte boundary must be 0
    if remaining_bits != 0 {
        let last_byte_mask = 0xff >> remaining_bits;
        if prefix[full_bytes] & last_byte_mask != 0 {
            return false;
        }
    }

    // Ensure that the remaining bytes are also 0
    prefix[bytes..].iter().all(|&b| b == 0)
}

/// Verifies that the prefixes of the two buffers match, where the length of the
/// prefix is given in bits
pub fn matches_prefix(buf1: &[u8], buf2: &[u8], prefix_len: u8) -> bool {
    let full_bytes = (prefix_len / 8) as usize;
    let remaining_bits = prefix_len % 8;
    let bytes = full_bytes + if remaining_bits != 0 { 1 } else { 0 };

    if bytes > buf1.len() || bytes > buf2.len() {
        return false;
    }

    // Ensure that the prefix bits in the last byte match
    if remaining_bits != 0 {
        let last_byte_mask = 0xff << (8 - remaining_bits);
        if (buf1[full_bytes] ^ buf2[full_bytes]) & last_byte_mask != 0 {
            return false;
        }
    }

    // Ensure that the prefix bytes before that match
    buf1[..full_bytes].iter().eq(buf2[..full_bytes].iter())
}

pub fn slice_to_u16(buf: &[u8]) -> u16 {
    ((buf[0] as u16) << 8) | (buf[1] as u16)
}

pub fn u16_to_slice(short: u16, slice: &mut [u8]) {
    slice[0] = (short >> 8) as u8;
    slice[1] = (short & 0xff) as u8;
}
