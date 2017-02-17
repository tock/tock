use core::intrinsics;
use kernel::common::volatile_cell::VolatileCell;

#[repr(C, packed)]
struct Nvic {
    iser: [VolatileCell<u32>; 7],
    _reserved0: [u32; 25],
    icer: [VolatileCell<u32>; 7],
    _reserved1: [u32; 25],
    ispr: [VolatileCell<u32>; 7],
    _reserved2: [u32; 25],
    icpr: [VolatileCell<u32>; 7],
}

pub mod raw {
    pub const HFLASHC: u32 = 0;
    pub const PDCA0: u32 = 1;
    pub const PDCA1: u32 = 2;
    pub const PDCA2: u32 = 3;
    pub const PDCA3: u32 = 4;
    pub const PDCA4: u32 = 5;
    pub const PDCA5: u32 = 6;
    pub const PDCA6: u32 = 7;
    pub const PDCA7: u32 = 8;
    pub const PDCA8: u32 = 9;
    pub const PDCA9: u32 = 10;
    pub const PDCA10: u32 = 11;
    pub const PDCA11: u32 = 12;
    pub const PDCA12: u32 = 13;
    pub const PDCA13: u32 = 14;
    pub const PDCA14: u32 = 15;
    pub const PDCA15: u32 = 16;
    pub const CRCCU: u32 = 17;
    pub const USBC: u32 = 18;
    pub const PEVCTR: u32 = 19;
    pub const PEVCOV: u32 = 20;
    pub const AESA: u32 = 21;
    pub const PM: u32 = 22;
    pub const SCIF: u32 = 23;
    pub const FREQM: u32 = 24;
    pub const GPIO0: u32 = 25;
    pub const GPIO1: u32 = 26;
    pub const GPIO2: u32 = 27;
    pub const GPIO3: u32 = 28;
    pub const GPIO4: u32 = 29;
    pub const GPIO5: u32 = 30;
    pub const GPIO6: u32 = 31;
    pub const GPIO7: u32 = 32;
    pub const GPIO8: u32 = 33;
    pub const GPIO9: u32 = 34;
    pub const GPIO10: u32 = 35;
    pub const GPIO11: u32 = 36;
    pub const BPM: u32 = 40;
    pub const BSCIF: u32 = 41;
    pub const ASTALARM: u32 = 42;
    pub const ASTPER: u32 = 43;
    pub const ASTOVF: u32 = 44;
    pub const ASTREADY: u32 = 45;
    pub const ASTCLKREADY: u32 = 46;
    pub const WDT: u32 = 47;
    pub const EIC1: u32 = 48;
    pub const EIC2: u32 = 49;
    pub const EIC3: u32 = 50;
    pub const EIC4: u32 = 51;
    pub const EIC5: u32 = 52;
    pub const EIC6: u32 = 53;
    pub const EIC7: u32 = 54;
    pub const EIC8: u32 = 55;
    pub const IISC: u32 = 56;
    pub const SPI: u32 = 57;
    pub const TC00: u32 = 58;
    pub const TC01: u32 = 59;
    pub const TC02: u32 = 60;
    pub const TC10: u32 = 61;
    pub const TC11: u32 = 62;
    pub const TC12: u32 = 63;
    pub const TWIM0: u32 = 64;
    pub const TWIS0: u32 = 65;
    pub const TWIM1: u32 = 66;
    pub const TWIS1: u32 = 67;
    pub const USART0: u32 = 68;
    pub const USART1: u32 = 69;
    pub const USART2: u32 = 70;
    pub const USART3: u32 = 71;
    pub const ADCIFE: u32 = 72;
    pub const DACC: u32 = 73;
    pub const ACIFC: u32 = 74;
    pub const ABDACB: u32 = 75;
    pub const TRNG: u32 = 76;
    pub const PARC: u32 = 77;
    pub const CATB: u32 = 78;
    pub const TWIM2: u32 = 80;
    pub const TWIM3: u32 = 81;
    pub const LCDCA: u32 = 82;
}

#[repr(C)]
#[derive(Copy,Clone)]
pub enum NvicIdx {
    HFLASHC = 0,
    PDCA0 = 1,
    PDCA1 = 2,
    PDCA2 = 3,
    PDCA3 = 4,
    PDCA4 = 5,
    PDCA5 = 6,
    PDCA6 = 7,
    PDCA7 = 8,
    PDCA8 = 9,
    PDCA9 = 10,
    PDCA10 = 11,
    PDCA11 = 12,
    PDCA12 = 13,
    PDCA13 = 14,
    PDCA14 = 15,
    PDCA15 = 16,
    CRCCU = 17,
    USBC = 18,
    PEVCTR = 19,
    PEVCOV = 20,
    AESA = 21,
    PM = 22,
    SCIF = 23,
    FREQM = 24,
    GPIO0 = 25,
    GPIO1 = 26,
    GPIO2 = 27,
    GPIO3 = 28,
    GPIO4 = 29,
    GPIO5 = 30,
    GPIO6 = 34,
    GPIO7 = 35,
    GPIO8 = 36,
    GPIO9 = 37,
    GPIO10 = 38,
    GPIO11 = 39,
    BPM = 40,
    BSCIF = 41,
    ASTALARM = 42,
    ASTPER = 43,
    ASTOVF = 44,
    ASTREADY = 45,
    ASTCLKREADY = 46,
    WDT = 47,
    EIC1 = 48,
    EIC2 = 49,
    EIC3 = 50,
    EIC4 = 51,
    EIC5 = 52,
    EIC6 = 53,
    EIC7 = 54,
    EIC8 = 55,
    IISC = 56,
    SPI = 57,
    TC00 = 58,
    TC01 = 59,
    TC02 = 60,
    TC10 = 61,
    TC11 = 62,
    TC12 = 63,
    TWIM0 = 64,
    TWIS0 = 65,
    TWIM1 = 66,
    TWIS1 = 67,
    USART0 = 68,
    USART1 = 69,
    USART2 = 70,
    USART3 = 71,
    ADCIFE = 72,
    DACC = 73,
    ACIFC = 74,
    ABDACB = 75,
    TRNG = 76,
    PARC = 77,
    CATB = 78,
    _RESERVED,
    TWIM2 = 80,
    TWIM3 = 81,
    LCDCA = 82,
}

impl ::core::default::Default for NvicIdx {
    fn default() -> NvicIdx {
        NvicIdx::HFLASHC
    }
}

const BASE_ADDRESS: usize = 0xe000e100;

pub unsafe fn enable(signal: NvicIdx) {
    let nvic: &mut Nvic = intrinsics::transmute(BASE_ADDRESS);
    let interrupt = signal as usize;

    nvic.iser[interrupt / 32].set(1 << (interrupt & 31));
}

pub unsafe fn disable(signal: NvicIdx) {
    let nvic: &mut Nvic = intrinsics::transmute(BASE_ADDRESS);
    let interrupt = signal as usize;

    nvic.icer[interrupt / 32].set(1 << (interrupt & 31));
}

pub unsafe fn clear_pending(signal: NvicIdx) {
    let nvic: &mut Nvic = intrinsics::transmute(BASE_ADDRESS);
    let interrupt = signal as usize;

    nvic.icpr[interrupt / 32].set(1 << (interrupt & 31));
}
