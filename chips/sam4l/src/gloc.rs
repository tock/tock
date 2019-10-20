//! Implementation of the SAM4L glue logic controller (GLOC).
//!
//! GLOC input and output pins must be selected appropriately from table 3-1 in
//! the SAM4l manual.

use crate::pm::{self, Clock, PBAClock};
use crate::scif::{self, ClockSource, GenericClock};
use kernel::common::registers::{register_bitfields, ReadWrite};
use kernel::common::StaticRef;

#[repr(C)]
pub struct GlocRegisters {
    cr: ReadWrite<u32, Control::Register>,
    truth: ReadWrite<u32, Truth::Register>,
}

register_bitfields![u32,
    Control [
        /// Filter Enable
        FILTEN OFFSET(31) NUMBITS(1) [
            NoGlitchFilter = 0,
            GlitchFilter = 1
        ],
        /// Enable IN Inputs
        AEN OFFSET(0) NUMBITS(4) []
    ],

    Truth [
        /// Truth table values
        TRUTH OFFSET(0) NUMBITS(16) []
    ]
];

/// The GLOC's base addresses in memory (Section 7.1 of manual).
const GLOC_BASE_ADDR: usize = 0x40060000;

/// The number of bytes between each memory mapped GLOC LUT (Section 36.7).
const GLOC_LUT_SIZE: usize = 0x8;

/// Bitmasks for selecting the four GLOC inputs.
pub const IN_0_4: u8 = 0b0001; // IN0/IN4
pub const IN_1_5: u8 = 0b0010; // IN1/IN5
pub const IN_2_6: u8 = 0b0100; // IN2/IN6
pub const IN_3_7: u8 = 0b1000; // IN3/IN7

/// Available look up tables.
pub enum Lut {
    Lut0 = 0,
    Lut1 = 1,
}

pub struct Gloc {
    lut_regs: [StaticRef<GlocRegisters>; 2],
}

pub static mut GLOC: Gloc = Gloc {
    lut_regs: [get_lut_reg(Lut::Lut0), get_lut_reg(Lut::Lut1)],
};

/// Gets the memory location of the memory-mapped registers of a LUT.
const fn get_lut_reg(lut: Lut) -> StaticRef<GlocRegisters> {
    unsafe {
        StaticRef::new((GLOC_BASE_ADDR + (lut as usize) * GLOC_LUT_SIZE) as *const GlocRegisters)
    }
}

impl Gloc {
    /// Enables the GLOC by enabling its clock.
    pub fn enable(&self) {
        pm::enable_clock(Clock::PBA(PBAClock::GLOC));
    }

    /// Disables the GLOC by resetting the registers and disabling the clocks.
    pub fn disable(&mut self) {
        self.disable_lut(Lut::Lut0);
        self.disable_lut(Lut::Lut1);
        scif::generic_clock_disable(GenericClock::GCLK5);
        pm::disable_clock(Clock::PBA(PBAClock::GLOC));
    }

    /// Gets the memory-mapped registers associated with a LUT.
    fn lut_registers(&self, lut: Lut) -> &GlocRegisters {
        &*self.lut_regs[lut as usize]
    }

    /// Set the truth table values.
    pub fn configure_lut(&mut self, lut: Lut, config: u16) {
        let registers = self.lut_registers(lut);
        registers.truth.write(Truth::TRUTH.val(config as u32));
    }

    /// Enable selected LUT inputs.
    pub fn enable_lut_inputs(&mut self, lut: Lut, inputs: u8) {
        let registers = self.lut_registers(lut);
        let aen: u32 = registers.cr.read(Control::AEN) | (inputs as u32);
        registers.cr.modify(Control::AEN.val(aen));
    }

    /// Disable selected LUT inputs.
    pub fn disable_lut_inputs(&mut self, lut: Lut, inputs: u8) {
        let registers = self.lut_registers(lut);
        let aen: u32 = registers.cr.read(Control::AEN) & !(inputs as u32);
        registers.cr.modify(Control::AEN.val(aen));
    }

    /// Disable LUT by resetting registers.
    pub fn disable_lut(&mut self, lut: Lut) {
        let registers = self.lut_registers(lut);
        registers.truth.write(Truth::TRUTH.val(0));
        registers.cr.modify(Control::AEN.val(0));
    }

    /// Enable filter on output to prevent glitches.  This will delay the given
    /// LUT's output by 3-4 GCLK cycles.
    pub fn enable_lut_filter(&mut self, lut: Lut) {
        scif::generic_clock_enable(GenericClock::GCLK5, ClockSource::CLK_CPU);
        let registers = self.lut_registers(lut);
        registers.cr.modify(Control::FILTEN::GlitchFilter);
    }

    /// Disable output filter.
    pub fn disable_lut_filter(&mut self, lut: Lut) {
        let registers = self.lut_registers(lut);
        registers.cr.modify(Control::FILTEN::NoGlitchFilter);
    }
}
