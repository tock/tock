// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Power Reset Clock Interrupt controller driver.

use core::cell::Cell;
use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::registers::interfaces::Readable;
use kernel::utilities::registers::{register_bitfields, ReadWrite};
use kernel::utilities::StaticRef;
use rv32i::csr;

#[repr(C)]
pub struct PrciRegisters {
    /// Clock Configuration Register
    hfrosccfg: ReadWrite<u32, hfrosccfg::Register>,
    /// Clock Configuration Register
    hfxosccfg: ReadWrite<u32, hfxosccfg::Register>,
    /// PLL Configuration Register
    pllcfg: ReadWrite<u32, pllcfg::Register>,
    /// PLL Divider Register
    plloutdiv: ReadWrite<u32, plloutdiv::Register>,
    /// Clock Configuration Register
    coreclkcfg: ReadWrite<u32>,
}

register_bitfields![u32,
    hfrosccfg [
        ready OFFSET(31) NUMBITS(1) [],
        enable OFFSET(30) NUMBITS(1) [],
        trim OFFSET(16) NUMBITS(5) [],
        div OFFSET(0) NUMBITS(6) []
    ],
    hfxosccfg [
        ready OFFSET(31) NUMBITS(1) [],
        enable OFFSET(30) NUMBITS(1) []
    ],
    pllcfg [
        lock OFFSET(31) NUMBITS(1) [],
        bypass OFFSET(18) NUMBITS(1) [],
        refsel OFFSET(17) NUMBITS(1) [],
        sel OFFSET(16) NUMBITS(1) [],
        pllq OFFSET(10) NUMBITS(2) [],
        pllf OFFSET(4) NUMBITS(6) [],
        pllr OFFSET(0) NUMBITS(3) [
            R1 = 0
        ]
    ],
    plloutdiv [
        divby1 OFFSET(8) NUMBITS(1) [],
        div OFFSET(0) NUMBITS(6) []
    ]
];

pub enum ClockFrequency {
    Freq16Mhz,
    Freq344Mhz,
}

pub struct Prci {
    registers: StaticRef<PrciRegisters>,
    current_frequency: Cell<ClockFrequency>,
}

impl Prci {
    pub const fn new(base: StaticRef<PrciRegisters>) -> Prci {
        Prci {
            registers: base,
            current_frequency: Cell::new(ClockFrequency::Freq16Mhz),
        }
    }

    pub fn switch_to_internal_clock(&self) {
        let regs = self.registers;
        // Enable internal high-frequency clock if it's not enabled
        if regs.hfrosccfg.read(hfrosccfg::enable) == 0 {
            regs.hfrosccfg.modify(hfrosccfg::enable::SET);
        }
        // ... Wait until the clock is ready
        while regs.hfrosccfg.read(hfrosccfg::ready) == 0 {}
        // ... and now actually switch
        regs.pllcfg
            .modify(pllcfg::sel::CLEAR + pllcfg::bypass::CLEAR);
    }

    pub fn set_internal_clock_default(&self) {
        let regs = self.registers;
        // Set to defaults, which according to data sheet should set to 14.4MHz +- 50%,
        regs.hfrosccfg
            .modify(hfrosccfg::div.val(4) + hfrosccfg::trim.val(0x10));
    }

    pub fn enable_external_clock(&self) {
        let regs = self.registers;
        // Make sure external crystal oscillator is enabled
        if regs.hfxosccfg.read(hfxosccfg::enable) == 0 {
            regs.hfxosccfg.modify(hfxosccfg::enable::SET);
        }
        // ... Wait until the clock is ready
        while regs.hfxosccfg.read(hfxosccfg::ready) == 0 {}
    }

    pub fn set_clock_frequency(&self, frequency: ClockFrequency) {
        let regs = self.registers;
        // According to someone affiliated with SiFive in forum post:
        // https://forums.sifive.com/t/is-it-possible-to-brick-the-hifive-board/751/6,
        // it is safe to adjust the internal high frequency clock while it
        // is in use, but not the PLL output.
        //
        // So first switch to internal clock before doing any other clock manipulation

        // Reset internal clock to defaults so we can estimate how long we're spinning for the PLL
        // lock delay. At default, the frequency should be 14.4MHz +- 50%, so a maximum frequency
        // of just under 22MHz
        self.set_internal_clock_default();
        self.switch_to_internal_clock();

        // Make sure external clock subsystem is enabled before adjusting the PLL
        self.enable_external_clock();

        match frequency {
            ClockFrequency::Freq16Mhz => {
                // Bypass enabled, feeds clock directly from external clock. For HiFive1 revB, this
                // is a 16 MHz clock
                regs.pllcfg
                    .modify(pllcfg::bypass::SET + pllcfg::refsel::SET);
                // ... configure final PLL divider to divide by 1
                regs.plloutdiv
                    .modify(plloutdiv::divby1.val(1) + plloutdiv::div.val(0));
                // ... and finally enable the output
                regs.pllcfg.modify(pllcfg::sel::SET);
            }
            ClockFrequency::Freq344Mhz => {
                // Disable bypass, and set external clock as source for PLL
                // divide 16 MHz input by 2 (pllr(1)), times 86 (pllf(42)), divide by 2 (pllq(1))
                // for a frequency of 344MHz
                regs.pllcfg.modify(
                    pllcfg::bypass::CLEAR
                        + pllcfg::refsel::SET
                        + pllcfg::pllr.val(1)
                        + pllcfg::pllf.val(42)
                        + pllcfg::pllq.val(1),
                );
                // Divide PLL output by 1
                regs.plloutdiv
                    .modify(plloutdiv::divby1.val(1) + plloutdiv::div.val(0));

                // We need to wait for PLL to settle before checking if it's stable, which takes
                // about 100 microseconds. Assuming internal clock is worst case of 22MHz (14.7 MHz
                // +- 50%), that's about 2200 cycles.
                let start = csr::CSR.mcycle.get();
                while csr::CSR.mcycle.get() - start < 2200 {}
                // ... and now wait for the PLL lock
                while regs.pllcfg.read(pllcfg::lock) == 0 {}
                // ... and finally switch to the PLL output
                regs.pllcfg.modify(pllcfg::sel::SET);
            }
        };
        self.current_frequency.set(frequency);

        // Finally, disable internal clock as we've now switched to something else.
        regs.hfrosccfg.modify(hfrosccfg::enable::CLEAR);
    }
}
