const BITMAP_SIZE: usize = 20;

pub struct Bitmap {
    map: [u8; BITMAP_SIZE],
}

impl Bitmap {
    pub fn new() -> Bitmap {
        Bitmap { map: [0; BITMAP_SIZE] }
    }

    pub fn clear(&mut self) {
        for i in 0..self.map.len() {
            self.map[i] = 0;
        }
    }

    pub fn clear_bit(&mut self, idx: usize) {
        let map_idx = idx / 8;
        self.map[map_idx] &= !(1 << (idx % 8));
    }

    pub fn set_bit(&mut self, idx: usize) {
        let map_idx = idx / 8;
        self.map[map_idx] |= 1 << (idx % 8);
    }

    // Sets bits from start_idx (inclusive) to end_idx (exclusive).
    // Returns false if any bits set overlap with already set bits,
    // true otherwise.

    // Returns true if successfully set bits, returns false if the bits
    // overlapped with already set bits
    // Note that each bit represents a multiple of 8 bytes (as everything
    // must be in 8-byte groups), and thus we can store 8*8 = 64 "bytes" per
    // byte in the bitmap.
    pub fn set_bits(&mut self, start_idx: usize, end_idx: usize) -> bool {
        if start_idx > end_idx {
            return false;
        }
        let start_byte_idx = start_idx / 8;
        let end_byte_idx = end_idx / 8;
        let first = 0xff << (start_idx % 8);
        let second = if end_idx % 8 == 0 {
            0x00
        } else {
            0xff >> (8 - (end_idx % 8))
        };
        if start_byte_idx == end_byte_idx {
            let result = (self.map[start_byte_idx] & (first & second)) == 0;
            self.map[start_byte_idx] |= first & second;
            result
        } else {
            let mut result = (self.map[start_byte_idx] & first) == 0;
            result = result && ((self.map[end_byte_idx] & second) == 0);
            self.map[start_byte_idx] |= first;
            self.map[end_byte_idx] |= second;
            // Set all bytes between start and end bytes.
            for i in start_byte_idx + 1..end_byte_idx {
                result = result && (self.map[i] == 0);
                self.map[i] = 0xff;
            }
            result
        }
    }

    pub fn is_complete(&self, total_length: usize) -> bool {
        let mut result = true;
        for i in 0..total_length / 8 {
            result = result && (self.map[i] == 0xff);
        }
        // Check last byte.
        let mask = 0xff >> (8 - (total_length % 8));
        result = result && (self.map[total_length / 8] == mask);
        result
    }
}
