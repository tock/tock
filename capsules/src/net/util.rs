/// Utility functions used in the 6LoWPAN implementation

/// Verifies that all bits beyond prefix_len in the slice are zero
pub fn verify_prefix_len(prefix: &[u8], prefix_len: u8) -> bool {
    let bytes: u8 = (prefix_len / 8) + ((prefix_len & 0x7 != 0) as u8);
    if bytes as usize > prefix.len() {
        return false;
    }

    // Ensure that the bits between the prefix and the next byte boundary are 0
    if prefix_len != bytes * 8 {
        let partial_byte_mask = (0x1 << (bytes * 8 - prefix_len)) - 1;
        if prefix[(prefix_len / 8) as usize] & partial_byte_mask != 0 {
            return false;
        }
    }

    // Ensure that the remaining bytes are also 0
    for i in (bytes as usize)..prefix.len() {
        if prefix[i] != 0 {
            return false;
        }
    }

    return true;
}

/// Verifies that all bytes are zero in the buffer
pub fn is_zero(buf: &[u8]) -> bool {
    for i in 0..buf.len() {
        if buf[i] != 0 {
            return false;
        }
    }

    return true;
}

pub fn matches_prefix(buf1: &[u8], buf2: &[u8], prefix_len: u8) -> bool {
    let full_bytes = (prefix_len / 8) as usize;
    let remaining = prefix_len & 0x7;
    let mut bytes = full_bytes;
    if remaining != 0 {
        bytes += 1;
    }

    if buf1.len() < bytes || buf2.len() < bytes {
        return false;
    }

    for i in 0..full_bytes {
        if buf1[i] != buf2[i] {
            return false;
        }
    }

    for i in 0..remaining {
        let mask: u8 = 0x80 >> i;
        if buf1[full_bytes] & mask != buf2[full_bytes] & mask {
            return false;
        }
    }

    true
}
