//! Power Control Manager (PCM)

use kernel::common::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::common::StaticRef;

const PCM_BASE: StaticRef<PcmRegisters> =
    unsafe { StaticRef::new(0x4001_0000 as *const PcmRegisters) };

const PCMKEY: u32 = 0x695A; // for unlocking PCMCTL0 and PCMCTL1

register_structs! {
    /// PCM
    PcmRegisters {
        /// Control 0 Register
        (0x000 => ctl0: ReadWrite<u32, PCMCTL0::Register>),
        /// Control 1 Register
        (0x004 => ctl1: ReadWrite<u32, PCMCTL1::Register>),
        /// Interrupt Enable Register
        (0x008 => ie: ReadWrite<u32, PCMIE::Register>),
        /// Interrupt Flag Register
        (0x00C => ifg: ReadOnly<u32, PCMIFG::Register>),
        /// Clear Interrupt Flag Register
        (0x010 => clrifg: WriteOnly<u32, PCMCLRIFG::Register>),
        (0x014 => @END),
    }
}

register_bitfields![u32,
    PCMCTL0 [
        /// Active Mode Request
        AMR OFFSET(0) NUMBITS(4) [
            /// LDO based Active Mode at Core voltage setting 0.
            LDOBasedActiveModeAtCoreVoltageSetting0 = 0,
            /// LDO based Active Mode at Core voltage setting 1.
            LDOBasedActiveModeAtCoreVoltageSetting1 = 1,
            /// DC-DC based Active Mode at Core voltage setting 0.
            DCDCBasedActiveModeAtCoreVoltageSetting0 = 4,
            /// DC-DC based Active Mode at Core voltage setting 1.
            DCDCBasedActiveModeAtCoreVoltageSetting1 = 5,
            /// Low-Frequency Active Mode at Core voltage setting 0.
            LowFrequencyActiveModeAtCoreVoltageSetting0 = 8,
            /// Low-Frequency Active Mode at Core voltage setting 1.
            LowFrequencyActiveModeAtCoreVoltageSetting1 = 9
        ],
        /// Low Power Mode Request
        LPMR OFFSET(4) NUMBITS(4) [
            /// LPM3. Core voltage setting is similar to the mode from which LPM3 is entered.
            LPM3CoreVoltageSettingIsSimilarToTheModeFromWhichLPM3IsEntered = 0,
            /// LPM3.5. Core voltage setting 0.
            LPM35CoreVoltageSetting0 = 10,
            /// LPM4.5
            LPM45 = 12
        ],
        /// Current Power Mode
        CPM OFFSET(8) NUMBITS(6) [
            /// LDO based Active Mode at Core voltage setting 0.
            LDOBasedActiveModeAtCoreVoltageSetting0 = 0,
            /// LDO based Active Mode at Core voltage setting 1.
            LDOBasedActiveModeAtCoreVoltageSetting1 = 1,
            /// DC-DC based Active Mode at Core voltage setting 0.
            DCDCBasedActiveModeAtCoreVoltageSetting0 = 4,
            /// DC-DC based Active Mode at Core voltage setting 1.
            DCDCBasedActiveModeAtCoreVoltageSetting1 = 5,
            /// Low-Frequency Active Mode at Core voltage setting 0.
            LowFrequencyActiveModeAtCoreVoltageSetting0 = 8,
            /// Low-Frequency Active Mode at Core voltage setting 1.
            LowFrequencyActiveModeAtCoreVoltageSetting1 = 9,
            /// LDO based LPM0 at Core voltage setting 0.
            LDOBasedLPM0AtCoreVoltageSetting0 = 16,
            /// LDO based LPM0 at Core voltage setting 1.
            LDOBasedLPM0AtCoreVoltageSetting1 = 17,
            /// DC-DC based LPM0 at Core voltage setting 0.
            DCDCBasedLPM0AtCoreVoltageSetting0 = 20,
            /// DC-DC based LPM0 at Core voltage setting 1.
            DCDCBasedLPM0AtCoreVoltageSetting1 = 21,
            /// Low-Frequency LPM0 at Core voltage setting 0.
            LowFrequencyLPM0AtCoreVoltageSetting0 = 24,
            /// Low-Frequency LPM0 at Core voltage setting 1.
            LowFrequencyLPM0AtCoreVoltageSetting1 = 25,
            /// LPM3
            LPM3 = 32
        ],
        /// PCM key
        PCMKEY OFFSET(16) NUMBITS(16) []
    ],
    PCMCTL1 [
        /// Lock LPM5
        LOCKLPM5 OFFSET(0) NUMBITS(1) [
            /// LPMx.5 configuration defaults to reset condition
            LPMx5ConfigurationDefaultsToResetCondition = 0,
            /// LPMx.5 configuration remains locked during LPMx.5 entry and exit
            LPMx5ConfigurationRemainsLockedDuringLPMx5EntryAndExit = 1
        ],
        /// Lock Backup
        LOCKBKUP OFFSET(1) NUMBITS(1) [
            /// Backup domain configuration defaults to reset condition
            BackupDomainConfigurationDefaultsToResetCondition = 0,
            /// Backup domain configuration remains locked during LPM3.5 entry and exit
            BackupDomainConfigurationRemainsLockedDuringLPM35EntryAndExit = 1
        ],
        /// Force LPM entry
        FORCE_LPM_ENTRY OFFSET(2) NUMBITS(1) [
            /// PCM aborts LPM3/LPMx.5 transition if the active clock configuration does not mee
            FORCE_LPM_ENTRY_0 = 0,
            /// PCM enters LPM3/LPMx.5 after shuting off the clocks forcefully. Application need
            FORCE_LPM_ENTRY_1 = 1
        ],
        /// Power mode request busy flag
        PMR_BUSY OFFSET(8) NUMBITS(1) [],
        /// PCM key
        PCMKEY OFFSET(16) NUMBITS(16) []
    ],
    PCMIE [
        /// LPM invalid transition interrupt enable
        LPM_INVALID_TR_IE OFFSET(0) NUMBITS(1) [
            /// Disabled
            Disabled = 0,
            /// Enabled
            Enabled = 1
        ],
        /// LPM invalid clock interrupt enable
        LPM_INVALID_CLK_IE OFFSET(1) NUMBITS(1) [
            /// Disabled
            Disabled = 0,
            /// Enabled
            Enabled = 1
        ],
        /// Active mode invalid transition interrupt enable
        AM_INVALID_TR_IE OFFSET(2) NUMBITS(1) [
            /// Disabled
            Disabled = 0,
            /// Enabled
            Enabled = 1
        ],
        /// DC-DC error interrupt enable
        DCDC_ERROR_IE OFFSET(6) NUMBITS(1) [
            /// Disabled
            Disabled = 0,
            /// Enabled
            Enabled = 1
        ]
    ],
    PCMIFG [
        /// LPM invalid transition flag
        LPM_INVALID_TR_IFG OFFSET(0) NUMBITS(1) [],
        /// LPM invalid clock flag
        LPM_INVALID_CLK_IFG OFFSET(1) NUMBITS(1) [],
        /// Active mode invalid transition flag
        AM_INVALID_TR_IFG OFFSET(2) NUMBITS(1) [],
        /// DC-DC error flag
        DCDC_ERROR_IFG OFFSET(6) NUMBITS(1) []
    ],
    PCMCLRIFG [
        /// Clear LPM invalid transition flag
        CLR_LPM_INVALID_TR_IFG OFFSET(0) NUMBITS(1) [],
        /// Clear LPM invalid clock flag
        CLR_LPM_INVALID_CLK_IFG OFFSET(1) NUMBITS(1) [],
        /// Clear active mode invalid transition flag
        CLR_AM_INVALID_TR_IFG OFFSET(2) NUMBITS(1) [],
        /// Clear DC-DC error flag
        CLR_DCDC_ERROR_IFG OFFSET(6) NUMBITS(1) []
    ]
];

pub struct Pcm {
    registers: StaticRef<PcmRegisters>,
}

impl Pcm {
    pub const fn new() -> Pcm {
        Pcm {
            registers: PCM_BASE,
        }
    }
    // currently not sure about the interface, so just implement a simple
    // method for activating AM_LDO_VCORE1 to provide enough power for 48MHz
    pub fn set_high_power(&self) {
        while self.registers.ctl1.is_set(PCMCTL1::PMR_BUSY) {}
        self.registers.ctl0.write(
            PCMCTL0::PCMKEY.val(PCMKEY) + PCMCTL0::AMR::DCDCBasedActiveModeAtCoreVoltageSetting1,
        );
        while self.registers.ctl1.is_set(PCMCTL1::PMR_BUSY) {}
    }
}
