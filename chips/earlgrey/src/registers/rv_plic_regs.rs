// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for rv_plic.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/top_earlgrey/ip_autogen/rv_plic/data/rv_plic.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number of interrupt sources
pub const RV_PLIC_PARAM_NUM_SRC: u32 = 185;
/// Number of Targets (Harts)
pub const RV_PLIC_PARAM_NUM_TARGET: u32 = 1;
/// Width of priority signals
pub const RV_PLIC_PARAM_PRIO_WIDTH: u32 = 2;
/// Number of alerts
pub const RV_PLIC_PARAM_NUM_ALERTS: u32 = 1;
/// Register width
pub const RV_PLIC_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub RvPlicRegisters {
        /// Interrupt Source 0 Priority
        (0x0000 => pub(crate) prio0: ReadWrite<u32, PRIO0::Register>),
        /// Interrupt Source 1 Priority
        (0x0004 => pub(crate) prio1: ReadWrite<u32, PRIO1::Register>),
        /// Interrupt Source 2 Priority
        (0x0008 => pub(crate) prio2: ReadWrite<u32, PRIO2::Register>),
        /// Interrupt Source 3 Priority
        (0x000c => pub(crate) prio3: ReadWrite<u32, PRIO3::Register>),
        /// Interrupt Source 4 Priority
        (0x0010 => pub(crate) prio4: ReadWrite<u32, PRIO4::Register>),
        /// Interrupt Source 5 Priority
        (0x0014 => pub(crate) prio5: ReadWrite<u32, PRIO5::Register>),
        /// Interrupt Source 6 Priority
        (0x0018 => pub(crate) prio6: ReadWrite<u32, PRIO6::Register>),
        /// Interrupt Source 7 Priority
        (0x001c => pub(crate) prio7: ReadWrite<u32, PRIO7::Register>),
        /// Interrupt Source 8 Priority
        (0x0020 => pub(crate) prio8: ReadWrite<u32, PRIO8::Register>),
        /// Interrupt Source 9 Priority
        (0x0024 => pub(crate) prio9: ReadWrite<u32, PRIO9::Register>),
        /// Interrupt Source 10 Priority
        (0x0028 => pub(crate) prio10: ReadWrite<u32, PRIO10::Register>),
        /// Interrupt Source 11 Priority
        (0x002c => pub(crate) prio11: ReadWrite<u32, PRIO11::Register>),
        /// Interrupt Source 12 Priority
        (0x0030 => pub(crate) prio12: ReadWrite<u32, PRIO12::Register>),
        /// Interrupt Source 13 Priority
        (0x0034 => pub(crate) prio13: ReadWrite<u32, PRIO13::Register>),
        /// Interrupt Source 14 Priority
        (0x0038 => pub(crate) prio14: ReadWrite<u32, PRIO14::Register>),
        /// Interrupt Source 15 Priority
        (0x003c => pub(crate) prio15: ReadWrite<u32, PRIO15::Register>),
        /// Interrupt Source 16 Priority
        (0x0040 => pub(crate) prio16: ReadWrite<u32, PRIO16::Register>),
        /// Interrupt Source 17 Priority
        (0x0044 => pub(crate) prio17: ReadWrite<u32, PRIO17::Register>),
        /// Interrupt Source 18 Priority
        (0x0048 => pub(crate) prio18: ReadWrite<u32, PRIO18::Register>),
        /// Interrupt Source 19 Priority
        (0x004c => pub(crate) prio19: ReadWrite<u32, PRIO19::Register>),
        /// Interrupt Source 20 Priority
        (0x0050 => pub(crate) prio20: ReadWrite<u32, PRIO20::Register>),
        /// Interrupt Source 21 Priority
        (0x0054 => pub(crate) prio21: ReadWrite<u32, PRIO21::Register>),
        /// Interrupt Source 22 Priority
        (0x0058 => pub(crate) prio22: ReadWrite<u32, PRIO22::Register>),
        /// Interrupt Source 23 Priority
        (0x005c => pub(crate) prio23: ReadWrite<u32, PRIO23::Register>),
        /// Interrupt Source 24 Priority
        (0x0060 => pub(crate) prio24: ReadWrite<u32, PRIO24::Register>),
        /// Interrupt Source 25 Priority
        (0x0064 => pub(crate) prio25: ReadWrite<u32, PRIO25::Register>),
        /// Interrupt Source 26 Priority
        (0x0068 => pub(crate) prio26: ReadWrite<u32, PRIO26::Register>),
        /// Interrupt Source 27 Priority
        (0x006c => pub(crate) prio27: ReadWrite<u32, PRIO27::Register>),
        /// Interrupt Source 28 Priority
        (0x0070 => pub(crate) prio28: ReadWrite<u32, PRIO28::Register>),
        /// Interrupt Source 29 Priority
        (0x0074 => pub(crate) prio29: ReadWrite<u32, PRIO29::Register>),
        /// Interrupt Source 30 Priority
        (0x0078 => pub(crate) prio30: ReadWrite<u32, PRIO30::Register>),
        /// Interrupt Source 31 Priority
        (0x007c => pub(crate) prio31: ReadWrite<u32, PRIO31::Register>),
        /// Interrupt Source 32 Priority
        (0x0080 => pub(crate) prio32: ReadWrite<u32, PRIO32::Register>),
        /// Interrupt Source 33 Priority
        (0x0084 => pub(crate) prio33: ReadWrite<u32, PRIO33::Register>),
        /// Interrupt Source 34 Priority
        (0x0088 => pub(crate) prio34: ReadWrite<u32, PRIO34::Register>),
        /// Interrupt Source 35 Priority
        (0x008c => pub(crate) prio35: ReadWrite<u32, PRIO35::Register>),
        /// Interrupt Source 36 Priority
        (0x0090 => pub(crate) prio36: ReadWrite<u32, PRIO36::Register>),
        /// Interrupt Source 37 Priority
        (0x0094 => pub(crate) prio37: ReadWrite<u32, PRIO37::Register>),
        /// Interrupt Source 38 Priority
        (0x0098 => pub(crate) prio38: ReadWrite<u32, PRIO38::Register>),
        /// Interrupt Source 39 Priority
        (0x009c => pub(crate) prio39: ReadWrite<u32, PRIO39::Register>),
        /// Interrupt Source 40 Priority
        (0x00a0 => pub(crate) prio40: ReadWrite<u32, PRIO40::Register>),
        /// Interrupt Source 41 Priority
        (0x00a4 => pub(crate) prio41: ReadWrite<u32, PRIO41::Register>),
        /// Interrupt Source 42 Priority
        (0x00a8 => pub(crate) prio42: ReadWrite<u32, PRIO42::Register>),
        /// Interrupt Source 43 Priority
        (0x00ac => pub(crate) prio43: ReadWrite<u32, PRIO43::Register>),
        /// Interrupt Source 44 Priority
        (0x00b0 => pub(crate) prio44: ReadWrite<u32, PRIO44::Register>),
        /// Interrupt Source 45 Priority
        (0x00b4 => pub(crate) prio45: ReadWrite<u32, PRIO45::Register>),
        /// Interrupt Source 46 Priority
        (0x00b8 => pub(crate) prio46: ReadWrite<u32, PRIO46::Register>),
        /// Interrupt Source 47 Priority
        (0x00bc => pub(crate) prio47: ReadWrite<u32, PRIO47::Register>),
        /// Interrupt Source 48 Priority
        (0x00c0 => pub(crate) prio48: ReadWrite<u32, PRIO48::Register>),
        /// Interrupt Source 49 Priority
        (0x00c4 => pub(crate) prio49: ReadWrite<u32, PRIO49::Register>),
        /// Interrupt Source 50 Priority
        (0x00c8 => pub(crate) prio50: ReadWrite<u32, PRIO50::Register>),
        /// Interrupt Source 51 Priority
        (0x00cc => pub(crate) prio51: ReadWrite<u32, PRIO51::Register>),
        /// Interrupt Source 52 Priority
        (0x00d0 => pub(crate) prio52: ReadWrite<u32, PRIO52::Register>),
        /// Interrupt Source 53 Priority
        (0x00d4 => pub(crate) prio53: ReadWrite<u32, PRIO53::Register>),
        /// Interrupt Source 54 Priority
        (0x00d8 => pub(crate) prio54: ReadWrite<u32, PRIO54::Register>),
        /// Interrupt Source 55 Priority
        (0x00dc => pub(crate) prio55: ReadWrite<u32, PRIO55::Register>),
        /// Interrupt Source 56 Priority
        (0x00e0 => pub(crate) prio56: ReadWrite<u32, PRIO56::Register>),
        /// Interrupt Source 57 Priority
        (0x00e4 => pub(crate) prio57: ReadWrite<u32, PRIO57::Register>),
        /// Interrupt Source 58 Priority
        (0x00e8 => pub(crate) prio58: ReadWrite<u32, PRIO58::Register>),
        /// Interrupt Source 59 Priority
        (0x00ec => pub(crate) prio59: ReadWrite<u32, PRIO59::Register>),
        /// Interrupt Source 60 Priority
        (0x00f0 => pub(crate) prio60: ReadWrite<u32, PRIO60::Register>),
        /// Interrupt Source 61 Priority
        (0x00f4 => pub(crate) prio61: ReadWrite<u32, PRIO61::Register>),
        /// Interrupt Source 62 Priority
        (0x00f8 => pub(crate) prio62: ReadWrite<u32, PRIO62::Register>),
        /// Interrupt Source 63 Priority
        (0x00fc => pub(crate) prio63: ReadWrite<u32, PRIO63::Register>),
        /// Interrupt Source 64 Priority
        (0x0100 => pub(crate) prio64: ReadWrite<u32, PRIO64::Register>),
        /// Interrupt Source 65 Priority
        (0x0104 => pub(crate) prio65: ReadWrite<u32, PRIO65::Register>),
        /// Interrupt Source 66 Priority
        (0x0108 => pub(crate) prio66: ReadWrite<u32, PRIO66::Register>),
        /// Interrupt Source 67 Priority
        (0x010c => pub(crate) prio67: ReadWrite<u32, PRIO67::Register>),
        /// Interrupt Source 68 Priority
        (0x0110 => pub(crate) prio68: ReadWrite<u32, PRIO68::Register>),
        /// Interrupt Source 69 Priority
        (0x0114 => pub(crate) prio69: ReadWrite<u32, PRIO69::Register>),
        /// Interrupt Source 70 Priority
        (0x0118 => pub(crate) prio70: ReadWrite<u32, PRIO70::Register>),
        /// Interrupt Source 71 Priority
        (0x011c => pub(crate) prio71: ReadWrite<u32, PRIO71::Register>),
        /// Interrupt Source 72 Priority
        (0x0120 => pub(crate) prio72: ReadWrite<u32, PRIO72::Register>),
        /// Interrupt Source 73 Priority
        (0x0124 => pub(crate) prio73: ReadWrite<u32, PRIO73::Register>),
        /// Interrupt Source 74 Priority
        (0x0128 => pub(crate) prio74: ReadWrite<u32, PRIO74::Register>),
        /// Interrupt Source 75 Priority
        (0x012c => pub(crate) prio75: ReadWrite<u32, PRIO75::Register>),
        /// Interrupt Source 76 Priority
        (0x0130 => pub(crate) prio76: ReadWrite<u32, PRIO76::Register>),
        /// Interrupt Source 77 Priority
        (0x0134 => pub(crate) prio77: ReadWrite<u32, PRIO77::Register>),
        /// Interrupt Source 78 Priority
        (0x0138 => pub(crate) prio78: ReadWrite<u32, PRIO78::Register>),
        /// Interrupt Source 79 Priority
        (0x013c => pub(crate) prio79: ReadWrite<u32, PRIO79::Register>),
        /// Interrupt Source 80 Priority
        (0x0140 => pub(crate) prio80: ReadWrite<u32, PRIO80::Register>),
        /// Interrupt Source 81 Priority
        (0x0144 => pub(crate) prio81: ReadWrite<u32, PRIO81::Register>),
        /// Interrupt Source 82 Priority
        (0x0148 => pub(crate) prio82: ReadWrite<u32, PRIO82::Register>),
        /// Interrupt Source 83 Priority
        (0x014c => pub(crate) prio83: ReadWrite<u32, PRIO83::Register>),
        /// Interrupt Source 84 Priority
        (0x0150 => pub(crate) prio84: ReadWrite<u32, PRIO84::Register>),
        /// Interrupt Source 85 Priority
        (0x0154 => pub(crate) prio85: ReadWrite<u32, PRIO85::Register>),
        /// Interrupt Source 86 Priority
        (0x0158 => pub(crate) prio86: ReadWrite<u32, PRIO86::Register>),
        /// Interrupt Source 87 Priority
        (0x015c => pub(crate) prio87: ReadWrite<u32, PRIO87::Register>),
        /// Interrupt Source 88 Priority
        (0x0160 => pub(crate) prio88: ReadWrite<u32, PRIO88::Register>),
        /// Interrupt Source 89 Priority
        (0x0164 => pub(crate) prio89: ReadWrite<u32, PRIO89::Register>),
        /// Interrupt Source 90 Priority
        (0x0168 => pub(crate) prio90: ReadWrite<u32, PRIO90::Register>),
        /// Interrupt Source 91 Priority
        (0x016c => pub(crate) prio91: ReadWrite<u32, PRIO91::Register>),
        /// Interrupt Source 92 Priority
        (0x0170 => pub(crate) prio92: ReadWrite<u32, PRIO92::Register>),
        /// Interrupt Source 93 Priority
        (0x0174 => pub(crate) prio93: ReadWrite<u32, PRIO93::Register>),
        /// Interrupt Source 94 Priority
        (0x0178 => pub(crate) prio94: ReadWrite<u32, PRIO94::Register>),
        /// Interrupt Source 95 Priority
        (0x017c => pub(crate) prio95: ReadWrite<u32, PRIO95::Register>),
        /// Interrupt Source 96 Priority
        (0x0180 => pub(crate) prio96: ReadWrite<u32, PRIO96::Register>),
        /// Interrupt Source 97 Priority
        (0x0184 => pub(crate) prio97: ReadWrite<u32, PRIO97::Register>),
        /// Interrupt Source 98 Priority
        (0x0188 => pub(crate) prio98: ReadWrite<u32, PRIO98::Register>),
        /// Interrupt Source 99 Priority
        (0x018c => pub(crate) prio99: ReadWrite<u32, PRIO99::Register>),
        /// Interrupt Source 100 Priority
        (0x0190 => pub(crate) prio100: ReadWrite<u32, PRIO100::Register>),
        /// Interrupt Source 101 Priority
        (0x0194 => pub(crate) prio101: ReadWrite<u32, PRIO101::Register>),
        /// Interrupt Source 102 Priority
        (0x0198 => pub(crate) prio102: ReadWrite<u32, PRIO102::Register>),
        /// Interrupt Source 103 Priority
        (0x019c => pub(crate) prio103: ReadWrite<u32, PRIO103::Register>),
        /// Interrupt Source 104 Priority
        (0x01a0 => pub(crate) prio104: ReadWrite<u32, PRIO104::Register>),
        /// Interrupt Source 105 Priority
        (0x01a4 => pub(crate) prio105: ReadWrite<u32, PRIO105::Register>),
        /// Interrupt Source 106 Priority
        (0x01a8 => pub(crate) prio106: ReadWrite<u32, PRIO106::Register>),
        /// Interrupt Source 107 Priority
        (0x01ac => pub(crate) prio107: ReadWrite<u32, PRIO107::Register>),
        /// Interrupt Source 108 Priority
        (0x01b0 => pub(crate) prio108: ReadWrite<u32, PRIO108::Register>),
        /// Interrupt Source 109 Priority
        (0x01b4 => pub(crate) prio109: ReadWrite<u32, PRIO109::Register>),
        /// Interrupt Source 110 Priority
        (0x01b8 => pub(crate) prio110: ReadWrite<u32, PRIO110::Register>),
        /// Interrupt Source 111 Priority
        (0x01bc => pub(crate) prio111: ReadWrite<u32, PRIO111::Register>),
        /// Interrupt Source 112 Priority
        (0x01c0 => pub(crate) prio112: ReadWrite<u32, PRIO112::Register>),
        /// Interrupt Source 113 Priority
        (0x01c4 => pub(crate) prio113: ReadWrite<u32, PRIO113::Register>),
        /// Interrupt Source 114 Priority
        (0x01c8 => pub(crate) prio114: ReadWrite<u32, PRIO114::Register>),
        /// Interrupt Source 115 Priority
        (0x01cc => pub(crate) prio115: ReadWrite<u32, PRIO115::Register>),
        /// Interrupt Source 116 Priority
        (0x01d0 => pub(crate) prio116: ReadWrite<u32, PRIO116::Register>),
        /// Interrupt Source 117 Priority
        (0x01d4 => pub(crate) prio117: ReadWrite<u32, PRIO117::Register>),
        /// Interrupt Source 118 Priority
        (0x01d8 => pub(crate) prio118: ReadWrite<u32, PRIO118::Register>),
        /// Interrupt Source 119 Priority
        (0x01dc => pub(crate) prio119: ReadWrite<u32, PRIO119::Register>),
        /// Interrupt Source 120 Priority
        (0x01e0 => pub(crate) prio120: ReadWrite<u32, PRIO120::Register>),
        /// Interrupt Source 121 Priority
        (0x01e4 => pub(crate) prio121: ReadWrite<u32, PRIO121::Register>),
        /// Interrupt Source 122 Priority
        (0x01e8 => pub(crate) prio122: ReadWrite<u32, PRIO122::Register>),
        /// Interrupt Source 123 Priority
        (0x01ec => pub(crate) prio123: ReadWrite<u32, PRIO123::Register>),
        /// Interrupt Source 124 Priority
        (0x01f0 => pub(crate) prio124: ReadWrite<u32, PRIO124::Register>),
        /// Interrupt Source 125 Priority
        (0x01f4 => pub(crate) prio125: ReadWrite<u32, PRIO125::Register>),
        /// Interrupt Source 126 Priority
        (0x01f8 => pub(crate) prio126: ReadWrite<u32, PRIO126::Register>),
        /// Interrupt Source 127 Priority
        (0x01fc => pub(crate) prio127: ReadWrite<u32, PRIO127::Register>),
        /// Interrupt Source 128 Priority
        (0x0200 => pub(crate) prio128: ReadWrite<u32, PRIO128::Register>),
        /// Interrupt Source 129 Priority
        (0x0204 => pub(crate) prio129: ReadWrite<u32, PRIO129::Register>),
        /// Interrupt Source 130 Priority
        (0x0208 => pub(crate) prio130: ReadWrite<u32, PRIO130::Register>),
        /// Interrupt Source 131 Priority
        (0x020c => pub(crate) prio131: ReadWrite<u32, PRIO131::Register>),
        /// Interrupt Source 132 Priority
        (0x0210 => pub(crate) prio132: ReadWrite<u32, PRIO132::Register>),
        /// Interrupt Source 133 Priority
        (0x0214 => pub(crate) prio133: ReadWrite<u32, PRIO133::Register>),
        /// Interrupt Source 134 Priority
        (0x0218 => pub(crate) prio134: ReadWrite<u32, PRIO134::Register>),
        /// Interrupt Source 135 Priority
        (0x021c => pub(crate) prio135: ReadWrite<u32, PRIO135::Register>),
        /// Interrupt Source 136 Priority
        (0x0220 => pub(crate) prio136: ReadWrite<u32, PRIO136::Register>),
        /// Interrupt Source 137 Priority
        (0x0224 => pub(crate) prio137: ReadWrite<u32, PRIO137::Register>),
        /// Interrupt Source 138 Priority
        (0x0228 => pub(crate) prio138: ReadWrite<u32, PRIO138::Register>),
        /// Interrupt Source 139 Priority
        (0x022c => pub(crate) prio139: ReadWrite<u32, PRIO139::Register>),
        /// Interrupt Source 140 Priority
        (0x0230 => pub(crate) prio140: ReadWrite<u32, PRIO140::Register>),
        /// Interrupt Source 141 Priority
        (0x0234 => pub(crate) prio141: ReadWrite<u32, PRIO141::Register>),
        /// Interrupt Source 142 Priority
        (0x0238 => pub(crate) prio142: ReadWrite<u32, PRIO142::Register>),
        /// Interrupt Source 143 Priority
        (0x023c => pub(crate) prio143: ReadWrite<u32, PRIO143::Register>),
        /// Interrupt Source 144 Priority
        (0x0240 => pub(crate) prio144: ReadWrite<u32, PRIO144::Register>),
        /// Interrupt Source 145 Priority
        (0x0244 => pub(crate) prio145: ReadWrite<u32, PRIO145::Register>),
        /// Interrupt Source 146 Priority
        (0x0248 => pub(crate) prio146: ReadWrite<u32, PRIO146::Register>),
        /// Interrupt Source 147 Priority
        (0x024c => pub(crate) prio147: ReadWrite<u32, PRIO147::Register>),
        /// Interrupt Source 148 Priority
        (0x0250 => pub(crate) prio148: ReadWrite<u32, PRIO148::Register>),
        /// Interrupt Source 149 Priority
        (0x0254 => pub(crate) prio149: ReadWrite<u32, PRIO149::Register>),
        /// Interrupt Source 150 Priority
        (0x0258 => pub(crate) prio150: ReadWrite<u32, PRIO150::Register>),
        /// Interrupt Source 151 Priority
        (0x025c => pub(crate) prio151: ReadWrite<u32, PRIO151::Register>),
        /// Interrupt Source 152 Priority
        (0x0260 => pub(crate) prio152: ReadWrite<u32, PRIO152::Register>),
        /// Interrupt Source 153 Priority
        (0x0264 => pub(crate) prio153: ReadWrite<u32, PRIO153::Register>),
        /// Interrupt Source 154 Priority
        (0x0268 => pub(crate) prio154: ReadWrite<u32, PRIO154::Register>),
        /// Interrupt Source 155 Priority
        (0x026c => pub(crate) prio155: ReadWrite<u32, PRIO155::Register>),
        /// Interrupt Source 156 Priority
        (0x0270 => pub(crate) prio156: ReadWrite<u32, PRIO156::Register>),
        /// Interrupt Source 157 Priority
        (0x0274 => pub(crate) prio157: ReadWrite<u32, PRIO157::Register>),
        /// Interrupt Source 158 Priority
        (0x0278 => pub(crate) prio158: ReadWrite<u32, PRIO158::Register>),
        /// Interrupt Source 159 Priority
        (0x027c => pub(crate) prio159: ReadWrite<u32, PRIO159::Register>),
        /// Interrupt Source 160 Priority
        (0x0280 => pub(crate) prio160: ReadWrite<u32, PRIO160::Register>),
        /// Interrupt Source 161 Priority
        (0x0284 => pub(crate) prio161: ReadWrite<u32, PRIO161::Register>),
        /// Interrupt Source 162 Priority
        (0x0288 => pub(crate) prio162: ReadWrite<u32, PRIO162::Register>),
        /// Interrupt Source 163 Priority
        (0x028c => pub(crate) prio163: ReadWrite<u32, PRIO163::Register>),
        /// Interrupt Source 164 Priority
        (0x0290 => pub(crate) prio164: ReadWrite<u32, PRIO164::Register>),
        /// Interrupt Source 165 Priority
        (0x0294 => pub(crate) prio165: ReadWrite<u32, PRIO165::Register>),
        /// Interrupt Source 166 Priority
        (0x0298 => pub(crate) prio166: ReadWrite<u32, PRIO166::Register>),
        /// Interrupt Source 167 Priority
        (0x029c => pub(crate) prio167: ReadWrite<u32, PRIO167::Register>),
        /// Interrupt Source 168 Priority
        (0x02a0 => pub(crate) prio168: ReadWrite<u32, PRIO168::Register>),
        /// Interrupt Source 169 Priority
        (0x02a4 => pub(crate) prio169: ReadWrite<u32, PRIO169::Register>),
        /// Interrupt Source 170 Priority
        (0x02a8 => pub(crate) prio170: ReadWrite<u32, PRIO170::Register>),
        /// Interrupt Source 171 Priority
        (0x02ac => pub(crate) prio171: ReadWrite<u32, PRIO171::Register>),
        /// Interrupt Source 172 Priority
        (0x02b0 => pub(crate) prio172: ReadWrite<u32, PRIO172::Register>),
        /// Interrupt Source 173 Priority
        (0x02b4 => pub(crate) prio173: ReadWrite<u32, PRIO173::Register>),
        /// Interrupt Source 174 Priority
        (0x02b8 => pub(crate) prio174: ReadWrite<u32, PRIO174::Register>),
        /// Interrupt Source 175 Priority
        (0x02bc => pub(crate) prio175: ReadWrite<u32, PRIO175::Register>),
        /// Interrupt Source 176 Priority
        (0x02c0 => pub(crate) prio176: ReadWrite<u32, PRIO176::Register>),
        /// Interrupt Source 177 Priority
        (0x02c4 => pub(crate) prio177: ReadWrite<u32, PRIO177::Register>),
        /// Interrupt Source 178 Priority
        (0x02c8 => pub(crate) prio178: ReadWrite<u32, PRIO178::Register>),
        /// Interrupt Source 179 Priority
        (0x02cc => pub(crate) prio179: ReadWrite<u32, PRIO179::Register>),
        /// Interrupt Source 180 Priority
        (0x02d0 => pub(crate) prio180: ReadWrite<u32, PRIO180::Register>),
        /// Interrupt Source 181 Priority
        (0x02d4 => pub(crate) prio181: ReadWrite<u32, PRIO181::Register>),
        /// Interrupt Source 182 Priority
        (0x02d8 => pub(crate) prio182: ReadWrite<u32, PRIO182::Register>),
        /// Interrupt Source 183 Priority
        (0x02dc => pub(crate) prio183: ReadWrite<u32, PRIO183::Register>),
        /// Interrupt Source 184 Priority
        (0x02e0 => pub(crate) prio184: ReadWrite<u32, PRIO184::Register>),
        (0x02e4 => _reserved1),
        /// Interrupt Pending
        (0x1000 => pub(crate) ip: [ReadWrite<u32, IP::Register>; 6]),
        (0x1018 => _reserved2),
        /// Interrupt Enable for Target 0
        (0x2000 => pub(crate) ie0: [ReadWrite<u32, IE0::Register>; 6]),
        (0x2018 => _reserved3),
        /// Threshold of priority for Target 0
        (0x200000 => pub(crate) threshold0: ReadWrite<u32, THRESHOLD0::Register>),
        /// Claim interrupt by read, complete interrupt by write for Target 0.
        (0x200004 => pub(crate) cc0: ReadWrite<u32, CC0::Register>),
        (0x200008 => _reserved4),
        /// msip for Hart 0.
        (0x4000000 => pub(crate) msip0: ReadWrite<u32, MSIP0::Register>),
        (0x4000004 => _reserved5),
        /// Alert Test Register.
        (0x4004000 => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        (0x4004004 => @END),
    }
}

register_bitfields![u32,
    pub(crate) PRIO0 [
        PRIO0 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO1 [
        PRIO1 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO2 [
        PRIO2 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO3 [
        PRIO3 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO4 [
        PRIO4 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO5 [
        PRIO5 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO6 [
        PRIO6 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO7 [
        PRIO7 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO8 [
        PRIO8 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO9 [
        PRIO9 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO10 [
        PRIO10 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO11 [
        PRIO11 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO12 [
        PRIO12 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO13 [
        PRIO13 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO14 [
        PRIO14 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO15 [
        PRIO15 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO16 [
        PRIO16 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO17 [
        PRIO17 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO18 [
        PRIO18 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO19 [
        PRIO19 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO20 [
        PRIO20 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO21 [
        PRIO21 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO22 [
        PRIO22 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO23 [
        PRIO23 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO24 [
        PRIO24 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO25 [
        PRIO25 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO26 [
        PRIO26 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO27 [
        PRIO27 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO28 [
        PRIO28 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO29 [
        PRIO29 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO30 [
        PRIO30 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO31 [
        PRIO31 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO32 [
        PRIO32 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO33 [
        PRIO33 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO34 [
        PRIO34 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO35 [
        PRIO35 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO36 [
        PRIO36 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO37 [
        PRIO37 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO38 [
        PRIO38 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO39 [
        PRIO39 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO40 [
        PRIO40 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO41 [
        PRIO41 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO42 [
        PRIO42 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO43 [
        PRIO43 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO44 [
        PRIO44 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO45 [
        PRIO45 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO46 [
        PRIO46 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO47 [
        PRIO47 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO48 [
        PRIO48 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO49 [
        PRIO49 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO50 [
        PRIO50 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO51 [
        PRIO51 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO52 [
        PRIO52 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO53 [
        PRIO53 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO54 [
        PRIO54 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO55 [
        PRIO55 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO56 [
        PRIO56 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO57 [
        PRIO57 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO58 [
        PRIO58 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO59 [
        PRIO59 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO60 [
        PRIO60 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO61 [
        PRIO61 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO62 [
        PRIO62 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO63 [
        PRIO63 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO64 [
        PRIO64 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO65 [
        PRIO65 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO66 [
        PRIO66 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO67 [
        PRIO67 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO68 [
        PRIO68 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO69 [
        PRIO69 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO70 [
        PRIO70 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO71 [
        PRIO71 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO72 [
        PRIO72 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO73 [
        PRIO73 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO74 [
        PRIO74 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO75 [
        PRIO75 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO76 [
        PRIO76 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO77 [
        PRIO77 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO78 [
        PRIO78 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO79 [
        PRIO79 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO80 [
        PRIO80 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO81 [
        PRIO81 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO82 [
        PRIO82 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO83 [
        PRIO83 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO84 [
        PRIO84 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO85 [
        PRIO85 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO86 [
        PRIO86 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO87 [
        PRIO87 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO88 [
        PRIO88 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO89 [
        PRIO89 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO90 [
        PRIO90 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO91 [
        PRIO91 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO92 [
        PRIO92 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO93 [
        PRIO93 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO94 [
        PRIO94 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO95 [
        PRIO95 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO96 [
        PRIO96 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO97 [
        PRIO97 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO98 [
        PRIO98 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO99 [
        PRIO99 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO100 [
        PRIO100 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO101 [
        PRIO101 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO102 [
        PRIO102 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO103 [
        PRIO103 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO104 [
        PRIO104 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO105 [
        PRIO105 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO106 [
        PRIO106 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO107 [
        PRIO107 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO108 [
        PRIO108 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO109 [
        PRIO109 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO110 [
        PRIO110 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO111 [
        PRIO111 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO112 [
        PRIO112 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO113 [
        PRIO113 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO114 [
        PRIO114 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO115 [
        PRIO115 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO116 [
        PRIO116 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO117 [
        PRIO117 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO118 [
        PRIO118 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO119 [
        PRIO119 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO120 [
        PRIO120 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO121 [
        PRIO121 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO122 [
        PRIO122 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO123 [
        PRIO123 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO124 [
        PRIO124 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO125 [
        PRIO125 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO126 [
        PRIO126 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO127 [
        PRIO127 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO128 [
        PRIO128 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO129 [
        PRIO129 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO130 [
        PRIO130 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO131 [
        PRIO131 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO132 [
        PRIO132 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO133 [
        PRIO133 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO134 [
        PRIO134 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO135 [
        PRIO135 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO136 [
        PRIO136 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO137 [
        PRIO137 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO138 [
        PRIO138 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO139 [
        PRIO139 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO140 [
        PRIO140 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO141 [
        PRIO141 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO142 [
        PRIO142 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO143 [
        PRIO143 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO144 [
        PRIO144 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO145 [
        PRIO145 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO146 [
        PRIO146 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO147 [
        PRIO147 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO148 [
        PRIO148 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO149 [
        PRIO149 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO150 [
        PRIO150 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO151 [
        PRIO151 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO152 [
        PRIO152 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO153 [
        PRIO153 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO154 [
        PRIO154 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO155 [
        PRIO155 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO156 [
        PRIO156 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO157 [
        PRIO157 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO158 [
        PRIO158 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO159 [
        PRIO159 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO160 [
        PRIO160 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO161 [
        PRIO161 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO162 [
        PRIO162 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO163 [
        PRIO163 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO164 [
        PRIO164 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO165 [
        PRIO165 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO166 [
        PRIO166 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO167 [
        PRIO167 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO168 [
        PRIO168 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO169 [
        PRIO169 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO170 [
        PRIO170 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO171 [
        PRIO171 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO172 [
        PRIO172 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO173 [
        PRIO173 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO174 [
        PRIO174 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO175 [
        PRIO175 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO176 [
        PRIO176 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO177 [
        PRIO177 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO178 [
        PRIO178 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO179 [
        PRIO179 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO180 [
        PRIO180 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO181 [
        PRIO181 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO182 [
        PRIO182 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO183 [
        PRIO183 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) PRIO184 [
        PRIO184 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) IP [
        P_0 OFFSET(0) NUMBITS(1) [],
        P_1 OFFSET(1) NUMBITS(1) [],
        P_2 OFFSET(2) NUMBITS(1) [],
        P_3 OFFSET(3) NUMBITS(1) [],
        P_4 OFFSET(4) NUMBITS(1) [],
        P_5 OFFSET(5) NUMBITS(1) [],
        P_6 OFFSET(6) NUMBITS(1) [],
        P_7 OFFSET(7) NUMBITS(1) [],
        P_8 OFFSET(8) NUMBITS(1) [],
        P_9 OFFSET(9) NUMBITS(1) [],
        P_10 OFFSET(10) NUMBITS(1) [],
        P_11 OFFSET(11) NUMBITS(1) [],
        P_12 OFFSET(12) NUMBITS(1) [],
        P_13 OFFSET(13) NUMBITS(1) [],
        P_14 OFFSET(14) NUMBITS(1) [],
        P_15 OFFSET(15) NUMBITS(1) [],
        P_16 OFFSET(16) NUMBITS(1) [],
        P_17 OFFSET(17) NUMBITS(1) [],
        P_18 OFFSET(18) NUMBITS(1) [],
        P_19 OFFSET(19) NUMBITS(1) [],
        P_20 OFFSET(20) NUMBITS(1) [],
        P_21 OFFSET(21) NUMBITS(1) [],
        P_22 OFFSET(22) NUMBITS(1) [],
        P_23 OFFSET(23) NUMBITS(1) [],
        P_24 OFFSET(24) NUMBITS(1) [],
        P_25 OFFSET(25) NUMBITS(1) [],
        P_26 OFFSET(26) NUMBITS(1) [],
        P_27 OFFSET(27) NUMBITS(1) [],
        P_28 OFFSET(28) NUMBITS(1) [],
        P_29 OFFSET(29) NUMBITS(1) [],
        P_30 OFFSET(30) NUMBITS(1) [],
        P_31 OFFSET(31) NUMBITS(1) [],
    ],
    pub(crate) IE0 [
        E_0 OFFSET(0) NUMBITS(1) [],
        E_1 OFFSET(1) NUMBITS(1) [],
        E_2 OFFSET(2) NUMBITS(1) [],
        E_3 OFFSET(3) NUMBITS(1) [],
        E_4 OFFSET(4) NUMBITS(1) [],
        E_5 OFFSET(5) NUMBITS(1) [],
        E_6 OFFSET(6) NUMBITS(1) [],
        E_7 OFFSET(7) NUMBITS(1) [],
        E_8 OFFSET(8) NUMBITS(1) [],
        E_9 OFFSET(9) NUMBITS(1) [],
        E_10 OFFSET(10) NUMBITS(1) [],
        E_11 OFFSET(11) NUMBITS(1) [],
        E_12 OFFSET(12) NUMBITS(1) [],
        E_13 OFFSET(13) NUMBITS(1) [],
        E_14 OFFSET(14) NUMBITS(1) [],
        E_15 OFFSET(15) NUMBITS(1) [],
        E_16 OFFSET(16) NUMBITS(1) [],
        E_17 OFFSET(17) NUMBITS(1) [],
        E_18 OFFSET(18) NUMBITS(1) [],
        E_19 OFFSET(19) NUMBITS(1) [],
        E_20 OFFSET(20) NUMBITS(1) [],
        E_21 OFFSET(21) NUMBITS(1) [],
        E_22 OFFSET(22) NUMBITS(1) [],
        E_23 OFFSET(23) NUMBITS(1) [],
        E_24 OFFSET(24) NUMBITS(1) [],
        E_25 OFFSET(25) NUMBITS(1) [],
        E_26 OFFSET(26) NUMBITS(1) [],
        E_27 OFFSET(27) NUMBITS(1) [],
        E_28 OFFSET(28) NUMBITS(1) [],
        E_29 OFFSET(29) NUMBITS(1) [],
        E_30 OFFSET(30) NUMBITS(1) [],
        E_31 OFFSET(31) NUMBITS(1) [],
    ],
    pub(crate) THRESHOLD0 [
        THRESHOLD0 OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) CC0 [
        CC0 OFFSET(0) NUMBITS(8) [],
    ],
    pub(crate) MSIP0 [
        MSIP0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
];

// End generated register constants for rv_plic
