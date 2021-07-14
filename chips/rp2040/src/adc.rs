use core::cell::Cell;
use kernel::common::{cells::OptionalCell, StaticRef};
use kernel::hil;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::{
    common::registers::{register_bitfields, register_structs, ReadWrite},
    ErrorCode,
};

register_structs! {
    /// Control and data interface to SAR ADC
    AdcRegisters {
        /// ADC Control and Status
        (0x000 => cs: ReadWrite<u32, CS::Register>),
        /// Result of most recent ADC conversion
        (0x004 => result: ReadWrite<u32, RESULT::Register>),
        /// FIFO control and status
        (0x008 => fcs: ReadWrite<u32, FCS::Register>),
        /// Conversion result FIFO
        (0x00C => fifo: ReadWrite<u32, FIFO::Register>),
        /// Clock divider. If non-zero, CS_START_MANY will start conversions
        /// at regular intervals rather than back-to-back.
        /// The divider is reset when either of these fields are written.
        /// Total period is 1 + INT + FRAC / 256
        (0x010 => div: ReadWrite<u32, DIV::Register>),
        /// Raw Interrupts
        (0x014 => intr: ReadWrite<u32, INTR::Register>),
        /// Interrupt Enable
        (0x018 => inte: ReadWrite<u32, INTE::Register>),
        /// Interrupt Force
        (0x01C => intf: ReadWrite<u32, INTE::Register>),
        /// Interrupt status after masking & forcing
        (0x020 => ints: ReadWrite<u32, INTE::Register>),
        (0x024 => @END),
    }
}
register_bitfields![u32,
CS [
    /// Round-robin sampling. 1 bit per channel. Set all bits to 0 to disable.
    /// Otherwise, the ADC will cycle through each enabled channel in a
    /// The first channel to be sampled will be the one currently indica
    /// AINSEL will be updated after each conversion with the newly-sele
    RROBIN OFFSET(16) NUMBITS(5) [],
    /// Select analog mux input. Updated automatically in round-robin mode.
    AINSEL OFFSET(12) NUMBITS(3) [],
    /// Some past ADC conversion encountered an error. Write 1 to clear.
    ERR_STICKY OFFSET(10) NUMBITS(1) [],
    /// The most recent ADC conversion encountered an error; result is undefined or nois
    ERR OFFSET(9) NUMBITS(1) [],
    /// 1 if the ADC is ready to start a new conversion. Implies any previous conversion
    /// 0 whilst conversion in progress.
    READY OFFSET(8) NUMBITS(1) [],
    /// Continuously perform conversions whilst this bit is 1. A new conversion will sta
    START_MANY OFFSET(3) NUMBITS(1) [],
    /// Start a single conversion. Self-clearing. Ignored if start_many is asserted.
    START_ONCE OFFSET(2) NUMBITS(1) [],
    /// Power on temperature sensor. 1 - enabled. 0 - disabled.
    TS_EN OFFSET(1) NUMBITS(1) [],
    /// Power on ADC and enable its clock.
    /// 1 - enabled. 0 - disabled.
    EN OFFSET(0) NUMBITS(1) []
],
RESULT [

    RESULT OFFSET(0) NUMBITS(12) []
],
FCS [
    /// DREQ/IRQ asserted when level >= threshold
    THRESH OFFSET(24) NUMBITS(4) [],
    /// The number of conversion results currently waiting in the FIFO
    LEVEL OFFSET(16) NUMBITS(4) [],
    /// 1 if the FIFO has been overflowed. Write 1 to clear.
    OVER OFFSET(11) NUMBITS(1) [],
    /// 1 if the FIFO has been underflowed. Write 1 to clear.
    UNDER OFFSET(10) NUMBITS(1) [],

    FULL OFFSET(9) NUMBITS(1) [],

    EMPTY OFFSET(8) NUMBITS(1) [],
    /// If 1: assert DMA requests when FIFO contains data
    DREQ_EN OFFSET(3) NUMBITS(1) [],
    /// If 1: conversion error bit appears in the FIFO alongside the result
    ERR OFFSET(2) NUMBITS(1) [],
    /// If 1: FIFO results are right-shifted to be one byte in size. Enables DMA to byte
    SHIFT OFFSET(1) NUMBITS(1) [],
    /// If 1: write result to the FIFO after each conversion.
    EN OFFSET(0) NUMBITS(1) []
],
FIFO [
    /// 1 if this particular sample experienced a conversion error. Remains in the same
    ERR OFFSET(15) NUMBITS(1) [],

    VAL OFFSET(0) NUMBITS(12) []
],
DIV [
    /// Integer part of clock divisor.
    INT OFFSET(8) NUMBITS(16) [],
    /// Fractional part of clock divisor. First-order delta-sigma.
    FRAC OFFSET(0) NUMBITS(8) []
],
INTR [
    /// Triggered when the sample FIFO reaches a certain level.
    /// This level can be programmed via the FCS_THRESH field.
    FIFO OFFSET(0) NUMBITS(1) []
],
INTE [
    /// Triggered when the sample FIFO reaches a certain level.
    /// This level can be programmed via the FCS_THRESH field.
    FIFO OFFSET(0) NUMBITS(1) []
],
INTF [
    /// Triggered when the sample FIFO reaches a certain level.
    /// This level can be programmed via the FCS_THRESH field.
    FIFO OFFSET(0) NUMBITS(1) []
],
INTS [
    /// Triggered when the sample FIFO reaches a certain level.
    /// This level can be programmed via the FCS_THRESH field.
    FIFO OFFSET(0) NUMBITS(1) []
]
];
const ADC_BASE: StaticRef<AdcRegisters> =
    unsafe { StaticRef::new(0x4004C000 as *const AdcRegisters) };

#[allow(dead_code)]
#[repr(u32)]
#[derive(Copy, Clone, PartialEq)]
pub enum Channel {
    Channel0 = 0b00000,
    Channel1 = 0b00001,
    Channel2 = 0b00010,
    Channel3 = 0b00011,
    Channel4 = 0b00100,
}

#[derive(Copy, Clone, PartialEq)]
enum ADCStatus {
    Idle,
    OneSample,
}

pub struct Adc {
    registers: StaticRef<AdcRegisters>,
    status: Cell<ADCStatus>,
    channel: Cell<Channel>,
    client: OptionalCell<&'static dyn hil::adc::Client>,
}

impl Adc {
    pub const fn new() -> Self {
        Self {
            registers: ADC_BASE,
            status: Cell::new(ADCStatus::Idle),
            channel: Cell::new(Channel::Channel0),
            client: OptionalCell::empty(),
        }
    }

    pub fn init(&self) {
        self.registers.cs.modify(CS::EN::SET);
        while !self.registers.cs.is_set(CS::READY) {}
    }

    pub fn disable(&self) {
        self.registers.cs.modify(CS::EN::CLEAR);
    }

    fn enable_interrupt(&self) {
        self.registers.inte.modify(INTE::FIFO::SET);
    }

    fn disable_interrupt(&self) {
        self.registers.inte.modify(INTE::FIFO::CLEAR);
    }

    fn enable_temperature(&self) {
        self.registers.cs.modify(CS::TS_EN::SET);
    }

    pub fn handle_interrupt(&self) {
        if self.registers.cs.is_set(CS::READY) {
            if self.status.get() == ADCStatus::OneSample {
                self.status.set(ADCStatus::Idle);
            }
            self.client.map(|client| {
                self.disable_interrupt();
                client.sample_ready(self.registers.fifo.read(FIFO::VAL) as u16)
            });
        }
    }
}

impl hil::adc::Adc for Adc {
    type Channel = Channel;

    fn sample(&self, channel: &Self::Channel) -> Result<(), ErrorCode> {
        if self.status.get() == ADCStatus::Idle {
            if *channel as u32 == 4 {
                self.enable_temperature();
            }
            self.status.set(ADCStatus::OneSample);
            self.channel.set(*channel);
            self.registers.cs.modify(CS::AINSEL.val(*channel as u32));
            self.registers
                .fcs
                .modify(FCS::THRESH.val(1 as u32) + FCS::EN::SET);
            self.enable_interrupt();
            self.registers.cs.modify(CS::START_ONCE::SET);
            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn sample_continuous(
        &self,
        _channel: &Self::Channel,
        _frequency: u32,
    ) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn stop_sampling(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn get_resolution_bits(&self) -> usize {
        12
    }

    fn get_voltage_reference_mv(&self) -> Option<usize> {
        Some(3300)
    }

    fn set_client(&self, client: &'static dyn hil::adc::Client) {
        self.client.set(client);
    }
}
