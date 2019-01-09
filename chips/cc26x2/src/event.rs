//CC26_EVENT_FABRIC_MAP1 Registers

// Table 5-13. CC26_EVENT_FABRIC_MAP1 Registers

// OffsetAcronym Register Name        Section
// 0h CPUIRQSEL0 Output Selection for CPU Interrupt 0 Section 5.7.1.2.1
// 4h CPUIRQSEL1 Output Selection for CPU Interrupt 1 Section 5.7.1.2.2
// 8h CPUIRQSEL2 Output Selection for CPU Interrupt 2 Section 5.7.1.2.3
// Ch CPUIRQSEL3 Output Selection for CPU Interrupt 3 Section 5.7.1.2.4
// 10h CPUIRQSEL4 Output Selection for CPU Interrupt 4 Section 5.7.1.2.5
// 14h CPUIRQSEL5 Output Selection for CPU Interrupt 5 Section 5.7.1.2.6
// 18h CPUIRQSEL6 Output Selection for CPU Interrupt 6 Section 5.7.1.2.7
// 1Ch CPUIRQSEL7 Output Selection for CPU Interrupt 7 Section 5.7.1.2.8
// 20h CPUIRQSEL8 Output Selection for CPU Interrupt 8 Section 5.7.1.2.9
// 24h CPUIRQSEL9 Output Selection for CPU Interrupt 9 Section 5.7.1.2.10
// 28h CPUIRQSEL10 Output Selection for CPU Interrupt 10 Section 5.7.1.2.11
// 2Ch CPUIRQSEL11 Output Selection for CPU Interrupt 11 Section 5.7.1.2.12
// 30h CPUIRQSEL12 Output Selection for CPU Interrupt 12 Section 5.7.1.2.13
// 34h CPUIRQSEL13 Output Selection for CPU Interrupt 13 Section 5.7.1.2.14
// 38h CPUIRQSEL14 Output Selection for CPU Interrupt 14 Section 5.7.1.2.15
// 3Ch CPUIRQSEL15 Output Selection for CPU Interrupt 15 Section 5.7.1.2.16
// 40h CPUIRQSEL16 Output Selection for CPU Interrupt 16 Section 5.7.1.2.17
// 44h CPUIRQSEL17 Output Selection for CPU Interrupt 17 Section 5.7.1.2.18
// 48h CPUIRQSEL18 Output Selection for CPU Interrupt 18 Section 5.7.1.2.19
// 4Ch CPUIRQSEL19 Output Selection for CPU Interrupt 19 Section 5.7.1.2.20
// 50h CPUIRQSEL20 Output Selection for CPU Interrupt 20 Section 5.7.1.2.21
// 54h CPUIRQSEL21 Output Selection for CPU Interrupt 21 Section 5.7.1.2.22
// 58h CPUIRQSEL22 Output Selection for CPU Interrupt 22 Section 5.7.1.2.23
// 5Ch CPUIRQSEL23 Output Selection for CPU Interrupt 23 Section 5.7.1.2.24
// 60h CPUIRQSEL24 Output Selection for CPU Interrupt 24 Section 5.7.1.2.25
// 64h CPUIRQSEL25 Output Selection for CPU Interrupt 25 Section 5.7.1.2.26
// 68h CPUIRQSEL26 Output Selection for CPU Interrupt 26 Section 5.7.1.2.27
// 6Ch CPUIRQSEL27 Output Selection for CPU Interrupt 27 Section 5.7.1.2.28
// 70h CPUIRQSEL28 Output Selection for CPU Interrupt 28 Section 5.7.1.2.29
// 74h CPUIRQSEL29 Output Selection for CPU Interrupt 29 Section 5.7.1.2.30
// 78h CPUIRQSEL30 Output Selection for CPU Interrupt 30 Section 5.7.1.2.31
// 7Ch CPUIRQSEL31 Output Selection for CPU Interrupt 31 Section 5.7.1.2.32
// 80h CPUIRQSEL32 Output Selection for CPU Interrupt 32 Section 5.7.1.2.33
// 84h CPUIRQSEL33 Output Selection for CPU Interrupt 33 Section 5.7.1.2.34
// 88h CPUIRQSEL34 Output Selection for CPU Interrupt 34 Section 5.7.1.2.35
// 8Ch CPUIRQSEL35 Output Selection for CPU Interrupt 35 Section 5.7.1.2.36
// 90h CPUIRQSEL36 Output Selection for CPU Interrupt 36 Section 5.7.1.2.37
// 94h CPUIRQSEL37 Output Selection for CPU Interrupt 37 Section 5.7.1.2.38
// 100h RFCSEL0 Output Selection for RFC Event 0 Section 5.7.1.2.39
// 104h RFCSEL1 Output Selection for RFC Event 1 Section 5.7.1.2.40
// 108h RFCSEL2 Output Selection for RFC Event 2 Section 5.7.1.2.41
// 10Ch RFCSEL3 Output Selection for RFC Event 3 Section 5.7.1.2.42
// 110h RFCSEL4 Output Selection for RFC Event 4 Section 5.7.1.2.43
// 114h RFCSEL5 Output Selection for RFC Event 5 Section 5.7.1.2.44
// 118h RFCSEL6 Output Selection for RFC Event 6 Section 5.7.1.2.45
// 11Ch RFCSEL7 Output Selection for RFC Event 7 Section 5.7.1.2.46
// 120h RFCSEL8 Output Selection for RFC Event 8 Section 5.7.1.2.47
// 124h RFCSEL9 Output Selection for RFC Event 9 Section 5.7.1.2.48
// 200h GPT0ACAPTSEL Output Selection for GPT0 0 Section 5.7.1.2.49
// 204h GPT0BCAPTSEL Output Selection for GPT0 1 Section 5.7.1.2.50
// 300h GPT1ACAPTSEL Output Selection for GPT1 0 Section 5.7.1.2.51
// 304h GPT1BCAPTSEL Output Selection for GPT1 1 Section 5.7.1.2.52
// 400h GPT2ACAPTSEL Output Selection for GPT2 0 Section 5.7.1.2.53
// 404h GPT2BCAPTSEL Output Selection for GPT2 1 Section 5.7.1.2.54
// 508h UDMACH1SSEL Output Selection for DMA Channel 1 SREQ Section 5.7.1.2.55
// 50Ch UDMACH1BSEL Output Selection for DMA Channel 1 REQ Section 5.7.1.2.56
// 510h UDMACH2SSEL Output Selection for DMA Channel 2 SREQ Section 5.7.1.2.57
// 514h UDMACH2BSEL Output Selection for DMA Channel 2 REQ Section 5.7.1.2.58
// 518h UDMACH3SSEL Output Selection for DMA Channel 3 SREQ Section 5.7.1.2.59
// 51Ch UDMACH3BSEL Output Selection for DMA Channel 3 REQ Section 5.7.1.2.60
// 520h UDMACH4SSEL Output Selection for DMA Channel 4 SREQ Section 5.7.1.2.61
// 524h UDMACH4BSEL Output Selection for DMA Channel 4 REQ Section 5.7.1.2.62
// 528h UDMACH5SSEL Output Selection for DMA Channel 5 SREQ Section 5.7.1.2.63
// 52Ch UDMACH5BSEL Output Selection for DMA Channel 5 REQ Section 5.7.1.2.64
// 530h UDMACH6SSEL Output Selection for DMA Channel 6 SREQ Section 5.7.1.2.65
// 534h UDMACH6BSEL Output Selection for DMA Channel 6 REQ Section 5.7.1.2.66
// 538h UDMACH7SSEL Output Selection for DMA Channel 7 SREQ Section 5.7.1.2.67
// 53Ch UDMACH7BSEL Output Selection for DMA Channel 7 REQ Section 5.7.1.2.68
// 540h UDMACH8SSEL Output Selection for DMA Channel 8 SREQ Section 5.7.1.2.69
// 544h UDMACH8BSEL Output Selection for DMA Channel 8 REQ Section 5.7.1.2.70
// 548h UDMACH9SSEL Output Selection for DMA Channel 9 SREQ Section 5.7.1.2.71
// 54Ch UDMACH9BSEL Output Selection for DMA Channel 9 REQ Section 5.7.1.2.72
// 550h UDMACH10SSEL Output Selection for DMA Channel 10 SREQ Section 5.7.1.2.73
// 554h UDMACH10BSEL Output Selection for DMA Channel 10 REQ Section 5.7.1.2.74
// 558h UDMACH11SSEL Output Selection for DMA Channel 11 SREQ Section 5.7.1.2.75
// 55Ch UDMACH11BSEL Output Selection for DMA Channel 11 REQ Section 5.7.1.2.76
// 560h UDMACH12SSEL Output Selection for DMA Channel 12 SREQ Section 5.7.1.2.77
// 564h UDMACH12BSEL Output Selection for DMA Channel 12 REQ Section 5.7.1.2.78
// 56Ch UDMACH13BSEL Output Selection for DMA Channel 13 REQ Section 5.7.1.2.79
// 574h UDMACH14BSEL Output Selection for DMA Channel 14 REQ Section 5.7.1.2.80
// 57Ch UDMACH15BSEL Output Selection for DMA Channel 15 REQ Section 5.7.1.2.81
// 580h UDMACH16SSEL Output Selection for DMA Channel 16 SREQ Section 5.7.1.2.82
// 584h UDMACH16BSEL Output Selection for DMA Channel 16 REQ Section 5.7.1.2.83
// 588h UDMACH17SSEL Output Selection for DMA Channel 17 SREQ Section 5.7.1.2.84
// 58Ch UDMACH17BSEL Output Selection for DMA Channel 17 REQ Section 5.7.1.2.85
// 5A8h UDMACH21SSEL Output Selection for DMA Channel 21 SREQ Section 5.7.1.2.86
// 5ACh UDMACH21BSEL Output Selection for DMA Channel 21 REQ Section 5.7.1.2.87
// 5B0h UDMACH22SSEL Output Selection for DMA Channel 22 SREQ Section 5.7.1.2.88
// 5B4h UDMACH22BSEL Output Selection for DMA Channel 22 REQ Section 5.7.1.2.89
// 5B8h UDMACH23SSEL Output Selection for DMA Channel 23 SREQ Section 5.7.1.2.90
// 5BCh UDMACH23BSEL Output Selection for DMA Channel 23 REQ Section 5.7.1.2.91
// 5C0h UDMACH24SSEL Output Selection for DMA Channel 24 SREQ Section 5.7.1.2.92
// 5C4h UDMACH24BSEL Output Selection for DMA Channel 24 REQ Section 5.7.1.2.93
// 600h GPT3ACAPTSEL Output Selection for GPT3 0 Section 5.7.1.2.94
// 604h GPT3BCAPTSEL Output Selection for GPT3 1 Section 5.7.1.2.95
// 700h AUXSEL0 Output Selection for AUX Subscriber 0 Section 5.7.1.2.96
// 800h CM3NMISEL0 Output Selection for NMI Subscriber 0 Section 5.7.1.2.97
// 900h I2SSTMPSEL0 Output Selection for I2S Subscriber 0 Section 5.7.1.2.98
// A00h FRZSEL0 Output Selection for FRZ Subscriber Section 5.7.1.2.99
// F00h SWEV Set or Clear Software Events Section 5.7.1.2.100

use kernel::common::registers::{register_bitfields, ReadWrite};
use kernel::common::StaticRef;

use crate::event;
use crate::memory_map::EVENT_BASE;

pub const REG: StaticRef<Register> =
    unsafe { StaticRef::new((EVENT_BASE + 0x200) as *const Register) };

#[repr(C)]
pub struct Register {
    _offset0: [ReadWrite<u8>; 0x200],
    pub gpt0a_sel: ReadWrite<u32, event::Gpt0A::Register>,
    pub gpt0b_sel: ReadWrite<u32, event::Gpt0B::Register>,
    _offset1: [ReadWrite<u8>; 0xF8],
    pub gpt1a_sel: ReadWrite<u32, event::Gpt1A::Register>,
    pub gpt1b_sel: ReadWrite<u32, event::Gpt1B::Register>,
    _offset2: [ReadWrite<u8>; 0xF8],
    pub gpt2a_sel: ReadWrite<u32, event::Gpt2A::Register>,
    pub gpt2b_sel: ReadWrite<u32, event::Gpt2B::Register>,
    _offset3: [ReadWrite<u8>; 0x1F8],
    pub gpt3a_sel: ReadWrite<u32, event::Gpt3A::Register>,
    pub gpt3b_sel: ReadWrite<u32, event::Gpt3B::Register>,
}

register_bitfields![
    u32,
    Gpt0A [
        EVENT OFFSET(0) NUMBITS(7) [
            PORT_EVENT0 = 0x55,
            PORT_EVENT1 = 0x56
        ]
    ],
    Gpt0B [
        EVENT OFFSET(0) NUMBITS(7) [
            PORT_EVENT0 = 0x55,
            PORT_EVENT1 = 0x56
        ]
    ],
    Gpt1A [
        EVENT OFFSET(0) NUMBITS(7) [
            PORT_EVENT2 = 0x57,
            PORT_EVENT3 = 0x58
        ]
    ],
    Gpt1B [
        EVENT OFFSET(0) NUMBITS(7) [
            PORT_EVENT2 = 0x57,
            PORT_EVENT3 = 0x58
        ]
    ],
    Gpt2A [
        EVENT OFFSET(0) NUMBITS(7) [
            PORT_EVENT4 = 0x59,
            PORT_EVENT5 = 0x5A
        ]
    ],
    Gpt2B [
        EVENT OFFSET(0) NUMBITS(7) [
            PORT_EVENT4 = 0x59,
            PORT_EVENT5 = 0x5A
        ]
    ],
    Gpt3A [
        EVENT OFFSET(0) NUMBITS(7) [
            PORT_EVENT6 = 0x5B,
            PORT_EVENT7 = 0x5C
        ]
    ],
    Gpt3B [
        EVENT OFFSET(0) NUMBITS(7) [
            PORT_EVENT6 = 0x5B,
            PORT_EVENT7 = 0x5C
        ]
    ]
];
