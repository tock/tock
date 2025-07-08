// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025 SRL.

use kernel::utilities::registers::{
    interfaces::ReadWriteable, register_bitfields, register_structs, ReadOnly, ReadWrite,
};
use kernel::utilities::StaticRef;

register_structs! {
    CpussRegisters {
        (0x000 => identity: ReadOnly<u32, IDENTITY::Register>),
        (0x004 => cm4_status: ReadOnly<u32, CM4_STATUS::Register>),
        (0x008 => cm4_clock_ctl: ReadWrite<u32>),
        (0x00C => cm4_ctl: ReadWrite<u32, CM4_CTL::Register>),
        (0x010 => _reserved0),
        (0x200 => cm4_vector_table_base: ReadWrite<u32>),
        (0x204 => _reserved1),
        (0x240 => cm4_nmi_ctl_0: ReadWrite<u32>),
        (0x244 => cm4_nmi_ctl_1: ReadWrite<u32>),
        (0x248 => cm4_nmi_ctl_2: ReadWrite<u32>),
        (0x24C => cm4_nmi_ctl_3: ReadWrite<u32>),
        (0x250 => _reserved2),
        (0x1000 => cm0_ctl: ReadWrite<u32, CM0_CTL::Register>),
        (0x1004 => cm0_status: ReadOnly<u32, CM0_STATUS::Register>),
        (0x1008 => cm0_clock_ctl: ReadWrite<u32, CM0_CLOCK_CTL::Register>),
        (0x100C => _reserved3),
        (0x1100 => cm0_int0_status: ReadOnly<u32, CM0_INT0_STATUS::Register>),
        (0x1104 => cm0_int1_status: ReadOnly<u32, CM0_INT1_STATUS::Register>),
        (0x1108 => cm0_int2_status: ReadOnly<u32, CM0_INT2_STATUS::Register>),
        (0x110C => cm0_int3_status: ReadOnly<u32, CM0_INT3_STATUS::Register>),
        (0x1110 => cm0_int4_status: ReadOnly<u32, CM0_INT4_STATUS::Register>),
        (0x1114 => cm0_int5_status: ReadOnly<u32, CM0_INT5_STATUS::Register>),
        (0x1118 => cm0_int6_status: ReadOnly<u32, CM0_INT6_STATUS::Register>),
        (0x111C => cm0_int7_status: ReadOnly<u32, CM0_INT7_STATUS::Register>),
        (0x1120 => cm0_vector_table_base: ReadWrite<u32>),
        (0x1124 => _reserved4),
        (0x1140 => cm0_nmi_ctl_0: ReadWrite<u32>),
        (0x1144 => cm0_nmi_ctl_1: ReadWrite<u32>),
        (0x1148 => cm0_nmi_ctl_2: ReadWrite<u32>),
        (0x114C => cm0_nmi_ctl_3: ReadWrite<u32>),
        (0x1150 => _reserved5),
        (0x1200 => cm4_pwr_ctl: ReadWrite<u32, CM4_PWR_CTL::Register>),
        (0x1204 => cm4_pwr_delay_ctl: ReadWrite<u32>),
        (0x1208 => _reserved6),
        (0x1300 => ram0_ctl0: ReadWrite<u32, RAM0_CTL0::Register>),
        (0x1304 => ram0_status: ReadOnly<u32>),
        (0x1308 => _reserved7),
        (0x1340 => ram0_pwr_macro_ctl_0: ReadWrite<u32, RAM0_PWR_MACRO_CTL0::Register>),
        (0x1344 => ram0_pwr_macro_ctl_1: ReadWrite<u32, RAM0_PWR_MACRO_CTL1::Register>),
        (0x1348 => ram0_pwr_macro_ctl_2: ReadWrite<u32, RAM0_PWR_MACRO_CTL2::Register>),
        (0x134C => ram0_pwr_macro_ctl_3: ReadWrite<u32, RAM0_PWR_MACRO_CTL3::Register>),
        (0x1350 => ram0_pwr_macro_ctl_4: ReadWrite<u32, RAM0_PWR_MACRO_CTL4::Register>),
        (0x1354 => ram0_pwr_macro_ctl_5: ReadWrite<u32, RAM0_PWR_MACRO_CTL5::Register>),
        (0x1358 => ram0_pwr_macro_ctl_6: ReadWrite<u32, RAM0_PWR_MACRO_CTL6::Register>),
        (0x135C => ram0_pwr_macro_ctl_7: ReadWrite<u32, RAM0_PWR_MACRO_CTL7::Register>),
        (0x1360 => ram0_pwr_macro_ctl_8: ReadWrite<u32, RAM0_PWR_MACRO_CTL8::Register>),
        (0x1364 => ram0_pwr_macro_ctl_9: ReadWrite<u32, RAM0_PWR_MACRO_CTL9::Register>),
        (0x1368 => ram0_pwr_macro_ctl_10: ReadWrite<u32, RAM0_PWR_MACRO_CTL10::Register>),
        (0x136C => ram0_pwr_macro_ctl_11: ReadWrite<u32, RAM0_PWR_MACRO_CTL11::Register>),
        (0x1370 => ram0_pwr_macro_ctl_12: ReadWrite<u32, RAM0_PWR_MACRO_CTL12::Register>),
        (0x1374 => ram0_pwr_macro_ctl_13: ReadWrite<u32, RAM0_PWR_MACRO_CTL13::Register>),
        (0x1378 => ram0_pwr_macro_ctl_14: ReadWrite<u32, RAM0_PWR_MACRO_CTL14::Register>),
        (0x137C => ram0_pwr_macro_ctl_15: ReadWrite<u32, RAM0_PWR_MACRO_CTL15::Register>),
        (0x1380 => ram1_ctl0: ReadWrite<u32, RAM1_CTL0::Register>),
        (0x1384 => ram1_status: ReadOnly<u32>),
        (0x1388 => ram1_pwr_ctl: ReadWrite<u32, RAM1_PWR_CTL::Register>),
        (0x138C => _reserved8),
        (0x13A0 => ram2_ctl0: ReadWrite<u32, RAM2_CTL0::Register>),
        (0x13A4 => ram2_status: ReadOnly<u32>),
        (0x13A8 => ram2_pwr_ctl: ReadWrite<u32, RAM2_PWR_CTL::Register>),
        (0x13AC => _reserved9),
        (0x13C0 => ram_pwr_delay_ctl: ReadWrite<u32>),
        (0x13C4 => rom_ctl: ReadWrite<u32, ROM_CTL::Register>),
        (0x13C8 => ecc_ctl: ReadWrite<u32, ECC_CTL::Register>),
        (0x13CC => _reserved10),
        (0x1400 => product_id: ReadOnly<u32, PRODUCT_ID::Register>),
        (0x1404 => _reserved11),
        (0x1410 => dp_status: ReadOnly<u32, DP_STATUS::Register>),
        (0x1414 => ap_ctl: ReadWrite<u32, AP_CTL::Register>),
        (0x1418 => _reserved12),
        (0x1500 => buff_ctl: ReadWrite<u32>),
        (0x1504 => _reserved13),
        (0x1600 => systick_ctl: ReadWrite<u32, SYSTICK_CTL::Register>),
        (0x1604 => _reserved14),
        (0x1704 => mbist_stat: ReadOnly<u32, MBIST_STAT::Register>),
        (0x1708 => _reserved15),
        (0x1800 => cal_sup_set: ReadWrite<u32>),
        (0x1804 => cal_sup_clr: ReadWrite<u32>),
        (0x1808 => _reserved16),
        (0x2000 => cm0_pc_ctl: ReadWrite<u32>),
        (0x2004 => _reserved17),
        (0x2040 => cm0_pc0_handler: ReadWrite<u32>),
        (0x2044 => cm0_pc1_handler: ReadWrite<u32>),
        (0x2048 => cm0_pc2_handler: ReadWrite<u32>),
        (0x204C => cm0_pc3_handler: ReadWrite<u32>),
        (0x2050 => _reserved18),
        (0x20C4 => protection: ReadWrite<u32>),
        (0x20C8 => _reserved19),
        (0x2100 => trim_rom_ctl: ReadWrite<u32>),
        (0x2104 => trim_ram_ctl: ReadWrite<u32>),
        (0x2108 => _reserved20),
        (0x8000 => cm0_system_int_ctl: [ReadWrite<u32, CM0_SYSTEM_INT_CTL::Register>; 168]),
        (0x82A0 => @END),
    }
}
register_bitfields![u32,
IDENTITY [
    P OFFSET(0) NUMBITS(1) [],
    NS OFFSET(1) NUMBITS(1) [],
    PC OFFSET(4) NUMBITS(4) [],
    MS OFFSET(8) NUMBITS(4) []
],
CM4_STATUS [
    SLEEPING OFFSET(0) NUMBITS(1) [],
    SLEEPDEEP OFFSET(1) NUMBITS(1) [],
    PWR_DONE OFFSET(4) NUMBITS(1) []
],
CM4_CLOCK_CTL [
    FAST_INT_DIV OFFSET(8) NUMBITS(8) []
],
CM4_CTL [
    IOC_MASK OFFSET(24) NUMBITS(1) [],
    DZC_MASK OFFSET(25) NUMBITS(1) [],
    OFC_MASK OFFSET(26) NUMBITS(1) [],
    UFC_MASK OFFSET(27) NUMBITS(1) [],
    IXC_MASK OFFSET(28) NUMBITS(1) [],
    IDC_MASK OFFSET(31) NUMBITS(1) []
],
CM4_INT0_STATUS [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) [],
    SYSTEM_INT_VALID OFFSET(31) NUMBITS(1) []
],
CM4_INT1_STATUS [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) [],
    SYSTEM_INT_VALID OFFSET(31) NUMBITS(1) []
],
CM4_INT2_STATUS [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) [],
    SYSTEM_INT_VALID OFFSET(31) NUMBITS(1) []
],
CM4_INT3_STATUS [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) [],
    SYSTEM_INT_VALID OFFSET(31) NUMBITS(1) []
],
CM4_INT4_STATUS [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) [],
    SYSTEM_INT_VALID OFFSET(31) NUMBITS(1) []
],
CM4_INT5_STATUS [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) [],
    SYSTEM_INT_VALID OFFSET(31) NUMBITS(1) []
],
CM4_INT6_STATUS [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) [],
    SYSTEM_INT_VALID OFFSET(31) NUMBITS(1) []
],
CM4_INT7_STATUS [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) [],
    SYSTEM_INT_VALID OFFSET(31) NUMBITS(1) []
],
CM4_VECTOR_TABLE_BASE [
    ADDR22 OFFSET(10) NUMBITS(22) []
],
UDB_PWR_CTL [
    PWR_MODE OFFSET(0) NUMBITS(2) [
        SeeCM4_PWR_CTL = 0
    ],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
UDB_PWR_DELAY_CTL [
    UP OFFSET(0) NUMBITS(10) []
],
CM0_CTL [
    SLV_STALL OFFSET(0) NUMBITS(1) [],
    ENABLED OFFSET(1) NUMBITS(1) [],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
CM0_STATUS [
    SLEEPING OFFSET(0) NUMBITS(1) [],
    SLEEPDEEP OFFSET(1) NUMBITS(1) []
],
CM0_CLOCK_CTL [
    SLOW_INT_DIV OFFSET(8) NUMBITS(8) [],
    PERI_INT_DIV OFFSET(24) NUMBITS(8) []
],
CM0_INT0_STATUS [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) [],
    SYSTEM_INT_VALID OFFSET(31) NUMBITS(1) []
],
CM0_INT1_STATUS [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) [],
    SYSTEM_INT_VALID OFFSET(31) NUMBITS(1) []
],
CM0_INT2_STATUS [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) [],
    SYSTEM_INT_VALID OFFSET(31) NUMBITS(1) []
],
CM0_INT3_STATUS [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) [],
    SYSTEM_INT_VALID OFFSET(31) NUMBITS(1) []
],
CM0_INT4_STATUS [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) [],
    SYSTEM_INT_VALID OFFSET(31) NUMBITS(1) []
],
CM0_INT5_STATUS [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) [],
    SYSTEM_INT_VALID OFFSET(31) NUMBITS(1) []
],
CM0_INT6_STATUS [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) [],
    SYSTEM_INT_VALID OFFSET(31) NUMBITS(1) []
],
CM0_INT7_STATUS [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) [],
    SYSTEM_INT_VALID OFFSET(31) NUMBITS(1) []
],
CM0_VECTOR_TABLE_BASE [
    ADDR24 OFFSET(8) NUMBITS(24) []
],
CM4_PWR_CTL [
    PWR_MODE OFFSET(0) NUMBITS(2) [
        SwitchCM4OffPowerOffClockOffIsolateResetAndNoRetain = 0,
        RESET = 1,
        RETAINED = 2,
        SwitchCM4OnPowerOnClockOnNoIsolateNoResetAndNoRetain = 3
    ],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
CM4_PWR_DELAY_CTL [
    UP OFFSET(0) NUMBITS(10) []
],
RAM0_CTL0 [
    SLOW_WS OFFSET(0) NUMBITS(2) [],
    FAST_WS OFFSET(8) NUMBITS(2) [],
    ECC_EN OFFSET(16) NUMBITS(1) [],
    ECC_AUTO_CORRECT OFFSET(17) NUMBITS(1) [],
    ECC_INJ_EN OFFSET(18) NUMBITS(1) []
],
RAM0_STATUS [
    WB_EMPTY OFFSET(0) NUMBITS(1) []
],
RAM1_CTL0 [
    SLOW_WS OFFSET(0) NUMBITS(2) [],
    FAST_WS OFFSET(8) NUMBITS(2) [],
    ECC_EN OFFSET(16) NUMBITS(1) [],
    ECC_AUTO_CORRECT OFFSET(17) NUMBITS(1) [],
    ECC_INJ_EN OFFSET(18) NUMBITS(1) []
],
RAM1_STATUS [
    WB_EMPTY OFFSET(0) NUMBITS(1) []
],
RAM1_PWR_CTL [
    PWR_MODE OFFSET(0) NUMBITS(2) [
        SeeRAM0_PWR_MACRO_CTL = 0,
        Undefined = 1
    ],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
RAM2_CTL0 [
    SLOW_WS OFFSET(0) NUMBITS(2) [],
    FAST_WS OFFSET(8) NUMBITS(2) [],
    ECC_EN OFFSET(16) NUMBITS(1) [],
    ECC_AUTO_CORRECT OFFSET(17) NUMBITS(1) [],
    ECC_INJ_EN OFFSET(18) NUMBITS(1) []
],
RAM2_STATUS [
    WB_EMPTY OFFSET(0) NUMBITS(1) []
],
RAM2_PWR_CTL [
    PWR_MODE OFFSET(0) NUMBITS(2) [
        SeeRAM0_PWR_MACRO_CTL = 0,
        Undefined = 1
    ],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
RAM_PWR_DELAY_CTL [
    UP OFFSET(0) NUMBITS(10) []
],
ROM_CTL [
    SLOW_WS OFFSET(0) NUMBITS(2) [],
    FAST_WS OFFSET(8) NUMBITS(2) []
],
ECC_CTL [
    WORD_ADDR OFFSET(0) NUMBITS(25) [],
    PARITY OFFSET(25) NUMBITS(7) []
],
PRODUCT_ID [
    FAMILY_ID OFFSET(0) NUMBITS(12) [],
    MAJOR_REV OFFSET(16) NUMBITS(4) [],
    MINOR_REV OFFSET(20) NUMBITS(4) []
],
DP_STATUS [
    SWJ_CONNECTED OFFSET(0) NUMBITS(1) [],
    SWJ_DEBUG_EN OFFSET(1) NUMBITS(1) [],
    SWJ_JTAG_SEL OFFSET(2) NUMBITS(1) []
],
AP_CTL [
    CM0_ENABLE OFFSET(0) NUMBITS(1) [],
    CM4_ENABLE OFFSET(1) NUMBITS(1) [],
    SYS_ENABLE OFFSET(2) NUMBITS(1) [],
    CM0_DISABLE OFFSET(16) NUMBITS(1) [],
    CM4_DISABLE OFFSET(17) NUMBITS(1) [],
    SYS_DISABLE OFFSET(18) NUMBITS(1) []
],
BUFF_CTL [
    WRITE_BUFF OFFSET(0) NUMBITS(1) []
],
SYSTICK_CTL [
    TENMS OFFSET(0) NUMBITS(24) [],
    CLOCK_SOURCE OFFSET(24) NUMBITS(2) [],
    SKEW OFFSET(30) NUMBITS(1) [],
    NOREF OFFSET(31) NUMBITS(1) []
],
MBIST_STAT [
    SFP_READY OFFSET(0) NUMBITS(1) [],
    SFP_FAIL OFFSET(1) NUMBITS(1) []
],
CAL_SUP_SET [
    DATA OFFSET(0) NUMBITS(32) []
],
CAL_SUP_CLR [
    DATA OFFSET(0) NUMBITS(32) []
],
CM0_PC_CTL [
    VALID OFFSET(0) NUMBITS(4) []
],
CM0_PC0_HANDLER [
    ADDR OFFSET(0) NUMBITS(32) []
],
CM0_PC1_HANDLER [
    ADDR OFFSET(0) NUMBITS(32) []
],
CM0_PC2_HANDLER [
    ADDR OFFSET(0) NUMBITS(32) []
],
CM0_PC3_HANDLER [
    ADDR OFFSET(0) NUMBITS(32) []
],
PROTECTION [
    STATE OFFSET(0) NUMBITS(3) []
],
TRIM_ROM_CTL [
    TRIM OFFSET(0) NUMBITS(32) []
],
TRIM_RAM_CTL [
    TRIM OFFSET(0) NUMBITS(32) []
],
CM4_NMI_CTL0 [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) []
],
CM4_NMI_CTL1 [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) []
],
CM4_NMI_CTL2 [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) []
],
CM4_NMI_CTL3 [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) []
],
CM0_NMI_CTL0 [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) []
],
CM0_NMI_CTL1 [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) []
],
CM0_NMI_CTL2 [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) []
],
CM0_NMI_CTL3 [
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) []
],
RAM0_PWR_MACRO_CTL0 [
    PWR_MODE OFFSET(0) NUMBITS(2) [
        OFF = 0,
        Undefined = 1,
        RETAINED = 2,
        ENABLED = 3
    ],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
RAM0_PWR_MACRO_CTL1 [
    PWR_MODE OFFSET(0) NUMBITS(2) [
        OFF = 0,
        Undefined = 1,
        RETAINED = 2,
        ENABLED = 3
    ],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
RAM0_PWR_MACRO_CTL2 [
    PWR_MODE OFFSET(0) NUMBITS(2) [
        OFF = 0,
        Undefined = 1,
        RETAINED = 2,
        ENABLED = 3
    ],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
RAM0_PWR_MACRO_CTL3 [
    PWR_MODE OFFSET(0) NUMBITS(2) [
        OFF = 0,
        Undefined = 1,
        RETAINED = 2,
        ENABLED = 3
    ],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
RAM0_PWR_MACRO_CTL4 [
    PWR_MODE OFFSET(0) NUMBITS(2) [
        OFF = 0,
        Undefined = 1,
        RETAINED = 2,
        ENABLED = 3
    ],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
RAM0_PWR_MACRO_CTL5 [
    PWR_MODE OFFSET(0) NUMBITS(2) [
        OFF = 0,
        Undefined = 1,
        RETAINED = 2,
        ENABLED = 3
    ],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
RAM0_PWR_MACRO_CTL6 [
    PWR_MODE OFFSET(0) NUMBITS(2) [
        OFF = 0,
        Undefined = 1,
        RETAINED = 2,
        ENABLED = 3
    ],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
RAM0_PWR_MACRO_CTL7 [
    PWR_MODE OFFSET(0) NUMBITS(2) [
        OFF = 0,
        Undefined = 1,
        RETAINED = 2,
        ENABLED = 3
    ],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
RAM0_PWR_MACRO_CTL8 [
    PWR_MODE OFFSET(0) NUMBITS(2) [
        OFF = 0,
        Undefined = 1,
        RETAINED = 2,
        ENABLED = 3
    ],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
RAM0_PWR_MACRO_CTL9 [
    PWR_MODE OFFSET(0) NUMBITS(2) [
        OFF = 0,
        Undefined = 1,
        RETAINED = 2,
        ENABLED = 3
    ],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
RAM0_PWR_MACRO_CTL10 [
    PWR_MODE OFFSET(0) NUMBITS(2) [
        OFF = 0,
        Undefined = 1,
        RETAINED = 2,
        ENABLED = 3
    ],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
RAM0_PWR_MACRO_CTL11 [
    PWR_MODE OFFSET(0) NUMBITS(2) [
        OFF = 0,
        Undefined = 1,
        RETAINED = 2,
        ENABLED = 3
    ],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
RAM0_PWR_MACRO_CTL12 [
    PWR_MODE OFFSET(0) NUMBITS(2) [
        OFF = 0,
        Undefined = 1,
        RETAINED = 2,
        ENABLED = 3
    ],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
RAM0_PWR_MACRO_CTL13 [
    PWR_MODE OFFSET(0) NUMBITS(2) [
        OFF = 0,
        Undefined = 1,
        RETAINED = 2,
        ENABLED = 3
    ],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
RAM0_PWR_MACRO_CTL14 [
    PWR_MODE OFFSET(0) NUMBITS(2) [
        OFF = 0,
        Undefined = 1,
        RETAINED = 2,
        ENABLED = 3
    ],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
RAM0_PWR_MACRO_CTL15 [
    PWR_MODE OFFSET(0) NUMBITS(2) [
        OFF = 0,
        Undefined = 1,
        RETAINED = 2,
        ENABLED = 3
    ],
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
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
