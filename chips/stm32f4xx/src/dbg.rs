use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;

/// Debug support
#[repr(C)]
struct DbgRegisters {
    /// IDCODE
    dbgmcu_idcode: ReadOnly<u32, DBGMCU_IDCODE::Register>,
    /// Control Register
    dbgmcu_cr: ReadWrite<u32, DBGMCU_CR::Register>,
    /// Debug MCU APB1 Freeze registe
    dbgmcu_apb1_fz: ReadWrite<u32, DBGMCU_APB1_FZ::Register>,
    /// Debug MCU APB2 Freeze registe
    dbgmcu_apb2_fz: ReadWrite<u32, DBGMCU_APB2_FZ::Register>,
}

register_bitfields![u32,
    DBGMCU_IDCODE [
        /// DEV_ID
        DEV_ID OFFSET(0) NUMBITS(12) [],
        /// REV_ID
        REV_ID OFFSET(16) NUMBITS(16) []
    ],
    DBGMCU_CR [
        /// DBG_SLEEP
        DBG_SLEEP OFFSET(0) NUMBITS(1) [],
        /// DBG_STOP
        DBG_STOP OFFSET(1) NUMBITS(1) [],
        /// DBG_STANDBY
        DBG_STANDBY OFFSET(2) NUMBITS(1) [],
        /// TRACE_IOEN
        TRACE_IOEN OFFSET(5) NUMBITS(1) [],
        /// TRACE_MODE
        TRACE_MODE OFFSET(6) NUMBITS(2) []
    ],
    DBGMCU_APB1_FZ [
        /// DBG_TIM2_STOP
        DBG_TIM2_STOP OFFSET(0) NUMBITS(1) [],
        /// DBG_TIM3 _STOP
        DBG_TIM3_STOP OFFSET(1) NUMBITS(1) [],
        /// DBG_TIM4_STOP
        DBG_TIM4_STOP OFFSET(2) NUMBITS(1) [],
        /// DBG_TIM5_STOP
        DBG_TIM5_STOP OFFSET(3) NUMBITS(1) [],
        /// DBG_TIM6_STOP
        DBG_TIM6_STOP OFFSET(4) NUMBITS(1) [],
        /// DBG_TIM7_STOP
        DBG_TIM7_STOP OFFSET(5) NUMBITS(1) [],
        /// DBG_TIM12_STOP
        DBG_TIM12_STOP OFFSET(6) NUMBITS(1) [],
        /// DBG_TIM13_STOP
        DBG_TIM13_STOP OFFSET(7) NUMBITS(1) [],
        /// DBG_TIM14_STOP
        DBG_TIM14_STOP OFFSET(8) NUMBITS(1) [],
        /// RTC stopped when Core is halted
        DBG_RTC_STOP OFFSET(10) NUMBITS(1) [],
        /// DBG_WWDG_STOP
        DBG_WWDG_STOP OFFSET(11) NUMBITS(1) [],
        /// DBG_IWDEG_STOP
        DBG_IWDEG_STOP OFFSET(12) NUMBITS(1) [],
        /// DBG_J2C1_SMBUS_TIMEOUT
        DBG_J2C1_SMBUS_TIMEOUT OFFSET(21) NUMBITS(1) [],
        /// DBG_J2C2_SMBUS_TIMEOUT
        DBG_J2C2_SMBUS_TIMEOUT OFFSET(22) NUMBITS(1) [],
        /// DBG_J2C3SMBUS_TIMEOUT
        DBG_J2C3SMBUS_TIMEOUT OFFSET(23) NUMBITS(1) [],
        /// SMBUS timeout mode stopped when Core is halted
        DBG_I2CFMP_SMBUS_TIMEOUT OFFSET(24) NUMBITS(1) [],
        /// DBG_CAN1_STOP
        DBG_CAN1_STOP OFFSET(25) NUMBITS(1) [],
        /// DBG_CAN2_STOP
        DBG_CAN2_STOP OFFSET(26) NUMBITS(1) []
    ],
    DBGMCU_APB2_FZ [
        /// TIM1 counter stopped when core is halted
        DBG_TIM1_STOP OFFSET(0) NUMBITS(1) [],
        /// TIM8 counter stopped when core is halted
        DBG_TIM8_STOP OFFSET(1) NUMBITS(1) [],
        /// TIM9 counter stopped when core is halted
        DBG_TIM9_STOP OFFSET(16) NUMBITS(1) [],
        /// TIM10 counter stopped when core is halted
        DBG_TIM10_STOP OFFSET(17) NUMBITS(1) [],
        /// TIM11 counter stopped when core is halted
        DBG_TIM11_STOP OFFSET(18) NUMBITS(1) []
    ]
];

const DBG_BASE: StaticRef<DbgRegisters> =
    unsafe { StaticRef::new(0xE0042000 as *const DbgRegisters) };

pub struct Dbg {
    registers: StaticRef<DbgRegisters>,
}

pub static mut DBG: Dbg = Dbg::new();

impl Dbg {
    const fn new() -> Dbg {
        Dbg {
            registers: DBG_BASE,
        }
    }

    pub fn disable_tim2_counter(&self) {
        self.registers
            .dbgmcu_apb1_fz
            .modify(DBGMCU_APB1_FZ::DBG_TIM2_STOP::SET);
    }
}
