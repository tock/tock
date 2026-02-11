// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {

    TicksRegisters {
        /// Controls the tick generator
        (0x000 => proc0_ctrl: ReadWrite<u32, PROC0_CTRL::Register>),

        (0x004 => proc0_cycles: ReadWrite<u32>),

        (0x008 => proc0_count: ReadWrite<u32>),
        /// Controls the tick generator
        (0x00C => proc1_ctrl: ReadWrite<u32, PROC1_CTRL::Register>),

        (0x010 => proc1_cycles: ReadWrite<u32>),

        (0x014 => proc1_count: ReadWrite<u32>),
        /// Controls the tick generator
        (0x018 => timer0_ctrl: ReadWrite<u32, TIMER0_CTRL::Register>),

        (0x01C => timer0_cycles: ReadWrite<u32, TIMER0_CYCLES::Register>),

        (0x020 => timer0_count: ReadOnly<u32, TIMER0_COUNT::Register>),
        /// Controls the tick generator
        (0x024 => timer1_ctrl: ReadWrite<u32, TIMER1_CTRL::Register>),

        (0x028 => timer1_cycles: ReadWrite<u32, TIMER1_CYCLES::Register>),

        (0x02C => timer1_count: ReadOnly<u32, TIMER1_COUNT::Register>),
        /// Controls the tick generator
        (0x030 => watchdog_ctrl: ReadWrite<u32, WATCHDOG_CTRL::Register>),

        (0x034 => watchdog_cycles: ReadWrite<u32>),

        (0x038 => watchdog_count: ReadWrite<u32>),
        /// Controls the tick generator
        (0x03C => riscv_ctrl: ReadWrite<u32, RISCV_CTRL::Register>),

        (0x040 => riscv_cycles: ReadWrite<u32>),

        (0x044 => riscv_count: ReadWrite<u32>),
        (0x048 => @END),
    }
}
register_bitfields![u32,
PROC0_CTRL [
    /// Is the tick generator running?
    RUNNING OFFSET(1) NUMBITS(1) [],
    /// start / stop tick generation
    ENABLE OFFSET(0) NUMBITS(1) []
],
PROC0_CYCLES [
    /// Total number of clk_tick cycles before the next tick.
    PROC0_CYCLES OFFSET(0) NUMBITS(9) []
],
PROC0_COUNT [
    /// Count down timer: the remaining number clk_tick cycles before the next tick is g
    PROC0_COUNT OFFSET(0) NUMBITS(9) []
],
PROC1_CTRL [
    /// Is the tick generator running?
    RUNNING OFFSET(1) NUMBITS(1) [],
    /// start / stop tick generation
    ENABLE OFFSET(0) NUMBITS(1) []
],
PROC1_CYCLES [
    /// Total number of clk_tick cycles before the next tick.
    PROC1_CYCLES OFFSET(0) NUMBITS(9) []
],
PROC1_COUNT [
    /// Count down timer: the remaining number clk_tick cycles before the next tick is g
    PROC1_COUNT OFFSET(0) NUMBITS(9) []
],
TIMER0_CTRL [
    /// Is the tick generator running?
    RUNNING OFFSET(1) NUMBITS(1) [],
    /// start / stop tick generation
    ENABLE OFFSET(0) NUMBITS(1) []
],
TIMER0_CYCLES [
    /// Total number of clk_tick cycles before the next tick.
    TIMER0_CYCLES OFFSET(0) NUMBITS(9) []
],
TIMER0_COUNT [
    /// Count down timer: the remaining number clk_tick cycles before the next tick is g
    TIMER0_COUNT OFFSET(0) NUMBITS(9) []
],
TIMER1_CTRL [
    /// Is the tick generator running?
    RUNNING OFFSET(1) NUMBITS(1) [],
    /// start / stop tick generation
    ENABLE OFFSET(0) NUMBITS(1) []
],
TIMER1_CYCLES [
    /// Total number of clk_tick cycles before the next tick.
    TIMER1_CYCLES OFFSET(0) NUMBITS(9) []
],
TIMER1_COUNT [
    /// Count down timer: the remaining number clk_tick cycles before the next tick is g
    TIMER1_COUNT OFFSET(0) NUMBITS(9) []
],
WATCHDOG_CTRL [
    /// Is the tick generator running?
    RUNNING OFFSET(1) NUMBITS(1) [],
    /// start / stop tick generation
    ENABLE OFFSET(0) NUMBITS(1) []
],
WATCHDOG_CYCLES [
    /// Total number of clk_tick cycles before the next tick.
    WATCHDOG_CYCLES OFFSET(0) NUMBITS(9) []
],
WATCHDOG_COUNT [
    /// Count down timer: the remaining number clk_tick cycles before the next tick is g
    WATCHDOG_COUNT OFFSET(0) NUMBITS(9) []
],
RISCV_CTRL [
    /// Is the tick generator running?
    RUNNING OFFSET(1) NUMBITS(1) [],
    /// start / stop tick generation
    ENABLE OFFSET(0) NUMBITS(1) []
],
RISCV_CYCLES [
    /// Total number of clk_tick cycles before the next tick.
    RISCV_CYCLES OFFSET(0) NUMBITS(9) []
],
RISCV_COUNT [
    /// Count down timer: the remaining number clk_tick cycles before the next tick is g
    RISCV_COUNT OFFSET(0) NUMBITS(9) []
]
];
const TICKS_BASE: StaticRef<TicksRegisters> =
    unsafe { StaticRef::new(0x40108000 as *const TicksRegisters) };

pub struct Ticks {
    registers: StaticRef<TicksRegisters>,
}

impl Ticks {
    pub fn new() -> Self {
        Self {
            registers: TICKS_BASE,
        }
    }

    pub fn set_timer0_generator(&self) {
        self.registers
            .timer0_ctrl
            .modify(TIMER0_CTRL::ENABLE::CLEAR);
        self.registers
            .timer0_cycles
            .modify(TIMER0_CYCLES::TIMER0_CYCLES.val(12));
        self.registers.timer0_ctrl.modify(TIMER0_CTRL::ENABLE::SET);
    }

    pub fn set_timer1_generator(&self) {
        self.registers
            .timer1_ctrl
            .modify(TIMER1_CTRL::ENABLE::CLEAR);
        self.registers
            .timer1_cycles
            .modify(TIMER1_CYCLES::TIMER1_CYCLES.val(12));
        self.registers.timer1_ctrl.modify(TIMER1_CTRL::ENABLE::SET);
    }

    pub fn is_timer0_on(&self) -> bool {
        self.registers.timer0_ctrl.is_set(TIMER0_CTRL::RUNNING)
    }
}
