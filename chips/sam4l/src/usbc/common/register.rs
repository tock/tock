/// Traits for types that manipulate memory-mapped registers

pub trait RegisterRW {
    fn read(&self) -> u32;
    fn write(&self, val: u32);

    #[inline]
    fn set_bit(&self, bit_index: u32) {
        let w = self.read();
        let bit = 1 << bit_index;
        self.write(w | bit);
    }

    #[inline]
    fn clear_bit(&self, bit_index: u32) {
        let w = self.read();
        let bit = 1 << bit_index;
        self.write(w & (!bit));
    }
}

pub trait RegisterR {
    fn read(&self) -> u32;
}

pub trait RegisterW {
    fn write(&self, val: u32);

    #[inline]
    fn set_bit(&self, bit_index: u32) {
        // For this kind of register, reads always return zero
        // and zero bits have no effect, so we simply write the
        // single bit requested.
        let bit = 1 << bit_index;
        self.write(bit);
    }
}

pub trait ToWord {
    fn to_word(self) -> u32;
}

impl ToWord for u32 {
    #[inline]
    fn to_word(self) -> u32 {
        self
    }
}

impl ToWord for u8 {
    #[inline]
    fn to_word(self) -> u32 {
        self as u32
    }
}

impl ToWord for bool {
    #[inline]
    fn to_word(self) -> u32 {
        if self {
            1
        } else {
            0
        }
    }
}

pub trait FromWord {
    fn from_word(u32) -> Self;
}

impl FromWord for u32 {
    #[inline]
    fn from_word(w: u32) -> Self {
        w
    }
}

impl FromWord for bool {
    #[inline]
    fn from_word(w: u32) -> bool {
        if w & 1 == 1 {
            true
        } else {
            false
        }
    }
}
