// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! SysReg driver.

use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

pub const SYS_REG_BASE: StaticRef<SysRegRegisters> =
    unsafe { StaticRef::new(0x600C_0000 as *const SysRegRegisters) };

register_structs! {
    pub SysRegRegisters {
        (0x000 => _reserved0),
        (0x008 => cpu_per_conf: ReadWrite<u32, CPU_PER_CONF::Register>),
        (0x00c => _reserved1),
        (0x010 => perip_clk_en0: ReadWrite<u32, PERIP_CLK_EN0::Register>),
        (0x014 => _reserved3),
        (0x058 => sysclk_config: ReadWrite<u32, SYSCLK_CONFIG::Register>),
        (0x05C => _reserved_unimplemented_yet),
        (0x1000 => @END),
    }
}

register_bitfields![u32,
    PERIP_CLK_EN0 [
        TIMERGROUP0 OFFSET(13) NUMBITS(1) []
    ],
    CPU_PER_CONF [
        CPUPERIOD_SEL OFFSET(0) NUMBITS(2) [
            MHz80 = 0,
            MHz160 = 1,
        ],
        PLL_FREQ_SEL OFFSET(2) NUMBITS(1) [
            MHz320 = 0,
            MHz480 = 1
        ],
        CPU_WAIT_MODE_FORCE_ON OFFSET(3) NUMBITS(1) [],
        CPU_WAIT_DELAY_NUM OFFSET(4) NUMBITS(4) [],
    ],
    SYSCLK_CONFIG [
        PRE_DIV_CNT OFFSET(0) NUMBITS(10) [],
        SOC_CLK_SEL OFFSET(10) NUMBITS(2) [
            Xtal = 0,
            Pll = 1,
            Fosc = 2
        ],
        CLK_XTAL_FREQ OFFSET(12) NUMBITS(6) [],
    ]
];

#[repr(u32)]
pub enum PllFrequency {
    MHz320 = 0,
    MHz480 = 1,
}

#[repr(u32)]
pub enum CpuFrequency {
    MHz80 = 0,
    MHz160 = 1,
}

pub struct SysReg {
    registers: StaticRef<SysRegRegisters>,
}

impl SysReg {
    pub const fn new() -> Self {
        SysReg {
            registers: SYS_REG_BASE,
        }
    }

    pub fn use_xtal_clock_source(&self) {
        self.registers
            .sysclk_config
            .modify(SYSCLK_CONFIG::SOC_CLK_SEL::Xtal);
    }

    pub fn use_pll_clock_source(&self, pll_frequency: PllFrequency, cpu_frequency: CpuFrequency) {
        self.registers
            .sysclk_config
            .modify(SYSCLK_CONFIG::SOC_CLK_SEL::Pll);
        self.registers.cpu_per_conf.modify(
            CPU_PER_CONF::PLL_FREQ_SEL.val(pll_frequency as u32)
                + CPU_PER_CONF::CPUPERIOD_SEL.val(cpu_frequency as u32),
        );
    }

    pub fn enable_timg0(&self) {
        self.registers
            .perip_clk_en0
            .modify(PERIP_CLK_EN0::TIMERGROUP0::SET);
    }

    pub fn disable_timg0(&self) {
        self.registers
            .perip_clk_en0
            .modify(PERIP_CLK_EN0::TIMERGROUP0::CLEAR);
    }

    pub fn is_enabled_timg0(&self) -> bool {
        self.registers
            .perip_clk_en0
            .is_set(PERIP_CLK_EN0::TIMERGROUP0)
    }
}
