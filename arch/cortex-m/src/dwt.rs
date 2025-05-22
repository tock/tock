// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Cortex-M Data Watchpoint and Trace Unit (DWT)
//!
//! <https://developer.arm.com/documentation/100166/0001/Data-Watchpoint-and-Trace-Unit/DWT-Programmers--model?lang=en>

use super::dcb;
use kernel::hil;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    /// In an ARMv7-M processor, a System Control Block (SCB) in the SCS
    /// provides key status information and control features for the processor.
    DwtRegisters {
        // Control Register
        (0x00 => ctrl: ReadWrite<u32, Control::Register>),

        // Cycle Count Register
        (0x04 => cyccnt: ReadWrite<u32, CycleCount::Register>),

        // CPI Count Register
        (0x08 => cpicnt: ReadWrite<u32, CpiCount::Register>),

        // Exception Overhead Register
        (0x0C => exccnt: ReadWrite<u32, ExceptionOverheadCount::Register>),

        // Sleep Count Register
        (0x10 => sleepcnt: ReadWrite<u32, SleepCount::Register>),

        // LSU Count Register
        (0x14 => lsucnt: ReadWrite<u32, LsuCount::Register>),

        // Folder-Instruction Count Register
        (0x18 => foldcnt: ReadWrite<u32, FoldedInstructionCount::Register>),

        // Program Count Sample Register
        (0x1C => pcsr: ReadOnly<u32, ProgramCounterSample::Register>),

        // Comparator Register0
        (0x20 => comp0: ReadWrite<u32, Comparator0::Register>),

        // Mask Register0
        // The maximum mask size is 32KB
        (0x24 => mask0: ReadWrite<u32, Comparator0Mask::Register>),

        // Function Register0
        (0x28 => function0: ReadWrite<u32, Comparator0Function::Register>),

        (0x2c => _reserved0),

        // Comparator Register1
        (0x30 => comp1: ReadWrite<u32, Comparator1::Register>),

        // Mask Register1
        // The maximum mask size is 32KB
        (0x34 => mask1: ReadWrite<u32, Comparator1Mask::Register>),

        // Function Register1
        (0x38 => function1: ReadWrite<u32, Comparator1Function::Register>),

        (0x3c => _reserved1),

        // Comparator Register2
        (0x40 => comp2: ReadWrite<u32, Comparator2::Register>),

        // Mask Register2
        // The maximum mask size is 32KB
        (0x44 => mask2: ReadWrite<u32, Comparator2Mask::Register>),

        // Function Register2
        (0x48 => function2: ReadWrite<u32, Comparator2Function::Register>),

        (0x4c => _reserved2),

        // Comparator Register3
        (0x50 => comp3: ReadWrite<u32, Comparator3::Register>),

        // Mask Register3
        // The maximum mask size is 33KB
        (0x54 => mask3: ReadWrite<u32, Comparator3Mask::Register>),

        // Function Register3
        (0x58 => function3: ReadWrite<u32, Comparator3Function::Register>),

        (0x5c => @END),
    }
}

register_bitfields![u32,
    Control [
        /// Number of Camparators implemented.
        /// RO.
        NUMCOMP         OFFSET(28)  NUMBITS(4),

        /// Shows if trace sampling and exception tracing is implemented
        /// Is 0 if supported, is 1 if it is not.
        /// RO
        NOTRCPKT        OFFSET(27)  NUMBITS(1),

        /// Shows if external match signals ([`CMPMATCH`]) are implemented
        /// Is 0 if supported, is 1 if it is not.
        /// RO
        NOEXITTRIG      OFFSET(26)  NUMBITS(1),

        /// Shows if the cycle counter is implemented
        /// Is 0 if supported, is 1 if it is not.
        /// RO
        NOCYCCNT        OFFSET(25)  NUMBITS(1),

        /// Shows if profiling counters are supported.
        /// Is 0 if supported, is 1 if it is not.
        /// RO
        NOPERFCNT       OFFSET(24)  NUMBITS(1),

        /// Writing 1 enables event counter packets generation if
        /// [`PCSAMPLENA`] is set to 0. Defaults to 0b0 on reset.
        /// WARN: This bit is UNKNOWN if [`NOTPRCPKT`] or [`NOCYCCNT`] is read
        /// as one.
        /// RW
        CYCEVTENA       OFFSET(22)  NUMBITS(1),

        /// Writing 1 enables generation of folded instruction counter overflow
        /// event. Defaults to 0b0 on reset.
        /// WARN: This bit is UNKNOWN if [`NOPERFCNT`] reads as one.
        /// RW
        FOLDEVTENA      OFFSET(21)  NUMBITS(1),

        /// Writing 1 enables generation of LSU counter overflow event.
        /// Defaults to 0b0 on reset.
        /// WARN: This bit is UNKNOWN if [`NOPERFCNT`] reads as one.
        /// RW
        LSUEVTENA       OFFSET(20)  NUMBITS(1),

        /// Writing 1 enables generation of sleep counter overflow event.
        /// Defaults to 0b0 on reset.
        /// WARN: This bit is UNKNOWN if [`NOPERFCNT`] reads as one.
        /// RW
        SLEEPEVTENA     OFFSET(19)  NUMBITS(1),

        /// Writing 1 enables generation of exception overhead counter overflow event.
        /// Defaults to 0b0 on reset.
        /// WARN: This bit is UNKNOWN if [`NOPERFCNT`] reads as one.
        /// RW
        EXCEVTENA       OFFSET(18)  NUMBITS(1),

        /// Writing 1 enables generation of the CPI counter overlow event.
        /// Defaults to 0b0 on reset.
        /// WARN: This bit is UNKNOWN if [`NOPERFCNT`] reads as one.
        /// RW
        CPIEVTENA       OFFSET(17)  NUMBITS(1),

        /// Writing 1 enables generation of exception trace.
        /// Defaults to 0b0 on reset.
        /// WARN: This bit is UNKNOWN if [`NOTRCPKT`] reads as one.
        /// RW
        EXCTRCENA       OFFSET(16)  NUMBITS(1),

        /// Writing 1 enables use of [`POSTCNT`] counter as a timer for Periodic
        /// PC sample packet generation.
        /// Defaults to 0b0 on reset.
        /// WARN: This bit is UNKNOWN if [`NOTRCPKT`] or [`NOCYCCNT`] read as one.
        /// RW
        PCSAMPLENA      OFFSET(12)  NUMBITS(1),

        /// Determines the position of synchronisation packet counter tap on the
        /// `CYCCNT` counter and thus the synchronisation packet rate. Defaults
        /// to UNKNOWN on reset.
        /// WARN: This bit is UNKNOWN if [`NOCYCCNT`] reads as one.
        /// RW
        SYNCTAP         OFFSET(10)  NUMBITS(2),

        /// Determines the position of the [`POSTCNT`] tap on the [`CYCCNT`] counter.
        /// Defaults to UNKNOWN on reset.
        /// WARN: This bit is UNKNOWN if [`NOCYCCNT`] reads as one.
        /// RW
        CYCTAP          OFFSET(9)  NUMBITS(1),

        /// Initial value for the [`POSTCNT`] counter.
        /// Defaults to UNKNOWN on reset.
        /// WARN: This bit is UNKNOWN if [`NOCYCCNT`] reads as one.
        /// RW
        POSTINIT        OFFSET(8)  NUMBITS(4),

        /// Reload value for the [`POSTCNT`] counter.
        /// Defaults to UNKNOWN on reset.
        /// WARN: This bit is UNKNOWN if [`NOCYCCNT`] reads as one.
        /// RW
        POSTPRESET      OFFSET(1)  NUMBITS(4),

        /// Writing 1 enables [`CYCCNT`].
        /// Defaults to 0b0 on reset.
        /// WARN: This bit is UNKNOWN if [`NOCYCCNT`] reads as one.
        /// RW
        CYCNTENA       OFFSET(0)  NUMBITS(1),
    ],

    CycleCount[
        /// When enabled, increases on each processor clock cycle when
        /// [`Control::CYCNTENA`] and [`DEMCRL::TRCENA`] read as one. Wraps to
        /// zero on overflow.
        CYCCNT          OFFSET(0)   NUMBITS(32),
    ],

    CpiCount[
        /// Base instruction overhead counter.
        CPICNT          OFFSET(0)   NUMBITS(8),
    ],

    ExceptionOverheadCount[
        /// Counts cycles spent in exception processing.
        EXCCNT          OFFSET(0)   NUMBITS(8),
    ],

    SleepCount[
        /// Counts each cycle the processor is sleeping.
        SLEEPCNT        OFFSET(0)   NUMBITS(8),
    ],

    LsuCount[
        /// Counts additional cycles required to execute all load store
        /// instructions
        LSUCNT          OFFSET(0)   NUMBITS(8),
    ],

    FoldedInstructionCount[
        /// Increments by one for each instruction that takes 0 cycles to
        /// execute.
        FOLDCNT          OFFSET(0)   NUMBITS(8),
    ],

    ProgramCounterSample[
        /// Samples current value of the program counter
        /// RO.
        EIASAMPLE       OFFSET(0)   NUMBITS(32),
    ],

    Comparator0[
        /// Reference value for comparator 0.
        COMP       OFFSET(0)   NUMBITS(32),
    ],

    Comparator0Mask[
        /// Size of ignore mask applied to the access address for address range
        /// matching by comparator 0.
        ///
        /// WARN: Maximum Mask size is IMPLEMENTATION DEFINED.
        MASK       OFFSET(0)   NUMBITS(5),
    ],

    Comparator0Function[
        /// Is one if comparator matches. Reading the register clears it to 0.
        /// RO.
        MATCHED     OFFSET(24)   NUMBITS(1),

        /// Second comparator number for linked address comparison.
        /// Works, when `DATAVMATCH` and `LNK1ENA` read as one.
        /// RW.
        DATAVADDR1  OFFSET(16)   NUMBITS(4),

        /// Comparator number for linked address comparison.
        /// Works, when `DATAVMATCH` reads as one.
        /// RW.
        DATAVADDR0  OFFSET(12)   NUMBITS(4),

        /// Size of data comparison (Byte, Halfword, Word).
        /// RW.
        DATAVSIZE   OFFSET(10)   NUMBITS(2),

        /// Reads as one if a second linked comparator is supported.
        LNK1ENA     OFFSET(9)    NUMBITS(1),

        /// Enables data value comparison
        /// When 0: Perform address comparison, when 1: data value comparison.
        /// RW.
        DATAVMATCH  OFFSET(8)    NUMBITS(1),

        /// Enable cycle count comparison for comparator 0.
        /// WARN: Only supported by FUNCTION0
        /// RW.
        CYCMATCH    OFFSET(7)    NUMBITS(1),

        /// Write 1 to enable generation of data trace address packets.
        /// WARN: If [`Control::NOTRCPKT`] reads as zero, this bit is UNKNOWN.
        /// RW.
        EMITRANGE   OFFSET(5)    NUMBITS(1),

        /// Selects action taken on comparator match.
        /// Resets to 0b0000.
        /// RW.
        FUNCTION    OFFSET(0)    NUMBITS(4),
    ],

    Comparator1[
        /// Reference value for comparator 0.
        COMP       OFFSET(0)   NUMBITS(32),
    ],

    Comparator1Mask[
        /// Size of ignore mask applied to the access address for address range
        /// matching by comparator 0.
        ///
        /// WARN: Maximum Mask size is IMPLEMENTATION DEFINED.
        MASK       OFFSET(0)   NUMBITS(5),
    ],

    Comparator1Function[
        /// Is one if comparator matches. Reading the register clears it to 0.
        /// RO.
        MATCHED     OFFSET(24)   NUMBITS(1),

        /// Second comparator number for linked address comparison.
        /// Works, when `DATAVMATCH` and `LNK1ENA` read as one.
        /// RW.
        DATAVADDR1  OFFSET(16)   NUMBITS(4),

        /// Comparator number for linked address comparison.
        /// Works, when [`DATAVMATCH`] reads as one.
        /// RW.
        DATAVADDR0  OFFSET(12)   NUMBITS(4),

        /// Size of data comparison (Byte, Halfword, Word).
        /// RW.
        DATAVSIZE   OFFSET(10)   NUMBITS(2),

        /// Reads as one if a second linked comparator is supported.
        LNK1ENA     OFFSET(9)    NUMBITS(1),

        /// Enables data value comparison
        /// When 0: Perform address comparison, when 1: data value comparison.
        /// RW.
        DATAVMATCH  OFFSET(8)    NUMBITS(1),

        /// Write 1 to enable generation of data trace address packets.
        /// WARN: If `Control::NOTRCPKT` reads as zero, this bit is UNKNOWN.
        /// RW.
        EMITRANGE   OFFSET(5)    NUMBITS(1),

        /// Selects action taken on comparator match.
        /// Resets to 0b0000.
        /// RW.
        FUNCTION    OFFSET(0)    NUMBITS(4),
    ],

    Comparator2[
        /// Reference value for comparator 0.
        COMP       OFFSET(0)   NUMBITS(32),
    ],

    Comparator2Mask[
        /// Size of ignore mask applied to the access address for address range
        /// matching by comparator 0.
        ///
        /// WARN: Maximum Mask size is IMPLEMENTATION DEFINED.
        MASK       OFFSET(0)   NUMBITS(5),
    ],

    Comparator2Function[
        /// Is one if comparator matches. Reading the register clears it to 0.
        /// RO.
        MATCHED     OFFSET(24)   NUMBITS(1),

        /// Second comparator number for linked address comparison.
        /// Works, when [`DATAVMATCH`] and [`LNK1ENA`] read as one.
        /// RW.
        DATAVADDR1  OFFSET(16)   NUMBITS(4),

        /// Comparator number for linked address comparison.
        /// Works, when `DATAVMATCH` reads as one.
        /// RW.
        DATAVADDR0  OFFSET(12)   NUMBITS(4),

        /// Size of data comparison (Byte, Halfword, Word).
        /// RW.
        DATAVSIZE   OFFSET(10)   NUMBITS(2),

        /// Reads as one if a second linked comparator is supported.
        LNK1ENA     OFFSET(9)    NUMBITS(1),

        /// Enables data value comparison
        /// When 0: Perform address comparison, when 1: data value comparison.
        /// RW.
        DATAVMATCH  OFFSET(8)    NUMBITS(1),

        /// Write 1 to enable generation of data trace address packets.
        /// WARN: If `Control::NOTRCPKT` reads as zero, this bit is UNKNOWN.
        /// RW.
        EMITRANGE   OFFSET(5)    NUMBITS(1),

        /// Selects action taken on comparator match.
        /// Resets to 0b0000.
        /// RW.
        FUNCTION    OFFSET(0)    NUMBITS(4),
    ],

    Comparator3[
        /// Reference value for comparator 0.
        COMP       OFFSET(0)   NUMBITS(32),
    ],

    Comparator3Mask[
        /// Size of ignore mask applied to the access address for address range
        /// matching by comparator 0.
        ///
        /// WARN: Maximum Mask size is IMPLEMENTATION DEFINED.
        MASK       OFFSET(0)   NUMBITS(5),
    ],

    Comparator3Function[
        /// Is one if comparator matches. Reading the register clears it to 0.
        /// RO.
        MATCHED     OFFSET(24)   NUMBITS(1),

        /// Second comparator number for linked address comparison.
        /// Works, when `DATAVMATCH` and `LNK1ENA` read as one.
        /// RW.
        DATAVADDR1  OFFSET(16)   NUMBITS(4),

        /// Comparator number for linked address comparison.
        /// Works, when `DATAVMATCH` reads as one.
        /// RW.
        DATAVADDR0  OFFSET(12)   NUMBITS(4),

        /// Size of data comparison (Byte, Halfword, Word).
        /// RW.
        DATAVSIZE   OFFSET(10)   NUMBITS(2),

        /// Reads as one if a second linked comparator is supported.
        LNK1ENA     OFFSET(9)    NUMBITS(1),

        /// Enables data value comparison
        /// When 0: Perform address comparison, when 1: data value comparison.
        /// RW.
        DATAVMATCH  OFFSET(8)    NUMBITS(1),

        /// Write 1 to enable generation of data trace address packets.
        /// WARN: If `Control::NOTRCPKT` reads as zero, this bit is UNKNOWN.
        /// RW.
        EMITRANGE   OFFSET(5)    NUMBITS(1),

        /// Selects action taken on comparator match.
        /// Resets to 0b0000.
        /// RW.
        FUNCTION    OFFSET(0)    NUMBITS(4),
    ],

];

const DWT: StaticRef<DwtRegisters> = unsafe { StaticRef::new(0xE0001000 as *const DwtRegisters) };

pub struct Dwt {
    registers: StaticRef<DwtRegisters>,
}

impl Dwt {
    pub const fn new() -> Self {
        Self { registers: DWT }
    }

    /// Returns whether a cycle counter is present on the chip.
    pub fn is_cycle_counter_present(&self) -> bool {
        DWT.ctrl.read(Control::NOCYCCNT) == 0
    }
}

impl hil::hw_debug::CycleCounter for Dwt {
    fn start(&self) {
        if self.is_cycle_counter_present() {
            // The cycle counter has to be enabled in the DCB block
            dcb::enable_debug_and_trace();
            self.registers.ctrl.modify(Control::CYCNTENA::SET);
        }
    }

    fn stop(&self) {
        self.registers.ctrl.modify(Control::CYCNTENA::CLEAR);
    }

    fn count(&self) -> u64 {
        self.registers.cyccnt.read(CycleCount::CYCCNT) as u64
    }

    fn reset(&self) {
        // disable the counter
        self.registers.ctrl.modify(Control::CYCNTENA::CLEAR);
        // reset the counter
        self.registers.cyccnt.set(0);
    }
}
