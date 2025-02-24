// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025 SRL.

use kernel::utilities::registers::{
    interfaces::ReadWriteable, register_bitfields, register_structs, ReadWrite,
};
use kernel::utilities::StaticRef;

register_structs! {
    SrssRegisters {
        (0x000 => _reserved0),
        (0x340 => clk_path_select: [ReadWrite<u32, CLK_PATH_SELECT::Register>; 6]),
        (0x358 => _reserved1),
        (0x380 => clk_root_select: [ReadWrite<u32, CLK_ROOT_SELECT::Register>; 6]),
        (0x398 => @END),
    }
}
register_bitfields![u32,
CLK_PATH_SELECT [
    PATH_MUX OFFSET(0) NUMBITS(3) [
        IMOInternalRCOscillator = 0,
        EXTCLKExternalClockPin = 1,
        ECOExternalCrystalOscillator = 2,
        ALTHFAlternateHighFrequencyClockInputProductSpecificClock = 3,
        DSI_MUX = 4
    ]
],
CLK_ROOT_SELECT [
    ROOT_MUX OFFSET(0) NUMBITS(4) [
        SelectPATH0CanBeConfiguredForFLL = 0,
        SelectPATH1CanBeConfiguredForPLL0IfAvailableInTheProduct = 1,
        SelectPATH2CanBeConfiguredForPLL1IfAvailableInTheProduct = 2,
        SelectPATH3CanBeConfiguredForPLL2IfAvailableInTheProduct = 3,
        SelectPATH4CanBeConfiguredForPLL3IfAvailableInTheProduct = 4,
        SelectPATH5CanBeConfiguredForPLL4IfAvailableInTheProduct = 5,
        SelectPATH6CanBeConfiguredForPLL5IfAvailableInTheProduct = 6,
        SelectPATH7CanBeConfiguredForPLL6IfAvailableInTheProduct = 7,
        SelectPATH8CanBeConfiguredForPLL7IfAvailableInTheProduct = 8,
        SelectPATH9CanBeConfiguredForPLL8IfAvailableInTheProduct = 9,
        SelectPATH10CanBeConfiguredForPLL9IfAvailableInTheProduct = 10,
        SelectPATH11CanBeConfiguredForPLL10IfAvailableInTheProduct = 11,
        SelectPATH12CanBeConfiguredForPLL11IfAvailableInTheProduct = 12,
        SelectPATH13CanBeConfiguredForPLL12IfAvailableInTheProduct = 13,
        SelectPATH14CanBeConfiguredForPLL13IfAvailableInTheProduct = 14,
        SelectPATH15CanBeConfiguredForPLL14IfAvailableInTheProduct = 15
    ],
    ROOT_DIV OFFSET(4) NUMBITS(2) [
        TransparentModeFeedThroughSelectedClockSourceWODividing = 0,
        DivideSelectedClockSourceBy2 = 1,
        DivideSelectedClockSourceBy4 = 2,
        DivideSelectedClockSourceBy8 = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
];
const SRSS_BASE: StaticRef<SrssRegisters> =
    unsafe { StaticRef::new(0x40260000 as *const SrssRegisters) };

pub struct Srss {
    registers: StaticRef<SrssRegisters>,
}

impl Srss {
    pub const fn new() -> Srss {
        Srss {
            registers: SRSS_BASE,
        }
    }

    pub fn init_clock(&self) {
        self.registers.clk_path_select[3]
            .modify(CLK_PATH_SELECT::PATH_MUX::IMOInternalRCOscillator);

        self.registers.clk_root_select[0].modify(CLK_ROOT_SELECT::ENABLE::SET + CLK_ROOT_SELECT::ROOT_MUX::SelectPATH3CanBeConfiguredForPLL2IfAvailableInTheProduct + CLK_ROOT_SELECT::ROOT_DIV::TransparentModeFeedThroughSelectedClockSourceWODividing);
    }
}
