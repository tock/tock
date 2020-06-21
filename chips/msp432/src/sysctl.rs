//! System Controller (SYSCTL)

use kernel::common::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::common::StaticRef;

pub static mut SYSCTL: SysCtl = SysCtl::new();

const SYSCTL_BASE: StaticRef<SysCtlRegisters> =
    unsafe { StaticRef::new(0xE004_3000 as *const SysCtlRegisters) };

register_structs! {
    /// SYSCTL
    SysCtlRegisters {
        /// Reboot Control Register
        (0x0000 => reboot_ctl: ReadWrite<u32, SYS_REBOOT_CTL::Register>),
        /// NMI Control and Status Register
        (0x0004 => nmi_ctlstat: ReadWrite<u32, SYS_NMI_CTLSTAT::Register>),
        /// Watchdog Reset Control Register
        (0x0008 => wdtreset_ctl: ReadWrite<u32, SYS_WDTRESET_CTL::Register>),
        /// Peripheral Halt Control Register
        (0x000C => perihalt_ctl: ReadWrite<u32, SYS_PERIHALT_CTL::Register>),
        /// SRAM Size Register
        (0x0010 => sram_size: ReadOnly<u32>),
        /// SRAM Bank Enable Register
        (0x0014 => sram_banken: ReadWrite<u32, SYS_SRAM_BANKEN::Register>),
        /// SRAM Bank Retention Control Register
        (0x0018 => sram_bankret: ReadWrite<u32, SYS_SRAM_BANKRET::Register>),
        (0x001C => _reserved0),
        /// Flash Size Register
        (0x0020 => flash_size: ReadOnly<u32>),
        (0x0024 => _reserved1),
        /// Digital I/O Glitch Filter Control Register
        (0x0030 => dio_gltflt_ctl: ReadWrite<u32>),
        (0x0034 => _reserved2),
        /// IP Protected Secure Zone Data Access Unlock Register
        (0x0040 => secdata_unlock: ReadWrite<u32>),
        (0x0044 => _reserved3),
        /// Master Unlock Register
        (0x1000 => master_unlock: ReadWrite<u32>),
        /// Boot Override Request Register
        (0x1004 => bootover_req_0: ReadWrite<u32>),
        /// Boot Override Request Register
        (0x1008 => bootover_req_1: ReadWrite<u32>),
        /// Boot Override Acknowledge Register
        (0x100C => bootover_ack: ReadWrite<u32>),
        /// Reset Request Register
        (0x1010 => reset_req: ReadWrite<u32, SYS_RESET_REQ::Register>),
        /// Reset Status and Override Register
        (0x1014 => reset_statover: ReadWrite<u32, SYS_RESET_STATOVER::Register>),
        (0x1018 => _reserved4),
        /// System Status Register
        (0x1020 => system_stat: ReadOnly<u32, SYS_SYSTEM_STAT::Register>),
        (0x1024 => @END),
    }
}

register_bitfields![u32,
    SYS_REBOOT_CTL [
        /// Write 1 initiates a Reboot of the device
        REBOOT OFFSET(0) NUMBITS(1) [],
        /// Key to enable writes to bit 0
        WKEY OFFSET(8) NUMBITS(8) []
    ],
    SYS_NMI_CTLSTAT [
        /// CS interrupt as a source of NMI
        CS_SRC OFFSET(0) NUMBITS(1) [
            /// Disables CS interrupt as a source of NMI
            DisablesCSInterruptAsASourceOfNMI = 0,
            /// Enables CS interrupt as a source of NMI
            EnablesCSInterruptAsASourceOfNMI = 1
        ],
        /// PSS interrupt as a source of NMI
        PSS_SRC OFFSET(1) NUMBITS(1) [
            /// Disables the PSS interrupt as a source of NMI
            DisablesThePSSInterruptAsASourceOfNMI = 0,
            /// Enables the PSS interrupt as a source of NMI
            EnablesThePSSInterruptAsASourceOfNMI = 1
        ],
        /// PCM interrupt as a source of NMI
        PCM_SRC OFFSET(2) NUMBITS(1) [
            /// Disbles the PCM interrupt as a source of NMI
            DisblesThePCMInterruptAsASourceOfNMI = 0,
            /// Enables the PCM interrupt as a source of NMI
            EnablesThePCMInterruptAsASourceOfNMI = 1
        ],
        /// RSTn/NMI pin configuration
        PIN_SRC OFFSET(3) NUMBITS(1) [
            /// Configures the RSTn_NMI pin as a source of POR Class Reset
            ConfiguresTheRSTn_NMIPinAsASourceOfPORClassReset = 0,
            /// Configures the RSTn_NMI pin as a source of NMI
            ConfiguresTheRSTn_NMIPinAsASourceOfNMI = 1
        ],
        /// CS interrupt was the source of NMI
        CS_FLG OFFSET(16) NUMBITS(1) [
            /// indicates CS interrupt was not the source of NMI
            IndicatesCSInterruptWasNotTheSourceOfNMI = 0,
            /// indicates CS interrupt was the source of NMI
            IndicatesCSInterruptWasTheSourceOfNMI = 1
        ],
        /// PSS interrupt was the source of NMI
        PSS_FLG OFFSET(17) NUMBITS(1) [
            /// indicates the PSS interrupt was not the source of NMI
            IndicatesThePSSInterruptWasNotTheSourceOfNMI = 0,
            /// indicates the PSS interrupt was the source of NMI
            IndicatesThePSSInterruptWasTheSourceOfNMI = 1
        ],
        /// PCM interrupt was the source of NMI
        PCM_FLG OFFSET(18) NUMBITS(1) [
            /// indicates the PCM interrupt was not the source of NMI
            IndicatesThePCMInterruptWasNotTheSourceOfNMI = 0,
            /// indicates the PCM interrupt was the source of NMI
            IndicatesThePCMInterruptWasTheSourceOfNMI = 1
        ],
        /// RSTn/NMI pin was the source of NMI
        PIN_FLG OFFSET(19) NUMBITS(1) [
            /// Indicates the RSTn_NMI pin was not the source of NMI
            IndicatesTheRSTn_NMIPinWasNotTheSourceOfNMI = 0,
            /// Indicates the RSTn_NMI pin was the source of NMI
            IndicatesTheRSTn_NMIPinWasTheSourceOfNMI = 1
        ]
    ],
    SYS_WDTRESET_CTL [
        /// WDT timeout reset type
        TIMEOUT OFFSET(0) NUMBITS(1) [
            /// WDT timeout event generates Soft reset
            WDTTimeoutEventGeneratesSoftReset = 0,
            /// WDT timeout event generates Hard reset
            WDTTimeoutEventGeneratesHardReset = 1
        ],
        /// WDT password violation reset type
        VIOLATION OFFSET(1) NUMBITS(1) [
            /// WDT password violation event generates Soft reset
            WDTPasswordViolationEventGeneratesSoftReset = 0,
            /// WDT password violation event generates Hard reset
            WDTPasswordViolationEventGeneratesHardReset = 1
        ]
    ],
    SYS_PERIHALT_CTL [
        /// Freezes IP operation when CPU is halted
        HALT_T16_0 OFFSET(0) NUMBITS(1) [
            /// IP operation unaffected when CPU is halted
            IPOperationUnaffectedWhenCPUIsHalted = 0,
            /// freezes IP operation when CPU is halted
            FreezesIPOperationWhenCPUIsHalted = 1
        ],
        /// Freezes IP operation when CPU is halted
        HALT_T16_1 OFFSET(1) NUMBITS(1) [
            /// IP operation unaffected when CPU is halted
            IPOperationUnaffectedWhenCPUIsHalted = 0,
            /// freezes IP operation when CPU is halted
            FreezesIPOperationWhenCPUIsHalted = 1
        ],
        /// Freezes IP operation when CPU is halted
        HALT_T16_2 OFFSET(2) NUMBITS(1) [
            /// IP operation unaffected when CPU is halted
            IPOperationUnaffectedWhenCPUIsHalted = 0,
            /// freezes IP operation when CPU is halted
            FreezesIPOperationWhenCPUIsHalted = 1
        ],
        /// Freezes IP operation when CPU is halted
        HALT_T16_3 OFFSET(3) NUMBITS(1) [
            /// IP operation unaffected when CPU is halted
            IPOperationUnaffectedWhenCPUIsHalted = 0,
            /// freezes IP operation when CPU is halted
            FreezesIPOperationWhenCPUIsHalted = 1
        ],
        /// Freezes IP operation when CPU is halted
        HALT_T32_0 OFFSET(4) NUMBITS(1) [
            /// IP operation unaffected when CPU is halted
            IPOperationUnaffectedWhenCPUIsHalted = 0,
            /// freezes IP operation when CPU is halted
            FreezesIPOperationWhenCPUIsHalted = 1
        ],
        /// Freezes IP operation when CPU is halted
        HALT_eUA0 OFFSET(5) NUMBITS(1) [
            /// IP operation unaffected when CPU is halted
            IPOperationUnaffectedWhenCPUIsHalted = 0,
            /// freezes IP operation when CPU is halted
            FreezesIPOperationWhenCPUIsHalted = 1
        ],
        /// Freezes IP operation when CPU is halted
        HALT_eUA1 OFFSET(6) NUMBITS(1) [
            /// IP operation unaffected when CPU is halted
            IPOperationUnaffectedWhenCPUIsHalted = 0,
            /// freezes IP operation when CPU is halted
            FreezesIPOperationWhenCPUIsHalted = 1
        ],
        /// Freezes IP operation when CPU is halted
        HALT_eUA2 OFFSET(7) NUMBITS(1) [
            /// IP operation unaffected when CPU is halted
            IPOperationUnaffectedWhenCPUIsHalted = 0,
            /// freezes IP operation when CPU is halted
            FreezesIPOperationWhenCPUIsHalted = 1
        ],
        /// Freezes IP operation when CPU is halted
        HALT_eUA3 OFFSET(8) NUMBITS(1) [
            /// IP operation unaffected when CPU is halted
            IPOperationUnaffectedWhenCPUIsHalted = 0,
            /// freezes IP operation when CPU is halted
            FreezesIPOperationWhenCPUIsHalted = 1
        ],
        /// Freezes IP operation when CPU is halted
        HALT_eUB0 OFFSET(9) NUMBITS(1) [
            /// IP operation unaffected when CPU is halted
            IPOperationUnaffectedWhenCPUIsHalted = 0,
            /// freezes IP operation when CPU is halted
            FreezesIPOperationWhenCPUIsHalted = 1
        ],
        /// Freezes IP operation when CPU is halted
        HALT_eUB1 OFFSET(10) NUMBITS(1) [
            /// IP operation unaffected when CPU is halted
            IPOperationUnaffectedWhenCPUIsHalted = 0,
            /// freezes IP operation when CPU is halted
            FreezesIPOperationWhenCPUIsHalted = 1
        ],
        /// Freezes IP operation when CPU is halted
        HALT_eUB2 OFFSET(11) NUMBITS(1) [
            /// IP operation unaffected when CPU is halted
            IPOperationUnaffectedWhenCPUIsHalted = 0,
            /// freezes IP operation when CPU is halted
            FreezesIPOperationWhenCPUIsHalted = 1
        ],
        /// Freezes IP operation when CPU is halted
        HALT_eUB3 OFFSET(12) NUMBITS(1) [
            /// IP operation unaffected when CPU is halted
            IPOperationUnaffectedWhenCPUIsHalted = 0,
            /// freezes IP operation when CPU is halted
            FreezesIPOperationWhenCPUIsHalted = 1
        ],
        /// Freezes IP operation when CPU is halted
        HALT_ADC OFFSET(13) NUMBITS(1) [
            /// IP operation unaffected when CPU is halted
            IPOperationUnaffectedWhenCPUIsHalted = 0,
            /// freezes IP operation when CPU is halted
            FreezesIPOperationWhenCPUIsHalted = 1
        ],
        /// Freezes IP operation when CPU is halted
        HALT_WDT OFFSET(14) NUMBITS(1) [
            /// IP operation unaffected when CPU is halted
            IPOperationUnaffectedWhenCPUIsHalted = 0,
            /// freezes IP operation when CPU is halted
            FreezesIPOperationWhenCPUIsHalted = 1
        ],
        /// Freezes IP operation when CPU is halted
        HALT_DMA OFFSET(15) NUMBITS(1) [
            /// IP operation unaffected when CPU is halted
            IPOperationUnaffectedWhenCPUIsHalted = 0,
            /// freezes IP operation when CPU is halted
            FreezesIPOperationWhenCPUIsHalted = 1
        ]
    ],
    SYS_SRAM_BANKEN [
        /// SRAM Bank0 enable
        BNK0_EN OFFSET(0) NUMBITS(1) [],
        /// SRAM Bank1 enable
        BNK1_EN OFFSET(1) NUMBITS(1) [
            /// Disables Bank1 of the SRAM
            DisablesBank1OfTheSRAM = 0,
            /// Enables Bank1 of the SRAM
            EnablesBank1OfTheSRAM = 1
        ],
        /// SRAM Bank1 enable
        BNK2_EN OFFSET(2) NUMBITS(1) [
            /// Disables Bank2 of the SRAM
            DisablesBank2OfTheSRAM = 0,
            /// Enables Bank2 of the SRAM
            EnablesBank2OfTheSRAM = 1
        ],
        /// SRAM Bank1 enable
        BNK3_EN OFFSET(3) NUMBITS(1) [
            /// Disables Bank3 of the SRAM
            DisablesBank3OfTheSRAM = 0,
            /// Enables Bank3 of the SRAM
            EnablesBank3OfTheSRAM = 1
        ],
        /// SRAM Bank1 enable
        BNK4_EN OFFSET(4) NUMBITS(1) [
            /// Disables Bank4 of the SRAM
            DisablesBank4OfTheSRAM = 0,
            /// Enables Bank4 of the SRAM
            EnablesBank4OfTheSRAM = 1
        ],
        /// SRAM Bank1 enable
        BNK5_EN OFFSET(5) NUMBITS(1) [
            /// Disables Bank5 of the SRAM
            DisablesBank5OfTheSRAM = 0,
            /// Enables Bank5 of the SRAM
            EnablesBank5OfTheSRAM = 1
        ],
        /// SRAM Bank1 enable
        BNK6_EN OFFSET(6) NUMBITS(1) [
            /// Disables Bank6 of the SRAM
            DisablesBank6OfTheSRAM = 0,
            /// Enables Bank6 of the SRAM
            EnablesBank6OfTheSRAM = 1
        ],
        /// SRAM Bank1 enable
        BNK7_EN OFFSET(7) NUMBITS(1) [
            /// Disables Bank7 of the SRAM
            DisablesBank7OfTheSRAM = 0,
            /// Enables Bank7 of the SRAM
            EnablesBank7OfTheSRAM = 1
        ],
        /// SRAM ready
        SRAM_RDY OFFSET(16) NUMBITS(1) [
            /// SRAM is not ready for accesses. Banks are undergoing an enable or disable sequen
            SRAM_RDY_0 = 0,
            /// SRAM is ready for accesses. All SRAM banks are enabled/disabled according to val
            SRAM_RDY_1 = 1
        ]
    ],
    SYS_SRAM_BANKRET [
        /// Bank0 retention
        BNK0_RET OFFSET(0) NUMBITS(1) [],
        /// Bank1 retention
        BNK1_RET OFFSET(1) NUMBITS(1) [
            /// Bank1 of the SRAM is not retained in LPM3 or LPM4
            Bank1OfTheSRAMIsNotRetainedInLPM3OrLPM4 = 0,
            /// Bank1 of the SRAM is retained in LPM3 and LPM4
            Bank1OfTheSRAMIsRetainedInLPM3AndLPM4 = 1
        ],
        /// Bank2 retention
        BNK2_RET OFFSET(2) NUMBITS(1) [
            /// Bank2 of the SRAM is not retained in LPM3 or LPM4
            Bank2OfTheSRAMIsNotRetainedInLPM3OrLPM4 = 0,
            /// Bank2 of the SRAM is retained in LPM3 and LPM4
            Bank2OfTheSRAMIsRetainedInLPM3AndLPM4 = 1
        ],
        /// Bank3 retention
        BNK3_RET OFFSET(3) NUMBITS(1) [
            /// Bank3 of the SRAM is not retained in LPM3 or LPM4
            Bank3OfTheSRAMIsNotRetainedInLPM3OrLPM4 = 0,
            /// Bank3 of the SRAM is retained in LPM3 and LPM4
            Bank3OfTheSRAMIsRetainedInLPM3AndLPM4 = 1
        ],
        /// Bank4 retention
        BNK4_RET OFFSET(4) NUMBITS(1) [
            /// Bank4 of the SRAM is not retained in LPM3 or LPM4
            Bank4OfTheSRAMIsNotRetainedInLPM3OrLPM4 = 0,
            /// Bank4 of the SRAM is retained in LPM3 and LPM4
            Bank4OfTheSRAMIsRetainedInLPM3AndLPM4 = 1
        ],
        /// Bank5 retention
        BNK5_RET OFFSET(5) NUMBITS(1) [
            /// Bank5 of the SRAM is not retained in LPM3 or LPM4
            Bank5OfTheSRAMIsNotRetainedInLPM3OrLPM4 = 0,
            /// Bank5 of the SRAM is retained in LPM3 and LPM4
            Bank5OfTheSRAMIsRetainedInLPM3AndLPM4 = 1
        ],
        /// Bank6 retention
        BNK6_RET OFFSET(6) NUMBITS(1) [
            /// Bank6 of the SRAM is not retained in LPM3 or LPM4
            Bank6OfTheSRAMIsNotRetainedInLPM3OrLPM4 = 0,
            /// Bank6 of the SRAM is retained in LPM3 and LPM4
            Bank6OfTheSRAMIsRetainedInLPM3AndLPM4 = 1
        ],
        /// Bank7 retention
        BNK7_RET OFFSET(7) NUMBITS(1) [
            /// Bank7 of the SRAM is not retained in LPM3 or LPM4
            Bank7OfTheSRAMIsNotRetainedInLPM3OrLPM4 = 0,
            /// Bank7 of the SRAM is retained in LPM3 and LPM4
            Bank7OfTheSRAMIsRetainedInLPM3AndLPM4 = 1
        ],
        /// SRAM ready
        SRAM_RDY OFFSET(16) NUMBITS(1) [
            /// SRAM banks are being set up for retention. Entry into LPM3, LPM4 should not be a
            SRAM_RDY_0 = 0,
            /// SRAM is ready for accesses. All SRAM banks are enabled/disabled for retention ac
            SRAM_RDY_1 = 1
        ]
    ],
    SYS_RESET_REQ [
        /// Generate POR
        POR OFFSET(0) NUMBITS(1) [],
        /// Generate Reboot_Reset
        REBOOT OFFSET(1) NUMBITS(1) [],
        /// Write key
        WKEY OFFSET(8) NUMBITS(8) []
    ],
    SYS_RESET_STATOVER [
        /// Indicates if SOFT Reset is active
        SOFT OFFSET(0) NUMBITS(1) [],
        /// Indicates if HARD Reset is active
        HARD OFFSET(1) NUMBITS(1) [],
        /// Indicates if Reboot Reset is active
        REBOOT OFFSET(2) NUMBITS(1) [],
        /// SOFT_Reset overwrite request
        SOFT_OVER OFFSET(8) NUMBITS(1) [],
        /// HARD_Reset overwrite request
        HARD_OVER OFFSET(9) NUMBITS(1) [],
        /// Reboot Reset overwrite request
        RBT_OVER OFFSET(10) NUMBITS(1) []
    ],
    SYS_SYSTEM_STAT [
        /// Debug Security active
        DBG_SEC_ACT OFFSET(3) NUMBITS(1) [],
        /// Indicates if JTAG and SWD Lock is active
        JTAG_SWD_LOCK_ACT OFFSET(4) NUMBITS(1) [],
        /// Indicates if IP protection is active
        IP_PROT_ACT OFFSET(5) NUMBITS(1) []
    ]
];

pub struct SysCtl {
    registers: StaticRef<SysCtlRegisters>,
}

impl SysCtl {
    const fn new() -> SysCtl {
        SysCtl {
            registers: SYSCTL_BASE,
        }
    }

    pub fn enable_all_sram_banks(&self) {
        self.registers
            .sram_banken
            .modify(SYS_SRAM_BANKEN::BNK7_EN::SET);
    }
}
