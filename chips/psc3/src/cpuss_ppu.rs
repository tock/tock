// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Infineon Technologies AG 2026.

use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    /// Power Policy Unit Registers for CPUSS
    Cpuss_PpuRegisters {
        /// Power Policy Register
        (0x000 => pwpr: ReadWrite<u32, PWPR::Register>),
        /// Power Mode Emulation Register
        (0x004 => pmer: ReadWrite<u32>),
        /// Power Status Register
        (0x008 => pwsr: ReadWrite<u32, PWSR::Register>),
        (0x00C => _reserved0),
        /// Device Interface Input Current Status Register
        (0x010 => disr: ReadWrite<u32, DISR::Register>),
        /// Miscellaneous Input Current Status Register
        (0x014 => misr: ReadWrite<u32, MISR::Register>),
        /// Stored Status Register
        (0x018 => stsr: ReadWrite<u32>),
        /// Unlock register
        (0x01C => unlk: ReadWrite<u32>),
        /// Power Configuration Register
        (0x020 => pwcr: ReadWrite<u32, PWCR::Register>),
        /// Power Mode Transition Configuration Register
        (0x024 => ptcr: ReadWrite<u32, PTCR::Register>),
        (0x028 => _reserved1),
        /// Interrupt Mask Register
        (0x030 => imr: ReadWrite<u32, IMR::Register>),
        /// Additional Interrupt Mask Register
        (0x034 => aimr: ReadWrite<u32, AIMR::Register>),
        /// Interrupt Status Register
        (0x038 => isr: ReadWrite<u32, ISR::Register>),
        /// Additional Interrupt Status Register
        (0x03C => aisr: ReadWrite<u32, AISR::Register>),
        /// Input Edge Sensitivity Register
        (0x040 => iesr: ReadWrite<u32, IESR::Register>),
        /// Operating Mode Active Edge Sensitivity Register
        (0x044 => opsr: ReadWrite<u32, OPSR::Register>),
        (0x048 => _reserved2),
        /// Functional Retention RAM Configuration Register
        (0x050 => funrr: ReadWrite<u32>),
        /// Full Retention RAM Configuration Register
        (0x054 => fulrr: ReadWrite<u32>),
        /// Memory Retention RAM Configuration Register
        (0x058 => memrr: ReadWrite<u32>),
        (0x05C => _reserved3),
        /// Power Mode Entry Delay Register 0
        (0x160 => edtr0: ReadWrite<u32, EDTR0::Register>),
        /// Power Mode Entry Delay Register 1
        (0x164 => edtr1: ReadWrite<u32, EDTR1::Register>),
        (0x168 => _reserved4),
        /// Device Control Delay Configuration Register 0
        (0x170 => dcdr0: ReadWrite<u32, DCDR0::Register>),
        /// Device Control Delay Configuration Register 1
        (0x174 => dcdr1: ReadWrite<u32, DCDR1::Register>),
        (0x178 => _reserved5),
        /// PPU Identification Register 0
        (0xFB0 => idr0: ReadWrite<u32, IDR0::Register>),
        /// PPU Identification Register 1
        (0xFB4 => idr1: ReadWrite<u32, IDR1::Register>),
        (0xFB8 => _reserved6),
        /// Implementation Identification Register
        (0xFC8 => iidr: ReadWrite<u32, IIDR::Register>),
        /// Architecture Identification Register
        (0xFCC => aidr: ReadWrite<u32, AIDR::Register>),
        /// Implementation Defined Identification Register (PID4)
        (0xFD0 => pid4: ReadWrite<u32>),
        (0xFD4 => _reserved7),
        /// Implementation Defined Identification Register (PID0)
        (0xFE0 => pid0: ReadWrite<u32>),
        /// Implementation Defined Identification Register (PID1)
        (0xFE4 => pid1: ReadWrite<u32, PID1::Register>),
        /// Implementation Defined Identification Register (PID2)
        (0xFE8 => pid2: ReadWrite<u32, PID2::Register>),
        /// Implementation Defined Identification Register (PID3)
        (0xFEC => pid3: ReadWrite<u32, PID3::Register>),
        /// Implementation Defined Identification Register (ID0)
        (0xFF0 => id0: ReadWrite<u32>),
        /// Implementation Defined Identification Register (ID1)
        (0xFF4 => id1: ReadWrite<u32>),
        /// Implementation Defined Identification Register (ID2)
        (0xFF8 => id2: ReadWrite<u32>),
        /// Implementation Defined Identification Register (ID3)
        (0xFFC => id3: ReadWrite<u32>),
        (0x1000 => @END),
    }
}
register_bitfields![u32,
PWPR [
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
    /// Power mode dynamic transition enable.  When this bit is set to 1 dynamic transitions are enabled for power modes, allowing transitions to be initiated by changes on power mode DEVACTIVE inputs.
    PWR_DYN_EN OFFSET(8) NUMBITS(1) [],
    /// N/A
    LOCK_EN OFFSET(12) NUMBITS(1) [],
    /// N/A
    OP_POLICY OFFSET(16) NUMBITS(4) [],
    /// N/A
    OP_DYN_EN OFFSET(24) NUMBITS(1) []
],
PMER [
    /// N/A
    EMU_EN OFFSET(0) NUMBITS(1) []
],
PWSR [
    /// Power mode status.  These bits reflect the current power mode of the PPU.  See PPU_PWPR.PWR_POLICY for power mode enumeration.
    PWR_STATUS OFFSET(0) NUMBITS(4) [],
    /// Power mode dynamic transition status.  When set to 1 power mode dynamic transitions are enabled.  There might be a delay in dynamic transitions becoming active or inactive if the PPU is transitioning when PWR_DYN_EN is programmed.
    PWR_DYN_STATUS OFFSET(8) NUMBITS(1) [],
    /// N/A
    LOCK_STATUS OFFSET(12) NUMBITS(1) [],
    /// N/A
    OP_STATUS OFFSET(16) NUMBITS(4) [],
    /// N/A
    OP_DYN_STATUS OFFSET(24) NUMBITS(1) []
],
DISR [
    /// Status of the power mode DEVACTIVE inputs.
///
/// There is one bit for each device interface Q-Channel DEVQACTIVE.  For example, bit 0 is for Q-channel device 0 DEVQACTIVE.  Refer to PPU_IDR0.DEVCHAN for device channel enumeration.
    PWR_DEVACTIVE_STATUS OFFSET(0) NUMBITS(11) [],
    /// N/A
    OP_DEVACTIVE_STATUS OFFSET(24) NUMBITS(8) []
],
MISR [
    /// The status of the PCSMPACCEPT input.
    PCSMPACCEPT_STATUS OFFSET(0) NUMBITS(1) [],
    /// Status of the device interface DEVACCEPT inputs.
///
/// There is one bit for each device interface DEVQACCEPTn.  For example, bit 8 is for Q-Channel 0 DEVQACCEPTn and bit 9 for Q-Channel 1 DEVQACCEPTn.   Refer to PPU_IDR0.DEVCHAN for device channel enumeration.
    DEVACCEPT_STATUS OFFSET(8) NUMBITS(8) [],
    /// Status of the device interface DEVDENY inputs.
///
/// There is one bit for each device interface DEVQDENY.  For example, bit 16 is for Q-Channel 0 DEVQDENY, and bit 17 for Q-Channel 1 DEVQDENY.  Refer to PPU_IDR0.DEVCHAN for device channel enumeration.
    DEVDENY_STATUS OFFSET(16) NUMBITS(8) []
],
STSR [
    /// Status of the DEVDENY signals from the last device interface Q-Channel transition.  For Q-Channel:  There is one bit for each device interface DEVQDENY.  For example, bit 0 is for Q-Channel 0 DEVQDENY, and bit 1 for Q-Channel 1 DEVQDENY.  Refer to PPU_DISR.PWR_DEVACTIVE_STATUS for device enumeration.
    STORED_DEVDENY OFFSET(0) NUMBITS(8) []
],
UNLK [
    /// N/A
    UNLOCK OFFSET(0) NUMBITS(1) []
],
PWCR [
    /// When set to 1 enables the device interface handshake for transitions.  All available bits are reset to 1.
///
/// There is one bit for each device interface channel.  For example, bit 0 is for Q-Channel 0, and bit 1 is for Q-Channel 1.  Refer to PPU_IDR0.DEVCHAN for device channel enumeration.
    DEVREQEN OFFSET(0) NUMBITS(8) [],
    /// These bits enable the power mode DEVACTIVE inputs.  When a bit is to 1 the related DEVACTIVE input is enabled, when set to 0 it is disabled.  All available bits are reset to 1.
///
/// There is one bit for each device interface Q-Channel DEVQACTIVE.  For example, bit 8 is for the Q-Channel 0 DEVQACTIVE, and bit 9 for the Q-Channel 1 DEVQACTIVE.  Refer to PPU_IDR0.DEVCHAN for device channel enumeration.
    PWR_DEVACTIVEEN OFFSET(8) NUMBITS(11) [],
    /// N/A
    OP_DEVACTIVEEN OFFSET(24) NUMBITS(8) []
],
PTCR [
    /// Transition behavior between ON and WARM_RST.  This bit should not be modified when the PPU is in WARM_RST, or if the PPU is performing a transition, otherwise PPU behavior is UNPREDICTABLE.
/// 0:  The PPU does not perform a device interface handshake when transitioning between ON and WARM_RST.
/// 1:  The PPU performs a device interface handshake when transitioning between ON and WARM_RST.  This disables all Q-Channels for this transition.
    WARM_RST_DEVREQEN OFFSET(0) NUMBITS(1) [],
    /// N/A
    DBG_RECOV_PORST_EN OFFSET(1) NUMBITS(1) []
],
IMR [
    /// Static full policy transition completion event mask.
    STA_POLICY_TRN_IRQ_MASK OFFSET(0) NUMBITS(1) [],
    /// Static transition acceptance event mask.
    STA_ACCEPT_IRQ_MASK OFFSET(1) NUMBITS(1) [],
    /// Static transition denial event mask.
    STA_DENY_IRQ_MASK OFFSET(2) NUMBITS(1) [],
    /// N/A
    EMU_ACCEPT_IRQ_MASK OFFSET(3) NUMBITS(1) [],
    /// N/A
    EMU_DENY_IRQ_MASK OFFSET(4) NUMBITS(1) [],
    /// N/A
    LOCKED_IRQ_MASK OFFSET(5) NUMBITS(1) []
],
AIMR [
    /// Unsupported Policy event mask.
    UNSPT_POLICY_IRQ_MASK OFFSET(0) NUMBITS(1) [],
    /// Dynamic transition acceptance event mask.
    DYN_ACCEPT_IRQ_MASK OFFSET(1) NUMBITS(1) [],
    /// Dynamic transition denial event mask.
    DYN_DENY_IRQ_MASK OFFSET(2) NUMBITS(1) [],
    /// N/A
    STA_POLICY_PWR_IRQ_MASK OFFSET(3) NUMBITS(1) [],
    /// N/A
    STA_POLICY_OP_IRQ_MASK OFFSET(4) NUMBITS(1) []
],
ISR [
    /// Static full policy transition completion event status.
    STA_POLICY_TRN_IRQ OFFSET(0) NUMBITS(1) [],
    /// Static transition acceptance event status.
    STA_ACCEPT_IRQ OFFSET(1) NUMBITS(1) [],
    /// Static transition denial event status.
    STA_DENY_IRQ OFFSET(2) NUMBITS(1) [],
    /// N/A
    EMU_ACCEPT_IRQ OFFSET(3) NUMBITS(1) [],
    /// N/A
    EMU_DENY_IRQ OFFSET(4) NUMBITS(1) [],
    /// N/A
    LOCKED_IRQ OFFSET(5) NUMBITS(1) [],
    /// Indicates there is an interrupt event pending in the Additional Interrupt Status Register (PPU_AISR).
    OTHER_IRQ OFFSET(7) NUMBITS(1) [],
    /// N/A
    PWR_ACTIVE_EDGE_IRQ OFFSET(8) NUMBITS(11) [],
    /// N/A
    OP_ACTIVE_EDGE_IRQ OFFSET(24) NUMBITS(8) []
],
AISR [
    /// Unsupported Policy event status.
    UNSPT_POLICY_IRQ OFFSET(0) NUMBITS(1) [],
    /// Dynamic transition acceptance event status.
    DYN_ACCEPT_IRQ OFFSET(1) NUMBITS(1) [],
    /// Dynamic transition denial event status.
    DYN_DENY_IRQ OFFSET(2) NUMBITS(1) [],
    /// N/A
    STA_POLICY_PWR_IRQ OFFSET(3) NUMBITS(1) [],
    /// N/A
    STA_POLICY_OP_IRQ OFFSET(4) NUMBITS(1) []
],
IESR [
    /// DEVACTIVE 0 edge sensitivity.
    DEVACTIVE00_EDGE OFFSET(0) NUMBITS(2) [],
    /// DEVACTIVE 1 edge sensitivity.
    DEVACTIVE01_EDGE OFFSET(2) NUMBITS(2) [],
    /// DEVACTIVE 2 edge sensitivity.
    DEVACTIVE02_EDGE OFFSET(4) NUMBITS(2) [],
    /// N/A
    DEVACTIVE03_EDGE OFFSET(6) NUMBITS(2) [],
    /// N/A
    DEVACTIVE04_EDGE OFFSET(8) NUMBITS(2) [],
    /// N/A
    DEVACTIVE05_EDGE OFFSET(10) NUMBITS(2) [],
    /// N/A
    DEVACTIVE06_EDGE OFFSET(12) NUMBITS(2) [],
    /// N/A
    DEVACTIVE07_EDGE OFFSET(14) NUMBITS(2) [],
    /// N/A
    DEVACTIVE08_EDGE OFFSET(16) NUMBITS(2) [],
    /// N/A
    DEVACTIVE09_EDGE OFFSET(18) NUMBITS(2) [],
    /// N/A
    DEVACTIVE10_EDGE OFFSET(20) NUMBITS(2) []
],
OPSR [
    /// N/A
    DEVACTIVE16_EDGE OFFSET(0) NUMBITS(2) [],
    /// N/A
    DEVACTIVE17_EDGE OFFSET(2) NUMBITS(2) [],
    /// N/A
    DEVACTIVE18_EDGE OFFSET(4) NUMBITS(2) [],
    /// N/A
    DEVACTIVE19_EDGE OFFSET(6) NUMBITS(2) [],
    /// N/A
    DEVACTIVE20_EDGE OFFSET(8) NUMBITS(2) [],
    /// N/A
    DEVACTIVE21_EDGE OFFSET(10) NUMBITS(2) [],
    /// N/A
    DEVACTIVE22_EDGE OFFSET(12) NUMBITS(2) [],
    /// N/A
    DEVACTIVE23_EDGE OFFSET(14) NUMBITS(2) []
],
FUNRR [
    /// N/A
    FUNC_RET_RAM_CFG OFFSET(0) NUMBITS(8) []
],
FULRR [
    /// N/A
    FULL_RET_RAM_CFG OFFSET(0) NUMBITS(8) []
],
MEMRR [
    /// N/A
    MEM_RET_RAM_CFG OFFSET(0) NUMBITS(8) []
],
EDTR0 [
    /// N/A
    OFF_DEL OFFSET(0) NUMBITS(8) [],
    /// N/A
    MEM_RET_DEL OFFSET(8) NUMBITS(8) [],
    /// N/A
    LOGIC_RET_DEL OFFSET(16) NUMBITS(8) [],
    /// N/A
    FULL_RET_DEL OFFSET(24) NUMBITS(8) []
],
EDTR1 [
    /// N/A
    MEM_OFF_DEL OFFSET(0) NUMBITS(8) [],
    /// N/A
    FUNC_RET_DEL OFFSET(8) NUMBITS(8) []
],
DCDR0 [
    /// N/A
    CLKEN_RST_DLY OFFSET(0) NUMBITS(8) [],
    /// N/A
    ISO_CLKEN_DLY OFFSET(8) NUMBITS(8) [],
    /// N/A
    RST_HWSTAT_DLY OFFSET(16) NUMBITS(8) []
],
DCDR1 [
    /// N/A
    ISO_RST_DLY OFFSET(0) NUMBITS(8) [],
    /// N/A
    CLKEN_ISO_DLY OFFSET(8) NUMBITS(8) []
],
IDR0 [
    /// No. of Device Interface Channels.
/// 0: This is a P-Channel PPU.  Refer to PPU_IDR1.OP_ACTIVE for the number of DEVPACTIVE inputs and their meaning.
/// non-zero: The value is the number of Q-Channels.
///
/// The device enumeration is:
/// Device 0: SRSS PDCM,
/// Device 1: CPUSS SEQ between SS level LPD500 EXP & PERI Q-Channel
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
IDR1 [
    /// Power mode entry delay support.
    PWR_MODE_ENTRY_DEL_SPT OFFSET(0) NUMBITS(1) [],
    /// Software device delay control configuration support.
    SW_DEV_DEL_SPT OFFSET(1) NUMBITS(1) [],
    /// Lock and the lock interrupt event are supported.
    LOCK_SPT OFFSET(2) NUMBITS(1) [],
    /// N/A
    MEM_RET_RAM_REG OFFSET(4) NUMBITS(1) [],
    /// N/A
    FULL_RET_RAM_REG OFFSET(5) NUMBITS(1) [],
    /// N/A
    FUNC_RET_RAM_REG OFFSET(6) NUMBITS(1) [],
    /// Power policy transition completion event status.
    STA_POLICY_PWR_IRQ_SPT OFFSET(8) NUMBITS(1) [],
    /// Operating policy transition completion event status.
    STA_POLICY_OP_IRQ_SPT OFFSET(9) NUMBITS(1) [],
    /// N/A
    OP_ACTIVE OFFSET(10) NUMBITS(1) [],
    /// OFF to MEM_RET direct transition.  Indicates if direct transitions from OFF to MEM_RET and from OFF_EMU to MEM_RET_EMU are supported.
    OFF_MEM_RET_TRANS OFFSET(12) NUMBITS(1) []
],
IIDR [
    /// Implementer identification.  [11:8] The JEP106 continuation code of the implementer.  [7] Always 0.  [6:0] The JEP106 identity code of the implementer.  For an Arm implementation, bits [11:0] are 0x43B.
    IMPLEMENTER OFFSET(0) NUMBITS(12) [],
    /// Minor revision of the product.
    REVISION OFFSET(12) NUMBITS(4) [],
    /// Major revision of the product.
    VARIANT OFFSET(16) NUMBITS(4) [],
    /// PPU part identification.
    PRODUCT_ID OFFSET(20) NUMBITS(12) []
],
AIDR [
    /// N/A
    ARCH_REV_MINOR OFFSET(0) NUMBITS(4) [],
    /// N/A
    ARCH_REV_MAJOR OFFSET(4) NUMBITS(4) []
],
PID4 [
    /// The JEP106 continuation code of the implementer, which is 0x4 hardcoded value.
    IMPLEMENTER_11_8 OFFSET(0) NUMBITS(4) []
],
PID0 [
    /// PPU part identification bits [7:0].
    PRODUCT_ID_7_0 OFFSET(0) NUMBITS(8) []
],
PID1 [
    /// PPU part identification bits [11:8]
    PRODUCT_ID_11_8 OFFSET(0) NUMBITS(4) [],
    /// JEP106_ID bits [3:0]
    IMPLEMENTER_3_0 OFFSET(4) NUMBITS(4) []
],
PID2 [
    /// JEP106_ID bits [6:4]
    IMPLEMENTER_6_4 OFFSET(0) NUMBITS(3) [],
    /// Constant HIGH
    CONST_HIGH OFFSET(3) NUMBITS(1) [],
    /// Constant LOW  Revision (4 bits)
    REV_CONST OFFSET(4) NUMBITS(4) []
],
PID3 [
    /// Constant LOW (4 bits)
    PID3_REV_CONST OFFSET(0) NUMBITS(4) [],
    /// Minor revision of the product.
    PID3_REVISION OFFSET(4) NUMBITS(4) []
],
ID0 [
    /// ID0 hard coded value
    ID0 OFFSET(0) NUMBITS(8) []
],
ID1 [
    /// ID1 hardcoded value
    ID1 OFFSET(0) NUMBITS(8) []
],
ID2 [
    /// ID2 hardcoded value
    ID2 OFFSET(0) NUMBITS(8) []
],
ID3 [
    /// ID3 hardcoded value
    ID3 OFFSET(0) NUMBITS(8) []
]
];
const CPUSS_PPU_BASE: StaticRef<Cpuss_PpuRegisters> =
    unsafe { StaticRef::new(0x42105000 as *const Cpuss_PpuRegisters) };

pub struct CpussPpu {
    registers: StaticRef<Cpuss_PpuRegisters>,
}

pub type PwrPolicy = PWPR::PWR_POLICY::Value;

impl CpussPpu {
    pub const fn new() -> CpussPpu {
        CpussPpu {
            registers: CPUSS_PPU_BASE,
        }
    }

    pub fn init_ppu(&self) {
        self.registers.iesr.write(IESR::DEVACTIVE00_EDGE::CLEAR); // disable all
        self.registers.imr.write(
            IMR::STA_POLICY_TRN_IRQ_MASK::SET
                + IMR::STA_ACCEPT_IRQ_MASK::SET
                + IMR::STA_DENY_IRQ_MASK::SET
                + IMR::EMU_ACCEPT_IRQ_MASK::SET
                + IMR::EMU_DENY_IRQ_MASK::SET
                + IMR::LOCKED_IRQ_MASK::SET,
        ); // mask accept events to avoid wakeup
        self.registers.isr.write(ISR::STA_POLICY_TRN_IRQ::CLEAR);
    }

    pub fn ppu_dynamic_enable(&self, min_dyn_state: PwrPolicy) {
        self.registers
            .pwpr
            .modify(PWPR::PWR_DYN_EN::SET + PWPR::PWR_POLICY.val(min_dyn_state as u32));

        while !self.registers.pwsr.is_set(PWSR::PWR_DYN_STATUS) {}
    }
}
