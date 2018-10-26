use kernel::common::registers::ReadWrite;

// Table 19-40. ADI_4_AUX_MMAP1 Registers

// Offset       Acronym      Register Name      Section
// 0h           MUX0         Internal           Section 19.8.1.1
// 1h           MUX1         Internal           Section 19.8.1.2
// 2h           MUX2         Internal           Section 19.8.1.3
// 3h           MUX3         Internal           Section 19.8.1.4
// 4h           ISRC         Current Source     Section 19.8.1.5
// 5h           COMP         Comparator         Section 19.8.1.6
// 7h           MUX4         Internal           Section 19.8.1.7
// 8h           ADC0         ADC Control 0      Section 19.8.1.8
// 9h           ADC1         ADC Control 1      Section 19.8.1.9
// Ah           ADCREF0      ADC Reference 0    Section 19.8.1.10
// Bh           ADCREF1      ADC Reference 1    Section 19.8.1.11
#[repr(C)]
pub struct AuxAdi4Registers {
    _reserved_mux0to3: [u8; 0x4], // the HWAPI touches these for you
    pub current_source: ReadWrite<u8, CurrentSource::Register>,
    pub comparator: ReadWrite<u8, Comparator::Register>,
    _reserved_skipped: u8,
    _reserved_mux4: u8,
    pub control0: ReadWrite<u8, Control0::Register>,
    _reserved_control1: u8, // the HWAPI touches these for you
    pub reference0: ReadWrite<u8, Reference0::Register>,
    pub reference1: ReadWrite<u8, Reference1::Register>,
}

register_bitfields![
    u8,
    CurrentSource [
        EN  OFFSET(0) NUMBITS(1) [],
        TRIM OFFSET(2) NUMBITS(6) [     //these may be combined
            _11P75_UA = 0x20, //11.75uA
            _4P5_UA = 0x10,   //4.5uA
            _2P0_UA = 0x08,   //2.0uA
            _1P0_UA = 0x04,   //1.0uA
            _0P5_UA = 0x02,   //0.5uA
            _0P25_UA = 0x01,  //0.25uA
            _NC = 0x00
        ]
    ],
    Comparator [
        A_EN  OFFSET(0) NUMBITS(1) [],
        B_EN  OFFSET(2) NUMBITS(1) [],
        COMPA_REF_CURR_EN OFFSET(6) NUMBITS(1) [],  // enable 2uA IPTAT current from ISRC to COMPA
        COMPA_REF_RES_EN OFFSET(7) NUMBITS(1) []    // enables 400kohm resisitance to ground
    ],
    Control0 [
        EN  OFFSET(0) NUMBITS(1) [],
        RESET_N  OFFSET(1) NUMBITS(1) [],       // reset required after every reconfigure
        SAMPLE_CYCLE OFFSET(3) NUMBITS(4) [     // only applies to synchronous mode sampling
            _2P7_US = 0x3,  // 2.7  uS
            _5P3_US = 0x4,  // 5.3  uS
            _10P6_US = 0x5, // 10.6 uS
            _21P3_US = 0x6, // 21.3 uS
            _42P6_US = 0x7, // 42.6 uS
            _85P3_US = 0x8, // 85.3.uS
            _170_US = 0x9,  // 170  uS
            _341_US = 0xA,  // 341  uS
            _682_US = 0xB,  // 682  uS
            _1P37_MS = 0xC, // 1.37 mS
            _2P73_MS = 0xD, // 2.73 mS
            _5P46_US = 0xE, // 5.46 mS
            _10P9_US = 0xF  // 10.9 mS
        ],
        SAMPLE_MODE OFFSET(7) NUMBITS(1) [
            SYNC = 0,
            ASYNC = 1
        ]
    ],
    Reference0 [
        EN OFFSET(0) NUMBITS(1) [],
        SRC OFFSET(3) NUMBITS(1) [
            FIXED_4P5V = 0,
            NOMINAL_VDDS =1
        ],
        // keep ADCREF powered up in idle state when ADC0.SMPL_MODE=0
        // set to 1 if ADC0.SMPLE is less than 6 (21.3us)
        REF_ON_IDLE OFFSET(6) NUMBITS(1) []
    ],
    Reference1 [
        VRTIM OFFSET(0) NUMBITS(6) [ // 64 steps, 2s complement
            // A few examples
            NOMINAL = 0x00,  //nominal voltage = 1.43V
            NOMINAL_PLUS_ONE = 0x01, //nominal voltage + 0.4% = 1.435V
            NOMINAL_MINUS_ONE = 0x3F, //nominal voltage - 0.4% = 1.425V
            MAX = 0x1F, // max voltage = 1.6V
            MIN = 0x20  // min voltage = 1.3V
        ]
    ]
];
