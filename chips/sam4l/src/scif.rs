// scif.rs -- System control interface for SAM4L
//
// This file includes support for the SCIF (Chapter 13 of SAML manual),
// which configures system clocks. Does not currently support all
// features/functionality: only main oscillator and generic clocks.
//
// Author: Philip Levis
// Date: Aug 2, 2015
//

use kernel::common::volatile_cell::VolatileCell;

pub enum Register {
    IER = 0x00,
    IDR = 0x04,
    IMR = 0x08,
    ISR = 0x0C,
    ICR = 0x10,
    PCLKSR = 0x14,
    UNLOCK = 0x18,
    CSCR = 0x1C,
    OSCCTRL0 = 0x20,
}

#[allow(non_camel_case_types)]
pub enum ClockSource {
    RCSYS = 0,
    OSC32K = 1,
    DFFL0 = 2,
    OSC0 = 3,
    RC80M = 4,
    RCFAST = 5,
    RC1M = 6,
    CLK_CPU = 7,
    CLK_HSB = 8,
    CLK_PBA = 9,
    CLK_PBB = 10,
    CLK_PBC = 11,
    CLK_PBD = 12,
    RC32K = 13,
    RESERVED1 = 14,
    CLK_1K = 15,
    PLL0 = 16,
    HRP = 17,
    FP = 18,
    GCLK_IN0 = 19,
    GCLK_IN1 = 20,
    GCLK11 = 21,
}

pub enum GenericClock {
    GCLK0,
    GCLK1,
    GCLK2,
    GCLK3,
    GCLK4,
    GCLK5,
    GCLK6,
    GCLK7,
    GCLK8,
    GCLK9,
    GCLK10,
    GCLK11,
}

#[repr(C, packed)]
struct Registers {
    ier: VolatileCell<u32>,
    idr: VolatileCell<u32>,
    imr: VolatileCell<u32>,
    isr: VolatileCell<u32>,
    icr: VolatileCell<u32>,
    pclksr: VolatileCell<u32>,
    unlock: VolatileCell<u32>,
    cscr: VolatileCell<u32>,
    // 0x20
    oscctrl0: VolatileCell<u32>,
    pll0: VolatileCell<u32>,
    dfll0conf: VolatileCell<u32>,
    dfll0val: VolatileCell<u32>,
    dfll0mul: VolatileCell<u32>,
    dfll0step: VolatileCell<u32>,
    dfll0ssg: VolatileCell<u32>,
    dfll0ratio: VolatileCell<u32>,
    // 0x40
    dfll0sync: VolatileCell<u32>,
    rccr: VolatileCell<u32>,
    rcfastcfg: VolatileCell<u32>,
    rfcastsr: VolatileCell<u32>,
    rc80mcr: VolatileCell<u32>,
    _reserved0: [u32; 4],
    // 0x64
    hrpcr: VolatileCell<u32>,
    fpcr: VolatileCell<u32>,
    fpmul: VolatileCell<u32>,
    fpdiv: VolatileCell<u32>,
    gcctrl0: VolatileCell<u32>,
    gcctrl1: VolatileCell<u32>,
    gcctrl2: VolatileCell<u32>,
    // 0x80
    gcctrl3: VolatileCell<u32>,
    gcctrl4: VolatileCell<u32>,
    gcctrl5: VolatileCell<u32>,
    gcctrl6: VolatileCell<u32>,
    gcctrl7: VolatileCell<u32>,
    gcctrl8: VolatileCell<u32>,
    gcctrl9: VolatileCell<u32>,
    gcctrl10: VolatileCell<u32>,
    // 0xa0
    gcctrl11: VolatileCell<u32>, // we leave out versions
}

const SCIF_BASE: usize = 0x400E0800;
static mut SCIF: *mut Registers = SCIF_BASE as *mut Registers;

#[repr(usize)]
pub enum Clock {
    ClockRCSys = 0,
    ClockOsc32 = 1,
    ClockAPB = 2,
    ClockGclk2 = 3,
    Clock1K = 4,
}

pub fn unlock(register: Register) {
    let val: u32 = 0xAA000000 | register as u32;
    unsafe {
        (*SCIF).unlock.set(val);
    }
}

pub fn oscillator_enable(internal: bool) {
    // Casting a bool to a u32 is 0,1
    let val: u32 = (1 << 16) | internal as u32;
    unlock(Register::OSCCTRL0);
    unsafe {
        (*SCIF).oscctrl0.set(val);
    }
}

pub fn oscillator_disable() {
    unlock(Register::OSCCTRL0);
    unsafe {
        (*SCIF).oscctrl0.set(0);
    }
}

pub fn generic_clock_disable(clock: GenericClock) {
    unsafe {
        match clock {
            GenericClock::GCLK0 => (*SCIF).gcctrl0.set(0),
            GenericClock::GCLK1 => (*SCIF).gcctrl1.set(0),
            GenericClock::GCLK2 => (*SCIF).gcctrl2.set(0),
            GenericClock::GCLK3 => (*SCIF).gcctrl3.set(0),
            GenericClock::GCLK4 => (*SCIF).gcctrl4.set(0),
            GenericClock::GCLK5 => (*SCIF).gcctrl5.set(0),
            GenericClock::GCLK6 => (*SCIF).gcctrl6.set(0),
            GenericClock::GCLK7 => (*SCIF).gcctrl7.set(0),
            GenericClock::GCLK8 => (*SCIF).gcctrl8.set(0),
            GenericClock::GCLK9 => (*SCIF).gcctrl9.set(0),
            GenericClock::GCLK10 => (*SCIF).gcctrl10.set(0),
            GenericClock::GCLK11 => (*SCIF).gcctrl11.set(0),
        };
    }
}

pub fn generic_clock_enable(clock: GenericClock, source: ClockSource) {
    // Oscillator field is bits 12:9, bit 0 is enable
    let val = (source as u32) << 8 | 1;
    unsafe {
        match clock {
            GenericClock::GCLK0 => (*SCIF).gcctrl0.set(val),
            GenericClock::GCLK1 => (*SCIF).gcctrl1.set(val),
            GenericClock::GCLK2 => (*SCIF).gcctrl2.set(val),
            GenericClock::GCLK3 => (*SCIF).gcctrl3.set(val),
            GenericClock::GCLK4 => (*SCIF).gcctrl4.set(val),
            GenericClock::GCLK5 => (*SCIF).gcctrl5.set(val),
            GenericClock::GCLK6 => (*SCIF).gcctrl6.set(val),
            GenericClock::GCLK7 => (*SCIF).gcctrl7.set(val),
            GenericClock::GCLK8 => (*SCIF).gcctrl8.set(val),
            GenericClock::GCLK9 => (*SCIF).gcctrl9.set(val),
            GenericClock::GCLK10 => (*SCIF).gcctrl10.set(val),
            GenericClock::GCLK11 => (*SCIF).gcctrl11.set(val),
        };
    }
}

// Note that most clocks can only support 8 bits of divider:
// interface does not currently check this. -pal
pub fn generic_clock_enable_divided(clock: GenericClock, source: ClockSource, divider: u16) {
    // Bits 31:16 -  divider
    // Bits 12:8  -  oscillator selection
    // Bit  1     -  divide enabled
    // Bit  0     -  clock enabled
    let val = (divider as u32) << 16 | ((source as u32) << 8) | 2 | 1;
    unsafe {
        match clock {
            GenericClock::GCLK0 => (*SCIF).gcctrl0.set(val),
            GenericClock::GCLK1 => (*SCIF).gcctrl1.set(val),
            GenericClock::GCLK2 => (*SCIF).gcctrl2.set(val),
            GenericClock::GCLK3 => (*SCIF).gcctrl3.set(val),
            GenericClock::GCLK4 => (*SCIF).gcctrl4.set(val),
            GenericClock::GCLK5 => (*SCIF).gcctrl5.set(val),
            GenericClock::GCLK6 => (*SCIF).gcctrl6.set(val),
            GenericClock::GCLK7 => (*SCIF).gcctrl7.set(val),
            GenericClock::GCLK8 => (*SCIF).gcctrl8.set(val),
            GenericClock::GCLK9 => (*SCIF).gcctrl9.set(val),
            GenericClock::GCLK10 => (*SCIF).gcctrl10.set(val),
            GenericClock::GCLK11 => (*SCIF).gcctrl11.set(val),
        };
    }
}
