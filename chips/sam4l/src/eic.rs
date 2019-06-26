//! Implementation of the SAM4L External Interrupt Controller (EIC).
//!
//! Datasheet section "21. External Interrupt Controller (EIC)".
//!
//! The External Interrupt Controller (EIC) allows pins to be configured as external
//! interrupts. Each external interrupt has its own interrupt request and can be individually
//! masked. Each external interrupt can generate an interrupt on rising or falling edge, or
//! high or low level. Every interrupt input has a configurable filter to remove spikes from
//! the interrupt source. Every interrupt pin can also be configured to be asynchronous in order
//! to wake up the part from sleep modes where the CLK_SYNC clock has been disabled.
//!
//! - Author: Josh Zhang  <jiashuoz@princeton.edu>
//! - Updated: June 25, 2019

use crate::pm::{self, Clock, PBDClock};
use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::debug;
use kernel::hil;

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
        NMI 0   // Non-Maskable Interrupt
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
    fn enable(&self) {
        pm::enable_clock(Clock::PBD(PBDClock::EIC));
        self.enabled.set(true);
        debug!("{}", pm::is_clock_enabled(Clock::PBD(PBDClock::EIC)));
    }

    fn disable(&self) {
        pm::disable_clock(Clock::PBD(PBDClock::EIC));
        self.enabled.set(false);
    }

    fn line_enable(&self, line_num: usize) {
        let regs: &EicRegisters = &*self.registers;
        let mask: u32 = 1 << line_num;
        // en == WriteOnly
        match line_num {
            0 => regs.en.write(Interrupt::INT.val(mask)),
            1 => regs.en.write(Interrupt::INT.val(mask)),
            2 => regs.en.write(Interrupt::INT.val(mask)),
            3 => regs.en.write(Interrupt::INT.val(mask)),
            4 => regs.en.write(Interrupt::INT.val(mask)),
            5 => regs.en.write(Interrupt::INT.val(mask)),
            6 => regs.en.write(Interrupt::INT.val(mask)),
            7 => regs.en.write(Interrupt::INT.val(mask)),
            8 => regs.en.write(Interrupt::INT.val(mask)),
            _ => debug!("not supported!"),
        }

        self.line_enable_interrupt(line_num);
    }

    fn line_disable(&self, line_num: usize) {
        let regs: &EicRegisters = &*self.registers;
        let mask: u32 = 1 << line_num;
        match line_num {
            0 => regs.dis.write(Interrupt::INT.val(mask)),
            1 => regs.dis.write(Interrupt::INT.val(mask)),
            2 => regs.dis.write(Interrupt::INT.val(mask)),
            3 => regs.dis.write(Interrupt::INT.val(mask)),
            4 => regs.dis.write(Interrupt::INT.val(mask)),
            5 => regs.dis.write(Interrupt::INT.val(mask)),
            6 => regs.dis.write(Interrupt::INT.val(mask)),
            7 => regs.dis.write(Interrupt::INT.val(mask)),
            8 => regs.dis.write(Interrupt::INT.val(mask)),
            _ => debug!("not supported!"),
        }

        self.line_disable_interrupt(line_num);
    }

    fn line_configure(
        &self,
        line_num: usize,
        int_mode: hil::eic::InterruptMode,
        filter: hil::eic::FilterMode,
        syn_mode: hil::eic::SynchronizationMode,
    ) {
        let mask: u32 = 1 << line_num;

        // regs.mode.set(original_mode & !mask);

        let mode_bits = match int_mode {
            hil::eic::InterruptMode::RisingEdge => 0b00,
            hil::eic::InterruptMode::FallingEdge => 0b01,
            hil::eic::InterruptMode::HighLevel => 0b10,
            hil::eic::InterruptMode::LowLevel => 0b11,
        };

        self.set_interrupt_mode(mode_bits, mask);

        match filter {
            hil::eic::FilterMode::FilterEnable => self.line_enable_filter(mask),
            hil::eic::FilterMode::FilterDisable => self.line_disable_filter(mask),
        }

        match syn_mode {
            hil::eic::SynchronizationMode::Synchronous => self.line_disable_asyn(mask),
            hil::eic::SynchronizationMode::Asynchronous => self.line_enable_asyn(mask),
        }
    }
}

impl<'a> Eic<'a> {
    pub fn set_interrupt_mode(&self, mode_bits: u8, mask: u32) {
        let regs: &EicRegisters = &*self.registers;
        let original_mode: u32 = regs.mode.get();
        let original_level: u32 = regs.level.get();
        let original_edge: u32 = regs.edge.get();

        if mode_bits & 0b10 != 0 {
            regs.mode.set(original_mode & !mask);
        } else {
            regs.mode.set(original_mode | mask);
        }

        if mode_bits & 0b01 != 0 {
            regs.edge.set(original_edge & !mask); // falling edge
            regs.level.set(original_level & !mask); // low level
        } else {
            regs.edge.set(original_edge | mask); // rising edge
            regs.level.set(original_level | mask); // high level
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

    pub fn set_client(&self, client: &'a hil::eic::Client, line_num: usize) {
        self.callbacks[line_num].set(client);
    }

    pub fn handle_interrupt(&self, line_num: usize) {
        self.line_clear_interrupt(line_num);
        self.callbacks[line_num].map(|cb| {
            cb.fired();
        });
    }

    /// Clears the interrupt flag of line. Should be called after handling interrupt
    /// Sets interrupt clear register
    pub fn line_clear_interrupt(&self, line_num: usize) {
        let regs: &EicRegisters = &*self.registers;
        let mask: u32 = 1 << line_num;
        // icr WriteOnly
        match line_num {
            0 => regs.icr.write(Interrupt::INT.val(mask)),
            1 => regs.icr.write(Interrupt::INT.val(mask)),
            2 => regs.icr.write(Interrupt::INT.val(mask)),
            3 => regs.icr.write(Interrupt::INT.val(mask)),
            4 => regs.icr.write(Interrupt::INT.val(mask)),
            5 => regs.icr.write(Interrupt::INT.val(mask)),
            6 => regs.icr.write(Interrupt::INT.val(mask)),
            7 => regs.icr.write(Interrupt::INT.val(mask)),
            8 => regs.icr.write(Interrupt::INT.val(mask)),
            _ => debug!("not supported!"),
        }
    }

    pub fn line_is_enabled(&self, line_num: usize) -> bool {
        let regs: &EicRegisters = &*self.registers;
        let mask: u32 = 1 << line_num;
        return (mask & regs.ctrl.get()) != 0;
    }

    // Enables the propagation from the EIC to the interrupt controller of the external interrupt on a specified
    // line.
    pub fn line_enable_interrupt(&self, line_num: usize) {
        let regs: &EicRegisters = &*self.registers;
        let mask: u32 = 1 << line_num;
        // ier WriteOnly
        match line_num {
            0 => regs.ier.write(Interrupt::INT.val(mask)),
            1 => regs.ier.write(Interrupt::INT.val(mask)),
            2 => regs.ier.write(Interrupt::INT.val(mask)),
            3 => regs.ier.write(Interrupt::INT.val(mask)),
            4 => regs.ier.write(Interrupt::INT.val(mask)),
            5 => regs.ier.write(Interrupt::INT.val(mask)),
            6 => regs.ier.write(Interrupt::INT.val(mask)),
            7 => regs.ier.write(Interrupt::INT.val(mask)),
            8 => regs.ier.write(Interrupt::INT.val(mask)),
            _ => debug!("not supported!"),
        }
    }

    pub fn line_disable_interrupt(&self, line_num: usize) {
        let regs: &EicRegisters = &*self.registers;
        let mask: u32 = 1 << line_num;
        match line_num {
            0 => regs.idr.write(Interrupt::INT.val(mask)),
            1 => regs.idr.write(Interrupt::INT.val(mask)),
            2 => regs.idr.write(Interrupt::INT.val(mask)),
            3 => regs.idr.write(Interrupt::INT.val(mask)),
            4 => regs.idr.write(Interrupt::INT.val(mask)),
            5 => regs.idr.write(Interrupt::INT.val(mask)),
            6 => regs.idr.write(Interrupt::INT.val(mask)),
            7 => regs.idr.write(Interrupt::INT.val(mask)),
            8 => regs.idr.write(Interrupt::INT.val(mask)),
            _ => debug!("not supported!"),
        }
    }

    /// Reads IMR register
    pub fn line_interrupt_is_enabled(&self, line_num: usize) -> bool {
        let regs: &EicRegisters = &*self.registers;
        let mask: u32 = 1 << line_num;
        return (mask & regs.imr.get()) != 0;
    }

    /// Tells whether an EIC interrupt line is pending.
    pub fn line_interrupt_pending(&self, line_num: usize) -> bool {
        let regs: &EicRegisters = &*self.registers;
        let mask: u32 = 1 << line_num;
        return (mask & regs.isr.get()) != 0;
    }

    pub fn line_enable_filter(&self, mask: u32) {
        let regs: &EicRegisters = &*self.registers;
        let original_filter: u32 = regs.filter.get();
        regs.filter.set(original_filter | mask);
    }

    pub fn line_disable_filter(&self, mask: u32) {
        let regs: &EicRegisters = &*self.registers;
        let original_filter: u32 = regs.filter.get();
        regs.filter.set(original_filter & (!mask));
    }

    pub fn line_enable_filter_is_enabled(&self, line_num: usize) -> bool {
        let regs: &EicRegisters = &*self.registers;
        let mask: u32 = 1 << line_num;
        return (mask & regs.filter.get()) != 0;
    }

    pub fn line_enable_asyn(&self, mask: u32) {
        let regs: &EicRegisters = &*self.registers;
        let original_asyn: u32 = regs.asynchronous.get();
        regs.asynchronous
            .modify(Interrupt::INT.val(original_asyn | mask));
    }

    pub fn line_disable_asyn(&self, mask: u32) {
        let regs: &EicRegisters = &*self.registers;
        let original_asyn: u32 = regs.asynchronous.get();
        regs.asynchronous
            .modify(Interrupt::INT.val(original_asyn & (!mask)));
    }

    pub fn line_asyn_is_enabled(&self, line_num: usize) -> bool {
        let regs: &EicRegisters = &*self.registers;
        let mask: u32 = 1 << line_num;
        return (mask & regs.asynchronous.get()) != 0;
    }
}

/// Static state to manage the EIC
pub static mut EIC: Eic = Eic::new();
