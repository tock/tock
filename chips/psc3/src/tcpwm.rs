// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025 SRL.

// TODO new registers
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::{
    interfaces::{ReadWriteable, Readable},
    register_bitfields, register_structs, ReadOnly, ReadWrite,
};
use kernel::utilities::StaticRef;
use kernel::{
    hil::{
        self,
        time::{self, Alarm, Ticks, Time},
    },
    utilities::registers::interfaces::Writeable,
};

use crate::tcpwm::CNT_CTRL::QUAD_ENCODING_MODE;

#[repr(C)]
pub struct CounterRegistersGrp0 {
    /// 0x00 - Counter control register
    pub ctrl: ReadWrite<u32, CNT_CTRL::Register>,
    /// 0x04 - Counter status register
    pub status: ReadOnly<u32, CNT_STATUS::Register>,
    /// 0x08 - Counter count register
    pub counter: ReadWrite<u32, CNT_COUNTER::Register>,
    _reserved0: [u8; 4],
    /// 0x10 - Counter compare/capture 0 register
    pub cc0: ReadWrite<u32, CNT_CC::Register>,
    /// 0x14 - Counter buffered compare/capture 0 register
    pub cc0_buff: ReadWrite<u32, CNT_CC_BUFF::Register>,
    /// 0x18 - Counter compare/capture 1 register
    pub cc1: ReadWrite<u32, CNT_CC::Register>,
    /// 0x1C - Counter buffered compare/capture 1 register
    pub cc1_buff: ReadWrite<u32, CNT_CC_BUFF::Register>,
    /// 0x20 - Counter period register
    pub period: ReadWrite<u32, CNT_PERIOD::Register>,
    /// 0x24 - Counter buffered period register
    pub period_buff: ReadWrite<u32, CNT_PERIOD_BUFF::Register>,
    /// 0x28 - Counter line selection register
    pub line_sel: ReadWrite<u32, CNT_LINE_SEL::Register>,
    /// 0x2C - Counter buffered line selection register
    pub line_sel_buff: ReadWrite<u32, CNT_LINE_SEL_BUFF::Register>,
    /// 0x30 - Counter PWM dead time register
    pub dt: ReadWrite<u32, CNT_DT::Register>,
    /// 0x34 - Counter buffered PWM dead time register
    pub dt_buff: ReadWrite<u32, CNT_DT_BUFF::Register>,
    /// 0x38 - Counter prescalar register
    pub ps: ReadWrite<u32, CNT_PS::Register>,
    _reserved1: [u8; 4],
    /// 0x40 - Counter trigger command register
    pub tr_cmd: ReadWrite<u32, CNT_TR_CMD::Register>,
    /// 0x44 - Counter input trigger selection register 0
    pub tr_in_sel0: ReadWrite<u32, CNT_TR_IN_SEL0::Register>,
    /// 0x48 - Counter input trigger selection register 1
    pub tr_in_sel1: ReadWrite<u32, CNT_TR_IN_SEL1::Register>,
    /// 0x4C - Counter input trigger edge selection register
    pub tr_in_edge_sel: ReadWrite<u32, CNT_TR_IN_EDGE_SEL::Register>,
    /// 0x50 - Counter trigger PWM control register
    pub tr_pwm_ctrl: ReadWrite<u32, CNT_TR_PWM_CTRL::Register>,
    /// 0x54 - Counter output trigger selection register
    pub tr_out_sel: ReadWrite<u32, CNT_TR_OUT_SEL::Register>,
    _reserved2: [u8; 24],
    /// 0x70 - Interrupt request register
    pub intr: ReadWrite<u32, CNT_INTR::Register>,
    /// 0x74 - Interrupt set request register
    pub intr_set: ReadWrite<u32, CNT_INTR_SET::Register>,
    /// 0x78 - Interrupt mask register
    pub intr_mask: ReadWrite<u32, CNT_INTR_MASK::Register>,
    /// 0x7C - Interrupt masked request register
    pub intr_masked: ReadOnly<u32, CNT_INTR_MASKED::Register>,
    _reserved3: [u8; 4],
    /// 0x84 - Glitch filter register for one to one trigger
    pub one_gf0: ReadWrite<u32, CNT_ONE_GF::Register>,
    _reserved4: [u8; 28],
    /// 0xA4 - Sync bypass register for one to one trigger
    pub tr_one_sync_bypass: ReadWrite<u32, CNT_TR_ONE_SYNC_BYPASS::Register>,
    _reserved5: [u8; 8],
    /// 0xB0 - Counter control register for HRPWM feature
    pub hrpwm_ctrl: ReadWrite<u32, CNT_HRPWM_CTRL::Register>,
    _reserved6: [u8; 76],
}

#[repr(C)]
pub struct CounterRegisters {
    /// 0x00 - Counter control register
    pub ctrl: ReadWrite<u32, CNT_CTRL::Register>,
    /// 0x04 - Counter status register
    pub status: ReadOnly<u32, CNT_STATUS::Register>,
    /// 0x08 - Counter count register
    pub counter: ReadWrite<u32, CNT_COUNTER::Register>,
    _reserved0: [u8; 4],
    /// 0x10 - Counter compare/capture 0 register
    pub cc0: ReadWrite<u32, CNT_CC::Register>,
    /// 0x14 - Counter buffered compare/capture 0 register
    pub cc0_buff: ReadWrite<u32, CNT_CC_BUFF::Register>,
    /// 0x18 - Counter compare/capture 1 register
    pub cc1: ReadWrite<u32, CNT_CC::Register>,
    /// 0x1C - Counter buffered compare/capture 1 register
    pub cc1_buff: ReadWrite<u32, CNT_CC_BUFF::Register>,
    /// 0x20 - Counter period register
    pub period: ReadWrite<u32, CNT_PERIOD::Register>,
    /// 0x24 - Counter buffered period register
    pub period_buff: ReadWrite<u32, CNT_PERIOD_BUFF::Register>,
    /// 0x28 - Counter line selection register
    pub line_sel: ReadWrite<u32, CNT_LINE_SEL::Register>,
    /// 0x2C - Counter buffered line selection register
    pub line_sel_buff: ReadWrite<u32, CNT_LINE_SEL_BUFF::Register>,
    /// 0x30 - Counter PWM dead time register
    pub dt: ReadWrite<u32, CNT_DT::Register>,
    /// 0x34 - Counter buffered PWM dead time register
    pub dt_buff: ReadWrite<u32, CNT_DT_BUFF::Register>,
    /// 0x38 - Counter prescalar register
    pub ps: ReadWrite<u32, CNT_PS::Register>,
    _reserved1: [u8; 4],
    /// 0x40 - Counter trigger command register
    pub tr_cmd: ReadWrite<u32, CNT_TR_CMD::Register>,
    /// 0x44 - Counter input trigger selection register 0
    pub tr_in_sel0: ReadWrite<u32, CNT_TR_IN_SEL0::Register>,
    /// 0x48 - Counter input trigger selection register 1
    pub tr_in_sel1: ReadWrite<u32, CNT_TR_IN_SEL1::Register>,
    /// 0x4C - Counter input trigger edge selection register
    pub tr_in_edge_sel: ReadWrite<u32, CNT_TR_IN_EDGE_SEL::Register>,
    /// 0x50 - Counter trigger PWM control register
    pub tr_pwm_ctrl: ReadWrite<u32, CNT_TR_PWM_CTRL::Register>,
    /// 0x54 - Counter output trigger selection register
    pub tr_out_sel: ReadWrite<u32, CNT_TR_OUT_SEL::Register>,
    _reserved2: [u8; 24],
    /// 0x70 - Interrupt request register
    pub intr: ReadWrite<u32, CNT_INTR::Register>,
    /// 0x74 - Interrupt set request register
    pub intr_set: ReadWrite<u32, CNT_INTR_SET::Register>,
    /// 0x78 - Interrupt mask register
    pub intr_mask: ReadWrite<u32, CNT_INTR_MASK::Register>,
    /// 0x7C - Interrupt masked request register
    pub intr_masked: ReadOnly<u32, CNT_INTR_MASKED::Register>,
    /// 0x80 - LFSR register
    pub lfsr: ReadWrite<u32, CNT_LFSR::Register>,
    _reserved3: [u8; 124],
}

register_structs! {
    Tcpwm0Registers {
        (0x000 => grp0_cnt0: CounterRegistersGrp0),
        (0x100 => grp0_cnt1: CounterRegistersGrp0),
        (0x200 => grp0_cnt2: CounterRegistersGrp0),
        (0x300 => grp0_cnt3: CounterRegistersGrp0),
        (0x400 => _reserved0),
        (0x10000 => grp1_cnt0: CounterRegisters),
        (0x10100 => grp1_cnt1: CounterRegisters),
        (0x10200 => grp1_cnt2: CounterRegisters),
        (0x10300 => grp1_cnt3: CounterRegisters),
        (0x10400 => grp1_cnt4: CounterRegisters),
        (0x10500 => grp1_cnt5: CounterRegisters),
        (0x10600 => grp1_cnt6: CounterRegisters),
        (0x10700 => grp1_cnt7: CounterRegisters),
        (0x10800 => @END),
    }
}
register_bitfields![u32,
CNT_CTRL [
    AUTO_RELOAD_CC0 OFFSET(0) NUMBITS(1) [],
    AUTO_RELOAD_CC1 OFFSET(1) NUMBITS(1) [],
    AUTO_RELOAD_PERIOD OFFSET(2) NUMBITS(1) [],
    AUTO_RELOAD_LINE_SEL OFFSET(3) NUMBITS(1) [],
    CC0_MATCH_UP_EN OFFSET(4) NUMBITS(1) [],
    CC0_MATCH_DOWN_EN OFFSET(5) NUMBITS(1) [],
    CC1_MATCH_UP_EN OFFSET(6) NUMBITS(1) [],
    CC1_MATCH_DOWN_EN OFFSET(7) NUMBITS(1) [],
    PWM_IMM_KILL OFFSET(8) NUMBITS(1) [],
    PWM_STOP_ON_KILL OFFSET(9) NUMBITS(1) [],
    PWM_SYNC_KILL OFFSET(10) NUMBITS(1) [],
    SWAP_ENABLE OFFSET(11) NUMBITS(1) [],
    PWM_DISABLE_MODE OFFSET(12) NUMBITS(2) [
        Z = 0,
        RETAIN = 1,
        L = 2,
        H = 3
    ],
    PWM_TC_SYNC_KILL_DT OFFSET(14) NUMBITS(1) [],
    PWM_SYNC_KILL_DT OFFSET(15) NUMBITS(1) [],
    UP_DOWN_MODE OFFSET(16) NUMBITS(2) [
        COUNT_UP = 0,
        COUNT_DOWN = 1,
        COUNT_UPDN1 = 2,
        COUNT_UPDN2 = 3
    ],
    ONE_SHOT OFFSET(18) NUMBITS(1) [],
    QUAD_ENCODING_MODE OFFSET(20) NUMBITS(2) [
        X1 = 0,
        /// X2 encoding (QUAD mode)
        X2 = 1,
        /// X4 encoding (QUAD mode)
        X4 = 2,
        UP_DOWN = 3
    ],
    /// When '0', dithering is disabled
    DITHEREN OFFSET(22) NUMBITS(2) [
        /// Period dithering is enabled
        PERIOD_DITHEN = 1,
        /// Duty dithering is enabled
        DUTY_DITHEN = 2,
        /// Period and Duty dithering is enabled
        PER_DUTY_DITHEN = 3
    ],
    /// Counter mode.
    MODE OFFSET(24) NUMBITS(3) [
        /// Timer mode
        TIMER = 0,
        /// N/A
        RSVD1 = 1,
        /// Capture mode
        CAPTURE = 2,
        QUAD = 3,
        PWM = 4,
        PWM_DT = 5,
        /// Pseudo random pulse width modulation
        PWM_PR = 6,
        /// Shift register mode.
        SR = 7
    ],
    KILL_LINE_POLARITY OFFSET(27) NUMBITS(2) [
        KILL_LINE_OUT_POLARITY = 1,
        KILL_LINE_COMPL_OUT_POLARITY = 2
    ],
    DBG_SUS_EN OFFSET(29) NUMBITS(1) [],
    DBG_FREEZE_EN OFFSET(30) NUMBITS(1) [],
    ENABLED OFFSET(31) NUMBITS(1) []
],
CNT_STATUS [
    DOWN OFFSET(0) NUMBITS(1) [],
    CC0_READ_MISS OFFSET(1) NUMBITS(1) [],
    CC1_READ_MISS OFFSET(2) NUMBITS(1) [],
    KILL_STATUS OFFSET(3) NUMBITS(1) [],
    /// Indicates the actual level of the selected capture 0 trigger.
    TR_CAPTURE0 OFFSET(4) NUMBITS(1) [],
    /// Indicates the actual level of the selected count trigger.
    TR_COUNT OFFSET(5) NUMBITS(1) [],
    /// Indicates the actual level of the selected reload trigger.
    TR_RELOAD OFFSET(6) NUMBITS(1) [],
    /// Indicates the actual level of the selected stop trigger.
    TR_STOP OFFSET(7) NUMBITS(1) [],
    /// Indicates the actual level of the selected start trigger.
    TR_START OFFSET(8) NUMBITS(1) [],
    /// Indicates the actual level of the selected capture 1 trigger.
    TR_CAPTURE1 OFFSET(9) NUMBITS(1) [],
    /// Indicates the actual level of the PWM line output signal.
    LINE_OUT OFFSET(10) NUMBITS(1) [],
    /// Indicates the actual level of the complementary PWM line output signal.
    LINE_COMPL_OUT OFFSET(11) NUMBITS(1) [],
    RUNNING OFFSET(15) NUMBITS(1) [],
    DT_CNT_L OFFSET(16) NUMBITS(8) [],
    DT_CNT_H OFFSET(24) NUMBITS(8) []
],
CNT_COUNTER [
    COUNTER OFFSET(0) NUMBITS(32) []
],
CNT_CC [
    CC OFFSET(0) NUMBITS(32) []
],
CNT_CC_BUFF [
    CC OFFSET(0) NUMBITS(32) []
],
CNT_PERIOD [
    PERIOD OFFSET(0) NUMBITS(32) []
],
CNT_PERIOD_BUFF [
    PERIOD OFFSET(0) NUMBITS(32) []
],
CNT_LINE_SEL [
    OUT_SEL OFFSET(0) NUMBITS(3) [
        /// fixed '0'
        L = 0,
        /// fixed '1'
        H = 1,
        /// PWM signal 'line'
        PWM = 2,
        /// inverted PWM signal 'line'
        PWM_INV = 3,
        Z = 4,
        MOTIF = 5,
        /// N/A
        RSVD6 = 6
    ],
    COMPL_OUT_SEL OFFSET(4) NUMBITS(3) [
        /// fixed '0'
        L = 0,
        /// fixed '1'
        H = 1,
        /// PWM signal 'line'
        PWM = 2,
        /// inverted PWM signal 'line'
        PWM_INV = 3,
        Z = 4,
        MOTIF = 5,
        /// N/A
        RSVD6 = 6
    ]
],
CNT_LINE_SEL_BUFF [
    OUT_SEL OFFSET(0) NUMBITS(3) [],
    COMPL_OUT_SEL OFFSET(4) NUMBITS(3) []
],
CNT_DT [
    DT_LINE_OUT_L OFFSET(0) NUMBITS(8) [],
    DT_LINE_OUT_H OFFSET(8) NUMBITS(8) [],
    DT_LINE_COMPL_OUT OFFSET(16) NUMBITS(16) []
],
CNT_DT_BUFF [
    DT_LINE_OUT_L OFFSET(0) NUMBITS(8) [],
    DT_LINE_OUT_H OFFSET(8) NUMBITS(8) [],
    DT_LINE_COMPL_OUT OFFSET(16) NUMBITS(16) []
],
CNT_PS [
    PS_DIV OFFSET(0) NUMBITS(3) [
        /// Pre-scaling of the selected counter clock. Divide by 1
        DIVBY1 = 0,
        /// Pre-scaling of the selected counter clock. Divide by 2
        DIVBY2 = 1,
        /// Pre-scaling of the selected counter clock. Divide by 4
        DIVBY4 = 2,
        /// Pre-scaling of the selected counter clock. Divide by 8
        DIVBY8 = 3,
        /// Pre-scaling of the selected counter clock. Divide by 16
        DIVBY16 = 4,
        /// Pre-scaling of the selected counter clock. Divide by 32
        DIVBY32 = 5,
        /// Pre-scaling of the selected counter clock. Divide by 64
        DIVBY64 = 6,
        /// Pre-scaling of the selected counter clock. Divide by 128
        DIVBY128 = 7
    ]
],
CNT_TR_CMD [
    CAPTURE0 OFFSET(0) NUMBITS(1) [],
    /// SW reload trigger. For HW behavior, see COUNTER_CAPTURE0 field.
    RELOAD OFFSET(2) NUMBITS(1) [],
    /// SW stop trigger. For HW behavior, see COUNTER_CAPTURE0 field.
    STOP OFFSET(3) NUMBITS(1) [],
    /// SW start trigger. For HW behavior, see COUNTER_CAPTURE0 field.
    START OFFSET(4) NUMBITS(1) [],
    /// SW capture 1 trigger. For HW behavior, see COUNTER_CAPTURE0 field.
    CAPTURE1 OFFSET(5) NUMBITS(1) []
],
CNT_TR_IN_SEL0 [
    CAPTURE0_SEL OFFSET(0) NUMBITS(8) [],
    COUNT_SEL OFFSET(8) NUMBITS(8) [],
    RELOAD_SEL OFFSET(16) NUMBITS(8) [],
    STOP_SEL OFFSET(24) NUMBITS(8) []
],
CNT_TR_IN_SEL1 [
    START_SEL OFFSET(0) NUMBITS(8) [],
    /// Selects one of the 256 input triggers as a capture 1 trigger.
    CAPTURE1_SEL OFFSET(8) NUMBITS(8) []
],
CNT_TR_IN_EDGE_SEL [
    /// A capture 0 event will copy the counter value into the CC0 register.
    CAPTURE0_EDGE OFFSET(0) NUMBITS(2) [
        /// Rising edge. Any rising edge generates an event.
        RISING_EDGE = 0,
        /// Falling edge. Any falling edge generates an event.
        FALLING_EDGE = 1,
        /// Rising AND falling edge. Any odd amount of edges generates an event.
        ANY_EDGE = 2,
        /// No edge detection, use trigger as is.
        NO_EDGE_DET = 3
    ],
    /// A counter event will increase or decrease the counter by '1'.
    COUNT_EDGE OFFSET(2) NUMBITS(2) [
        /// Rising edge. Any rising edge generates an event.
        RISING_EDGE = 0,
        /// Falling edge. Any falling edge generates an event.
        FALLING_EDGE = 1,
        /// Rising AND falling edge. Any odd amount of edges generates an event.
        ANY_EDGE = 2,
        /// No edge detection, use trigger as is.
        NO_EDGE_DET = 3
    ],
    RELOAD_EDGE OFFSET(4) NUMBITS(2) [
        /// Rising edge. Any rising edge generates an event.
        RISING_EDGE = 0,
        /// Falling edge. Any falling edge generates an event.
        FALLING_EDGE = 1,
        /// Rising AND falling edge. Any odd amount of edges generates an event.
        ANY_EDGE = 2,
        /// No edge detection, use trigger as is.
        NO_EDGE_DET = 3
    ],
    STOP_EDGE OFFSET(6) NUMBITS(2) [
        /// Rising edge. Any rising edge generates an event.
        RISING_EDGE = 0,
        /// Falling edge. Any falling edge generates an event.
        FALLING_EDGE = 1,
        /// Rising AND falling edge. Any odd amount of edges generates an event.
        ANY_EDGE = 2,
        /// No edge detection, use trigger as is.
        NO_EDGE_DET = 3
    ],
    START_EDGE OFFSET(8) NUMBITS(2) [
        /// Rising edge. Any rising edge generates an event.
        RISING_EDGE = 0,
        /// Falling edge. Any falling edge generates an event.
        FALLING_EDGE = 1,
        /// Rising AND falling edge. Any odd amount of edges generates an event.
        ANY_EDGE = 2,
        /// No edge detection, use trigger as is.
        NO_EDGE_DET = 3
    ],
    /// A capture 1 event will copy the counter value into the CC1 register.
    CAPTURE1_EDGE OFFSET(10) NUMBITS(2) [
        /// Rising edge. Any rising edge generates an event.
        RISING_EDGE = 0,
        /// Falling edge. Any falling edge generates an event.
        FALLING_EDGE = 1,
        /// Rising AND falling edge. Any odd amount of edges generates an event.
        ANY_EDGE = 2,
        /// No edge detection, use trigger as is.
        NO_EDGE_DET = 3
    ]
],
CNT_TR_PWM_CTRL [
    CC0_MATCH_MODE OFFSET(0) NUMBITS(2) [
        /// Set to '1'
        SET_TO_1 = 0,
        /// Set to '0'
        CLEAR_TO_0 = 1,
        /// Invert
        INVERT = 2,
        /// No Change
        NO_CHANGE = 3
    ],
    OVERFLOW_MODE OFFSET(2) NUMBITS(2) [
        /// Set to '1'
        SET_TO_1 = 0,
        /// Set to '0'
        CLEAR_TO_0 = 1,
        /// Invert
        INVERT = 2,
        /// No Change
        NO_CHANGE = 3
    ],
    UNDERFLOW_MODE OFFSET(4) NUMBITS(2) [
        /// Set to '1'
        SET_TO_1 = 0,
        /// Set to '0'
        CLEAR_TO_0 = 1,
        /// Invert
        INVERT = 2,
        /// No Change
        NO_CHANGE = 3
    ],
    CC1_MATCH_MODE OFFSET(6) NUMBITS(2) [
        /// Set to '1'
        SET_TO_1 = 0,
        /// Set to '0'
        CLEAR_TO_0 = 1,
        /// Invert
        INVERT = 2,
        /// No Change
        NO_CHANGE = 3
    ]
],
CNT_TR_OUT_SEL [
    OUT0 OFFSET(0) NUMBITS(3) [
        /// Overflow event
        OVERFLOW = 0,
        /// Underflow event
        UNDERFLOW = 1,
        /// Terminal count event (default selection)
        TC = 2,
        /// Compare match 0 event
        CC0_MATCH = 3,
        /// Compare match 1 event
        CC1_MATCH = 4,
        /// PWM output signal 'line_out'
        LINE_OUT = 5,
        /// Compare match 0 event or Compare match 1 event
        CC0_CC1_MATCH = 6,
        /// Output trigger disabled.
        Disabled = 7
    ],
    OUT1 OFFSET(4) NUMBITS(3) [
        /// Overflow event
        OVERFLOW = 0,
        /// Underflow event
        UNDERFLOW = 1,
        /// Terminal count event
        TC = 2,
        /// Compare match 0 event (default selection)
        CC0_MATCH = 3,
        /// Compare match 1 event
        CC1_MATCH = 4,
        /// PWM output signal 'line_out'
        LINE_OUT = 5,
        /// Compare match 0 event or Compare match 1 event
        CC0_CC1_MATCH = 6,
        /// Output trigger disabled.
        Disabled = 7
    ]
],
CNT_INTR [
    /// Terminal count event. Set to '1', when event is detected. Write with '1' to clear bit.
    TC OFFSET(0) NUMBITS(1) [],
    CC0_MATCH OFFSET(1) NUMBITS(1) [],
    CC1_MATCH OFFSET(2) NUMBITS(1) []
],
CNT_INTR_SET [
    /// Write with '1' to set corresponding bit in interrupt request register.
    TC OFFSET(0) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    CC0_MATCH OFFSET(1) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    CC1_MATCH OFFSET(2) NUMBITS(1) []
],
CNT_INTR_MASK [
    /// Mask bit for corresponding bit in interrupt request register.
    TC OFFSET(0) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    CC0_MATCH OFFSET(1) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    CC1_MATCH OFFSET(2) NUMBITS(1) []
],
CNT_INTR_MASKED [
    /// Logical and of corresponding request and mask bits.
    TC OFFSET(0) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    CC0_MATCH OFFSET(1) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    CC1_MATCH OFFSET(2) NUMBITS(1) []
],
CNT_LFSR [
    PLFSR OFFSET(0) NUMBITS(8) [],
    DLFSR OFFSET(8) NUMBITS(8) [],
    LIMITER OFFSET(16) NUMBITS(3) [
        Limit_1 = 1,
        Limit_2 = 2,
        Limit_3 = 3,
        Limit_4 = 4,
        Limit_5 = 5,
        Limit_6 = 6,
        Limit_7 = 7
    ]
],
CNT_ONE_GF [
    /// Select the glitch filter depth
    GF_DEPTH OFFSET(0) NUMBITS(3) [
        /// Glitch filter is turned off
        DEPTHX0 = 0,
        /// Glitch filter depth is set to 1 x GFPS_DIVBYx of prescalar, where x is 1, 2, 4, 8
        DEPTHX1 = 1,
        /// Glitch filter depth is set to 2 x GFPS_DIVBYx of prescalar, where x is 1, 2, 4, 8
        DEPTHX2 = 2,
        /// Glitch filter depth is set to 4 x GFPS_DIVBYx of prescalar, where x is 1, 2, 4, 8
        DEPTHX4 = 3,
        /// Glitch filter depth is set to 8 x GFPS_DIVBYx of prescalar, where x is 1, 2, 4, 8
        DEPTHX8 = 4,
        /// Glitch filter depth is set to 16 x GFPS_DIVBYx of prescalar, where x is 1, 2, 4, 8
        DEPTHX16 = 5,
        /// Glitch filter depth is set to 32 x GFPS_DIVBYx of prescalar, where x is 1, 2, 4, 8
        DEPTHX32 = 6,
        /// Glitch filter depth is set to 64 x GFPS_DIVBYx of prescalar, where x is 1, 2, 4, 8
        DEPTHX64 = 7
    ],
    /// Select the glitch filter pre-scaling of the selected counter clock
    GFPS_DIV OFFSET(3) NUMBITS(2) [
        /// Glitch filter pre-scaling of the selected counter clock. Divide by 1
        GFPS_DIVBY1 = 0,
        /// Glitch filter pre-scaling of the selected counter clock. Divide by 2
        GFPS_DIVBY2 = 1,
        /// Glitch filter pre-scaling of the selected counter clock. Divide by 4
        GFPS_DIVBY4 = 2,
        /// Glitch filter pre-scaling of the selected counter clock. Divide by 8
        GFPS_DIVBY8 = 3
    ]
],
CNT_TR_ONE_SYNC_BYPASS [
    /// When set='1', bypass the sync stage for the corresponding one to one trigger
    SYNC_BYPASS OFFSET(0) NUMBITS(8) []
],
CNT_HRPWM_CTRL [
    HRPWM_EN OFFSET(0) NUMBITS(1) [],
    DATA_IN_CC0_EN OFFSET(3) NUMBITS(1) [],
    DATA_IN_CC1_EN OFFSET(4) NUMBITS(1) [],
    FREQ_SEL OFFSET(5) NUMBITS(2) [
        /// CLK_OUT = 160MHz
        FREQ_SELX0 = 0,
        /// CLK_OUT = 200 MHz
        FREQ_SELX1 = 1,
        /// CLK_OUT = 240 MHz
        FREQ_SELX2 = 2,
        /// N/A
        FREQ_SELX3 = 3
    ]
],

];
const TCPWM0_BASE: StaticRef<Tcpwm0Registers> =
    unsafe { StaticRef::new(0x42A00000 as *const Tcpwm0Registers) };

pub struct Tcpwm0<'a> {
    registers: StaticRef<Tcpwm0Registers>,
    client: OptionalCell<&'a dyn hil::time::AlarmClient>,
}

impl Tcpwm0<'_> {
    pub const fn new() -> Self {
        Self {
            registers: TCPWM0_BASE,
            client: OptionalCell::empty(),
        }
    }

    pub fn enable_interrupt(&self) {
        self.registers
            .grp0_cnt0
            .intr_mask
            .modify(CNT_INTR_MASK::CC0_MATCH::SET);
        // todo CC1?
    }

    pub fn init_timer(&self) {
        self.registers.grp0_cnt0.ps.modify(CNT_PS::PS_DIV::DIVBY1);
        self.registers.grp0_cnt0.ctrl.modify(
            CNT_CTRL::ONE_SHOT::CLEAR
                + CNT_CTRL::UP_DOWN_MODE::COUNT_UP
                + CNT_CTRL::MODE::TIMER
                + CNT_CTRL::AUTO_RELOAD_CC0::CLEAR
                + CNT_CTRL::SWAP_ENABLE::CLEAR
                + QUAD_ENCODING_MODE::CLEAR,
        );
        self.registers.grp0_cnt0.counter.set(0);

        self.registers.grp0_cnt0.cc0.modify(CNT_CC::CC::SET);
        self.registers
            .grp0_cnt0
            .cc0_buff
            .modify(CNT_CC_BUFF::CC::SET);

        self.registers
            .grp0_cnt0
            .period
            .modify(CNT_PERIOD::PERIOD.val(0xFFFF_FFFF));

        // Disable all special edge detection, use trigger as is.
        self.registers.grp0_cnt0.tr_in_edge_sel.set(0xFFFF_FFFF);

        self.registers
            .grp0_cnt0
            .tr_out_sel
            .modify(CNT_TR_OUT_SEL::OUT0::Disabled + CNT_TR_OUT_SEL::OUT1::Disabled);

        self.registers
            .grp0_cnt0
            .intr_mask
            .write(CNT_INTR_MASK::CC0_MATCH::SET);

        self.registers
            .grp0_cnt0
            .intr
            .write(CNT_INTR::TC::SET + CNT_INTR::CC0_MATCH::SET + CNT_INTR::CC1_MATCH::SET);

        self.registers.grp0_cnt0.ctrl.modify(CNT_CTRL::ENABLED::SET);

        self.registers
            .grp0_cnt0
            .tr_cmd
            .write(CNT_TR_CMD::START::SET);
    }

    pub fn set_timer_ticks(&self, ticks: u32) {
        self.registers
            .grp0_cnt0
            .cc0
            .modify(CNT_CC::CC.val(ticks - 1));
    }

    pub fn disable_interrupt(&self) {
        self.registers
            .grp0_cnt0
            .intr_mask
            .modify(CNT_INTR_MASK::CC0_MATCH::CLEAR);
    }

    pub fn handle_interrupt(&self) {
        if self.registers.grp0_cnt0.intr.is_set(CNT_INTR::CC0_MATCH) {
            self.client.map(|client| client.alarm());
            self.registers
                .grp0_cnt0
                .intr
                .modify(CNT_INTR::CC0_MATCH::SET); // clear bit by writing '1'
        }
    }
}

impl Time for Tcpwm0<'_> {
    type Frequency = time::Freq1MHz;
    type Ticks = time::Ticks32;

    fn now(&self) -> Self::Ticks {
        time::Ticks32::from(self.registers.grp0_cnt0.counter.read(CNT_COUNTER::COUNTER))
    }
}

impl<'a> Alarm<'a> for Tcpwm0<'a> {
    fn set_alarm_client(&self, client: &'a dyn time::AlarmClient) {
        self.client.set(client)
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        let mut expire = reference.wrapping_add(dt);
        let now = self.now();
        if !now.within_range(reference, expire) {
            expire = now;
        }

        if expire.wrapping_sub(now) < self.minimum_dt() {
            expire = now.wrapping_add(self.minimum_dt());
        }

        self.set_timer_ticks(expire.into_u32());
        self.enable_interrupt();
    }

    fn disarm(&self) -> Result<(), kernel::ErrorCode> {
        self.disable_interrupt();
        Ok(())
    }

    fn is_armed(&self) -> bool {
        self.registers
            .grp0_cnt0
            .intr_mask
            .is_set(CNT_INTR_MASK::CC0_MATCH)
    }

    fn minimum_dt(&self) -> Self::Ticks {
        // This is a small arbitrary value.
        Self::Ticks::from(50)
    }

    fn get_alarm(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.grp0_cnt0.cc0.read(CNT_CC::CC))
    }
}
