//! Implementation of the system control interface for the SAM4L.
//!
//! This file includes support for the SCIF (Chapter 13 of SAML manual), which
//! configures system clocks. Does not currently support all
//! features/functionality: only main oscillator and generic clocks.
//!
//! - Author: Philip Levis
//! - Date: Aug 2, 2015

use bscif;
use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly, FieldValue};

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
    PLL0 = 0x24,
    DFLL0CONF = 0x28,
    DFLL0MUL = 0x30,
    DFLL0STEP = 0x34,
    DFLL0SSG = 0x38,
}

#[allow(non_camel_case_types)]
pub enum ClockSource {
    RCSYS = 0,
    OSC32K = 1,
    DFLL0 = 2,
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

#[repr(C)]
struct ScifRegisters {
    ier: WriteOnly<u32, Interrupt::Register>,
    idr: WriteOnly<u32, Interrupt::Register>,
    imr: ReadOnly<u32, Interrupt::Register>,
    isr: ReadOnly<u32, Interrupt::Register>,
    icr: WriteOnly<u32, Interrupt::Register>,
    pclksr: ReadOnly<u32, Interrupt::Register>,
    unlock: WriteOnly<u32, Unlock::Register>,
    cscr: ReadWrite<u32>,
    oscctrl0: ReadWrite<u32, Oscillator::Register>,
    pll0: ReadWrite<u32, PllControl::Register>,
    dfll0conf: ReadWrite<u32, Dfll::Register>,
    dfll0val: ReadWrite<u32>,
    dfll0mul: ReadWrite<u32>,
    dfll0step: ReadWrite<u32, DfllStep::Register>,
    dfll0ssg: ReadWrite<u32>,
    dfll0ratio: ReadOnly<u32>,
    dfll0sync: WriteOnly<u32>,
    rccr: ReadWrite<u32>,
    rcfastcfg: ReadWrite<u32>,
    rcfastsr: ReadOnly<u32>,
    rc80mcr: ReadWrite<u32>,
    _reserved0: [u32; 4],
    hrpcr: ReadWrite<u32>,
    fpcr: ReadWrite<u32>,
    fpmul: ReadWrite<u32>,
    fpdiv: ReadWrite<u32>,
    gcctrl0: ReadWrite<u32, GenericClockControl::Register>,
    gcctrl1: ReadWrite<u32, GenericClockControl::Register>,
    gcctrl2: ReadWrite<u32, GenericClockControl::Register>,
    gcctrl3: ReadWrite<u32, GenericClockControl::Register>,
    gcctrl4: ReadWrite<u32, GenericClockControl::Register>,
    gcctrl5: ReadWrite<u32, GenericClockControl::Register>,
    gcctrl6: ReadWrite<u32, GenericClockControl::Register>,
    gcctrl7: ReadWrite<u32, GenericClockControl::Register>,
    gcctrl8: ReadWrite<u32, GenericClockControl::Register>,
    gcctrl9: ReadWrite<u32, GenericClockControl::Register>,
    gcctrl10: ReadWrite<u32, GenericClockControl::Register>,
    gcctrl11: ReadWrite<u32, GenericClockControl::Register>,
    // Version registers are omitted here
}

register_bitfields![u32,
    Interrupt [
        RCFASTLOCKLOST 14,
        RCFASTLOCK 13,
        PLL0LOCKLOST 7,
        PLL0LOCK 6,
        DFLL0RCS 4,
        DFLL0RDY 3,
        DFLL0LOCKF 2,
        DFLL0LOCKC 1,
        OSC0RDY 0
    ],
    Unlock [
        KEY OFFSET(24) NUMBITS(8) [],
        ADDR OFFSET(0) NUMBITS(10) []
    ],
    Oscillator [
        OSCEN OFFSET(16) NUMBITS(1) [],
        STARTUP OFFSET(8) NUMBITS(4) [
            Cycles64 = 1,
            Cycles1024 = 14
        ],
        AGC OFFSET(3) NUMBITS(1) [],
        GAIN OFFSET(1) NUMBITS(2) [
            G0 = 0, G1 = 1, G2 = 2, G3 = 3, G4 = 4
        ],
        MODE OFFSET(0) NUMBITS(1) [
            External = 0,
            Crystal = 1
        ]
    ],
    Dfll [
        CALIB OFFSET(24) NUMBITS(4) [],
        FCD OFFSET(23) NUMBITS(1) [],
        RANGE OFFSET(16) NUMBITS(2) [],
        QLDIS OFFSET(6) NUMBITS(1) [],
        CCDIS OFFSET(5) NUMBITS(1) [],
        LLAW OFFSET(3) NUMBITS(1) [],
        STABLE OFFSET(2) NUMBITS(1) [],
        MODE OFFSET(1) NUMBITS(1) [
            OpenLoop = 0,
            ClosedLoop = 1
        ],
        EN OFFSET(0) NUMBITS(1) []
    ],
    DfllStep [
        CSTEP OFFSET(16) NUMBITS(5) [],
        FSTEP OFFSET(0) NUMBITS(8) []
    ],
    GenericClockControl [
        DIV OFFSET(16) NUMBITS(16) [],
        OCSEL OFFSET(8) NUMBITS(5) [
            // values available from enum ClockSource
        ],
        DIVEN OFFSET(1) NUMBITS(1) [],
        CEN OFFSET(0) NUMBITS(1) []
    ],
    PllControl [
        PLLCOUNT OFFSET(24) NUMBITS(6) [
            Max = 0x3F
        ],
        PLLMUL OFFSET(16) NUMBITS(4) [],
        PLLDIV OFFSET(8) NUMBITS(4) [],
        PLLOSC OFFSET(1) NUMBITS(2) [
            OSC0 = 0,
            GCLK9 = 1
        ],
        PLLOPT OFFSET(3) NUMBITS(3) [
            DivideBy2 = 2
            // Other option combinations omitted here, as it
            // is not clear in which order the bits are stored
        ],
        PLLEN OFFSET(0) NUMBITS(1) []
    ]
];

const SCIF_BASE: usize = 0x400E0800;
static mut SCIF: *mut ScifRegisters = SCIF_BASE as *mut ScifRegisters;

#[repr(usize)]
pub enum Clock {
    ClockRCSys = 0,
    ClockOsc32 = 1,
    ClockAPB = 2,
    ClockGclk2 = 3,
    Clock1K = 4,
}

pub fn unlock(register: Register) {
    unsafe {
        (*SCIF).unlock.write(Unlock::KEY.val(0xAA) + Unlock::ADDR.val(register as u32));
    }
}

pub fn oscillator_enable(internal: bool) {
    let mode = if internal { Oscillator::MODE::Crystal } else { Oscillator::MODE::External };
    unlock(Register::OSCCTRL0);
    unsafe {
        (*SCIF).oscctrl0.write(Oscillator::OSCEN::SET + mode);
    }
}

pub fn oscillator_disable() {
    unlock(Register::OSCCTRL0);
    unsafe {
        (*SCIF).oscctrl0.write(Oscillator::OSCEN::CLEAR);
    }
}

pub unsafe fn setup_dfll_rc32k_48mhz() {

    unsafe fn wait_dfll0_ready() {
        while (*SCIF).pclksr.matches(Interrupt::DFLL0RDY::CLEAR) {}
    }

    // Check to see if the DFLL is already setup or is not locked
    if (*SCIF).dfll0conf.matches(Dfll::MODE::OpenLoop + Dfll::EN::CLEAR) ||
       (*SCIF).pclksr.matches(Interrupt::DFLL0LOCKF::CLEAR) {

        // Enable the GENCLK_SRC_RC32K
        bscif::enable_rc32k();

        // Next, initialize closed-loop mode ...

        // Must do a SCIF sync before reading the SCIF registers?
        // 13.7.16: "To be able to read the current value of DFLLxVAL or DFLLxRATIO, this bit must
        //    be written to one. The updated value are available in DFLLxVAL or DFLLxRATIO when
        //    PCLKSR.DFLL0RDY is set."
        (*SCIF).dfll0sync.set(0x01);
        wait_dfll0_ready();

        // Read the current DFLL settings
        let scif_dfll0conf = (*SCIF).dfll0conf.get();
        // Compute some new configuration field values
        let new_config_fields = Dfll::EN::SET +
                                Dfll::MODE::ClosedLoop +
                                Dfll::RANGE.val(2);
        // Apply the new field values to the current config value,
        // for use further below ...
        let scif_dfll0conf_new = new_config_fields.modify(scif_dfll0conf);

        // Enable the generic clock with RC32K and no divider
        (*SCIF).gcctrl0.write(GenericClockControl::CEN::SET +
                              GenericClockControl::OCSEL.val(ClockSource::RC32K as u32) +
                              GenericClockControl::DIVEN::CLEAR +
                              GenericClockControl::DIV.val(0));

        // Setup DFLL. Must wait after every operation for the ready bit to go high.
        //
        // First, enable dfll
        unlock(Register::DFLL0CONF);
        (*SCIF).dfll0conf.write(Dfll::EN::SET);
        wait_dfll0_ready();

        // Set step values
        unlock(Register::DFLL0STEP);
        (*SCIF).dfll0step.write(DfllStep::FSTEP.val(4) + DfllStep::CSTEP.val(4));
        wait_dfll0_ready();

        // Set multiply value
        unlock(Register::DFLL0MUL);
        // 1464 = 48000000 / 32768
        (*SCIF).dfll0mul.set(1464);
        wait_dfll0_ready();

        // Set SSG value
        unlock(Register::DFLL0SSG);
        // just set to zero to disable
        (*SCIF).dfll0ssg.set(0);
        wait_dfll0_ready();

        // Set actual configuration
        unlock(Register::DFLL0CONF);
        // we already prepared this value
        (*SCIF).dfll0conf.set(scif_dfll0conf_new);

        // Now wait for the DFLL to become locked
        while (*SCIF).pclksr.matches(Interrupt::DFLL0LOCKF::CLEAR) {}
    }
}

pub unsafe fn setup_osc_16mhz_fast_startup() {
    // Enable the OSC0 with ~557us startup time
    unlock(Register::OSCCTRL0);
    (*SCIF).oscctrl0.write(Oscillator::OSCEN::SET +
                           Oscillator::STARTUP::Cycles64 +
                           Oscillator::GAIN::G4 +
                           Oscillator::MODE::Crystal);

    // Wait for oscillator to be ready
    while (*SCIF).pclksr.matches(Interrupt::OSC0RDY::CLEAR) {}
}

pub unsafe fn setup_osc_16mhz_slow_startup() {
    // Enable the OSC0 with ~8.9ms startup time
    unlock(Register::OSCCTRL0);
    (*SCIF).oscctrl0.write(Oscillator::OSCEN::SET +
                           Oscillator::STARTUP::Cycles1024 +
                           Oscillator::GAIN::G4 +
                           Oscillator::MODE::Crystal);

    // Wait for oscillator to be ready
    while (*SCIF).pclksr.matches(Interrupt::OSC0RDY::CLEAR) {}
}

pub unsafe fn setup_pll_osc_48mhz() {
    unlock(Register::PLL0);
    (*SCIF).pll0.write(PllControl::PLLCOUNT::Max +
                       PllControl::PLLMUL.val(5) +
                       PllControl::PLLDIV.val(1) +
                       PllControl::PLLOPT::DivideBy2 +
                       PllControl::PLLOSC::OSC0 +
                       PllControl::PLLEN::SET);

    // Wait for the PLL to become locked
    while (*SCIF).pclksr.matches(Interrupt::PLL0LOCK::CLEAR) {}
}

pub fn generic_clock_disable(clock: GenericClock) {
    generic_clock_control_write(clock, GenericClockControl::CEN::CLEAR);
}

pub fn generic_clock_enable(clock: GenericClock, source: ClockSource) {
    generic_clock_control_write(clock, GenericClockControl::OCSEL.val(source as u32) +
                                       GenericClockControl::CEN::SET);
}

// Note that most clocks can only support 8 bits of divider:
// interface does not currently check this. -pal
pub fn generic_clock_enable_divided(clock: GenericClock, source: ClockSource, divider: u16) {
    generic_clock_control_write(clock, GenericClockControl::OCSEL.val(source as u32) +
                                       GenericClockControl::DIVEN::SET +
                                       GenericClockControl::DIV.val(divider as u32) +
                                       GenericClockControl::CEN::SET);
}

fn generic_clock_control_write(clock: GenericClock, val: FieldValue<u32, GenericClockControl::Register>) {
    unsafe {
        match clock {
            GenericClock::GCLK0 => (*SCIF).gcctrl0.write(val),
            GenericClock::GCLK1 => (*SCIF).gcctrl1.write(val),
            GenericClock::GCLK2 => (*SCIF).gcctrl2.write(val),
            GenericClock::GCLK3 => (*SCIF).gcctrl3.write(val),
            GenericClock::GCLK4 => (*SCIF).gcctrl4.write(val),
            GenericClock::GCLK5 => (*SCIF).gcctrl5.write(val),
            GenericClock::GCLK6 => (*SCIF).gcctrl6.write(val),
            GenericClock::GCLK7 => (*SCIF).gcctrl7.write(val),
            GenericClock::GCLK8 => (*SCIF).gcctrl8.write(val),
            GenericClock::GCLK9 => (*SCIF).gcctrl9.write(val),
            GenericClock::GCLK10 => (*SCIF).gcctrl10.write(val),
            GenericClock::GCLK11 => (*SCIF).gcctrl11.write(val),
        };
    }
}
