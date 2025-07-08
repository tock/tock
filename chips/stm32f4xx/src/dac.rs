// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use crate::clocks::{phclk, Stm32f4Clocks};
use core::cell::Cell;
use kernel::hil;
use kernel::platform::chip::ClockInterface;
use kernel::utilities::registers::interfaces::{ReadWriteable, Writeable};
use kernel::utilities::registers::{register_bitfields, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

/// DAC
#[repr(C)]
pub struct DacRegisters {
    cr: ReadWrite<u32, CR::Register>,
    swtrigr: WriteOnly<u32, SWTRIGR::Register>,
    dhr12r1: ReadWrite<u32, DHR12R1::Register>,
    dhr8r1: ReadWrite<u32, DHR8R1::Register>,
    dhr12r2: ReadWrite<u32, DHR12R2::Register>,
    dhr12l2: ReadWrite<u32, DHR12L2::Register>,
    dhr8r2: ReadWrite<u32, DHR8R2::Register>,
    dhr12rd: ReadWrite<u32, DHR12RD::Register>,
    dhr12ld: ReadWrite<u32, DHR12LD::Register>,
    dhr8rd: ReadWrite<u32, DHR8RD::Register>,
    dor1: ReadWrite<u32, DOR1::Register>,
    dor2: ReadWrite<u32, DOR2::Register>,
}

register_bitfields![u32,
        /// Control register
        CR [
            /// DAC channel 2 DMA underrun interrupt enable
            DMAUDRIE2 OFFSET(29) NUMBITS(1) [],
            /// DAC channel 2 DMA enable
            DMAEN2 OFFSET(28) NUMBITS(1) [],
            /// DAC channel2 mask/amplitude selector
            MAMP2 OFFSET(24) NUMBITS(4) [],
            /// DAC channel2 noise/triangle wave generation enable
            WAVE2 OFFSET(22) NUMBITS(2) [],
            /// DAC channel2 trigger selection
            TSEL2 OFFSET(19) NUMBITS(3) [],
            /// DAC channel2 trigger enable
            TEN2 OFFSET(18) NUMBITS(1) [],
            /// DAC channel2 output buffer disable
            BOFF2 OFFSET(17) NUMBITS(1) [],
            /// DAC channel2 enable
            EN2 OFFSET(16) NUMBITS(1) [],
            /// DAC channel 1 DMA underrun interrupt enable
            DMAUDRIE1 OFFSET(13) NUMBITS(1) [],
            /// DAC channel 1 DMA enable
            DMAEN1 OFFSET(12) NUMBITS(1) [],
            /// DAC channel1 mask/amplitude selector
            MAMP1 OFFSET(8) NUMBITS(4) [],
            /// DAC channel1 noise/triangle wave generation enable
            WAVE1 OFFSET(6) NUMBITS(2) [],
            /// DAC channel2 trigger selection
            TSEL1 OFFSET(3) NUMBITS(3) [],
            /// DAC channel2 trigger enable
            TEN1 OFFSET(2) NUMBITS(1) [],
            /// DAC channel2 output buffer disable
            BOFF1 OFFSET(1) NUMBITS(1) [],
            /// DAC channel1 enable
            EN1 OFFSET(0) NUMBITS(1) [],
        ],
        /// Software trigger register
        SWTRIGR [
            /// DAC channel2 software trigger
            SWTRIG2 OFFSET(1) NUMBITS(1) [],
            /// DAC channel1 software trigger
            SWTRIG1 OFFSET(0) NUMBITS(1) []
        ],
        /// Channel1 12-bit right-aligned data holding register
        DHR12R1 [
            /// DAC channel1 12-bit right-aligned data
            DACC1DHR OFFSET(0) NUMBITS(12) []
        ],
        /// Channel1 8-bit right aligned data holding register
        DHR8R1 [
            /// DAC Channel1 8-bit right-aligned data
            DACC1DHR OFFSET(0) NUMBITS(8) []
        ],
        /// Channel2 12-bit right aligned data holding register
        DHR12R2 [
            /// DAC channel2 12-bit right aligned data
            DACC2DHR OFFSET(0) NUMBITS(12) []
        ],
        /// Channel2 12-bit left aligned data holding register
        DHR12L2 [
            /// DAC channel2 12-bit left-aligned data
            DACC2DHR OFFSET(0) NUMBITS(12) []
        ],
        /// Channel2 8-bit right-aligned data holding register
        DHR8R2 [
            /// DAC channel2 8-bit right-aligned data
            DACC2DHR OFFSET(0) NUMBITS(8) []
        ],
        /// Dual DAC 12-bit right-aligned data holding register
        DHR12RD [
            /// DAC channel2 12-bit right-aligned data
            DACC2DHR OFFSET(16) NUMBITS(12) [],
            /// DAC channel1 12-bit right-aligned data
            DACC1DHR OFFSET(0) NUMBITS(12) []
        ],
        /// Dual DAC 12-bit left aligned data holding register
        DHR12LD [
            /// DAC channel2 12-bit left-aligned data
            DACC2DHR OFFSET(16) NUMBITS(12) [],
            /// DAC channel1 12-bit left-aligned data
            DACC1DHR OFFSET(0) NUMBITS(12) []
        ],
        /// Dual DAC 8-bit right aligned data holding register
        DHR8RD [
            /// DAC channel2 8-bit right-aligned data
            DACC2DHR OFFSET(8) NUMBITS(8) [],
            /// DAC channel1 8-bit right-aligned data
            DACC1DHR OFFSET(0) NUMBITS(8) []
        ],
        /// DAC Channel 1 data output register
        DOR1 [
            /// DAC channel1 data output
            DACC1DOR OFFSET(0) NUMBITS(12) []
        ],
        /// DAC Channel 2 data output register
        DOR2 [
            /// DAC channel2 data output
            DACC2DOR OFFSET(0) NUMBITS(12) []
        ],
        /// DAC status register
        SR [
            /// DAC channel2 DMA underrun flag
            DMAUDR2 OFFSET(29) NUMBITS(1) [],
            /// DAC channel1 DMA underrun flag
            DMAUDR1 OFFSET(13) NUMBITS(1) []
        ]
];

const DAC_BASE: StaticRef<DacRegisters> =
    unsafe { StaticRef::new(0x40007400 as *const DacRegisters) };

pub struct Dac<'a> {
    registers: StaticRef<DacRegisters>,
    clock: DacClock<'a>,
    initialized: Cell<bool>,
    enabled: Cell<bool>,
}

impl<'a> Dac<'a> {
    pub const fn new(clocks: &'a dyn Stm32f4Clocks) -> Self {
        Self {
            registers: DAC_BASE,
            clock: DacClock(phclk::PeripheralClock::new(
                phclk::PeripheralClockType::APB1(phclk::PCLK1::DAC),
                clocks,
            )),
            initialized: Cell::new(false),
            enabled: Cell::new(false),
        }
    }

    fn initialize(&self) -> Result<(), ErrorCode> {
        if !self.is_enabled_clock() {
            self.enable_clock();
        }

        // Clear BOFF1, TEN1, TSEL1, WAVE1 and MAMP1 bits
        self.registers.cr.modify(CR::BOFF1::CLEAR);
        self.registers.cr.modify(CR::TEN1::CLEAR);
        self.registers.cr.modify(CR::TSEL1::CLEAR);
        self.registers.cr.modify(CR::WAVE1::CLEAR);
        self.registers.cr.modify(CR::MAMP1::CLEAR);

        self.enable();

        Ok(())
    }

    fn enable(&self) {
        self.registers.cr.modify(CR::EN1::SET);
    }

    // Not currently using interrupt.
    pub fn handle_interrupt(&self) {}

    fn is_enabled_clock(&self) -> bool {
        self.clock.is_enabled()
    }

    fn enable_clock(&self) {
        self.clock.enable();
    }
}

struct DacClock<'a>(phclk::PeripheralClock<'a>);

impl ClockInterface for DacClock<'_> {
    fn is_enabled(&self) -> bool {
        self.0.is_enabled()
    }

    fn enable(&self) {
        self.0.enable();
    }

    fn disable(&self) {
        self.0.disable();
    }
}

impl hil::dac::DacChannel for Dac<'_> {
    fn set_value(&self, value: usize) -> Result<(), ErrorCode> {
        if !self.initialized.get() {
            self.initialize()?;
        }

        if !self.enabled.get() {
            self.enable();
        }

        self.registers
            .dhr12r1
            .write(DHR12R1::DACC1DHR.val(value as u32));
        Ok(())
    }
}
