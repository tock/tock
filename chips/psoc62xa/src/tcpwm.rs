// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025 SRL.

use kernel::hil::{
    self,
    time::{self, Alarm, Ticks, Time},
};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::{
    interfaces::{ReadWriteable, Readable},
    register_bitfields, register_structs, ReadOnly, ReadWrite,
};
use kernel::utilities::StaticRef;

register_structs! {
    Tcpwm0Registers {
        (0x000 => ctrl: ReadWrite<u32, CTRL::Register>),
        (0x004 => ctrl_clr: ReadWrite<u32, CTRL_CLR::Register>),
        (0x008 => ctrl_set: ReadWrite<u32, CTRL_SET::Register>),
        (0x00C => cmd_capture: ReadWrite<u32, CMD_CAPTURE::Register>),
        (0x010 => cmd_reload: ReadWrite<u32, CMD_RELOAD::Register>),
        (0x014 => cmd_stop: ReadWrite<u32, CMD_STOP::Register>),
        (0x018 => cmd_start: ReadWrite<u32, CMD_START::Register>),
        (0x01C => intr_cause: ReadOnly<u32, INTR_CAUSE::Register>),
        (0x020 => _reserved0),
        (0x100 => cnt0_ctrl: ReadWrite<u32, CNT_CTRL::Register>),
        (0x104 => cnt0_status: ReadWrite<u32, CNT_STATUS::Register>),
        (0x108 => cnt0_counter: ReadWrite<u32, CNT_COUNTER::Register>),
        (0x10c => cnt0_cc: ReadWrite<u32, CNT_CC::Register>),
        (0x110 => cnt0_cc_buff: ReadWrite<u32, CNT_CC_BUFF::Register>),
        (0x114 => cnt0_period: ReadWrite<u32, CNT_PERIOD::Register>),
        (0x118 => cnt0_period_buff: ReadWrite<u32, CNT_PERIOD_BUFF::Register>),
        (0x11c => _reserved1),
        (0x120 => cnt0_tr_ctrl0: ReadWrite<u32, CNT_TR_CTRL0::Register>),
        (0x124 => cnt0_tr_ctrl1: ReadWrite<u32, CNT_TR_CTRL1::Register>),
        (0x128 => cnt0_tr_ctrl2: ReadWrite<u32, CNT_TR_CTRL2::Register>),
        (0x12c => _reserved2),
        (0x130 => cnt0_intr: ReadWrite<u32, CNT_INTR::Register>),
        (0x134 => cnt0_intr_set: ReadWrite<u32, CNT_INTR_SET::Register>),
        (0x138 => cnt0_intr_mask: ReadWrite<u32, CNT_INTR_MASK::Register>),
        (0x13c => cnt0_intr_masked: ReadWrite<u32, CNT_INTR_MASKED::Register>),
        (0x140 => @END),
    }
}
register_bitfields![u32,
CTRL [
    COUNTER_ENABLED OFFSET(0) NUMBITS(32) []
],
CTRL_CLR [
    COUNTER_ENABLED OFFSET(0) NUMBITS(32) []
],
CTRL_SET [
    COUNTER_ENABLED OFFSET(0) NUMBITS(32) []
],
CMD_CAPTURE [
    COUNTER_CAPTURE OFFSET(0) NUMBITS(32) []
],
CMD_RELOAD [
    COUNTER_RELOAD OFFSET(0) NUMBITS(32) []
],
CMD_STOP [
    COUNTER_STOP OFFSET(0) NUMBITS(32) []
],
CMD_START [
    COUNTER_START OFFSET(0) NUMBITS(32) []
],
INTR_CAUSE [
    COUNTER_INT OFFSET(0) NUMBITS(32) []
],
CNT_CTRL [
    AUTO_RELOAD_CC OFFSET(0) NUMBITS(1) [],
    AUTO_RELOAD_PERIOD OFFSET(1) NUMBITS(1) [],
    PWM_SYNC_KILL OFFSET(2) NUMBITS(1) [],
    PWM_STOP_ON_KILL OFFSET(3) NUMBITS(1) [],
    GENERIC OFFSET(8) NUMBITS(8) [
        DivBy1 = 0,
        DivBy2 = 1,
        DivBy4 = 2,
        DivBy8 = 3,
        DivBy16 = 4,
        DivBy32 = 5,
        DivBy64 = 6,
        DivBy128 = 7,
    ],
    UP_DOWN_MODE OFFSET(16) NUMBITS(2) [
        Count_UP = 0,
        Count_DOWN = 1,
        Count_UPDN1 = 2,
        Count_UPDN2 = 3,
    ],
    ONE_SHOT OFFSET(18) NUMBITS(1) [],
    QUADRATURE_MODE OFFSET(20) NUMBITS(2) [],
    MODE OFFSET(24) NUMBITS(3) [
        Timer = 0,
        Capture = 2,
        Quad = 3,
        Pwm = 4,
        Pwm_DT = 5,
        Pwm_PR = 6
    ]
],
CNT_STATUS [
    DOWN OFFSET(0) NUMBITS(1) [],
    GENERIC OFFSET(8) NUMBITS(8) [],
    RUNNING OFFSET(31) NUMBITS(1) [],
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
CNT_TR_CTRL0 [
    CAPTURE_SEL OFFSET(0) NUMBITS(4) [],
    COUNT_SEL OFFSET(4) NUMBITS(4) [],
    RELOAD_SEL OFFSET(8) NUMBITS(4) [],
    STOP_SEL OFFSET(12) NUMBITS(4) [],
    START_SEL OFFSET(16) NUMBITS(4) []
],
CNT_TR_CTRL1 [
    CAPTURE_EDGE OFFSET(0) NUMBITS(2) [
        Rising = 0,
        Falling = 1,
        Both = 2,
        NoEdge = 3,
    ],
    COUNT_EDGE OFFSET(2) NUMBITS(2) [
        Rising = 0,
        Falling = 1,
        Both = 2,
        NoEdge = 3,
    ],
    RELOAD_EDGE OFFSET(4) NUMBITS(2) [
        Rising = 0,
        Falling = 1,
        Both = 2,
        NoEdge = 3,
    ],
    STOP_EDGE OFFSET(6) NUMBITS(2) [
        Rising = 0,
        Falling = 1,
        Both = 2,
        NoEdge = 3,
    ],
    START_EDGE OFFSET(8) NUMBITS(2) [
        Rising = 0,
        Falling = 1,
        Both = 2,
        NoEdge = 3,
    ]
],
CNT_TR_CTRL2 [
    CC_MATCH_MODE OFFSET(0) NUMBITS(2) [
        Set = 0,
        Clear = 1,
        Invert = 2,
        NoChange = 3
    ],
    OVERFLOW_MODE OFFSET(2) NUMBITS(2) [
        Set = 0,
        Clear = 1,
        Invert = 2,
        NoChange = 3
    ],
    UNDERFLOW_MODE OFFSET(4) NUMBITS(2) [
        Set = 0,
        Clear = 1,
        Invert = 2,
        NoChange = 3
    ]
],
CNT_INTR [
    TC OFFSET(0) NUMBITS(1) [],
    CC_MATCH OFFSET(1) NUMBITS(1) []
],
CNT_INTR_SET [
    TC OFFSET(0) NUMBITS(1) [],
    CC_MATCH OFFSET(1) NUMBITS(1) []
],
CNT_INTR_MASK [
    TC OFFSET(0) NUMBITS(1) [],
    CC_MATCH OFFSET(1) NUMBITS(1) []
],
CNT_INTR_MASKED [
    TC OFFSET(0) NUMBITS(1) [],
    CC_MATCH OFFSET(1) NUMBITS(1) []
]
];
const TCPWM0_BASE: StaticRef<Tcpwm0Registers> =
    unsafe { StaticRef::new(0x40380000 as *const Tcpwm0Registers) };

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
            .cnt0_intr_mask
            .modify(CNT_INTR_MASK::CC_MATCH::SET);
    }

    pub fn init_timer(&self) {
        self.registers
            .ctrl_clr
            .modify(CTRL_CLR::COUNTER_ENABLED.val(1));
        self.registers.cnt0_ctrl.modify(CNT_CTRL::MODE::Timer);
        self.registers
            .cnt0_period
            .modify(CNT_PERIOD::PERIOD.val(!0));
        self.registers.cnt0_cc.modify(CNT_CC::CC.val(!0));
        self.registers.cnt0_cc_buff.modify(CNT_CC_BUFF::CC.val(0));
        self.registers.cnt0_ctrl.modify(
            CNT_CTRL::AUTO_RELOAD_CC::CLEAR
                + CNT_CTRL::GENERIC::DivBy1
                + CNT_CTRL::UP_DOWN_MODE::Count_UP
                + CNT_CTRL::ONE_SHOT::CLEAR,
        );
        self.registers.cnt0_tr_ctrl0.modify(
            CNT_TR_CTRL0::STOP_SEL::CLEAR
                + CNT_TR_CTRL0::COUNT_SEL.val(1)
                + CNT_TR_CTRL0::CAPTURE_SEL::CLEAR
                + CNT_TR_CTRL0::RELOAD_SEL::CLEAR
                + CNT_TR_CTRL0::START_SEL::CLEAR,
        );
        self.registers
            .ctrl_set
            .modify(CTRL_SET::COUNTER_ENABLED.val(1));
        self.registers
            .cmd_reload
            .modify(CMD_RELOAD::COUNTER_RELOAD.val(1));
        while self.registers.cmd_reload.read(CMD_RELOAD::COUNTER_RELOAD) == 1 {}
    }

    pub fn set_timer_ticks(&self, ticks: u32) {
        self.registers.cnt0_cc.modify(CNT_CC::CC.val(ticks - 1));
    }

    pub fn disable_interrupt(&self) {
        self.registers
            .cnt0_intr_mask
            .modify(CNT_INTR_MASK::CC_MATCH::CLEAR);
    }

    pub fn handle_interrupt(&self) {
        if self.registers.cnt0_intr.is_set(CNT_INTR::CC_MATCH) {
            self.client.map(|client| client.alarm());
            self.registers.cnt0_intr.modify(CNT_INTR::CC_MATCH::SET);
        }
    }
}

impl Time for Tcpwm0<'_> {
    type Frequency = time::Freq1MHz;
    type Ticks = time::Ticks32;

    fn now(&self) -> Self::Ticks {
        time::Ticks32::from(self.registers.cnt0_counter.read(CNT_COUNTER::COUNTER))
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
            .cnt0_intr_mask
            .is_set(CNT_INTR_MASK::CC_MATCH)
    }

    fn minimum_dt(&self) -> Self::Ticks {
        // This is a small arbitrary value.
        Self::Ticks::from(50)
    }

    fn get_alarm(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.cnt0_cc.read(CNT_CC::CC))
    }
}
