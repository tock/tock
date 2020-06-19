//! Power Mangement for LowRISC

use kernel::common::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::common::StaticRef;

register_structs! {
    pub PwrMgrRegisters {
        (0x00 => ctrl_cfg_regwen: ReadOnly<u32, CTRL_CFG_REGWEN::Register>),
        (0x04 => control: ReadWrite<u32, CONTROL::Register>),
        (0x08 => cfg_cdc_sync: ReadWrite<u32, CFG_CDC_SYNC::Register>),
        (0x0C => wakeup_en_regwen: ReadWrite<u32, WAKEUP_EN_REGWEN::Register>),
        (0x10 => wakeup_en: ReadWrite<u32, WAKEUP_EN::Register>),
        (0x14 => wake_status: ReadOnly<u32, WAKE_STATUS::Register>),
        (0x18 => reset_en_regwen: ReadWrite<u32, RESET_EN_REGWEN::Register>),
        (0x1C => reset_en: ReadWrite<u32, RESET_EN::Register>),
        (0x20 => reset_status: ReadOnly<u32, RESET_STATUS::Register>),
        (0x24 => wake_info_capture_dis: ReadWrite<u32, WAKE_INFO_CAPTURE_DIS::Register>),
        (0x28 => wake_info: ReadWrite<u32, WAKE_INFO::Register>),
        (0x2C => @END),
    }
}

register_bitfields![u32,
    CTRL_CFG_REGWEN [
        EN OFFSET(0) NUMBITS(1) []
    ],
    CONTROL [
        LOW_POWER_HINT OFFSET(0) NUMBITS(1) [],
        CORE_CLK_EN OFFSET(4) NUMBITS(1) [],
        IO_CLK_EN OFFSET(5) NUMBITS(1) [],
        MAIN_PD_N OFFSET(6) NUMBITS(1) []
    ],
    CFG_CDC_SYNC [
        SYNC OFFSET(0) NUMBITS(1) []
    ],
    WAKEUP_EN_REGWEN [
        EN OFFSET(0) NUMBITS(1) []
    ],
    WAKEUP_EN [
        START OFFSET(0) NUMBITS(16) []
    ],
    WAKE_STATUS [
        VAL OFFSET(0) NUMBITS(16) []
    ],
    RESET_EN_REGWEN [
        EN OFFSET(0) NUMBITS(1) []
    ],
    RESET_EN [
        EN OFFSET(0) NUMBITS(2) []
    ],
    RESET_STATUS [
        VAL OFFSET(0) NUMBITS(2) []
    ],
    WAKE_INFO_CAPTURE_DIS [
        VAL OFFSET(0) NUMBITS(1) []
    ],
    WAKE_INFO [
        REASONS OFFSET(0) NUMBITS(16) [],
        FALL_THROUGH OFFSET(16) NUMBITS(1) [],
        ABORT OFFSET(17) NUMBITS(1) []
    ]
];

pub struct PwrMgr {
    registers: StaticRef<PwrMgrRegisters>,
}

impl PwrMgr {
    pub const fn new(base: StaticRef<PwrMgrRegisters>) -> PwrMgr {
        PwrMgr { registers: base }
    }

    pub fn check_clock_propagation(&self) -> bool {
        let regs = self.registers;

        if regs.cfg_cdc_sync.read(CFG_CDC_SYNC::SYNC) == 0 {
            return true;
        }

        false
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;

        // Disable power saving
        regs.control.write(CONTROL::LOW_POWER_HINT::CLEAR);

        // Propagate changes to slow clock domain
        regs.cfg_cdc_sync.write(CFG_CDC_SYNC::SYNC::SET);
    }

    pub fn enable_low_power(&self) {
        let regs = self.registers;

        if regs.control.read(CONTROL::LOW_POWER_HINT) != 1 {
            // Next WFI should trigger low power entry
            // Leave the IO clock enabled as we need to get interrupts
            regs.control.write(
                CONTROL::LOW_POWER_HINT::SET
                    + CONTROL::CORE_CLK_EN::CLEAR
                    + CONTROL::IO_CLK_EN::SET
                    + CONTROL::MAIN_PD_N::CLEAR,
            );

            // Propagate changes to slow clock domain
            regs.cfg_cdc_sync.write(CFG_CDC_SYNC::SYNC::SET);
        }
    }
}
