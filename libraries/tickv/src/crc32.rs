//! A standalone CRC32 implementation
//!
//! This is based on the CRC32 implementation
//!  from: https://github.com/mrhooray/crc-rs
//!
//! This implemented the CRC-32 checksum
//!     poly: 0x04c11db7
//!     init: 0x00000000
//!     refin: false
//!     refout: false
//!     xorout: 0xffffffff
//!
//! Licensed under either of:
//!     Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
//!     MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
//! at your option.

const fn crc32_table(poly: u32) -> [u32; 256] {
    let mut table = [0u32; 256];
    table[0] = crc32(poly, 0);
    table[1] = crc32(poly, 1);
    table[2] = crc32(poly, 2);
    table[3] = crc32(poly, 3);
    table[4] = crc32(poly, 4);
    table[5] = crc32(poly, 5);
    table[6] = crc32(poly, 6);
    table[7] = crc32(poly, 7);
    table[8] = crc32(poly, 8);
    table[9] = crc32(poly, 9);
    table[10] = crc32(poly, 10);
    table[11] = crc32(poly, 11);
    table[12] = crc32(poly, 12);
    table[13] = crc32(poly, 13);
    table[14] = crc32(poly, 14);
    table[15] = crc32(poly, 15);
    table[16] = crc32(poly, 16);
    table[17] = crc32(poly, 17);
    table[18] = crc32(poly, 18);
    table[19] = crc32(poly, 19);
    table[20] = crc32(poly, 20);
    table[21] = crc32(poly, 21);
    table[22] = crc32(poly, 22);
    table[23] = crc32(poly, 23);
    table[24] = crc32(poly, 24);
    table[25] = crc32(poly, 25);
    table[26] = crc32(poly, 26);
    table[27] = crc32(poly, 27);
    table[28] = crc32(poly, 28);
    table[29] = crc32(poly, 29);
    table[30] = crc32(poly, 30);
    table[31] = crc32(poly, 31);
    table[32] = crc32(poly, 32);
    table[33] = crc32(poly, 33);
    table[34] = crc32(poly, 34);
    table[35] = crc32(poly, 35);
    table[36] = crc32(poly, 36);
    table[37] = crc32(poly, 37);
    table[38] = crc32(poly, 38);
    table[39] = crc32(poly, 39);
    table[40] = crc32(poly, 40);
    table[41] = crc32(poly, 41);
    table[42] = crc32(poly, 42);
    table[43] = crc32(poly, 43);
    table[44] = crc32(poly, 44);
    table[45] = crc32(poly, 45);
    table[46] = crc32(poly, 46);
    table[47] = crc32(poly, 47);
    table[48] = crc32(poly, 48);
    table[49] = crc32(poly, 49);
    table[50] = crc32(poly, 50);
    table[51] = crc32(poly, 51);
    table[52] = crc32(poly, 52);
    table[53] = crc32(poly, 53);
    table[54] = crc32(poly, 54);
    table[55] = crc32(poly, 55);
    table[56] = crc32(poly, 56);
    table[57] = crc32(poly, 57);
    table[58] = crc32(poly, 58);
    table[59] = crc32(poly, 59);
    table[60] = crc32(poly, 60);
    table[61] = crc32(poly, 61);
    table[62] = crc32(poly, 62);
    table[63] = crc32(poly, 63);
    table[64] = crc32(poly, 64);
    table[65] = crc32(poly, 65);
    table[66] = crc32(poly, 66);
    table[67] = crc32(poly, 67);
    table[68] = crc32(poly, 68);
    table[69] = crc32(poly, 69);
    table[70] = crc32(poly, 70);
    table[71] = crc32(poly, 71);
    table[72] = crc32(poly, 72);
    table[73] = crc32(poly, 73);
    table[74] = crc32(poly, 74);
    table[75] = crc32(poly, 75);
    table[76] = crc32(poly, 76);
    table[77] = crc32(poly, 77);
    table[78] = crc32(poly, 78);
    table[79] = crc32(poly, 79);
    table[80] = crc32(poly, 80);
    table[81] = crc32(poly, 81);
    table[82] = crc32(poly, 82);
    table[83] = crc32(poly, 83);
    table[84] = crc32(poly, 84);
    table[85] = crc32(poly, 85);
    table[86] = crc32(poly, 86);
    table[87] = crc32(poly, 87);
    table[88] = crc32(poly, 88);
    table[89] = crc32(poly, 89);
    table[90] = crc32(poly, 90);
    table[91] = crc32(poly, 91);
    table[92] = crc32(poly, 92);
    table[93] = crc32(poly, 93);
    table[94] = crc32(poly, 94);
    table[95] = crc32(poly, 95);
    table[96] = crc32(poly, 96);
    table[97] = crc32(poly, 97);
    table[98] = crc32(poly, 98);
    table[99] = crc32(poly, 99);
    table[100] = crc32(poly, 100);
    table[101] = crc32(poly, 101);
    table[102] = crc32(poly, 102);
    table[103] = crc32(poly, 103);
    table[104] = crc32(poly, 104);
    table[105] = crc32(poly, 105);
    table[106] = crc32(poly, 106);
    table[107] = crc32(poly, 107);
    table[108] = crc32(poly, 108);
    table[109] = crc32(poly, 109);
    table[110] = crc32(poly, 110);
    table[111] = crc32(poly, 111);
    table[112] = crc32(poly, 112);
    table[113] = crc32(poly, 113);
    table[114] = crc32(poly, 114);
    table[115] = crc32(poly, 115);
    table[116] = crc32(poly, 116);
    table[117] = crc32(poly, 117);
    table[118] = crc32(poly, 118);
    table[119] = crc32(poly, 119);
    table[120] = crc32(poly, 120);
    table[121] = crc32(poly, 121);
    table[122] = crc32(poly, 122);
    table[123] = crc32(poly, 123);
    table[124] = crc32(poly, 124);
    table[125] = crc32(poly, 125);
    table[126] = crc32(poly, 126);
    table[127] = crc32(poly, 127);
    table[128] = crc32(poly, 128);
    table[129] = crc32(poly, 129);
    table[130] = crc32(poly, 130);
    table[131] = crc32(poly, 131);
    table[132] = crc32(poly, 132);
    table[133] = crc32(poly, 133);
    table[134] = crc32(poly, 134);
    table[135] = crc32(poly, 135);
    table[136] = crc32(poly, 136);
    table[137] = crc32(poly, 137);
    table[138] = crc32(poly, 138);
    table[139] = crc32(poly, 139);
    table[140] = crc32(poly, 140);
    table[141] = crc32(poly, 141);
    table[142] = crc32(poly, 142);
    table[143] = crc32(poly, 143);
    table[144] = crc32(poly, 144);
    table[145] = crc32(poly, 145);
    table[146] = crc32(poly, 146);
    table[147] = crc32(poly, 147);
    table[148] = crc32(poly, 148);
    table[149] = crc32(poly, 149);
    table[150] = crc32(poly, 150);
    table[151] = crc32(poly, 151);
    table[152] = crc32(poly, 152);
    table[153] = crc32(poly, 153);
    table[154] = crc32(poly, 154);
    table[155] = crc32(poly, 155);
    table[156] = crc32(poly, 156);
    table[157] = crc32(poly, 157);
    table[158] = crc32(poly, 158);
    table[159] = crc32(poly, 159);
    table[160] = crc32(poly, 160);
    table[161] = crc32(poly, 161);
    table[162] = crc32(poly, 162);
    table[163] = crc32(poly, 163);
    table[164] = crc32(poly, 164);
    table[165] = crc32(poly, 165);
    table[166] = crc32(poly, 166);
    table[167] = crc32(poly, 167);
    table[168] = crc32(poly, 168);
    table[169] = crc32(poly, 169);
    table[170] = crc32(poly, 170);
    table[171] = crc32(poly, 171);
    table[172] = crc32(poly, 172);
    table[173] = crc32(poly, 173);
    table[174] = crc32(poly, 174);
    table[175] = crc32(poly, 175);
    table[176] = crc32(poly, 176);
    table[177] = crc32(poly, 177);
    table[178] = crc32(poly, 178);
    table[179] = crc32(poly, 179);
    table[180] = crc32(poly, 180);
    table[181] = crc32(poly, 181);
    table[182] = crc32(poly, 182);
    table[183] = crc32(poly, 183);
    table[184] = crc32(poly, 184);
    table[185] = crc32(poly, 185);
    table[186] = crc32(poly, 186);
    table[187] = crc32(poly, 187);
    table[188] = crc32(poly, 188);
    table[189] = crc32(poly, 189);
    table[190] = crc32(poly, 190);
    table[191] = crc32(poly, 191);
    table[192] = crc32(poly, 192);
    table[193] = crc32(poly, 193);
    table[194] = crc32(poly, 194);
    table[195] = crc32(poly, 195);
    table[196] = crc32(poly, 196);
    table[197] = crc32(poly, 197);
    table[198] = crc32(poly, 198);
    table[199] = crc32(poly, 199);
    table[200] = crc32(poly, 200);
    table[201] = crc32(poly, 201);
    table[202] = crc32(poly, 202);
    table[203] = crc32(poly, 203);
    table[204] = crc32(poly, 204);
    table[205] = crc32(poly, 205);
    table[206] = crc32(poly, 206);
    table[207] = crc32(poly, 207);
    table[208] = crc32(poly, 208);
    table[209] = crc32(poly, 209);
    table[210] = crc32(poly, 210);
    table[211] = crc32(poly, 211);
    table[212] = crc32(poly, 212);
    table[213] = crc32(poly, 213);
    table[214] = crc32(poly, 214);
    table[215] = crc32(poly, 215);
    table[216] = crc32(poly, 216);
    table[217] = crc32(poly, 217);
    table[218] = crc32(poly, 218);
    table[219] = crc32(poly, 219);
    table[220] = crc32(poly, 220);
    table[221] = crc32(poly, 221);
    table[222] = crc32(poly, 222);
    table[223] = crc32(poly, 223);
    table[224] = crc32(poly, 224);
    table[225] = crc32(poly, 225);
    table[226] = crc32(poly, 226);
    table[227] = crc32(poly, 227);
    table[228] = crc32(poly, 228);
    table[229] = crc32(poly, 229);
    table[230] = crc32(poly, 230);
    table[231] = crc32(poly, 231);
    table[232] = crc32(poly, 232);
    table[233] = crc32(poly, 233);
    table[234] = crc32(poly, 234);
    table[235] = crc32(poly, 235);
    table[236] = crc32(poly, 236);
    table[237] = crc32(poly, 237);
    table[238] = crc32(poly, 238);
    table[239] = crc32(poly, 239);
    table[240] = crc32(poly, 240);
    table[241] = crc32(poly, 241);
    table[242] = crc32(poly, 242);
    table[243] = crc32(poly, 243);
    table[244] = crc32(poly, 244);
    table[245] = crc32(poly, 245);
    table[246] = crc32(poly, 246);
    table[247] = crc32(poly, 247);
    table[248] = crc32(poly, 248);
    table[249] = crc32(poly, 249);
    table[250] = crc32(poly, 250);
    table[251] = crc32(poly, 251);
    table[252] = crc32(poly, 252);
    table[253] = crc32(poly, 253);
    table[254] = crc32(poly, 254);
    table[255] = crc32(poly, 255);
    table
}

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

pub struct Crc {
    table: [u32; 256],
}

pub struct Digest<'a> {
    crc: &'a Crc,
    value: u32,
}

impl Crc {
    pub const fn new() -> Self {
        let table = crc32_table(0x04c11db7);
        Self { table }
    }

    const fn init(&self) -> u32 {
        0x00000000
    }

    const fn table_entry(&self, index: u32) -> u32 {
        self.table[(index & 0xFF) as usize]
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
