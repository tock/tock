// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025 SRL.

use kernel::utilities::registers::{
    interfaces::ReadWriteable, register_bitfields, register_structs, ReadWrite,
};
use kernel::utilities::StaticRef;

#[repr(C)]
struct Port {
    port_sel0: ReadWrite<u32, PRT_PORT_SEL0::Register>,
    port_sel1: ReadWrite<u32, PRT_PORT_SEL1::Register>,
    _reserved: [u8; 8],
}

register_structs! {
    HsiomRegisters {
        (0x000 => ports: [Port; 15]),
        (0x0f0 => @END),
    }
}
register_bitfields![u32,
PRT_PORT_SEL0 [
    IO0_SEL OFFSET(0) NUMBITS(5) [],
    IO1_SEL OFFSET(8) NUMBITS(5) [],
    IO2_SEL OFFSET(16) NUMBITS(5) [],
    IO3_SEL OFFSET(16) NUMBITS(5) [],
],
PRT_PORT_SEL1 [
    IO4_SEL OFFSET(0) NUMBITS(5) [],
    IO5_SEL OFFSET(8) NUMBITS(5) [],
    IO6_SEL OFFSET(16) NUMBITS(5) [],
    IO7_SEL OFFSET(16) NUMBITS(5) [],
],
];
const HSIOM_BASE: StaticRef<HsiomRegisters> =
    unsafe { StaticRef::new(0x40300000 as *const HsiomRegisters) };

pub struct Hsiom {
    registers: StaticRef<HsiomRegisters>,
}

impl Hsiom {
    pub const fn new() -> Hsiom {
        Hsiom {
            registers: HSIOM_BASE,
        }
    }

    pub fn enable_uart(&self) {
        self.registers.ports[5]
            .port_sel0
            .modify(PRT_PORT_SEL0::IO1_SEL.val(0x12));
        self.registers.ports[5]
            .port_sel0
            .modify(PRT_PORT_SEL0::IO0_SEL.val(0x12));
    }
}
