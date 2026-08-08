// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// Adapted from embassy-rs/stm32-data-generated/stm32-metapac/src/peripherals/rcc_u5.rs

use super::hertz;

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum MsiRange {
    /// range 0 around 48 MHz
    Range48mhz = 0x0,
    /// range 1 around 24 MHz
    Range24mhz = 0x01,
    /// range 2 around 16 MHz
    Range16mhz = 0x02,
    /// range 3 around 12 MHz
    Range12mhz = 0x03,
    /// range 4 around 4 MHz (reset value)
    Range4mhz = 0x04,
    /// range 5 around 2 MHz
    Range2mhz = 0x05,
    /// range 6 around 1.33 MHz
    Range133mhz = 0x06,
    /// range 7 around 1 MHz
    Range1mhz = 0x07,
    /// range 8 around 3.072 MHz
    Range3072mhz = 0x08,
    /// range 9 around 1.536 MHz
    Range1536mhz = 0x09,
    /// range 10 around 1.024 MHz
    Range1024mhz = 0x0a,
    /// range 11 around 768 kHz
    Range768khz = 0x0b,
    /// range 12 around 400 kHz
    Range400khz = 0x0c,
    /// range 13 around 200 kHz
    Range200khz = 0x0d,
    /// range 14 around 133 kHz
    Range133khz = 0x0e,
    /// range 15 around 100 kHz
    Range100khz = 0x0f,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum PllSource {
    /// No clock sent to PLL
    Disable = 0x0,
    /// MSIS clock selected as PLL clock entry
    Msis = 0x01,
    /// HSI clock selected as PLL clock entry
    Hsi = 0x02,
    /// HSE clock selected as PLL clock entry
    Hse = 0x03,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum Sysclk {
    /// MSIS selected as system clock
    Msis = 0x0,
    /// HSI selected as system clock
    Hsi = 0x01,
    /// HSE selected as system clock
    Hse = 0x02,
    /// PLL pll1_r_ck selected as system clock
    Pll1R = 0x03,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum PllPreDiv {
    Div1 = 0x0,
    Div2 = 0x01,
    Div3 = 0x02,
    Div4 = 0x03,
    Div5 = 0x04,
    Div6 = 0x05,
    Div7 = 0x06,
    Div8 = 0x07,
    Div9 = 0x08,
    Div10 = 0x09,
    Div11 = 0x0a,
    Div12 = 0x0b,
    Div13 = 0x0c,
    Div14 = 0x0d,
    Div15 = 0x0e,
    Div16 = 0x0f,
}
impl hertz::Prescaler for PllPreDiv {
    fn num(&self) -> u32 {
        *self as u32 + 1
    }
    fn denom(&self) -> u32 {
        1
    }
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum PllMul {
    Mul4 = 0x03,
    Mul5 = 0x04,
    Mul6 = 0x05,
    Mul7 = 0x06,
    Mul8 = 0x07,
    Mul9 = 0x08,
    Mul10 = 0x09,
    Mul11 = 0x0a,
    Mul12 = 0x0b,
    Mul13 = 0x0c,
    Mul14 = 0x0d,
    Mul15 = 0x0e,
    Mul16 = 0x0f,
    Mul17 = 0x10,
    Mul18 = 0x11,
    Mul19 = 0x12,
    Mul20 = 0x13,
    Mul21 = 0x14,
    Mul22 = 0x15,
    Mul23 = 0x16,
    Mul24 = 0x17,
    Mul25 = 0x18,
    Mul26 = 0x19,
    Mul27 = 0x1a,
    Mul28 = 0x1b,
    Mul29 = 0x1c,
    Mul30 = 0x1d,
    Mul31 = 0x1e,
    Mul32 = 0x1f,
    Mul33 = 0x20,
    Mul34 = 0x21,
    Mul35 = 0x22,
    Mul36 = 0x23,
    Mul37 = 0x24,
    Mul38 = 0x25,
    Mul39 = 0x26,
    Mul40 = 0x27,
    Mul41 = 0x28,
    Mul42 = 0x29,
    Mul43 = 0x2a,
    Mul44 = 0x2b,
    Mul45 = 0x2c,
    Mul46 = 0x2d,
    Mul47 = 0x2e,
    Mul48 = 0x2f,
    Mul49 = 0x30,
    Mul50 = 0x31,
    Mul51 = 0x32,
    Mul52 = 0x33,
    Mul53 = 0x34,
    Mul54 = 0x35,
    Mul55 = 0x36,
    Mul56 = 0x37,
    Mul57 = 0x38,
    Mul58 = 0x39,
    Mul59 = 0x3a,
    Mul60 = 0x3b,
    Mul61 = 0x3c,
    Mul62 = 0x3d,
    Mul63 = 0x3e,
    Mul64 = 0x3f,
    Mul65 = 0x40,
    Mul66 = 0x41,
    Mul67 = 0x42,
    Mul68 = 0x43,
    Mul69 = 0x44,
    Mul70 = 0x45,
    Mul71 = 0x46,
    Mul72 = 0x47,
    Mul73 = 0x48,
    Mul74 = 0x49,
    Mul75 = 0x4a,
    Mul76 = 0x4b,
    Mul77 = 0x4c,
    Mul78 = 0x4d,
    Mul79 = 0x4e,
    Mul80 = 0x4f,
    Mul81 = 0x50,
    Mul82 = 0x51,
    Mul83 = 0x52,
    Mul84 = 0x53,
    Mul85 = 0x54,
    Mul86 = 0x55,
    Mul87 = 0x56,
    Mul88 = 0x57,
    Mul89 = 0x58,
    Mul90 = 0x59,
    Mul91 = 0x5a,
    Mul92 = 0x5b,
    Mul93 = 0x5c,
    Mul94 = 0x5d,
    Mul95 = 0x5e,
    Mul96 = 0x5f,
    Mul97 = 0x60,
    Mul98 = 0x61,
    Mul99 = 0x62,
    Mul100 = 0x63,
    Mul101 = 0x64,
    Mul102 = 0x65,
    Mul103 = 0x66,
    Mul104 = 0x67,
    Mul105 = 0x68,
    Mul106 = 0x69,
    Mul107 = 0x6a,
    Mul108 = 0x6b,
    Mul109 = 0x6c,
    Mul110 = 0x6d,
    Mul111 = 0x6e,
    Mul112 = 0x6f,
    Mul113 = 0x70,
    Mul114 = 0x71,
    Mul115 = 0x72,
    Mul116 = 0x73,
    Mul117 = 0x74,
    Mul118 = 0x75,
    Mul119 = 0x76,
    Mul120 = 0x77,
    Mul121 = 0x78,
    Mul122 = 0x79,
    Mul123 = 0x7a,
    Mul124 = 0x7b,
    Mul125 = 0x7c,
    Mul126 = 0x7d,
    Mul127 = 0x7e,
    Mul128 = 0x7f,
    Mul129 = 0x80,
    Mul130 = 0x81,
    Mul131 = 0x82,
    Mul132 = 0x83,
    Mul133 = 0x84,
    Mul134 = 0x85,
    Mul135 = 0x86,
    Mul136 = 0x87,
    Mul137 = 0x88,
    Mul138 = 0x89,
    Mul139 = 0x8a,
    Mul140 = 0x8b,
    Mul141 = 0x8c,
    Mul142 = 0x8d,
    Mul143 = 0x8e,
    Mul144 = 0x8f,
    Mul145 = 0x90,
    Mul146 = 0x91,
    Mul147 = 0x92,
    Mul148 = 0x93,
    Mul149 = 0x94,
    Mul150 = 0x95,
    Mul151 = 0x96,
    Mul152 = 0x97,
    Mul153 = 0x98,
    Mul154 = 0x99,
    Mul155 = 0x9a,
    Mul156 = 0x9b,
    Mul157 = 0x9c,
    Mul158 = 0x9d,
    Mul159 = 0x9e,
    Mul160 = 0x9f,
    Mul161 = 0xa0,
    Mul162 = 0xa1,
    Mul163 = 0xa2,
    Mul164 = 0xa3,
    Mul165 = 0xa4,
    Mul166 = 0xa5,
    Mul167 = 0xa6,
    Mul168 = 0xa7,
    Mul169 = 0xa8,
    Mul170 = 0xa9,
    Mul171 = 0xaa,
    Mul172 = 0xab,
    Mul173 = 0xac,
    Mul174 = 0xad,
    Mul175 = 0xae,
    Mul176 = 0xaf,
    Mul177 = 0xb0,
    Mul178 = 0xb1,
    Mul179 = 0xb2,
    Mul180 = 0xb3,
    Mul181 = 0xb4,
    Mul182 = 0xb5,
    Mul183 = 0xb6,
    Mul184 = 0xb7,
    Mul185 = 0xb8,
    Mul186 = 0xb9,
    Mul187 = 0xba,
    Mul188 = 0xbb,
    Mul189 = 0xbc,
    Mul190 = 0xbd,
    Mul191 = 0xbe,
    Mul192 = 0xbf,
    Mul193 = 0xc0,
    Mul194 = 0xc1,
    Mul195 = 0xc2,
    Mul196 = 0xc3,
    Mul197 = 0xc4,
    Mul198 = 0xc5,
    Mul199 = 0xc6,
    Mul200 = 0xc7,
    Mul201 = 0xc8,
    Mul202 = 0xc9,
    Mul203 = 0xca,
    Mul204 = 0xcb,
    Mul205 = 0xcc,
    Mul206 = 0xcd,
    Mul207 = 0xce,
    Mul208 = 0xcf,
    Mul209 = 0xd0,
    Mul210 = 0xd1,
    Mul211 = 0xd2,
    Mul212 = 0xd3,
    Mul213 = 0xd4,
    Mul214 = 0xd5,
    Mul215 = 0xd6,
    Mul216 = 0xd7,
    Mul217 = 0xd8,
    Mul218 = 0xd9,
    Mul219 = 0xda,
    Mul220 = 0xdb,
    Mul221 = 0xdc,
    Mul222 = 0xdd,
    Mul223 = 0xde,
    Mul224 = 0xdf,
    Mul225 = 0xe0,
    Mul226 = 0xe1,
    Mul227 = 0xe2,
    Mul228 = 0xe3,
    Mul229 = 0xe4,
    Mul230 = 0xe5,
    Mul231 = 0xe6,
    Mul232 = 0xe7,
    Mul233 = 0xe8,
    Mul234 = 0xe9,
    Mul235 = 0xea,
    Mul236 = 0xeb,
    Mul237 = 0xec,
    Mul238 = 0xed,
    Mul239 = 0xee,
    Mul240 = 0xef,
    Mul241 = 0xf0,
    Mul242 = 0xf1,
    Mul243 = 0xf2,
    Mul244 = 0xf3,
    Mul245 = 0xf4,
    Mul246 = 0xf5,
    Mul247 = 0xf6,
    Mul248 = 0xf7,
    Mul249 = 0xf8,
    Mul250 = 0xf9,
    Mul251 = 0xfa,
    Mul252 = 0xfb,
    Mul253 = 0xfc,
    Mul254 = 0xfd,
    Mul255 = 0xfe,
    Mul256 = 0xff,
    Mul257 = 0x0100,
    Mul258 = 0x0101,
    Mul259 = 0x0102,
    Mul260 = 0x0103,
    Mul261 = 0x0104,
    Mul262 = 0x0105,
    Mul263 = 0x0106,
    Mul264 = 0x0107,
    Mul265 = 0x0108,
    Mul266 = 0x0109,
    Mul267 = 0x010a,
    Mul268 = 0x010b,
    Mul269 = 0x010c,
    Mul270 = 0x010d,
    Mul271 = 0x010e,
    Mul272 = 0x010f,
    Mul273 = 0x0110,
    Mul274 = 0x0111,
    Mul275 = 0x0112,
    Mul276 = 0x0113,
    Mul277 = 0x0114,
    Mul278 = 0x0115,
    Mul279 = 0x0116,
    Mul280 = 0x0117,
    Mul281 = 0x0118,
    Mul282 = 0x0119,
    Mul283 = 0x011a,
    Mul284 = 0x011b,
    Mul285 = 0x011c,
    Mul286 = 0x011d,
    Mul287 = 0x011e,
    Mul288 = 0x011f,
    Mul289 = 0x0120,
    Mul290 = 0x0121,
    Mul291 = 0x0122,
    Mul292 = 0x0123,
    Mul293 = 0x0124,
    Mul294 = 0x0125,
    Mul295 = 0x0126,
    Mul296 = 0x0127,
    Mul297 = 0x0128,
    Mul298 = 0x0129,
    Mul299 = 0x012a,
    Mul300 = 0x012b,
    Mul301 = 0x012c,
    Mul302 = 0x012d,
    Mul303 = 0x012e,
    Mul304 = 0x012f,
    Mul305 = 0x0130,
    Mul306 = 0x0131,
    Mul307 = 0x0132,
    Mul308 = 0x0133,
    Mul309 = 0x0134,
    Mul310 = 0x0135,
    Mul311 = 0x0136,
    Mul312 = 0x0137,
    Mul313 = 0x0138,
    Mul314 = 0x0139,
    Mul315 = 0x013a,
    Mul316 = 0x013b,
    Mul317 = 0x013c,
    Mul318 = 0x013d,
    Mul319 = 0x013e,
    Mul320 = 0x013f,
    Mul321 = 0x0140,
    Mul322 = 0x0141,
    Mul323 = 0x0142,
    Mul324 = 0x0143,
    Mul325 = 0x0144,
    Mul326 = 0x0145,
    Mul327 = 0x0146,
    Mul328 = 0x0147,
    Mul329 = 0x0148,
    Mul330 = 0x0149,
    Mul331 = 0x014a,
    Mul332 = 0x014b,
    Mul333 = 0x014c,
    Mul334 = 0x014d,
    Mul335 = 0x014e,
    Mul336 = 0x014f,
    Mul337 = 0x0150,
    Mul338 = 0x0151,
    Mul339 = 0x0152,
    Mul340 = 0x0153,
    Mul341 = 0x0154,
    Mul342 = 0x0155,
    Mul343 = 0x0156,
    Mul344 = 0x0157,
    Mul345 = 0x0158,
    Mul346 = 0x0159,
    Mul347 = 0x015a,
    Mul348 = 0x015b,
    Mul349 = 0x015c,
    Mul350 = 0x015d,
    Mul351 = 0x015e,
    Mul352 = 0x015f,
    Mul353 = 0x0160,
    Mul354 = 0x0161,
    Mul355 = 0x0162,
    Mul356 = 0x0163,
    Mul357 = 0x0164,
    Mul358 = 0x0165,
    Mul359 = 0x0166,
    Mul360 = 0x0167,
    Mul361 = 0x0168,
    Mul362 = 0x0169,
    Mul363 = 0x016a,
    Mul364 = 0x016b,
    Mul365 = 0x016c,
    Mul366 = 0x016d,
    Mul367 = 0x016e,
    Mul368 = 0x016f,
    Mul369 = 0x0170,
    Mul370 = 0x0171,
    Mul371 = 0x0172,
    Mul372 = 0x0173,
    Mul373 = 0x0174,
    Mul374 = 0x0175,
    Mul375 = 0x0176,
    Mul376 = 0x0177,
    Mul377 = 0x0178,
    Mul378 = 0x0179,
    Mul379 = 0x017a,
    Mul380 = 0x017b,
    Mul381 = 0x017c,
    Mul382 = 0x017d,
    Mul383 = 0x017e,
    Mul384 = 0x017f,
    Mul385 = 0x0180,
    Mul386 = 0x0181,
    Mul387 = 0x0182,
    Mul388 = 0x0183,
    Mul389 = 0x0184,
    Mul390 = 0x0185,
    Mul391 = 0x0186,
    Mul392 = 0x0187,
    Mul393 = 0x0188,
    Mul394 = 0x0189,
    Mul395 = 0x018a,
    Mul396 = 0x018b,
    Mul397 = 0x018c,
    Mul398 = 0x018d,
    Mul399 = 0x018e,
    Mul400 = 0x018f,
    Mul401 = 0x0190,
    Mul402 = 0x0191,
    Mul403 = 0x0192,
    Mul404 = 0x0193,
    Mul405 = 0x0194,
    Mul406 = 0x0195,
    Mul407 = 0x0196,
    Mul408 = 0x0197,
    Mul409 = 0x0198,
    Mul410 = 0x0199,
    Mul411 = 0x019a,
    Mul412 = 0x019b,
    Mul413 = 0x019c,
    Mul414 = 0x019d,
    Mul415 = 0x019e,
    Mul416 = 0x019f,
    Mul417 = 0x01a0,
    Mul418 = 0x01a1,
    Mul419 = 0x01a2,
    Mul420 = 0x01a3,
    Mul421 = 0x01a4,
    Mul422 = 0x01a5,
    Mul423 = 0x01a6,
    Mul424 = 0x01a7,
    Mul425 = 0x01a8,
    Mul426 = 0x01a9,
    Mul427 = 0x01aa,
    Mul428 = 0x01ab,
    Mul429 = 0x01ac,
    Mul430 = 0x01ad,
    Mul431 = 0x01ae,
    Mul432 = 0x01af,
    Mul433 = 0x01b0,
    Mul434 = 0x01b1,
    Mul435 = 0x01b2,
    Mul436 = 0x01b3,
    Mul437 = 0x01b4,
    Mul438 = 0x01b5,
    Mul439 = 0x01b6,
    Mul440 = 0x01b7,
    Mul441 = 0x01b8,
    Mul442 = 0x01b9,
    Mul443 = 0x01ba,
    Mul444 = 0x01bb,
    Mul445 = 0x01bc,
    Mul446 = 0x01bd,
    Mul447 = 0x01be,
    Mul448 = 0x01bf,
    Mul449 = 0x01c0,
    Mul450 = 0x01c1,
    Mul451 = 0x01c2,
    Mul452 = 0x01c3,
    Mul453 = 0x01c4,
    Mul454 = 0x01c5,
    Mul455 = 0x01c6,
    Mul456 = 0x01c7,
    Mul457 = 0x01c8,
    Mul458 = 0x01c9,
    Mul459 = 0x01ca,
    Mul460 = 0x01cb,
    Mul461 = 0x01cc,
    Mul462 = 0x01cd,
    Mul463 = 0x01ce,
    Mul464 = 0x01cf,
    Mul465 = 0x01d0,
    Mul466 = 0x01d1,
    Mul467 = 0x01d2,
    Mul468 = 0x01d3,
    Mul469 = 0x01d4,
    Mul470 = 0x01d5,
    Mul471 = 0x01d6,
    Mul472 = 0x01d7,
    Mul473 = 0x01d8,
    Mul474 = 0x01d9,
    Mul475 = 0x01da,
    Mul476 = 0x01db,
    Mul477 = 0x01dc,
    Mul478 = 0x01dd,
    Mul479 = 0x01de,
    Mul480 = 0x01df,
    Mul481 = 0x01e0,
    Mul482 = 0x01e1,
    Mul483 = 0x01e2,
    Mul484 = 0x01e3,
    Mul485 = 0x01e4,
    Mul486 = 0x01e5,
    Mul487 = 0x01e6,
    Mul488 = 0x01e7,
    Mul489 = 0x01e8,
    Mul490 = 0x01e9,
    Mul491 = 0x01ea,
    Mul492 = 0x01eb,
    Mul493 = 0x01ec,
    Mul494 = 0x01ed,
    Mul495 = 0x01ee,
    Mul496 = 0x01ef,
    Mul497 = 0x01f0,
    Mul498 = 0x01f1,
    Mul499 = 0x01f2,
    Mul500 = 0x01f3,
    Mul501 = 0x01f4,
    Mul502 = 0x01f5,
    Mul503 = 0x01f6,
    Mul504 = 0x01f7,
    Mul505 = 0x01f8,
    Mul506 = 0x01f9,
    Mul507 = 0x01fa,
    Mul508 = 0x01fb,
    Mul509 = 0x01fc,
    Mul510 = 0x01fd,
    Mul511 = 0x01fe,
    Mul512 = 0x01ff,
}
impl hertz::Prescaler for PllMul {
    fn num(&self) -> u32 {
        *self as u32 + 1
    }
    fn denom(&self) -> u32 {
        1
    }
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum PllDiv {
    Div1 = 0x0,
    Div2 = 0x01,
    Div3 = 0x02,
    Div4 = 0x03,
    Div5 = 0x04,
    Div6 = 0x05,
    Div7 = 0x06,
    Div8 = 0x07,
    Div9 = 0x08,
    Div10 = 0x09,
    Div11 = 0x0a,
    Div12 = 0x0b,
    Div13 = 0x0c,
    Div14 = 0x0d,
    Div15 = 0x0e,
    Div16 = 0x0f,
    Div17 = 0x10,
    Div18 = 0x11,
    Div19 = 0x12,
    Div20 = 0x13,
    Div21 = 0x14,
    Div22 = 0x15,
    Div23 = 0x16,
    Div24 = 0x17,
    Div25 = 0x18,
    Div26 = 0x19,
    Div27 = 0x1a,
    Div28 = 0x1b,
    Div29 = 0x1c,
    Div30 = 0x1d,
    Div31 = 0x1e,
    Div32 = 0x1f,
    Div33 = 0x20,
    Div34 = 0x21,
    Div35 = 0x22,
    Div36 = 0x23,
    Div37 = 0x24,
    Div38 = 0x25,
    Div39 = 0x26,
    Div40 = 0x27,
    Div41 = 0x28,
    Div42 = 0x29,
    Div43 = 0x2a,
    Div44 = 0x2b,
    Div45 = 0x2c,
    Div46 = 0x2d,
    Div47 = 0x2e,
    Div48 = 0x2f,
    Div49 = 0x30,
    Div50 = 0x31,
    Div51 = 0x32,
    Div52 = 0x33,
    Div53 = 0x34,
    Div54 = 0x35,
    Div55 = 0x36,
    Div56 = 0x37,
    Div57 = 0x38,
    Div58 = 0x39,
    Div59 = 0x3a,
    Div60 = 0x3b,
    Div61 = 0x3c,
    Div62 = 0x3d,
    Div63 = 0x3e,
    Div64 = 0x3f,
    Div65 = 0x40,
    Div66 = 0x41,
    Div67 = 0x42,
    Div68 = 0x43,
    Div69 = 0x44,
    Div70 = 0x45,
    Div71 = 0x46,
    Div72 = 0x47,
    Div73 = 0x48,
    Div74 = 0x49,
    Div75 = 0x4a,
    Div76 = 0x4b,
    Div77 = 0x4c,
    Div78 = 0x4d,
    Div79 = 0x4e,
    Div80 = 0x4f,
    Div81 = 0x50,
    Div82 = 0x51,
    Div83 = 0x52,
    Div84 = 0x53,
    Div85 = 0x54,
    Div86 = 0x55,
    Div87 = 0x56,
    Div88 = 0x57,
    Div89 = 0x58,
    Div90 = 0x59,
    Div91 = 0x5a,
    Div92 = 0x5b,
    Div93 = 0x5c,
    Div94 = 0x5d,
    Div95 = 0x5e,
    Div96 = 0x5f,
    Div97 = 0x60,
    Div98 = 0x61,
    Div99 = 0x62,
    Div100 = 0x63,
    Div101 = 0x64,
    Div102 = 0x65,
    Div103 = 0x66,
    Div104 = 0x67,
    Div105 = 0x68,
    Div106 = 0x69,
    Div107 = 0x6a,
    Div108 = 0x6b,
    Div109 = 0x6c,
    Div110 = 0x6d,
    Div111 = 0x6e,
    Div112 = 0x6f,
    Div113 = 0x70,
    Div114 = 0x71,
    Div115 = 0x72,
    Div116 = 0x73,
    Div117 = 0x74,
    Div118 = 0x75,
    Div119 = 0x76,
    Div120 = 0x77,
    Div121 = 0x78,
    Div122 = 0x79,
    Div123 = 0x7a,
    Div124 = 0x7b,
    Div125 = 0x7c,
    Div126 = 0x7d,
    Div127 = 0x7e,
    Div128 = 0x7f,
}
impl hertz::Prescaler for PllDiv {
    fn num(&self) -> u32 {
        *self as u32 + 1
    }
    fn denom(&self) -> u32 {
        1
    }
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum AHBPrescaler {
    /// SYSCLK not divided
    Div1 = 0x0,
    /// SYSCLK divided by 2
    Div2 = 0x08,
    /// SYSCLK divided by 4
    Div4 = 0x09,
    /// SYSCLK divided by 8
    Div8 = 0x0a,
    /// SYSCLK divided by 16
    Div16 = 0x0b,
    /// SYSCLK divided by 64
    Div64 = 0x0c,
    /// SYSCLK divided by 128
    Div128 = 0x0d,
    /// SYSCLK divided by 256
    Div256 = 0x0e,
    /// SYSCLK divided by 512
    Div512 = 0x0f,
}
impl hertz::Prescaler for AHBPrescaler {
    fn num(&self) -> u32 {
        match *self {
            Self::Div1 => 1,
            Self::Div2 => 2,
            Self::Div4 => 4,
            Self::Div8 => 8,
            Self::Div16 => 16,
            Self::Div64 => 64,
            Self::Div128 => 128,
            Self::Div256 => 256,
            Self::Div512 => 512,
        }
    }
    fn denom(&self) -> u32 {
        1
    }
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum APBPrescaler {
    /// HCLK not divided
    Div1 = 0x0,
    /// HCLK divided by 2
    Div2 = 0x04,
    /// HCLK divided by 4
    Div4 = 0x05,
    /// HCLK divided by 8
    Div8 = 0x06,
    /// HCLK divided by 16
    Div16 = 0x07,
}
impl hertz::Prescaler for APBPrescaler {
    fn num(&self) -> u32 {
        match *self {
            Self::Div1 => 1,
            Self::Div2 => 2,
            Self::Div4 => 4,
            Self::Div8 => 8,
            Self::Div16 => 16,
        }
    }
    fn denom(&self) -> u32 {
        1
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum Rtcsel {
    /// No clock selected
    #[default]
    Disable = 0x0,
    /// LSE oscillator clock selected
    Lse = 0x01,
    /// LSI oscillator clock selected
    Lsi = 0x02,
    /// HSE oscillator clock divided by 32 selected
    Hse = 0x03,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum Fdcansel {
    /// HSE clock selected
    #[default]
    Hse = 0x0,
    /// PLL1 Q (pll1_q_ck) selected
    Pll1Q = 0x01,
    /// PLL2 P (pll2_p_ck) selected
    Pll2P = 0x02,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum I2csel {
    /// PCLK1 selected
    #[default]
    Pclk1 = 0x0,
    /// SYSCLK selected
    Sys = 0x01,
    /// HSI selected
    Hsi = 0x02,
    /// MSIK selected
    Msik = 0x03,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum Iclksel {
    /// HSI48 clock selected
    #[default]
    Hsi48 = 0x0,
    /// PLL2 Q (pll2_q_ck) selected
    Pll2Q = 0x01,
    /// PLL1 Q (pll1_q_ck) selected
    Pll1Q = 0x02,
    /// MSIK clock selected
    Msik = 0x03,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum Lptim2sel {
    /// PCLK1 selected
    #[default]
    Pclk1 = 0x0,
    /// LSI selected
    Lsi = 0x01,
    /// HSI selected
    Hsi = 0x02,
    /// LSE selected
    Lse = 0x03,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum Spi1sel {
    /// PCLK2 selected
    #[default]
    Pclk2 = 0x0,
    /// SYSCLK selected
    Sys = 0x01,
    /// HSI selected
    Hsi = 0x02,
    /// MSIK selected
    Msik = 0x03,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum Spi2sel {
    /// PCLK2 selected
    #[default]
    Pclk1 = 0x0,
    /// SYSCLK selected
    Sys = 0x01,
    /// HSI selected
    Hsi = 0x02,
    /// MSIK selected
    Msik = 0x03,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum Usartsel {
    /// PCLK1 selected
    #[default]
    Pclk1 = 0x0,
    /// SYSCLK selected
    Sys = 0x01,
    /// HSI selected
    Hsi = 0x02,
    /// LSE selected
    Lse = 0x03,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum Usart1sel {
    /// PCLK2 selected
    #[default]
    Pclk2 = 0x0,
    /// SYSCLK selected
    Sys = 0x01,
    /// HSI selected
    Hsi = 0x02,
    /// LSE selected
    Lse = 0x03,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum Octospisel {
    /// SYSCLK selected
    #[default]
    Sys = 0x0,
    /// MSIK selected
    Msik = 0x01,
    /// PLL1 Q (pll1_q_ck) selected, can be up to 200 MHz
    Pll1Q = 0x02,
    /// PLL2 Q (pll2_q_ck) selected, can be up to 200 MHz
    Pll2Q = 0x03,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum Rngsel {
    /// HSI48 selected
    #[default]
    Hsi48 = 0x0,
    /// HSI48 / 2 selected, can be used in Range 4
    Hsi48Div2 = 0x01,
    /// HSI selected
    Hsi = 0x02,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum Saessel {
    /// SHSI selected
    #[default]
    Shsi = 0x0,
    /// SHSI / 2 selected, can be used in Range 4
    ShsiDiv2 = 0x01,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum Saisel {
    /// PLL2 P (pll2_p_ck) selected
    #[default]
    Pll2P = 0x0,
    /// PLL3 P (pll3_p_ck) selected
    Pll3P = 0x01,
    /// PLL1 P (pll1_p_ck) selected
    Pll1P = 0x02,
    /// input pin AUDIOCLK selected
    Audioclk = 0x03,
    /// HSI clock selected
    Hsi = 0x04,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum Sdmmcsel {
    /// ICLK clock selected
    #[default]
    Iclk = 0x0,
    /// PLL1 P (pll1_p_ck) selected, in case higher than 48 MHz is needed (for SDR50 mode)
    Pll1P = 0x01,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum Adcdacsel {
    /// HCLK clock selected
    #[default]
    Hclk1 = 0x0,
    /// SYSCLK selected
    Sys = 0x01,
    /// PLL2 R (pll2_r_ck) selected
    Pll2R = 0x02,
    /// HSE clock selected
    Hse = 0x03,
    /// HSI clock selected
    Hsi = 0x04,
    /// MSIK clock selected
    Msik = 0x05,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum Adfsel {
    /// HCLK selected
    #[default]
    Hclk3 = 0x0,
    /// PLL1 P (pll1_p_ck) selected
    Pll1P = 0x01,
    /// PLL3 Q (pll3_q_ck) selected
    Pll3Q = 0x02,
    /// input pin AUDIOCLK selected
    Audioclk = 0x03,
    /// MSIK clock selected
    Msik = 0x04,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum Dacsel {
    /// LSE selected
    #[default]
    Lse = 0x0,
    /// LSI selected
    Lsi = 0x01,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum I2c3sel {
    /// PCLK3 selected
    #[default]
    Pclk3 = 0x0,
    /// SYSCLK selected
    Sys = 0x01,
    /// HSI selected
    Hsi = 0x02,
    /// MSIK selected
    Msik = 0x03,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum Lptimsel {
    /// MSIK selected
    #[default]
    Msik = 0x0,
    /// LSI selected
    Lsi = 0x01,
    /// HSI selected
    Hsi = 0x02,
    /// LSE selected
    Lse = 0x03,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum Lpusartsel {
    /// PCLK3 selected
    #[default]
    Pclk3 = 0x0,
    /// SYSCLK selected
    Sys = 0x01,
    /// HSI selected
    Hsi = 0x02,
    /// LSE selected
    Lse = 0x03,
    /// MSIK selected
    Msik = 0x04,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum Spi3sel {
    /// PCLK2 selected
    #[default]
    Pclk3 = 0x0,
    /// SYSCLK selected
    Sys = 0x01,
    /// HSI selected
    Hsi = 0x02,
    /// MSIK selected
    Msik = 0x03,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum Pllrge {
    /// PLL2 input (ref2_ck) clock range frequency between 4 and 8 MHz
    #[default]
    Freq4to8mhz = 0x0,
    /// PLL2 input (ref2_ck) clock range frequency between 8 and 16 MHz
    Freq8to16mhz = 0x03,
}

#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum PllMboost {
    /// division by 1 (bypass)
    #[default]
    Div1 = 0x0,
    /// division by 2
    Div2 = 0x01,
    /// division by 4
    Div4 = 0x02,
    /// division by 6
    Div6 = 0x03,
    /// division by 8
    Div8 = 0x04,
    /// division by 10
    Div10 = 0x05,
    /// division by 12
    Div12 = 0x06,
    /// division by 14
    Div14 = 0x07,
    /// division by 16
    Div16 = 0x08,
}
