// CRC test cases
//
// Output values can be computed here:
//
//   http://www.zorc.breitbandkatze.de/crc.html
//
// with the "reverse data bytes" option set.
//
// Below, "no post processing" is achieved by configuring that calculator with:
//
//   "Final XOR value" = 0,
//
// while "output reversed then inverted" uses:
//
//   "reverse CRC result before Final XOR"
//   "Final XOR value" = FFFFFFFF

// Polynomial: 0x1021, no post-processing

CASE(CRC_SAM4L_16, 0xffff1541, "ABCDEFG")
CASE(CRC_SAM4L_16, 0xffffB34B, "ABCD")
CASE(CRC_SAM4L_16, 0xffff1C2D, "0123456")
CASE(CRC_SAM4L_16, 0xffffD5A8, "0123")
CASE(CRC_SAM4L_16, 0xffffC21F, "01234567")
CASE(CRC_SAM4L_16, 0xffff35B3, "012345678")
CASE(CRC_SAM4L_16, 0xffff57C4, "01234567A")
CASE(CRC_SAM4L_16, 0xffffE06E, "01234567ABCDE")
CASE(CRC_SAM4L_16, 0xffffEC86, "0000000000000")
CASE(CRC_SAM4L_16, 0xffff7B2E, "00000000000000")
CASE(CRC_SAM4L_16, 0xffffDFCA, "01234567ABCDEF")
CASE(CRC_SAM4L_16, 0xffff2DFE, "01234567ABCDEFG")
CASE(CRC_SAM4L_16, 0xffff39BC, "01234567ABCDEFGH")
CASE(CRC_SAM4L_16, 0xffffB881, "01234567ABCDEFGHI")

// Polynomial: 0x04C11DB7, no post-processing

CASE(CRC_SAM4L_32, 0xC2D6098F, "ABCDEFG")
CASE(CRC_SAM4L_32, 0x4146999A, "0123")
CASE(CRC_SAM4L_32, 0xA4CF5FDD, "A man, a plan, a canal, Panama")

// Polynomial 0x1EDC6F41, no post-processing

CASE(CRC_SAM4L_32C, 0x599511CB, "ABCDEFG")
CASE(CRC_SAM4L_32C, 0x62B9639F, "0123")
CASE(CRC_SAM4L_32C, 0xDD284452, "A man, a plan, a canal, Panama")

// Polynomial: 0x04C11DB7, output reversed then inverted

CASE(CRC_32, 0x0E6F94BC, "ABCDEFG")
CASE(CRC_32, 0xA6669D7D, "0123")
CASE(CRC_32, 0x44050CDA, "A man, a plan, a canal, Panama")

// Polynomial 0x1EDC6F41, output reversed then inverted

CASE(CRC_32C, 0x2C775665, "ABCDEFG")
CASE(CRC_32C, 0x063962B9, "0123")
CASE(CRC_32C, 0xB5DDEB44, "A man, a plan, a canal, Panama")
