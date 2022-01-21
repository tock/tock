//! Power Mangement for LowRISC

use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    pub PwrMgrRegisters {
        (0x00 => intr_state: ReadOnly<u32, INTR::Register>),
        (0x04 => intr_enable: ReadOnly<u32, INTR::Register>),
        (0x08 => intr_test: ReadOnly<u32, INTR::Register>),
        (0x0C => ctrl_cfg_regwen: ReadOnly<u32, CTRL_CFG_REGWEN::Register>),
        (0x10 => control: ReadWrite<u32, CONTROL::Register>),
        (0x14 => cfg_cdc_sync: ReadWrite<u32, CFG_CDC_SYNC::Register>),
        (0x18 => wakeup_en_regwen: ReadWrite<u32, WAKEUP_EN_REGWEN::Register>),
        (0x1C => wakeup_en: ReadWrite<u32, WAKEUP_EN::Register>),
        (0x20 => wake_status: ReadOnly<u32, WAKE_STATUS::Register>),
        (0x24 => reset_en_regwen: ReadWrite<u32, RESET_EN_REGWEN::Register>),
        (0x28 => reset_en: ReadWrite<u32, RESET_EN::Register>),
        (0x2C => reset_status: ReadOnly<u32, RESET_STATUS::Register>),
        (0x30 => escalate_reset_status: ReadOnly<u32>),
        (0x34 => wake_info_capture_dis: ReadWrite<u32, WAKE_INFO_CAPTURE_DIS::Register>),
        (0x38 => wake_info: ReadWrite<u32, WAKE_INFO::Register>),
        (0x3C => @END),
    }
}

register_bitfields![u32,
    INTR [
        WAKEUP OFFSET(0) NUMBITS(1) []
    ],
    CTRL_CFG_REGWEN [
        EN OFFSET(0) NUMBITS(1) []
    ],
    CONTROL [
        LOW_POWER_HINT OFFSET(0) NUMBITS(1) [],
        CORE_CLK_EN OFFSET(4) NUMBITS(1) [],
        IO_CLK_EN OFFSET(5) NUMBITS(1) [],
        USB_CLKC_EN_LP OFFSET(6) NUMBITS(1) [],
        USB_CLK_EN_ACTIVE OFFSET(7) NUMBITS(1) [],
        MAIN_PD_N OFFSET(8) NUMBITS(1) [],
    ],
    CFG_CDC_SYNC [
        SYNC OFFSET(0) NUMBITS(1) []
    ],
    WAKEUP_EN_REGWEN [
        EN OFFSET(0) NUMBITS(1) []
    ],
    WAKEUP_EN [
        EN0 OFFSET(0) NUMBITS(1) [],
        EN1 OFFSET(1) NUMBITS(1) [],
        EN2 OFFSET(2) NUMBITS(1) [],
        EN3 OFFSET(3) NUMBITS(1) [],
        EN4 OFFSET(4) NUMBITS(1) [],
    ],
    WAKE_STATUS [
        VAL0 OFFSET(0) NUMBITS(1) [],
        VAL1 OFFSET(1) NUMBITS(1) [],
        VAL2 OFFSET(2) NUMBITS(1) [],
        VAL3 OFFSET(3) NUMBITS(1) [],
        VAL4 OFFSET(4) NUMBITS(1) [],
    ],
    RESET_EN_REGWEN [
        EN OFFSET(0) NUMBITS(1) []
    ],
    RESET_EN [
        EN0 OFFSET(0) NUMBITS(1) [],
        EN1 OFFSET(1) NUMBITS(1) [],
    ],
    RESET_STATUS [
        VAL0 OFFSET(0) NUMBITS(1) [],
        VAL1 OFFSET(1) NUMBITS(1) [],
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
