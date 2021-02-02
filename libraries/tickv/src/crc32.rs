//! A standalone CRC32 implementation
//!
//! This is based on the CRC32 implementation
//! from: crc-rs <https://github.com/mrhooray/crc-rs>
//!
//! This implemented the CRC-32 checksum
//!     poly: 0x04c11db7
//!     init: 0x00000000
//!     refin: false
//!     refout: false
//!     xorout: 0xffffffff
//!
//! Licensed under either of:
//!     Apache License, Version 2.0 (LICENSE-APACHE <http://www.apache.org/licenses/LICENSE-2.0>)
//!     MIT License (LICENSE-MIT <http://opensource.org/licenses/MIT>)
//! at your option.

const CRC_TABLE: [u32; 256] = [
    crc32(0x04c11db7, 0),
    crc32(0x04c11db7, 1),
    crc32(0x04c11db7, 2),
    crc32(0x04c11db7, 3),
    crc32(0x04c11db7, 4),
    crc32(0x04c11db7, 5),
    crc32(0x04c11db7, 6),
    crc32(0x04c11db7, 7),
    crc32(0x04c11db7, 8),
    crc32(0x04c11db7, 9),
    crc32(0x04c11db7, 10),
    crc32(0x04c11db7, 11),
    crc32(0x04c11db7, 12),
    crc32(0x04c11db7, 13),
    crc32(0x04c11db7, 14),
    crc32(0x04c11db7, 15),
    crc32(0x04c11db7, 16),
    crc32(0x04c11db7, 17),
    crc32(0x04c11db7, 18),
    crc32(0x04c11db7, 19),
    crc32(0x04c11db7, 20),
    crc32(0x04c11db7, 21),
    crc32(0x04c11db7, 22),
    crc32(0x04c11db7, 23),
    crc32(0x04c11db7, 24),
    crc32(0x04c11db7, 25),
    crc32(0x04c11db7, 26),
    crc32(0x04c11db7, 27),
    crc32(0x04c11db7, 28),
    crc32(0x04c11db7, 29),
    crc32(0x04c11db7, 30),
    crc32(0x04c11db7, 31),
    crc32(0x04c11db7, 32),
    crc32(0x04c11db7, 33),
    crc32(0x04c11db7, 34),
    crc32(0x04c11db7, 35),
    crc32(0x04c11db7, 36),
    crc32(0x04c11db7, 37),
    crc32(0x04c11db7, 38),
    crc32(0x04c11db7, 39),
    crc32(0x04c11db7, 40),
    crc32(0x04c11db7, 41),
    crc32(0x04c11db7, 42),
    crc32(0x04c11db7, 43),
    crc32(0x04c11db7, 44),
    crc32(0x04c11db7, 45),
    crc32(0x04c11db7, 46),
    crc32(0x04c11db7, 47),
    crc32(0x04c11db7, 48),
    crc32(0x04c11db7, 49),
    crc32(0x04c11db7, 50),
    crc32(0x04c11db7, 51),
    crc32(0x04c11db7, 52),
    crc32(0x04c11db7, 53),
    crc32(0x04c11db7, 54),
    crc32(0x04c11db7, 55),
    crc32(0x04c11db7, 56),
    crc32(0x04c11db7, 57),
    crc32(0x04c11db7, 58),
    crc32(0x04c11db7, 59),
    crc32(0x04c11db7, 60),
    crc32(0x04c11db7, 61),
    crc32(0x04c11db7, 62),
    crc32(0x04c11db7, 63),
    crc32(0x04c11db7, 64),
    crc32(0x04c11db7, 65),
    crc32(0x04c11db7, 66),
    crc32(0x04c11db7, 67),
    crc32(0x04c11db7, 68),
    crc32(0x04c11db7, 69),
    crc32(0x04c11db7, 70),
    crc32(0x04c11db7, 71),
    crc32(0x04c11db7, 72),
    crc32(0x04c11db7, 73),
    crc32(0x04c11db7, 74),
    crc32(0x04c11db7, 75),
    crc32(0x04c11db7, 76),
    crc32(0x04c11db7, 77),
    crc32(0x04c11db7, 78),
    crc32(0x04c11db7, 79),
    crc32(0x04c11db7, 80),
    crc32(0x04c11db7, 81),
    crc32(0x04c11db7, 82),
    crc32(0x04c11db7, 83),
    crc32(0x04c11db7, 84),
    crc32(0x04c11db7, 85),
    crc32(0x04c11db7, 86),
    crc32(0x04c11db7, 87),
    crc32(0x04c11db7, 88),
    crc32(0x04c11db7, 89),
    crc32(0x04c11db7, 90),
    crc32(0x04c11db7, 91),
    crc32(0x04c11db7, 92),
    crc32(0x04c11db7, 93),
    crc32(0x04c11db7, 94),
    crc32(0x04c11db7, 95),
    crc32(0x04c11db7, 96),
    crc32(0x04c11db7, 97),
    crc32(0x04c11db7, 98),
    crc32(0x04c11db7, 99),
    crc32(0x04c11db7, 100),
    crc32(0x04c11db7, 101),
    crc32(0x04c11db7, 102),
    crc32(0x04c11db7, 103),
    crc32(0x04c11db7, 104),
    crc32(0x04c11db7, 105),
    crc32(0x04c11db7, 106),
    crc32(0x04c11db7, 107),
    crc32(0x04c11db7, 108),
    crc32(0x04c11db7, 109),
    crc32(0x04c11db7, 110),
    crc32(0x04c11db7, 111),
    crc32(0x04c11db7, 112),
    crc32(0x04c11db7, 113),
    crc32(0x04c11db7, 114),
    crc32(0x04c11db7, 115),
    crc32(0x04c11db7, 116),
    crc32(0x04c11db7, 117),
    crc32(0x04c11db7, 118),
    crc32(0x04c11db7, 119),
    crc32(0x04c11db7, 120),
    crc32(0x04c11db7, 121),
    crc32(0x04c11db7, 122),
    crc32(0x04c11db7, 123),
    crc32(0x04c11db7, 124),
    crc32(0x04c11db7, 125),
    crc32(0x04c11db7, 126),
    crc32(0x04c11db7, 127),
    crc32(0x04c11db7, 128),
    crc32(0x04c11db7, 129),
    crc32(0x04c11db7, 130),
    crc32(0x04c11db7, 131),
    crc32(0x04c11db7, 132),
    crc32(0x04c11db7, 133),
    crc32(0x04c11db7, 134),
    crc32(0x04c11db7, 135),
    crc32(0x04c11db7, 136),
    crc32(0x04c11db7, 137),
    crc32(0x04c11db7, 138),
    crc32(0x04c11db7, 139),
    crc32(0x04c11db7, 140),
    crc32(0x04c11db7, 141),
    crc32(0x04c11db7, 142),
    crc32(0x04c11db7, 143),
    crc32(0x04c11db7, 144),
    crc32(0x04c11db7, 145),
    crc32(0x04c11db7, 146),
    crc32(0x04c11db7, 147),
    crc32(0x04c11db7, 148),
    crc32(0x04c11db7, 149),
    crc32(0x04c11db7, 150),
    crc32(0x04c11db7, 151),
    crc32(0x04c11db7, 152),
    crc32(0x04c11db7, 153),
    crc32(0x04c11db7, 154),
    crc32(0x04c11db7, 155),
    crc32(0x04c11db7, 156),
    crc32(0x04c11db7, 157),
    crc32(0x04c11db7, 158),
    crc32(0x04c11db7, 159),
    crc32(0x04c11db7, 160),
    crc32(0x04c11db7, 161),
    crc32(0x04c11db7, 162),
    crc32(0x04c11db7, 163),
    crc32(0x04c11db7, 164),
    crc32(0x04c11db7, 165),
    crc32(0x04c11db7, 166),
    crc32(0x04c11db7, 167),
    crc32(0x04c11db7, 168),
    crc32(0x04c11db7, 169),
    crc32(0x04c11db7, 170),
    crc32(0x04c11db7, 171),
    crc32(0x04c11db7, 172),
    crc32(0x04c11db7, 173),
    crc32(0x04c11db7, 174),
    crc32(0x04c11db7, 175),
    crc32(0x04c11db7, 176),
    crc32(0x04c11db7, 177),
    crc32(0x04c11db7, 178),
    crc32(0x04c11db7, 179),
    crc32(0x04c11db7, 180),
    crc32(0x04c11db7, 181),
    crc32(0x04c11db7, 182),
    crc32(0x04c11db7, 183),
    crc32(0x04c11db7, 184),
    crc32(0x04c11db7, 185),
    crc32(0x04c11db7, 186),
    crc32(0x04c11db7, 187),
    crc32(0x04c11db7, 188),
    crc32(0x04c11db7, 189),
    crc32(0x04c11db7, 190),
    crc32(0x04c11db7, 191),
    crc32(0x04c11db7, 192),
    crc32(0x04c11db7, 193),
    crc32(0x04c11db7, 194),
    crc32(0x04c11db7, 195),
    crc32(0x04c11db7, 196),
    crc32(0x04c11db7, 197),
    crc32(0x04c11db7, 198),
    crc32(0x04c11db7, 199),
    crc32(0x04c11db7, 200),
    crc32(0x04c11db7, 201),
    crc32(0x04c11db7, 202),
    crc32(0x04c11db7, 203),
    crc32(0x04c11db7, 204),
    crc32(0x04c11db7, 205),
    crc32(0x04c11db7, 206),
    crc32(0x04c11db7, 207),
    crc32(0x04c11db7, 208),
    crc32(0x04c11db7, 209),
    crc32(0x04c11db7, 210),
    crc32(0x04c11db7, 211),
    crc32(0x04c11db7, 212),
    crc32(0x04c11db7, 213),
    crc32(0x04c11db7, 214),
    crc32(0x04c11db7, 215),
    crc32(0x04c11db7, 216),
    crc32(0x04c11db7, 217),
    crc32(0x04c11db7, 218),
    crc32(0x04c11db7, 219),
    crc32(0x04c11db7, 220),
    crc32(0x04c11db7, 221),
    crc32(0x04c11db7, 222),
    crc32(0x04c11db7, 223),
    crc32(0x04c11db7, 224),
    crc32(0x04c11db7, 225),
    crc32(0x04c11db7, 226),
    crc32(0x04c11db7, 227),
    crc32(0x04c11db7, 228),
    crc32(0x04c11db7, 229),
    crc32(0x04c11db7, 230),
    crc32(0x04c11db7, 231),
    crc32(0x04c11db7, 232),
    crc32(0x04c11db7, 233),
    crc32(0x04c11db7, 234),
    crc32(0x04c11db7, 235),
    crc32(0x04c11db7, 236),
    crc32(0x04c11db7, 237),
    crc32(0x04c11db7, 238),
    crc32(0x04c11db7, 239),
    crc32(0x04c11db7, 240),
    crc32(0x04c11db7, 241),
    crc32(0x04c11db7, 242),
    crc32(0x04c11db7, 243),
    crc32(0x04c11db7, 244),
    crc32(0x04c11db7, 245),
    crc32(0x04c11db7, 246),
    crc32(0x04c11db7, 247),
    crc32(0x04c11db7, 248),
    crc32(0x04c11db7, 249),
    crc32(0x04c11db7, 250),
    crc32(0x04c11db7, 251),
    crc32(0x04c11db7, 252),
    crc32(0x04c11db7, 253),
    crc32(0x04c11db7, 254),
    crc32(0x04c11db7, 255),
];

const fn reflect_8(mut b: u8) -> u8 {
    b = (b & 0xF0) >> 4 | (b & 0x0F) << 4;
    b = (b & 0xCC) >> 2 | (b & 0x33) << 2;
    b = (b & 0xAA) >> 1 | (b & 0x55) << 1;
    b
}

const fn reflect_32(mut b: u32) -> u32 {
    b = (b & 0xFFFF0000) >> 16 | (b & 0x0000FFFF) << 16;
    b = (b & 0xFF00FF00) >> 8 | (b & 0x00FF00FF) << 8;
    b = (b & 0xF0F0F0F0) >> 4 | (b & 0x0F0F0F0F) << 4;
    b = (b & 0xCCCCCCCC) >> 2 | (b & 0x33333333) << 2;
    b = (b & 0xAAAAAAAA) >> 1 | (b & 0x55555555) << 1;
    b
}

const fn crc32(poly: u32, mut byte: u8) -> u32 {
    const fn poly_sum_crc32(poly: u32, value: u32) -> u32 {
        (value << 1) ^ ((value >> 31) * poly)
    }
    byte = [byte, reflect_8(byte)][0];
    let mut value = (byte as u32) << 24;
    value = poly_sum_crc32(poly, value);
    value = poly_sum_crc32(poly, value);
    value = poly_sum_crc32(poly, value);
    value = poly_sum_crc32(poly, value);
    value = poly_sum_crc32(poly, value);
    value = poly_sum_crc32(poly, value);
    value = poly_sum_crc32(poly, value);
    value = poly_sum_crc32(poly, value);
    [value, reflect_32(value)][0]
}

pub struct Crc {}

pub struct Digest<'a> {
    crc: &'a Crc,
    value: u32,
}

impl Crc {
    pub const fn new() -> Self {
        Self {}
    }

    const fn init(&self) -> u32 {
        0x00000000
    }

    const fn table_entry(&self, index: u32) -> u32 {
        CRC_TABLE[(index & 0xFF) as usize]
    }

    const fn update(&self, mut crc: u32, bytes: &[u8]) -> u32 {
        let mut i = 0;
        while i < bytes.len() {
            crc = self.table_entry(bytes[i] as u32 ^ (crc >> 24)) ^ (crc << 8);
            i += 1;
        }
        crc
    }

    const fn finalise(&self, crc: u32) -> u32 {
        crc ^ 0xffffffff
    }

    pub const fn digest(&self) -> Digest {
        Digest::new(self)
    }
}

impl<'a> Digest<'a> {
    const fn new(crc: &'a Crc) -> Self {
        let value = crc.init();
        Digest { crc, value }
    }

    pub fn update(&mut self, bytes: &[u8]) {
        self.value = self.crc.update(self.value, bytes);
    }

    pub const fn finalise(self) -> u32 {
        self.crc.finalise(self.value)
    }
}
