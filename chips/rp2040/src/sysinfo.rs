// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::utilities::registers::interfaces::Readable;
use kernel::utilities::StaticRef;

use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};

register_structs! {

    SysInfoRegisters {

        (0x000 => chip_id: ReadWrite<u32, CHIP_ID::Register>),

        (0x004 => platform: ReadWrite<u32, PLATFORM::Register>),

        (0x008 => _reserved1),

        (0x040 => gitref_rp2040: ReadWrite<u32, GITREF_RP2040::Register>),

        (0x044 => @END),
    }
}
register_bitfields![u32,
    CHIP_ID [

        REVISION OFFSET(28) NUMBITS(4) [],

        PART OFFSET(12) NUMBITS(16) [],

        MANUFACTURER OFFSET(0) NUMBITS(12) []

    ],
    PLATFORM [
        ASIC OFFSET(1) NUMBITS(1) [],

        FPGA OFFSET(0) NUMBITS(1) []

    ],
    GITREF_RP2040 [
        SOURCE_GIT_HASH OFFSET(0) NUMBITS(32) []
    ]
];

const SYSINFO_BASE: StaticRef<SysInfoRegisters> =
    unsafe { StaticRef::new(0x40000000 as *const SysInfoRegisters) };

pub enum Platform {
    Asic,
    Fpga,
}

pub struct SysInfo {
    registers: StaticRef<SysInfoRegisters>,
}

impl SysInfo {
    pub const fn new() -> SysInfo {
        SysInfo {
            registers: SYSINFO_BASE,
        }
    }

    pub fn get_revision(&self) -> u8 {
        self.registers.chip_id.read(CHIP_ID::REVISION) as u8
    }

    pub fn get_part(&self) -> u16 {
        self.registers.chip_id.read(CHIP_ID::PART) as u16
    }

    pub fn get_manufacturer_rp2040(&self) -> u16 {
        self.registers.chip_id.read(CHIP_ID::MANUFACTURER) as u16
    }

    pub fn get_asic(&self) -> u32 {
        self.registers.platform.read(PLATFORM::ASIC)
    }

    pub fn get_fpga(&self) -> u32 {
        self.registers.platform.read(PLATFORM::FPGA)
    }

    pub fn get_platform(&self) -> Platform {
        if self.registers.platform.is_set(PLATFORM::ASIC) {
            Platform::Asic
        } else {
            Platform::Fpga
        }
    }

    pub fn get_git_ref(&self) -> u32 {
        self.registers
            .gitref_rp2040
            .read(GITREF_RP2040::SOURCE_GIT_HASH)
    }
}
