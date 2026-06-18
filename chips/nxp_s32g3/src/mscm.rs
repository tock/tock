// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

pub const NUM_EXTERNAL_IRQS: usize = 240;

// MSCM Interrupts
// Documented in RM §8.5.2.2 (MSCM interrupt router functional description) and
// docs/pcu/mscm_interrupt_management.md (MSCM C2C Interrupt Mapping (S32G3)).
//
// These represent the 14 inter-core interrupt registers per target core, where
// Index 3 and 4 are PCIe MSI, and the remaining 12 are mapped to GIC SPIs.
// On the Cortex-M7 core, they are routed directly to the NVIC external vectors.

/// MSCM PCIe1 Interrupt (Index 4, GIC SPI 0).
/// RM §8.4.21, §8.5.1.2.
pub const MSCM_PCIE_1_MSI: u32 = 0;

/// MSCM Core-to-Core Interrupt 0 (Index 0, GIC SPI 1, INTID 33).
/// Used by IPCF/MSCM DTS (RM §8.4.19, §8.5.1.1).
pub const MSCM_INT0: u32 = 1;

/// MSCM Core-to-Core Interrupt 1 (Index 1, GIC SPI 2, INTID 34).
/// Used by IPCF Instance 0 (M7→A53) (RM §8.4.19, §8.5.1.1).
pub const MSCM_INT1: u32 = 2;

/// MSCM Core-to-Core Interrupt 2 (Index 2, GIC SPI 3, INTID 35).
/// Used by IPCF/MSCM DTS (RM §8.4.19, §8.5.1.1).
pub const MSCM_INT2: u32 = 3;

/// MSCM PCIe0 Interrupt (Index 3, GIC SPI 4).
/// RM §8.4.21, §8.5.1.2.
pub const MSCM_PCIE_0_MSI: u32 = 4;

/// MSCM Core-to-Core Interrupt 3 (Index 5, GIC SPI 22, INTID 54).
/// Used by IPCF Instance 1 (M7→A53, Valeo SHM) (RM §8.4.19, §8.5.1.1).
pub const MSCM_INT3: u32 = 22;

/// MSCM Core-to-Core Interrupt 4 (Index 6, GIC SPI 23, INTID 55).
/// Used by IPCF/MSCM DTS (RM §8.4.19, §8.5.1.1).
pub const MSCM_INT4: u32 = 23;

/// MSCM Core-to-Core Interrupt 5 (Index 7, GIC SPI 68, INTID 100).
/// Used by IPCF Instance 1 (A53→M7 tx confirm) (RM §8.4.19, §8.5.1.1).
pub const MSCM_INT5: u32 = 68;

/// MSCM Core-to-Core Interrupt 6 (Index 8, GIC SPI 69, INTID 101).
/// Used by IPCF/MSCM DTS (RM §8.4.19, §8.5.1.1).
pub const MSCM_INT6: u32 = 69;

/// MSCM Core-to-Core Interrupt 7 (Index 9, GIC SPI 164, INTID 196).
/// Used by IRC M7→VM Doorbell (RM §8.4.19, §8.5.1.1).
pub const MSCM_INT7: u32 = 164;

/// MSCM Core-to-Core Interrupt 8 (Index 10, GIC SPI 165, INTID 197).
/// Used by IRC QNX→VM Doorbell (RM §8.4.19, §8.5.1.1).
pub const MSCM_INT8: u32 = 165;

/// MSCM Core-to-Core Interrupt 9 (Index 11, GIC SPI 166, INTID 198).
/// Reserved for IRC future use (RM §8.4.19, §8.5.1.1).
pub const MSCM_INT9: u32 = 166;

/// MSCM Core-to-Core Interrupt 10 (Index 12, GIC SPI 167, INTID 199).
/// Reserved for IRC future use (RM §8.4.19, §8.5.1.1).
pub const MSCM_INT10: u32 = 167;

/// MSCM Core-to-Core Interrupt 11 (Index 13, GIC SPI 168, INTID 200).
/// Reserved for IRC future use (RM §8.4.19, §8.5.1.1).
pub const MSCM_INT11: u32 = 168;

// CTI Interrupts
pub const CTI_INT0: u32 = 5;
pub const CTI_INT1: u32 = 6;

// MCM Interrupt
pub const MCM: u32 = 7;

// eDMA Interrupts
pub const DMA0_0_15: u32 = 8;
pub const DMA0_16_31: u32 = 9;
pub const DMA0_ERR0: u32 = 10;
pub const DMA1_0_15: u32 = 11;
pub const DMA1_16_31: u32 = 12;
pub const DMA1_ERR0: u32 = 13;

// Software Watchdog Timer (SWT) Interrupts
pub const SWT_0: u32 = 14;
pub const SWT_1: u32 = 15;
pub const SWT_2: u32 = 16;
pub const SWT_3: u32 = 17;
pub const SWT_4: u32 = 18;
pub const SWT_5: u32 = 19;
pub const SWT_6: u32 = 20;
pub const SWT_7: u32 = 21;
pub const SWT_8: u32 = 156;
pub const SWT_9: u32 = 157;
pub const SWT_10: u32 = 158;
pub const SWT_11: u32 = 159;

// System Timer Module (STM) Interrupts
pub const STM_0: u32 = 24;
pub const STM_1: u32 = 25;
pub const STM_2: u32 = 26;
pub const STM_3: u32 = 27;
pub const STM_4: u32 = 28;
pub const STM_5: u32 = 29;
pub const STM_6: u32 = 30;
pub const STM_7: u32 = 31;
pub const STM_8: u32 = 160;
pub const STM_9: u32 = 161;
pub const STM_10: u32 = 162;
pub const STM_11: u32 = 163;
pub const STM_TS_CH_REQ: u32 = 204;

// Quad Serial Peripheral Interface (QSPI) Interrupts
pub const QSPI0: u32 = 32;
pub const QSPI1: u32 = 33;
pub const QSPI2: u32 = 34;

// Self-Test Control Unit (STCU2) Interrupt
pub const STCU2_LBIST_MBIST: u32 = 35;

// SD Host Controller (uSDHC) Interrupt
pub const USDHC: u32 = 36;

// FlexCAN Interrupts
pub const CAN0_ORED: u32 = 37;
pub const CAN0_ERR: u32 = 38;
pub const CAN0_ORED_0_7_MB: u32 = 39;
pub const CAN0_ORED_8_127_MB: u32 = 40;
pub const CAN1_ORED: u32 = 41;
pub const CAN1_ERR: u32 = 42;
pub const CAN1_ORED_0_7_MB: u32 = 43;
pub const CAN1_ORED_8_127_MB: u32 = 44;
pub const CAN2_ORED: u32 = 45;
pub const CAN2_ERR: u32 = 46;
pub const CAN2_ORED_0_7_MB: u32 = 47;
pub const CAN2_ORED_8_127_MB: u32 = 48;
pub const CAN3_ORED: u32 = 49;
pub const CAN3_ERR: u32 = 50;
pub const CAN3_ORED_0_7_MB: u32 = 51;
pub const CAN3_ORED_8_127_MB: u32 = 52;

// Periodic Interrupt Timer (PIT) Interrupts
pub const PIT_0: u32 = 53;
pub const PIT_1: u32 = 54;

// FlexTimer Module (FTM) Interrupts
pub const FTM_0: u32 = 55;
pub const FTM_1: u32 = 56;

// Gigabit Ethernet (GMAC0) Interrupts
pub const GMAC0_COMMON: u32 = 57;
pub const GMAC0_CH0_TX: u32 = 58;
pub const GMAC0_CH0_RX: u32 = 59;
pub const GMAC0_CH1_TX: u32 = 60;
pub const GMAC0_CH1_RX: u32 = 61;
pub const GMAC0_CH2_TX: u32 = 62;
pub const GMAC0_CH2_RX: u32 = 63;
pub const GMAC0_CH3_TX: u32 = 64;
pub const GMAC0_CH3_RX: u32 = 65;
pub const GMAC0_CH4_TX: u32 = 66;
pub const GMAC0_CH4_RX: u32 = 67;

// SAR ADC Interrupts
pub const SAR_ADC0_INT: u32 = 70;
pub const SAR_ADC1_INT: u32 = 71;

// FlexRay Interrupts
pub const FLEXRAY0_NCERR: u32 = 72;
pub const FLEXRAY0_CERR: u32 = 73;
pub const FLEXRAY0_CH0_RX_FIFO: u32 = 74;
pub const FLEXRAY0_CH1_RX_FIFO: u32 = 75;
pub const FLEXRAY0_WKUP: u32 = 76;
pub const FLEXRAY0_STATUS: u32 = 77;
pub const FLEXRAY0_CMBERR: u32 = 78;
pub const FLEXRAY0_TX_BUFF: u32 = 79;
pub const FLEXRAY0_RX_BUFF: u32 = 80;
pub const FLEXRAY0_MODULE: u32 = 81;

// LINFlexD Interrupts
pub const LINFLEXD_0: u32 = 82;
pub const LINFLEXD_1: u32 = 83;
pub const LINFLEXD_2: u32 = 84;

// Deserial Peripheral Interface (SPI) Interrupts
pub const SPI0: u32 = 85;
pub const SPI1: u32 = 86;
pub const SPI2: u32 = 87;
pub const SPI3: u32 = 88;
pub const SPI4: u32 = 89;
pub const SPI5: u32 = 90;

// Inter-Integrated Circuit (I2C) Interrupts
pub const I2C0: u32 = 92;
pub const I2C1: u32 = 93;
pub const I2C2: u32 = 94;
pub const I2C3: u32 = 95;
pub const I2C4: u32 = 96;

// MC_RGM Interrupt
pub const MC_RGM: u32 = 98;

// Fault Control and Monitoring Unit (FCCU) Interrupts
pub const FCCU_ALARM: u32 = 100;
pub const FCCU_MISC: u32 = 101;

// SBSW Interrupt
pub const SBSW: u32 = 102;

// Hardware Security Engine MU (HSE MU) Interrupts
pub const HSE_MU0_TX: u32 = 103;
pub const HSE_MU0_RX: u32 = 104;
pub const HSE_MU0_ORED: u32 = 105;
pub const HSE_MU1_TX: u32 = 106;
pub const HSE_MU1_RX: u32 = 107;
pub const HSE_MU1_ORED: u32 = 108;
pub const HSE_MU2_TX: u32 = 109;
pub const HSE_MU2_RX: u32 = 110;
pub const HSE_MU2_ORED: u32 = 111;
pub const HSE_MU3_TX: u32 = 112;
pub const HSE_MU3_RX: u32 = 113;
pub const HSE_MU3_ORED: u32 = 114;

// DDR0 Interrupts
pub const DDR0_SCRUB: u32 = 115;
pub const DDR0_PHY: u32 = 116;

// Trigger Unit (CTU) Interrupts
pub const CTU_FIFO_FULL_EMPTY: u32 = 117;
pub const CTU_M_RELOAD: u32 = 118;
pub const CTU_ERR: u32 = 119;

// Temperature Monitoring Unit (TMU) Interrupt
pub const TMU_ALARM: u32 = 120;

// Real Time Clock (RTC) Interrupt
pub const RTC_SYS_CONT: u32 = 121;

// PCIe Controller 0 (PCIE0) Interrupts
pub const PCIE0_ORED_DMA: u32 = 123;
pub const PCIE0_LINK: u32 = 124;
pub const PCIE0_AXI_MSI: u32 = 125;
pub const PCIE0_PHY_DOWN: u32 = 126;
pub const PCIE0_PHY_UP: u32 = 127;
pub const PCIE0_INTA: u32 = 128;
pub const PCIE0_INTB: u32 = 129;
pub const PCIE0_INTC: u32 = 130;
pub const PCIE0_INTD: u32 = 131;
pub const PCIE0_MISC: u32 = 132;
pub const PCIE0_PCS: u32 = 133;
pub const PCIE0_TLP_NC: u32 = 134;

// Cortex-A53 Cluster Interrupts
pub const CORTEX_A53_ERR_L2RAM_CLUSTER0: u32 = 151;
pub const CORTEX_A53_ERR_LIVLOCK_CLUSTER0: u32 = 152;
pub const CORTEX_A53_ERR_L2RAM_CLUSTER1: u32 = 153;
pub const CORTEX_A53_ERR_LIVLOCK_CLUSTER1: u32 = 154;

// JTAG Data Communication (JDC) Interrupt
pub const JDC: u32 = 155;

// Light-Latency Communication Engine (LLCE) Interrupts
pub const LLCE0_INT0: u32 = 170;
pub const LLCE0_INT1: u32 = 171;
pub const LLCE0_INT2: u32 = 172;
pub const LLCE0_INT3: u32 = 173;
pub const LLCE0_ICSR14: u32 = 174;
pub const LLCE0_ICSR15: u32 = 175;
pub const LLCE0_ICSR16: u32 = 176;
pub const LLCE0_ICSR17: u32 = 177;
pub const LLCE0_ICSR18: u32 = 178;
pub const LLCE0_ICSR19: u32 = 179;
pub const LLCE0_ICSR20: u32 = 180;
pub const LLCE0_ICSR21: u32 = 181;
pub const LLCE0_ICSR22: u32 = 182;
pub const LLCE0_ICSR23: u32 = 183;
pub const LLCE0_ICSR24: u32 = 184;
pub const LLCE0_ICSR25: u32 = 185;
pub const LLCE0_ICSR26: u32 = 186;
pub const LLCE0_ICSR27: u32 = 187;

// Packet Forwarding Engine (PFE0) Interrupts
pub const PFE0_CH0_STAT: u32 = 190;
pub const PFE0_CH1_STAT: u32 = 191;
pub const PFE0_CH2_STAT: u32 = 192;
pub const PFE0_CH3_STAT: u32 = 193;
pub const PFE0_BMU1_BMU2: u32 = 194;
pub const PFE0_HIF_NC: u32 = 195;
pub const PFE0_UT_GPT: u32 = 196;
pub const PFE0_PMT: u32 = 197;
pub const PFE0_ORED: u32 = 198;

// System Integration Unit Lite (SIUL1) Ored Interrupt
pub const SIUL1_ORED: u32 = 210;

// USB OTG Interrupts
pub const USB0_OTG_CORE: u32 = 211;
pub const USB0_OTG_WKP: u32 = 212;

// Wakeup Unit (WKPU) Pad Group Interrupt
pub const WKPU_GRP: u32 = 213;

// PCIe Controller 1 (PCIE1) Interrupts
pub const PCIE1_ORED_DMA: u32 = 214;
pub const PCIE1_STAT: u32 = 215;
pub const PCIE1_AXI_MSI: u32 = 216;
pub const PCIE1_PHY_LDOWN: u32 = 217;
pub const PCIE1_PHY_LUP: u32 = 218;
pub const PCIE1_INTA: u32 = 219;
pub const PCIE1_INTB: u32 = 220;
pub const PCIE1_INTC: u32 = 221;
pub const PCIE1_INTD: u32 = 222;
pub const PCIE1_MISC: u32 = 223;
pub const PCIE1_PCS: u32 = 224;
pub const PCIE1_TLP: u32 = 225;

// Extended Resource Domain Controller (XRDC) Interrupts
pub const XRDC_ERR: u32 = 229;
pub const XRDC_MANAGER_ERR: u32 = 230;

/// Base address of MSCM (Miscellaneous System Control Module).
/// RM §8.4.1 MSCM memory map.
pub const MSCM_BASE: StaticRef<MscmRegisters> =
    unsafe { StaticRef::new(0x4019_8000 as *const MscmRegisters) };

register_structs! {
    /// Processor CPn Registers.
    /// RM §8.4.1 MSCM memory map.
    pub CpRegisters {
        /// Processor Type.
        /// RM §8.4.9 Processor Type (CP0TYPE - CP11TYPE).
        (0x00 => pub cptype: ReadOnly<u32>),
        /// Processor Number.
        /// RM §8.4.10 Processor Number (CP0NUM - CP11NUM).
        (0x04 => pub cpnum: ReadOnly<u32>),
        /// Processor Count / Revision.
        /// RM §8.4.11 Processor Count (CP0REV - CP11REV).
        (0x08 => pub cprev: ReadOnly<u32>),
        /// Processor Configuration 0.
        /// RM §8.4.12 Processor Configuration 0 (CP0CFG0 - CP11CFG0).
        (0x0C => pub cpcfg0: ReadOnly<u32>),
        /// Processor Configuration 0.
        /// RM §8.4.13 Processor Configuration 1 (CP0CFG1 - CP11CFG1).
        (0x10 => pub cpcfg1: ReadOnly<u32>),
        /// Processor Configuration 2.
        /// RM §8.4.14 Processor Configuration 2 (CP0CFG2 - CP11CFG2).
        (0x14 => pub cpcfg2: ReadOnly<u32>),
        /// Processor Configuration 3.
        /// RM §8.4.15 Processor Configuration 3 (CP0CFG3 - CP11CFG3).
        (0x18 => pub cpcfg3: ReadOnly<u32>),
        /// Reserved.
        (0x1C => _reserved),
        (0x20 => @END),
    }
}

register_structs! {
    /// MSCM Register Map.
    /// RM §8.4.1 MSCM memory map.
    pub MscmRegisters {
        /// Processor X Type Register.
        /// RM §8.4.2 Processor X Type (CPXTYPE).
        (0x000 => pub cpxtype: ReadOnly<u32>),
        /// Processor X Number Register.
        /// RM §8.4.3 Processor X Number (CPXNUM).
        (0x004 => pub cpxnum: ReadOnly<u32>),
        /// Processor X Revision Register.
        /// RM §8.4.4 Processor X Revision (CPXREV).
        (0x008 => pub cpxrev: ReadOnly<u32>),
        /// Processor X Configuration Register 0.
        /// RM §8.4.5 Processor X Configuration 0 (CPXCFG0).
        (0x00C => pub cpxcfg0: ReadOnly<u32>),
        /// Processor X Configuration Register 1.
        /// RM §8.4.6 Processor X Configuration 1 (CPXCFG1).
        (0x010 => pub cpxcfg1: ReadOnly<u32>),
        /// Processor X Configuration Register 2.
        /// RM §8.4.7 Processor X Configuration 2 (CPXCFG2).
        (0x014 => pub cpxcfg2: ReadOnly<u32>),
        /// Processor X Configuration Register 3.
        /// RM §8.4.8 Processor X Configuration 3 (CPXCFG3).
        (0x018 => pub cpxcfg3: ReadOnly<u32>),
        /// Reserved gap.
        (0x01C => _reserved0),
        /// Processor CP0 to CP11 registers (12 processors, each has TYPE, NUM, REV, CFG0, CFG1, CFG2, CFG3).
        /// RM §8.4.9 to §8.4.15.
        (0x020 => pub cp_regs: [CpRegisters; 12]),
        /// Reserved gap.
        (0x1A0 => _reserved1),
        /// Interrupt Router Configuration Register.
        /// RM §8.4.16 Interrupt Router Configuration (IRCPCFG).
        (0x400 => pub ircpcfg: ReadWrite<u32, IrcpCfg::Register>),
        /// Reserved gap.
        (0x404 => _reserved2),
        /// Interrupt Router Nonmaskable Interrupt Control Register.
        /// RM §8.4.17 Interrupt Router Nonmaskable Interrupt Control (IRNMIC).
        (0x800 => pub irnmic: ReadWrite<u32, IrNmic::Register>),
        /// Reserved gap.
        (0x804 => _reserved3),
        /// Interrupt Router Shared Peripheral Routing Control (IRSPRC0 - IRSPRC239).
        /// RM §8.4.18 Interrupt Router Shared Peripheral Routing Control (IRSPRC0 - IRSPRC239).
        (0x880 => pub irsprc: [ReadWrite<u16, Irsprc::Register>; 240]),
        (0xA60 => @END),
    }
}

register_bitfields![u32,
    /// Interrupt Router Configuration Register (IRCPCFG).
    /// RM §8.4.16.
    IrcpCfg [
        /// Lock. Provides a locking mechanism that can be used to limit the ability to write to the register.
        /// RM §8.4.16 field `31 LOCK`.
        LOCK OFFSET(31) NUMBITS(1) [
            /// Register can be written to by any privileged write.
            Unlocked = 0,
            /// Register is locked (read-only) until the next reset.
            Locked = 1,
        ],
        /// Reserved. Reads return 0 (RM §8.4.16 field `30-12`).
        _RSV_12_30 OFFSET(12) NUMBITS(19) [],
        /// Trusted Cores Configuration. Designates specific cores in the system as trusted.
        /// RM §8.4.16 field `11-0 TRST_CP`.
        TRST_CP OFFSET(0) NUMBITS(12) [],
    ],

    /// Interrupt Router Nonmaskable Interrupt Control Register (IRNMIC).
    /// RM §8.4.17.
    IrNmic [
        /// LOCK. Non-maskable interrupt configuration lock.
        /// RM §8.4.17 field `31 LOCK`.
        LOCK OFFSET(31) NUMBITS(1) [
            /// Writes to register are allowed.
            Unlocked = 0,
            /// Writes to register are ignored.
            Locked = 1,
        ],
        /// Reserved. Reads return 0 (RM §8.4.17 field `30-0`).
        _RSV_0_30 OFFSET(0) NUMBITS(31) [],
    ]
];

register_bitfields![u16,
    /// Interrupt Router Shared Peripheral Routing Control Register (IRSPRC0 - IRSPRC239).
    /// RM §8.4.18.
    pub Irsprc [
        /// Lock Interrupt Request.
        /// RM §8.4.18 field `15 LOCK`.
        LOCK OFFSET(15) NUMBITS(1) [
            /// Writes to IRSPRCn are allowed.
            Unlocked = 0,
            /// Writes to IRSPRCn are ignored.
            Locked = 1,
        ],
        /// Reserved. Reads return 0 (RM §8.4.18 field `14-5`).
        _RSV_5_14 OFFSET(5) NUMBITS(10) [],
        /// Enable Cortex-M7_3 Interrupt Steering.
        /// RM §8.4.18 field `4 M7_3`.
        M7_3 OFFSET(4) NUMBITS(1) [
            /// Routing disabled.
            Disabled = 0,
            /// Routing enabled.
            Enabled = 1,
        ],
        /// Enable Cortex-M7_2 Interrupt Steering.
        /// RM §8.4.18 field `3 M7_2`.
        M7_2 OFFSET(3) NUMBITS(1) [
            /// Routing disabled.
            Disabled = 0,
            /// Routing enabled.
            Enabled = 1,
        ],
        /// Enable Cortex-M7_1 Interrupt Steering.
        /// RM §8.4.18 field `2 M7_1`.
        M7_1 OFFSET(2) NUMBITS(1) [
            /// Routing disabled.
            Disabled = 0,
            /// Routing enabled.
            Enabled = 1,
        ],
        /// Enable Cortex-M7_0 Interrupt Steering.
        /// RM §8.4.18 field `1 M7_0`.
        M7_0 OFFSET(1) NUMBITS(1) [
            /// Routing disabled.
            Disabled = 0,
            /// Routing enabled.
            Enabled = 1,
        ],
        /// Enable GIC500 Interrupt Steering.
        /// RM §8.4.18 field `0 GIC500`.
        GIC500 OFFSET(0) NUMBITS(1) [
            /// Routing disabled.
            Disabled = 0,
            /// Routing enabled.
            Enabled = 1,
        ],
    ]
];

/// Enumeration of S32G3 cores for interrupt steering.
pub enum S32G3Core {
    M7_0 = 0,
    M7_1 = 1,
    M7_2 = 2,
    M7_3 = 3,
}

/// MSCM (Miscellaneous System Control Module) driver.
/// MSCM contains CPU configuration and on-chip memory controller registers.
/// * Availability of software-accessible processor core configuration information
/// * Interrupt router control registers
/// * Message-based interrupt configuration and management registers
pub struct Mscm {
    registers: StaticRef<MscmRegisters>,
}
impl Mscm {
    pub const fn new() -> Self {
        Self {
            registers: MSCM_BASE,
        }
    }
    pub fn enable_interrupt(&self, irq: u32, core: S32G3Core) {
        if irq >= NUM_EXTERNAL_IRQS as u32 {
            panic!("Invalid IRQ number: {}", irq);
        }
        let irsprc = &self.registers.irsprc[irq as usize];
        match core {
            S32G3Core::M7_0 => irsprc.modify(Irsprc::M7_0::Enabled),
            S32G3Core::M7_1 => irsprc.modify(Irsprc::M7_1::Enabled),
            S32G3Core::M7_2 => irsprc.modify(Irsprc::M7_2::Enabled),
            S32G3Core::M7_3 => irsprc.modify(Irsprc::M7_3::Enabled),
        }
    }
}
