//! Implementation of the SAM4L External Interrupt Controller (EIC).
//!
//! Datasheet section "21. External Interrupt Controller (EIC)".
//!
//! The External Interrupt Controller (EIC) allows pins to be configured as external
//! interrupts. Each external interrupt has its own interrupt request and can be individually
//! interrupt_lineed. Each external interrupt can generate an interrupt on rising or falling edge, or
//! high or low level. Every interrupt input has a configurable filter to remove spikes from
//! the interrupt source. Every interrupt pin can also be configured to be asynchronous in order
//! to wake up the part from sleep modes where the CLK_SYNC clock has been disabled.
//!
//! - Author: Josh Zhang  <jiashuoz@cs.princeton.edu>
//! - Updated: June 25, 2019

use crate::pm::{self, Clock, PBDClock};
use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::debug;
use kernel::hil;

/// Representation of an EIC line on the SAM4L.
pub struct EicLine {
    line_number: u32,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum Line {
    LINE0 = 0x00, // NMI
    LINE1 = 0x01, // EXT1
    LINE2 = 0x02, // EXT2
    LINE3 = 0x03, // EXT3
    LINE4 = 0x04, // EXT4
    LINE5 = 0x05, // EXT5
    LINE6 = 0x06, // EXT6
    LINE7 = 0x07, // EXT7
    LINE8 = 0x08, // EXT8
}

/// Initialization of an EIC line.
impl EicLine {
    /// Create a new EIC line.
    ///
    /// - `line`: Line enum representing the line number
    const fn new(line: Line) -> EicLine {
        EicLine {
            line_number: ((line as u8) & 0x0F) as u32,
        }
    }
}

pub static mut LINE_EIC0: EicLine = EicLine::new(Line::LINE0);
pub static mut LINE_EIC1: EicLine = EicLine::new(Line::LINE1);
pub static mut LINE_EIC2: EicLine = EicLine::new(Line::LINE2);
pub static mut LINE_EIC3: EicLine = EicLine::new(Line::LINE3);
pub static mut LINE_EIC4: EicLine = EicLine::new(Line::LINE4);
pub static mut LINE_EIC5: EicLine = EicLine::new(Line::LINE5);
pub static mut LINE_EIC6: EicLine = EicLine::new(Line::LINE6);
pub static mut LINE_EIC7: EicLine = EicLine::new(Line::LINE7);
pub static mut LINE_EIC8: EicLine = EicLine::new(Line::LINE8);

#[repr(C)]
struct EicRegisters {
    ier: WriteOnly<u32, Interrupt::Register>,          // 0x00
    idr: WriteOnly<u32, Interrupt::Register>,          // 0x04
    imr: ReadOnly<u32, Interrupt::Register>,           // 0x08
    isr: ReadOnly<u32, Interrupt::Register>,           // 0x0c
    icr: WriteOnly<u32, Interrupt::Register>,          // 0x10
    mode: ReadWrite<u32, Interrupt::Register>,         // 0x14
    edge: ReadWrite<u32, Interrupt::Register>,         // 0x18
    level: ReadWrite<u32, Interrupt::Register>,        // 0x1c
    filter: ReadWrite<u32, Interrupt::Register>,       // 0x20
    test: ReadWrite<u32, Test::Register>,              // 0x24
    asynchronous: ReadWrite<u32, Interrupt::Register>, // 0x28
    _reserved0: ReadOnly<u32>,                         // 0x02c, skip
    en: WriteOnly<u32, Interrupt::Register>,           // 0x30
    dis: WriteOnly<u32, Interrupt::Register>,          // 0x34
    ctrl: ReadOnly<u32, Interrupt::Register>,          // 0x38
}

// IER: Writing a one to this bit will set the corresponding bit in IMR.
// IDR: Writing a one to this bit will clear the corresponding bit in IMR.
// IMR: 0: The corresponding interrupt is disabled.
//      1: The corresponding interrupt is enabled.
// ISR: 0: An interrupt event has not occurred.
//      1: An interrupt event has occurred.
// ICR: Writing a one to this bit will clear the corresponding bit in ISR.
// MODE:    0: The external interrupt is edge triggered.
//          1: The external interrupt is level triggered.'
// EDGE:    0: The external interrupt triggers on falling edge.
//          1: The external interrupt triggers on rising edge.
// LEVEL:   0: The external interrupt triggers on low level.
//          1: The external interrupt triggers on high level.
// FILTER:  0: The external interrupt is not filtered.
//          1: The external interrupt is filtered.
// ASYNC:   0: The external interrupt is synchronized to CLK_SYNC.
//          1: The external interrupt is asynchronous.
// EN: Writing a one to this bit will enable the corresponding external interrupt.
// DIS: Writing a one to this bit will disable the corresponding external interrupt.
// CTRL:    0: The corresponding external interrupt is disabled.
//          1: The corresponding external interrupt is enabled.

register_bitfields![
    u32,
    Interrupt [
        INT OFFSET(0) NUMBITS(32) []
    ],
    Test [
        //0: This bit disables external interrupt test mode.
        //1: This bit enables external interrupt test mode.
        TESTEN 31,

        // Writing a zero to this bit will set the input value to INTn to zero, if test mode is enabled. 
        // Writing a one to this bit will set the input value to INTn to one, if test mode is enabled.
        INT30 30,
        INT29 29,
        INT28 28,
        INT27 27,
        INT26 26,
        INT25 25,
        INT24 24,
        INT23 23,
        INT22 22,
        INT21 21,
        INT20 20,
        INT19 19,
        INT18 18,
        INT17 17,
        INT16 16,
        INT15 15,
        INT14 14,
        INT13 13,
        INT12 12,
        INT11 11,
        INT10 10,
        INT9 9,
        INT8 8,
        INT7 7,
        INT6 6,
        INT5 5,
        INT4 4,
        INT3 3,
        INT2 2,
        INT1 1,
        NMI 0   // Non-interrupt_lineable Interrupt
    ]
];

// Page 59 of SAM4L data sheet
const EIC_BASE: StaticRef<EicRegisters> =
    unsafe { StaticRef::new(0x400F1000 as *const EicRegisters) };

pub struct Eic<'a> {
    registers: StaticRef<EicRegisters>,
    enabled: Cell<bool>,
    callbacks: [OptionalCell<&'a hil::eic::Client>; 9],
}

impl<'a> hil::eic::ExternalInterruptController for Eic<'a> {
    type Line = EicLine;

    fn line_enable(
        &self,
        line: &Self::Line,
        int_mode: hil::eic::InterruptMode,
        filter_mode: hil::eic::FilterMode,
        syn_mode: hil::eic::SynchronizationMode,
    ) {
        if !self.is_enabled() {
            self.enable();
        }

        let interrupt_line: u32 = 1 << line.line_number;
        let regs: &EicRegisters = &*self.registers;

        regs.en.write(Interrupt::INT.val(interrupt_line));

        self.line_configure(interrupt_line, int_mode, filter_mode, syn_mode);
        self.line_enable_interrupt(interrupt_line);
    }

    fn line_disable(&self, line: &Self::Line) {
        if !self.is_enabled() {
            return;
        }

        let interrupt_line: u32 = 1 << line.line_number;
        let regs: &EicRegisters = &*self.registers;
        regs.dis.write(Interrupt::INT.val(interrupt_line));
        self.line_disable_interrupt(interrupt_line);
    }
}

impl<'a> Eic<'a> {
    pub fn line_configure(
        &self,
        interrupt_line: u32,
        int_mode: hil::eic::InterruptMode,
        filter_mode: hil::eic::FilterMode,
        syn_mode: hil::eic::SynchronizationMode,
    ) {
        let mode_bits = match int_mode {
            hil::eic::InterruptMode::RisingEdge => 0b00,
            hil::eic::InterruptMode::FallingEdge => 0b01,
            hil::eic::InterruptMode::HighLevel => 0b10,
            hil::eic::InterruptMode::LowLevel => 0b11,
        };

        self.set_interrupt_mode(mode_bits, interrupt_line);

        match filter_mode {
            hil::eic::FilterMode::FilterEnable => self.line_enable_filter(interrupt_line),
            hil::eic::FilterMode::FilterDisable => self.line_disable_filter(interrupt_line),
        }

        match syn_mode {
            hil::eic::SynchronizationMode::Synchronous => self.line_disable_asyn(interrupt_line),
            hil::eic::SynchronizationMode::Asynchronous => self.line_enable_asyn(interrupt_line),
        }
    }

    pub fn enable(&self) {
        pm::enable_clock(Clock::PBD(PBDClock::EIC));
        self.enabled.set(true);
    }

    pub fn disable(&self) {
        pm::disable_clock(Clock::PBD(PBDClock::EIC));
        self.enabled.set(false);
    }

    pub fn set_interrupt_mode(&self, mode_bits: u8, interrupt_line: u32) {
        let regs: &EicRegisters = &*self.registers;
        let original_mode: u32 = regs.mode.get();
        let original_level: u32 = regs.level.get();
        let original_edge: u32 = regs.edge.get();

        if mode_bits & 0b10 != 0 {
            regs.mode.set(original_mode | interrupt_line); // 0b10 or 0b11 -> level
        } else {
            regs.mode.set(original_mode & !interrupt_line); // 0b00 or 0b01 -> edge
        }

        if mode_bits & 0b01 != 0 {
            regs.edge.set(original_edge & !interrupt_line); // falling edge
            regs.level.set(original_level & !interrupt_line); // low level
        } else {
            regs.edge.set(original_edge | interrupt_line); // rising edge
            regs.level.set(original_level | interrupt_line); // high level
        }
    }

    const fn new() -> Eic<'a> {
        Eic {
            registers: EIC_BASE,
            enabled: Cell::new(false),
            callbacks: [
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
            ],
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.get()
    }

    pub fn set_client(&self, client: &'a hil::eic::Client, line_number: usize) {
        if line_number > 8 {
            return;
        }
        self.callbacks.get(line_number).map(|c| c.set(client));
    }

    pub fn handle_interrupt(&self, line_number: usize) {
        self.line_clear_interrupt(line_number);
        self.callbacks[line_number].map(|cb| {
            cb.fired();
        });
    }

    /// Clears the interrupt flag of line. Should be called after handling interrupt
    /// Sets interrupt clear register
    pub fn line_clear_interrupt(&self, line_number: usize) {
        let regs: &EicRegisters = &*self.registers;

        // line_number always sits in the range 1-8, no need to check shift overflow
        let interrupt_line: u32 = 1 << line_number;

        regs.icr.write(Interrupt::INT.val(interrupt_line));
    }

    pub fn line_is_enabled(&self, line_number: usize) -> bool {
        let regs: &EicRegisters = &*self.registers;
        let interrupt_line: u32 = 1 << line_number;
        (interrupt_line & regs.ctrl.get()) != 0
    }

    /// Enables the propagation from the EIC to the interrupt controller of the external interrupt on a specified
    /// line.
    pub fn line_enable_interrupt(&self, interrupt_line: u32) {
        let regs: &EicRegisters = &*self.registers;

        regs.ier.write(Interrupt::INT.val(interrupt_line));
    }

    pub fn line_disable_interrupt(&self, interrupt_line: u32) {
        let regs: &EicRegisters = &*self.registers;

        regs.idr.write(Interrupt::INT.val(interrupt_line));
    }

    pub fn line_interrupt_is_enabled(&self, line_number: usize) -> bool {
        let regs: &EicRegisters = &*self.registers;
        let interrupt_line: u32 = 1 << line_number;
        (interrupt_line & regs.imr.get()) != 0
    }

    pub fn line_interrupt_pending(&self, line_number: usize) -> bool {
        let regs: &EicRegisters = &*self.registers;
        let interrupt_line: u32 = 1 << line_number;
        (interrupt_line & regs.isr.get()) != 0
    }

    /// Enables filtering mode on synchronous interrupt
    pub fn line_enable_filter(&self, interrupt_line: u32) {
        let regs: &EicRegisters = &*self.registers;
        let original_filter: u32 = regs.filter.get();
        regs.filter.set(original_filter | interrupt_line);
    }

    /// Disables filtering mode on synchronous interrupt
    pub fn line_disable_filter(&self, interrupt_line: u32) {
        let regs: &EicRegisters = &*self.registers;
        let original_filter: u32 = regs.filter.get();
        regs.filter.set(original_filter & (!interrupt_line));
    }

    pub fn line_enable_filter_is_enabled(&self, line_number: usize) -> bool {
        let regs: &EicRegisters = &*self.registers;
        let interrupt_line: u32 = 1 << line_number;
        (interrupt_line & regs.filter.get()) != 0
    }

    /// Enables asynchronous mode
    pub fn line_enable_asyn(&self, interrupt_line: u32) {
        let regs: &EicRegisters = &*self.registers;
        let original_asyn: u32 = regs.asynchronous.get();
        regs.asynchronous
            .modify(Interrupt::INT.val(original_asyn | interrupt_line));
    }

    /// Disables asynchronous mode, goes back to synchronous mode
    pub fn line_disable_asyn(&self, interrupt_line: u32) {
        let regs: &EicRegisters = &*self.registers;
        let original_asyn: u32 = regs.asynchronous.get();
        regs.asynchronous
            .modify(Interrupt::INT.val(original_asyn & (!interrupt_line)));
    }

    pub fn line_asyn_is_enabled(&self, line_number: usize) -> bool {
        let regs: &EicRegisters = &*self.registers;
        let interrupt_line: u32 = 1 << line_number;
        (interrupt_line & regs.asynchronous.get()) != 0
    }
}

/// Static state to manage the EIC
pub static mut EIC: Eic = Eic::new();
