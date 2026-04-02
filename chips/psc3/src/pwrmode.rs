// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Infineon Technologies AG 2026.

//! Power mode control interface

use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    /// SRSS Power Mode Control Registers
    PwrmodeRegisters {
        /// Dependency Sense Register
        (0x000 => pd_0_pd_sense: ReadWrite<u32, PD_SENSE::Register>),
        /// Dependency Support Register
        (0x004 => pd_0_pd_spt: ReadWrite<u32, PD_SPT::Register>),
        (0x008 => _reserved0),
        (0x010 => pd_1_pd_sense: ReadWrite<u32, PD_SENSE::Register>),
        (0x014 => pd_1_pd_spt: ReadWrite<u32, PD_SPT::Register>),
        (0x018 => _reserved1),
        (0x020 => pd_2_pd_sense: ReadWrite<u32, PD_SENSE::Register>),
        (0x024 => pd_2_pd_spt: ReadWrite<u32, PD_SPT::Register>),
        (0x028 => _reserved2),
        (0x030 => pd_3_pd_sense: ReadWrite<u32, PD_SENSE::Register>),
        (0x034 => pd_3_pd_spt: ReadWrite<u32, PD_SPT::Register>),
        (0x038 => _reserved3),
        (0x040 => pd_4_pd_sense: ReadWrite<u32, PD_SENSE::Register>),
        (0x044 => pd_4_pd_spt: ReadWrite<u32, PD_SPT::Register>),
        (0x048 => _reserved4),
        (0x050 => pd_5_pd_sense: ReadWrite<u32, PD_SENSE::Register>),
        (0x054 => pd_5_pd_spt: ReadWrite<u32, PD_SPT::Register>),
        (0x058 => _reserved5),
        (0x060 => pd_6_pd_sense: ReadWrite<u32, PD_SENSE::Register>),
        (0x064 => pd_6_pd_spt: ReadWrite<u32, PD_SPT::Register>),
        (0x068 => _reserved6),
        (0x070 => pd_7_pd_sense: ReadWrite<u32, PD_SENSE::Register>),
        (0x074 => pd_7_pd_spt: ReadWrite<u32, PD_SPT::Register>),
        (0x078 => _reserved7),
        (0x080 => pd_8_pd_sense: ReadWrite<u32, PD_SENSE::Register>),
        (0x084 => pd_8_pd_spt: ReadWrite<u32, PD_SPT::Register>),
        (0x088 => _reserved8),
        (0x090 => pd_9_pd_sense: ReadWrite<u32, PD_SENSE::Register>),
        (0x094 => pd_9_pd_spt: ReadWrite<u32, PD_SPT::Register>),
        (0x098 => _reserved9),
        (0x0A0 => pd_10_pd_sense: ReadWrite<u32, PD_SENSE::Register>),
        (0x0A4 => pd_10_pd_spt: ReadWrite<u32, PD_SPT::Register>),
        (0x0A8 => _reserved10),
        (0x0B0 => pd_11_pd_sense: ReadWrite<u32, PD_SENSE::Register>),
        (0x0B4 => pd_11_pd_spt: ReadWrite<u32, PD_SPT::Register>),
        (0x0B8 => _reserved11),
        (0x0C0 => pd_12_pd_sense: ReadWrite<u32, PD_SENSE::Register>),
        (0x0C4 => pd_12_pd_spt: ReadWrite<u32, PD_SPT::Register>),
        (0x0C8 => _reserved12),
        (0x0D0 => pd_13_pd_sense: ReadWrite<u32, PD_SENSE::Register>),
        (0x0D4 => pd_13_pd_spt: ReadWrite<u32, PD_SPT::Register>),
        (0x0D8 => _reserved13),
        (0x0E0 => pd_14_pd_sense: ReadWrite<u32, PD_SENSE::Register>),
        (0x0E4 => pd_14_pd_spt: ReadWrite<u32, PD_SPT::Register>),
        (0x0E8 => _reserved14),
        (0x0F0 => pd_15_pd_sense: ReadWrite<u32, PD_SENSE::Register>),
        (0x0F4 => pd_15_pd_spt: ReadWrite<u32, PD_SPT::Register>),
        (0x0F8 => _reserved15),

        // --- PPU Main Registers ---
        /// Power Policy Register
        (0x1000 => ppu_main_pwpr: ReadWrite<u32, PPU_PWPR::Register>),
        /// Power Mode Emulation Register
        (0x1004 => ppu_main_pmer: ReadWrite<u32, PPU_PMER::Register>),
        /// Power Status Register
        (0x1008 => ppu_main_pwsr: ReadOnly<u32, PPU_PWSR::Register>),
        (0x100C => _reserved_main0),
        /// Device Interface Input Current Status Register
        (0x1010 => ppu_main_disr: ReadOnly<u32, PPU_DISR::Register>),
        /// Miscellaneous Input Current Status Register
        (0x1014 => ppu_main_misr: ReadOnly<u32, PPU_MISR::Register>),
        /// Stored Status Register
        (0x1018 => ppu_main_stsr: ReadOnly<u32, PPU_STSR::Register>),
        /// Unlock register
        (0x101C => ppu_main_unlk: ReadWrite<u32, PPU_UNLK::Register>),
        /// Power Configuration Register
        (0x1020 => ppu_main_pwcr: ReadWrite<u32, PPU_PWCR::Register>),
        /// Power Mode Transition Configuration Register
        (0x1024 => ppu_main_ptcr: ReadWrite<u32, PPU_PTCR::Register>),
        (0x1028 => _reserved_main1),
        /// Interrupt Mask Register
        (0x1030 => ppu_main_imr: ReadWrite<u32, PPU_IMR::Register>),
        /// Additional Interrupt Mask Register
        (0x1034 => ppu_main_aimr: ReadWrite<u32, PPU_AIMR::Register>),
        /// Interrupt Status Register
        (0x1038 => ppu_main_isr: ReadWrite<u32, PPU_ISR::Register>),
        /// Additional Interrupt Status Register
        (0x103C => ppu_main_aisr: ReadWrite<u32, PPU_AISR::Register>),
        /// Input Edge Sensitivity Register
        (0x1040 => ppu_main_iesr: ReadWrite<u32, PPU_IESR::Register>),
        /// Operating Mode Active Edge Sensitivity Register
        (0x1044 => ppu_main_opsr: ReadWrite<u32, PPU_OPSR::Register>),
        (0x1048 => _reserved_main2),
        /// Functional Retention RAM Configuration Register
        (0x1050 => ppu_main_funrr: ReadWrite<u32, PPU_FUNRR::Register>),
        /// Full Retention RAM Configuration Register
        (0x1054 => ppu_main_fulrr: ReadWrite<u32, PPU_FULRR::Register>),
        /// Memory Retention RAM Configuration Register
        (0x1058 => ppu_main_memrr: ReadWrite<u32, PPU_MEMRR::Register>),
        (0x105C => _reserved_main3),
        /// Power Mode Entry Delay Register 0
        (0x1160 => ppu_main_edtr0: ReadWrite<u32, PPU_EDTR0::Register>),
        /// Power Mode Entry Delay Register 1
        (0x1164 => ppu_main_edtr1: ReadWrite<u32, PPU_EDTR1::Register>),
        (0x1168 => _reserved_main4),
        /// Device Control Delay Configuration Register 0
        (0x1170 => ppu_main_dcdr0: ReadWrite<u32, PPU_DCDR0::Register>),
        /// Device Control Delay Configuration Register 1
        (0x1174 => ppu_main_dcdr1: ReadWrite<u32, PPU_DCDR1::Register>),
        (0x1178 => _reserved_main5),
        /// PPU Identification Register 0
        (0x1FB0 => ppu_main_idr0: ReadOnly<u32, PPU_IDR0::Register>),
        /// PPU Identification Register 1
        (0x1FB4 => ppu_main_idr1: ReadOnly<u32, PPU_IDR1::Register>),
        (0x1FB8 => _reserved_main6),
        /// Implementation Identification Register
        (0x1FC8 => ppu_main_iidr: ReadOnly<u32, PPU_IIDR::Register>),
        /// Architecture Identification Register
        (0x1FCC => ppu_main_aidr: ReadOnly<u32, PPU_AIDR::Register>),
        (0x1FD0 => _reserved16),

        // --- Clock Selection ---
        /// Clock Selection for Power Mode Components
        (0x2000 => clk_select: ReadWrite<u32, CLK_SELECT::Register>),

        (0x2004 => @END),
    }
}
register_bitfields![u32,
CLK_SELECT [
    /// clk_pwr is generated by dividing the CLK_PWR_MUX selection by (CLK_PWR_DIV+1).
    CLK_PWR_DIV OFFSET(0) NUMBITS(8) [],
    /// Selects a source for the clock used by power control components.  Note that not all products
    /// support all clock sources.  Selecting a clock source that is not supported will result in
    /// undefined behavior.  It takes four cycles of the originally selected clock to switch away
    /// from it.  Do not disable the original clock during this time.
    CLK_PWR_MUX OFFSET(16) NUMBITS(2) [
        /// IMO - Internal R/C Oscillator
        IMOInternalRCOscillator = 0,
        /// IHO - Internal High-speed Oscillator
        IHOInternalHighSpeedOscillator = 1,
        NA = 2
    ]
],
PD_SENSE [
    /// Each bit <i> indicates whether PD<j> is directly kept on when PD<i> is on.  Indirect
    /// dependency is still possible if multiple direct dependencies work together to create a
    /// transitive relationship.  For example, if PD1 depends upon PD2; and PD2 dpends upon PD3;
    /// then PD1 indirectly depends upon PD3 regardless of whether there is a direct dependency from
    /// PD3 to PD1.  Some bits are implemented as constants, and some bits are implemented as
    /// user-configurable registers.  Refer to PD_SPT register to see how each bit is implemented.
    PD_ON OFFSET(0) NUMBITS(16) []
],
PD_SPT [
    /// Each bit <i> indicates whether PD<j> is always kept on when PD<i> is on for sense bits that
    /// are not configurable.  For configurable bits, this indicates the reset value of the
    /// configurable bit.
    PD_FORCE_ON OFFSET(0) NUMBITS(16) [],
    /// Each bit <i> indicates whether PD<j> can be configured on when PD<i> is on.
    PD_CONFIG_ON OFFSET(16) NUMBITS(16) []
],
PPU_PWPR [
    /// Power mode policy.  When static power mode transitions are enabled, PWR_DYN_EN is set to 0,
    /// this is the target power mode for the PPU.  When dynamic power mode transitions are enabled,
    /// PWR_DYN_EN is set to 1, this is the minimum power mode for the PPU.
    ///
    /// This PPU supports the following modes: OFF(0), MEM_RET(2), FULL_RET(5), ON(8).  Do not use
    /// WARM_RST(9) or other unsupported modes.
    PWR_POLICY OFFSET(0) NUMBITS(4) [
        Off = 0,
        MemoryRetention = 2,
        FullRetention = 5,
        On = 8
    ],
    /// Power mode dynamic transition enable.  For main PPU, keep this bit 1.
    PWR_DYN_EN OFFSET(8) NUMBITS(1) [],
    LOCK_EN OFFSET(12) NUMBITS(1) [],
    OP_POLICY OFFSET(16) NUMBITS(4) [],
    OP_DYN_EN OFFSET(24) NUMBITS(1) []
],
PPU_PMER [
    EMU_EN OFFSET(0) NUMBITS(1) []
],
PPU_PWSR [
    /// Power mode status.  These bits reflect the current power mode of the PPU.  See
    /// PPU_PWPR.PWR_POLICY for power mode enumeration.
    PWR_STATUS OFFSET(0) NUMBITS(4) [
        Off = 0,
        MemoryRetention = 2,
        FullRetention = 5,
        On = 8
    ],
    /// Power mode dynamic transition status.  When set to 1 power mode dynamic transitions are
    /// enabled.  There might be a delay in dynamic transitions becoming active or inactive if the
    /// PPU is transitioning when PWR_DYN_EN is programmed.
    PWR_DYN_STATUS OFFSET(8) NUMBITS(1) [],
    LOCK_STATUS OFFSET(12) NUMBITS(1) [],
    OP_STATUS OFFSET(16) NUMBITS(4) [],
    OP_DYN_STATUS OFFSET(24) NUMBITS(1) []
],
PPU_DISR [
    /// Status of the power mode DEVACTIVE inputs.
    ///
    /// There is one bit for each device interface Q-Channel DEVQACTIVE.  For example, bit 0 is for
    /// Q-channel device 0 DEVQACTIVE.  Refer to PPU_IDR0.DEVCHAN for device channel enumeration.
    PWR_DEVACTIVE_STATUS OFFSET(0) NUMBITS(11) [],
    OP_DEVACTIVE_STATUS OFFSET(24) NUMBITS(8) []
],
PPU_MISR [
    /// The status of the PCSMPACCEPT input.
    PCSMPACCEPT_STATUS OFFSET(0) NUMBITS(1) [],
    /// Status of the device interface DEVACCEPT inputs.
    ///
    /// There is one bit for each device interface DEVQACCEPTn.  For example, bit 8 is for Q-Channel
    /// 0 DEVQACCEPTn and bit 9 for Q-Channel 1 DEVQACCEPTn.   Refer to PPU_IDR0.DEVCHAN for device
    /// channel enumeration.
    DEVACCEPT_STATUS OFFSET(8) NUMBITS(8) [],
    /// Status of the device interface DEVDENY inputs.
    ///
    /// There is one bit for each device interface DEVQDENY.  For example, bit 16 is for Q-Channel 0
    /// DEVQDENY, and bit 17 for Q-Channel 1 DEVQDENY.  Refer to PPU_IDR0.DEVCHAN for device channel
    /// enumeration.
    DEVDENY_STATUS OFFSET(16) NUMBITS(8) []
],
PPU_STSR [
    /// Status of the DEVDENY signals from the last device interface Q-Channel transition.  For
    /// Q-Channel:  There is one bit for each device interface DEVQDENY.  For example, bit 0 is for
    /// Q-Channel 0 DEVQDENY, and bit 1 for Q-Channel 1 DEVQDENY.  Refer to
    /// PPU_DISR.PWR_DEVACTIVE_STATUS for device enumeration.
    STORED_DEVDENY OFFSET(0) NUMBITS(8) []
],
PPU_UNLK [
    UNLOCK OFFSET(0) NUMBITS(1) []
],
PPU_PWCR [
    /// When set to 1 enables the device interface handshake for transitions.  All available bits
    /// are reset to 1.
    ///
    /// There is one bit for each device interface channel.  For example, bit 0 is for Q-Channel 0,
    /// and bit 1 is for Q-Channel 1.  Refer to PPU_IDR0.DEVCHAN for device channel enumeration.
    DEVREQEN OFFSET(0) NUMBITS(8) [],
    /// These bits enable the power mode DEVACTIVE inputs.  When a bit is to 1 the related DEVACTIVE
    /// input is enabled, when set to 0 it is disabled.  All available bits are reset to 1.
    ///
    /// There is one bit for each device interface Q-Channel DEVQACTIVE.  For example, bit 8 is for
    /// the Q-Channel 0 DEVQACTIVE, and bit 9 for the Q-Channel 1 DEVQACTIVE.  Refer to
    /// PPU_IDR0.DEVCHAN for device channel enumeration.
    PWR_DEVACTIVEEN OFFSET(8) NUMBITS(11) [],
    OP_DEVACTIVEEN OFFSET(24) NUMBITS(8) []
],
PPU_PTCR [
    /// Transition behavior between ON and WARM_RST.  This bit should not be modified when the PPU
    /// is in WARM_RST, or if the PPU is performing a transition, otherwise PPU behavior is
    /// UNPREDICTABLE.  0:  The PPU does not perform a device interface handshake when transitioning
    /// between ON and WARM_RST.  1:  The PPU performs a device interface handshake when
    /// transitioning between ON and WARM_RST.  This disables all Q-Channels for this transition.
    WARM_RST_DEVREQEN OFFSET(0) NUMBITS(1) [],
    DBG_RECOV_PORST_EN OFFSET(1) NUMBITS(1) []
],
PPU_IMR [
    /// Static full policy transition completion event mask.  For main PPU, this bit has no function
    /// because no static transitions are supported (see PWPR.PWR_DYN_EN).
    STA_POLICY_TRN_IRQ_MASK OFFSET(0) NUMBITS(1) [],
    /// Static transition acceptance event mask.  For main PPU, keep this bit 1 to mask the event,
    /// otherwise the interrupt may trigger a wakeup.
    STA_ACCEPT_IRQ_MASK OFFSET(1) NUMBITS(1) [],
    /// Static transition denial event mask.
    STA_DENY_IRQ_MASK OFFSET(2) NUMBITS(1) [],
    EMU_ACCEPT_IRQ_MASK OFFSET(3) NUMBITS(1) [],
    EMU_DENY_IRQ_MASK OFFSET(4) NUMBITS(1) [],
    LOCKED_IRQ_MASK OFFSET(5) NUMBITS(1) []
],
PPU_AIMR [
    /// Unsupported Policy event mask.
    UNSPT_POLICY_IRQ_MASK OFFSET(0) NUMBITS(1) [],
    /// Dynamic transition acceptance event mask.  For main PPU, keep this bit 1 to mask the event,
    /// otherwise the interrupt that occurs when entering a low power mode may trigger a wakeup.
    DYN_ACCEPT_IRQ_MASK OFFSET(1) NUMBITS(1) [],
    /// Dynamic transition denial event mask.
    DYN_DENY_IRQ_MASK OFFSET(2) NUMBITS(1) [],
    STA_POLICY_PWR_IRQ_MASK OFFSET(3) NUMBITS(1) [],
    STA_POLICY_OP_IRQ_MASK OFFSET(4) NUMBITS(1) []
],
PPU_ISR [
    /// Static full policy transition completion event status.
    STA_POLICY_TRN_IRQ OFFSET(0) NUMBITS(1) [],
    /// Static transition acceptance event status.
    STA_ACCEPT_IRQ OFFSET(1) NUMBITS(1) [],
    /// Static transition denial event status.
    STA_DENY_IRQ OFFSET(2) NUMBITS(1) [],
    EMU_ACCEPT_IRQ OFFSET(3) NUMBITS(1) [],
    EMU_DENY_IRQ OFFSET(4) NUMBITS(1) [],
    LOCKED_IRQ OFFSET(5) NUMBITS(1) [],
    /// Indicates there is an interrupt event pending in the Additional Interrupt Status Register
    /// (PPU_AISR).
    OTHER_IRQ OFFSET(7) NUMBITS(1) [],
    PWR_ACTIVE_EDGE_IRQ OFFSET(8) NUMBITS(11) [],
    OP_ACTIVE_EDGE_IRQ OFFSET(24) NUMBITS(8) []
],
PPU_AISR [
    /// Unsupported Policy event status.
    UNSPT_POLICY_IRQ OFFSET(0) NUMBITS(1) [],
    /// Dynamic transition acceptance event status.
    DYN_ACCEPT_IRQ OFFSET(1) NUMBITS(1) [],
    /// Dynamic transition denial event status.
    DYN_DENY_IRQ OFFSET(2) NUMBITS(1) [],
    STA_POLICY_PWR_IRQ OFFSET(3) NUMBITS(1) [],
    STA_POLICY_OP_IRQ OFFSET(4) NUMBITS(1) []
],
PPU_IESR [
    /// DEVACTIVE 0 edge sensitivity.
    DEVACTIVE00_EDGE OFFSET(0) NUMBITS(2) [
        Disabled = 0,
        RisingEdge = 1,
        FallingEdge = 2,
        BothEdges = 3
    ],
    /// DEVACTIVE 1 edge sensitivity.
    DEVACTIVE01_EDGE OFFSET(2) NUMBITS(2) [
        Disabled = 0,
        RisingEdge = 1,
        FallingEdge = 2,
        BothEdges = 3
    ],
    /// DEVACTIVE 2 edge sensitivity.
    DEVACTIVE02_EDGE OFFSET(4) NUMBITS(2) [
        Disabled = 0,
        RisingEdge = 1,
        FallingEdge = 2,
        BothEdges = 3
    ],
    DEVACTIVE03_EDGE OFFSET(6) NUMBITS(2) [
        Disabled = 0,
        RisingEdge = 1,
        FallingEdge = 2,
        BothEdges = 3
    ],
    DEVACTIVE04_EDGE OFFSET(8) NUMBITS(2) [
        Disabled = 0,
        RisingEdge = 1,
        FallingEdge = 2,
        BothEdges = 3
    ],
    DEVACTIVE05_EDGE OFFSET(10) NUMBITS(2) [
        Disabled = 0,
        RisingEdge = 1,
        FallingEdge = 2,
        BothEdges = 3
    ],
    DEVACTIVE06_EDGE OFFSET(12) NUMBITS(2) [
        Disabled = 0,
        RisingEdge = 1,
        FallingEdge = 2,
        BothEdges = 3
    ],
    DEVACTIVE07_EDGE OFFSET(14) NUMBITS(2) [
        Disabled = 0,
        RisingEdge = 1,
        FallingEdge = 2,
        BothEdges = 3
    ],
    DEVACTIVE08_EDGE OFFSET(16) NUMBITS(2) [
        Disabled = 0,
        RisingEdge = 1,
        FallingEdge = 2,
        BothEdges = 3
    ],
    DEVACTIVE09_EDGE OFFSET(18) NUMBITS(2) [
        Disabled = 0,
        RisingEdge = 1,
        FallingEdge = 2,
        BothEdges = 3
    ],
    DEVACTIVE10_EDGE OFFSET(20) NUMBITS(2) [
        Disabled = 0,
        RisingEdge = 1,
        FallingEdge = 2,
        BothEdges = 3
    ]
],
PPU_OPSR [
    DEVACTIVE16_EDGE OFFSET(0) NUMBITS(2) [
        Disabled = 0,
        RisingEdge = 1,
        FallingEdge = 2,
        BothEdges = 3
    ],
    DEVACTIVE17_EDGE OFFSET(2) NUMBITS(2) [
        Disabled = 0,
        RisingEdge = 1,
        FallingEdge = 2,
        BothEdges = 3
    ],
    DEVACTIVE18_EDGE OFFSET(4) NUMBITS(2) [
        Disabled = 0,
        RisingEdge = 1,
        FallingEdge = 2,
        BothEdges = 3
    ],
    DEVACTIVE19_EDGE OFFSET(6) NUMBITS(2) [
        Disabled = 0,
        RisingEdge = 1,
        FallingEdge = 2,
        BothEdges = 3
    ],
    DEVACTIVE20_EDGE OFFSET(8) NUMBITS(2) [
        Disabled = 0,
        RisingEdge = 1,
        FallingEdge = 2,
        BothEdges = 3
    ],
    DEVACTIVE21_EDGE OFFSET(10) NUMBITS(2) [
        Disabled = 0,
        RisingEdge = 1,
        FallingEdge = 2,
        BothEdges = 3
    ],
    DEVACTIVE22_EDGE OFFSET(12) NUMBITS(2) [
        Disabled = 0,
        RisingEdge = 1,
        FallingEdge = 2,
        BothEdges = 3
    ],
    DEVACTIVE23_EDGE OFFSET(14) NUMBITS(2) [
        Disabled = 0,
        RisingEdge = 1,
        FallingEdge = 2,
        BothEdges = 3
    ]
],
PPU_FUNRR [
    FUNC_RET_RAM_CFG OFFSET(0) NUMBITS(8) []
],
PPU_FULRR [
    FULL_RET_RAM_CFG OFFSET(0) NUMBITS(8) []
],
PPU_MEMRR [
    MEM_RET_RAM_CFG OFFSET(0) NUMBITS(8) []
],
PPU_EDTR0 [
    OFF_DEL OFFSET(0) NUMBITS(8) [],
    MEM_RET_DEL OFFSET(8) NUMBITS(8) [],
    LOGIC_RET_DEL OFFSET(16) NUMBITS(8) [],
    FULL_RET_DEL OFFSET(24) NUMBITS(8) []
],
PPU_EDTR1 [
    MEM_OFF_DEL OFFSET(0) NUMBITS(8) [],
    FUNC_RET_DEL OFFSET(8) NUMBITS(8) []
],
PPU_DCDR0 [
    CLKEN_RST_DLY OFFSET(0) NUMBITS(8) [],
    ISO_CLKEN_DLY OFFSET(8) NUMBITS(8) [],
    RST_HWSTAT_DLY OFFSET(16) NUMBITS(8) []
],
PPU_DCDR1 [
    ISO_RST_DLY OFFSET(0) NUMBITS(8) [],
    CLKEN_ISO_DLY OFFSET(8) NUMBITS(8) []
],
PPU_IDR0 [
    /// No. of Device Interface Channels.  0: This is a P-Channel PPU.  Refer to PPU_IDR1.OP_ACTIVE
    /// for the number of DEVPACTIVE inputs and their meaning.  non-zero: The value is the number of
    /// Q-Channels.
    ///
    /// The device enumeration is: Device 0: PDCM
    DEVCHAN OFFSET(0) NUMBITS(4) [],
    /// No. of operating modes supported is NUM_OPMODE + 1.
    NUM_OPMODE OFFSET(4) NUMBITS(4) [],
    /// OFF support.
    STA_OFF_SPT OFFSET(8) NUMBITS(1) [],
    /// OFF_EMU support.
    STA_OFF_EMU_SPT OFFSET(9) NUMBITS(1) [],
    /// MEM_RET support.
    STA_MEM_RET_SPT OFFSET(10) NUMBITS(1) [],
    /// MEM_RET_EMU support.
    STA_MEM_RET_EMU_SPT OFFSET(11) NUMBITS(1) [],
    /// LOGIC_RET support.
    STA_LGC_RET_SPT OFFSET(12) NUMBITS(1) [],
    /// MEM_OFF support.
    STA_MEM_OFF_SPT OFFSET(13) NUMBITS(1) [],
    /// FULL_RET support.
    STA_FULL_RET_SPT OFFSET(14) NUMBITS(1) [],
    /// FUNC_RET support.
    STA_FUNC_RET_SPT OFFSET(15) NUMBITS(1) [],
    /// ON support.
    STA_ON_SPT OFFSET(16) NUMBITS(1) [],
    /// WARM_RST support.  Ignore this bit.  Do not use WARM_RST.
    STA_WRM_RST_SPT OFFSET(17) NUMBITS(1) [],
    /// DBG_RECOV support.
    STA_DBG_RECOV_SPT OFFSET(18) NUMBITS(1) [],
    /// Dynamic OFF support.
    DYN_OFF_SPT OFFSET(20) NUMBITS(1) [],
    /// Dynamic OFF_EMU support.
    DYN_OFF_EMU_SPT OFFSET(21) NUMBITS(1) [],
    /// Dynamic MEM_RET support.
    DYN_MEM_RET_SPT OFFSET(22) NUMBITS(1) [],
    /// Dynamic MEM_RET_EMU support
    DYN_MEM_RET_EMU_SPT OFFSET(23) NUMBITS(1) [],
    /// Dynamic LOGIC_RET support.
    DYN_LGC_RET_SPT OFFSET(24) NUMBITS(1) [],
    /// Dynamic MEM_OFF support.
    DYN_MEM_OFF_SPT OFFSET(25) NUMBITS(1) [],
    /// Dynamic FULL_RET support.
    DYN_FULL_RET_SPT OFFSET(26) NUMBITS(1) [],
    /// Dynamic FUNC_RET support.
    DYN_FUNC_RET_SPT OFFSET(27) NUMBITS(1) [],
    /// Dynamic ON support.
    DYN_ON_SPT OFFSET(28) NUMBITS(1) [],
    /// Dynamic WARM_RST support.
    DYN_WRM_RST_SPT OFFSET(29) NUMBITS(1) []
],
PPU_IDR1 [
    /// Power mode entry delay support.
    PWR_MODE_ENTRY_DEL_SPT OFFSET(0) NUMBITS(1) [],
    /// Software device delay control configuration support.
    SW_DEV_DEL_SPT OFFSET(1) NUMBITS(1) [],
    /// Lock and the lock interrupt event are supported.
    LOCK_SPT OFFSET(2) NUMBITS(1) [],
    MEM_RET_RAM_REG OFFSET(4) NUMBITS(1) [],
    FULL_RET_RAM_REG OFFSET(5) NUMBITS(1) [],
    FUNC_RET_RAM_REG OFFSET(6) NUMBITS(1) [],
    /// Power policy transition completion event status.
    STA_POLICY_PWR_IRQ_SPT OFFSET(8) NUMBITS(1) [],
    /// Operating policy transition completion event status.
    STA_POLICY_OP_IRQ_SPT OFFSET(9) NUMBITS(1) [],
    OP_ACTIVE OFFSET(10) NUMBITS(1) [],
    /// OFF to MEM_RET direct transition.  Indicates if direct transitions from OFF to MEM_RET and
    /// from OFF_EMU to MEM_RET_EMU are supported.
    OFF_MEM_RET_TRANS OFFSET(12) NUMBITS(1) []
],
PPU_IIDR [
    /// Implementer identification.  [11:8] The JEP106 continuation code of the implementer.  [7]
    /// Always 0.  [6:0] The JEP106 identity code of the implementer.  For an Arm implementation,
    /// bits [11:0] are 0x43B.
    IMPLEMENTER OFFSET(0) NUMBITS(12) [],
    /// Minor revision of the product.
    REVISION OFFSET(12) NUMBITS(4) [],
    /// Major revision of the product.
    VARIANT OFFSET(16) NUMBITS(4) [],
    /// PPU part identification.
    PRODUCT_ID OFFSET(20) NUMBITS(12) []
],
PPU_AIDR [
    ARCH_REV_MINOR OFFSET(0) NUMBITS(4) [],
    ARCH_REV_MAJOR OFFSET(4) NUMBITS(4) []
]
];

const PWRMODE_BASE: StaticRef<PwrmodeRegisters> =
    unsafe { StaticRef::new(0x42210000 as *const PwrmodeRegisters) };

pub struct PwrMode {
    registers: StaticRef<PwrmodeRegisters>,
}

pub type PwrPolicy = PPU_PWPR::PWR_POLICY::Value;

impl PwrMode {
    pub const fn new() -> PwrMode {
        PwrMode {
            registers: PWRMODE_BASE,
        }
    }

    /// Initializes the PPU
    pub fn ppu_init(&self) {
        self.registers
            .ppu_main_iesr
            .write(PPU_IESR::DEVACTIVE00_EDGE::Disabled); // disable all
        self.registers.ppu_main_imr.write(
            PPU_IMR::STA_POLICY_TRN_IRQ_MASK::SET
                + PPU_IMR::STA_ACCEPT_IRQ_MASK::SET
                + PPU_IMR::STA_DENY_IRQ_MASK::SET
                + PPU_IMR::EMU_ACCEPT_IRQ_MASK::SET
                + PPU_IMR::EMU_DENY_IRQ_MASK::SET
                + PPU_IMR::LOCKED_IRQ_MASK::SET,
        ); // mask accept events to avoid wakeup
        self.registers
            .ppu_main_isr
            .write(PPU_ISR::STA_POLICY_TRN_IRQ::CLEAR);
    }

    /// Enables dynamic power mode transitions with the specified minimum dynamic power mode.
    pub fn ppu_dynamic_enable(&self, min_dyn_state: PwrPolicy) {
        self.registers
            .ppu_main_pwpr
            .modify(PPU_PWPR::PWR_DYN_EN::SET + PPU_PWPR::PWR_POLICY.val(min_dyn_state as u32));

        while !self
            .registers
            .ppu_main_pwsr
            .is_set(PPU_PWSR::PWR_DYN_STATUS)
        {}
    }
}
