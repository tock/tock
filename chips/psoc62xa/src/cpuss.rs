// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025 SRL.

use kernel::utilities::registers::{
    interfaces::ReadWriteable, register_bitfields, register_structs, ReadWrite,
};
use kernel::utilities::StaticRef;

register_structs! {
    CpussRegisters {
        (0x0000 => _reserved0),
        (0x1008 => cm0_clock_ctl: ReadWrite<u32, CM0_CLOCK_CTL::Register>),
        (0x100c => _reserved1),
        (0x8000 => cm0_system_int_ctl: [ReadWrite<u32, CM0_SYSTEM_INT_CTL::Register>; 168]),
        (0x82A0 => @END),
    }
}
register_bitfields![u32,
CM0_CLOCK_CTL [
    SLOW_INT_DIV OFFSET(8) NUMBITS(8) [],
    PERI_INT_DIV OFFSET(24) NUMBITS(8) []
],
CM0_SYSTEM_INT_CTL [
    CPU_INT_IDX OFFSET(0) NUMBITS(3) [],
    CPU_INT_VALID OFFSET(31) NUMBITS(1) []
],
];
const CPUSS_BASE: StaticRef<CpussRegisters> =
    unsafe { StaticRef::new(0x40200000 as *const CpussRegisters) };

const SCB5_ID: usize = 44;
const TCPWM0_ID: usize = 123;

pub struct Cpuss {
    registers: StaticRef<CpussRegisters>,
}

impl Cpuss {
    pub const fn new() -> Cpuss {
        Cpuss {
            registers: CPUSS_BASE,
        }
    }

    pub fn init_clock(&self) {
        self.registers
            .cm0_clock_ctl
            .modify(CM0_CLOCK_CTL::PERI_INT_DIV.val(0));
    }

    pub fn enable_int_for_scb5(&self) {
        self.registers.cm0_system_int_ctl[SCB5_ID].modify(
            CM0_SYSTEM_INT_CTL::CPU_INT_IDX.val(0) + CM0_SYSTEM_INT_CTL::CPU_INT_VALID::SET,
        );
    }

    pub fn enable_int_for_tcpwm00(&self) {
        self.registers.cm0_system_int_ctl[TCPWM0_ID].modify(
            CM0_SYSTEM_INT_CTL::CPU_INT_IDX.val(0) + CM0_SYSTEM_INT_CTL::CPU_INT_VALID::SET,
        );
    }

    pub fn enable_int_for_gpio0(&self) {
        self.registers.cm0_system_int_ctl[15].modify(
            CM0_SYSTEM_INT_CTL::CPU_INT_IDX.val(1) + CM0_SYSTEM_INT_CTL::CPU_INT_VALID::SET,
        );
    }
}
