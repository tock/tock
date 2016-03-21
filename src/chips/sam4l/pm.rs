#[allow(dead_code)]

use helpers::*;

#[repr(C, packed)]
struct PmRegisters {
    mcctrl: u32,
    cpusel: u32,
    reserved0: u32,
    pbasel: u32,
    pbbsel: u32,
    pbcsel: u32,
    pbdsel: u32,
    reserved1: u32,
    cpumask: u32, // 0x020
    hsbmask: u32,
    pbamask: u32,
    pbbmask: u32,
    pbcmask: u32,
    pbdmask: u32,
    reserved2: [u32; 2],
    pbadivmask: u32, // 0x040
    reserved3: [u32; 4],
    cfdctrl: u32,
    unlock: u32,
    reserved4: u32,
    reserved5: [u32; 24], // 0x60
    ier: u32, // 0xC0
    idr: u32,
    imr: u32,
    isr: u32,
    icr: u32,
    sr: u32,
    reserved6: [u32; 2],
    reserved7: [u32; 24], // 0x100
    ppcr: u32, // 0x160
    reserved8: [u32; 7],
    rcause: u32, // 0x180
    wcause: u32,
    awen: u32,
    protctrl: u32,
    reserved9: u32,
    fastsleep: u32,
    reserved10: [u32; 2],
    config: u32, // 0x200
    version: u32
}

const PM_BASE: isize = 0x400E0000;
const HSB_MASK_OFFSET: u32 = 0x24;
const PBA_MASK_OFFSET: u32 = 0x28;
const PBB_MASK_OFFSET: u32 = 0x2C;
const PBD_MASK_OFFSET: u32 = 0x34;

static mut PM: *mut PmRegisters = PM_BASE as *mut PmRegisters;

pub enum MainClock {
    RCSYS, OSC0, PLL, DFLL, RC80M, RCFAST, RC1M
}

#[derive(Copy,Clone)]
pub enum Clock {
    HSB(HSBClock),
    PBA(PBAClock),
    PBB(PBBClock),
    PBD(PBDClock),
}

#[derive(Copy,Clone)]
pub enum HSBClock {
    PDCA, FLASHCALW, FLASHCALWP, USBC, CRCCU, APBA, APBB, APBC, APBD, AESA
}

#[derive(Copy,Clone)]
pub enum PBAClock {
    IISC, SPI, TC0, TC1, TWIM0, TWIS0, TWIM1, TWIS1,
    USART0, USART1, USART2, USART3, ADCIFE, DACC, ACIFC, GLOC, ABSACB,
    TRNG, PARC, CATB, NULL, TWIM2, TWIM3, LCDCA
}

#[derive(Copy,Clone)]
pub enum PBBClock {
    FLASHCALW, HRAMC1, HMATRIX, PDCA, CRCCU, USBC, PEVC
}

#[derive(Copy,Clone)]
pub enum PBDClock {
    BPM, BSCIF, AST, WDT, EIC, PICOUART
}

unsafe fn unlock(register_offset: u32) {
    volatile_store(&mut (*PM).unlock, 0xAA000000 | register_offset);
}

pub unsafe fn select_main_clock(clock: MainClock) {
    volatile_store(&mut (*PM).mcctrl, clock as u32);
}

macro_rules! mask_clock {
    ($module:ident: $field:ident | $mask:expr) => ({
        unlock(concat_idents!($module, _MASK_OFFSET));
        let val = volatile_load(&(*PM).$field) | ($mask);
        volatile_store(&mut (*PM).$field, val);
    });
}

pub unsafe fn enable_clock(clock: Clock) {
    match clock {
        Clock::HSB(v) => mask_clock!(HSB: hsbmask | 1 << (v as u32)),
        Clock::PBA(v) => mask_clock!(PBA: pbamask | 1 << (v as u32)),
        Clock::PBB(v) => mask_clock!(PBB: pbbmask | 1 << (v as u32)),
        Clock::PBD(v) => mask_clock!(PBD: pbdmask | 1 << (v as u32)),
    }
}

pub unsafe fn disable_clock(clock: Clock) {
    match clock {
        Clock::HSB(v) => mask_clock!(HSB: hsbmask | !(1 << (v as u32))),
        Clock::PBA(v) => mask_clock!(PBA: pbamask | !(1 << (v as u32))),
        Clock::PBB(v) => mask_clock!(PBB: pbbmask | !(1 << (v as u32))),
        Clock::PBD(v) => mask_clock!(PBD: pbdmask | !(1 << (v as u32))),
    }
}

