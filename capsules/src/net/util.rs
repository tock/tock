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
