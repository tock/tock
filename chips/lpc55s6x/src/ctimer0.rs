// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use cortexm33::support::with_interrupts_disabled;
use kernel::hil;
use kernel::hil::time::{Alarm, Ticks, Ticks32, Time};
use kernel::utilities::cells::{OptionalCell, VolatileCell};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

use crate::interrupts::CTIMER0;

register_structs! {
    /// Standard counter/timers (CTIMER0 to 4)
    Ctimer0Registers {
        /// Interrupt Register. The IR can be written to clear interrupts. The IR can be rea
        (0x000 => ir: ReadWrite<u32, IR::Register>),
        /// Timer Control Register. The TCR is used to control the Timer Counter functions.
        (0x004 => tcr: ReadWrite<u32, TCR::Register>),
        /// Timer Counter
        (0x008 => tc: ReadWrite<u32>),
        /// Prescale Register
        (0x00C => pr: ReadWrite<u32, PR::Register>),
        /// Prescale Counter
        (0x010 => pc: ReadWrite<u32, PC::Register>),
        /// Match Control Register
        (0x014 => mcr: ReadWrite<u32, MCR::Register>),
        /// Match Register . MR can be enabled through the MCR to reset the TC, stop both th
        (0x018 => mr_0: ReadWrite<u32, MR0::Register>),
        /// Match Register . MR can be enabled through the MCR to reset the TC, stop both th
        (0x01C => mr_1: ReadWrite<u32>),
        /// Match Register . MR can be enabled through the MCR to reset the TC, stop both th
        (0x020 => mr_2: ReadWrite<u32>),
        /// Match Register . MR can be enabled through the MCR to reset the TC, stop both th
        (0x024 => mr_3: ReadWrite<u32>),
        /// Capture Control Register. The CCR controls which edges of the capture inputs are
        (0x028 => ccr: ReadWrite<u32, CCR::Register>),
        /// Capture Register . CR is loaded with the value of TC when there is an event on t
        (0x02C => cr_0: ReadOnly<u32>),
        /// Capture Register . CR is loaded with the value of TC when there is an event on t
        (0x030 => cr_1: ReadOnly<u32>),
        /// Capture Register . CR is loaded with the value of TC when there is an event on t
        (0x034 => cr_2: ReadOnly<u32>),
        /// Capture Register . CR is loaded with the value of TC when there is an event on t
        (0x038 => cr_3: ReadOnly<u32>),
        /// External Match Register. The EMR controls the match function and the external ma
        (0x03C => emr: ReadWrite<u32, EMR::Register>),
        (0x040 => _reserved0),
        /// Count Control Register. The CTCR selects between Timer and Counter mode, and in
        (0x070 => ctcr: ReadWrite<u32, CTCR::Register>),
        /// PWM Control Register. This register enables PWM mode for the external match pins
        (0x074 => pwmc: ReadWrite<u32, PWMC::Register>),
        /// Match Shadow Register
        (0x078 => msr_0: ReadWrite<u32>),
        /// Match Shadow Register
        (0x07C => msr_1: ReadWrite<u32>),
        /// Match Shadow Register
        (0x080 => msr_2: ReadWrite<u32>),
        /// Match Shadow Register
        (0x084 => msr_3: ReadWrite<u32>),
        (0x088 => @END),
    }
}
register_bitfields![u32,
IR [
    MR0INT OFFSET(0) NUMBITS(1) [],
    MR1INT OFFSET(1) NUMBITS(1) [],
    MR2INT OFFSET(2) NUMBITS(1) [],
    MR3INT OFFSET(3) NUMBITS(1) [],
    CR0INT OFFSET(4) NUMBITS(1) [],
    CR1INT OFFSET(5) NUMBITS(1) [],
    CR2INT OFFSET(6) NUMBITS(1) [],
    CR3INT OFFSET(7) NUMBITS(1) [],
],
TCR [
    /// Counter enable.
    CEN OFFSET(0) NUMBITS(1) [
        /// Disabled.The counters are disabled.
        DisabledTheCountersAreDisabled = 0,
        /// Enabled. The Timer Counter and Prescale Counter are enabled.
        EnabledTheTimerCounterAndPrescaleCounterAreEnabled = 1
    ],
    /// Counter reset.
    CRST OFFSET(1) NUMBITS(1) [
        /// Disabled. Do nothing.
        DisabledDoNothing = 0,
        /// Enabled. The Timer Counter and the Prescale Counter are synchronously reset on t
        ENABLED = 1
    ]
],
TC [
    /// Timer counter value.
    TCVAL OFFSET(0) NUMBITS(32) []
],
PR [
    /// Prescale counter value.
    PRVAL OFFSET(0) NUMBITS(32) []
],
PC [
    /// Prescale counter value.
    PCVAL OFFSET(0) NUMBITS(32) []
],
MCR [
    /// Interrupt on MR0: an interrupt is generated when MR0 matches the value in the TC
    MR0I OFFSET(0) NUMBITS(1) [],
    /// Reset on MR0: the TC will be reset if MR0 matches it.
    MR0R OFFSET(1) NUMBITS(1) [],
    /// Stop on MR0: the TC and PC will be stopped and TCR[0] will be set to 0 if MR0 ma
    MR0S OFFSET(2) NUMBITS(1) [],
    /// Interrupt on MR1: an interrupt is generated when MR1 matches the value in the TC
    MR1I OFFSET(3) NUMBITS(1) [],
    /// Reset on MR1: the TC will be reset if MR1 matches it.
    MR1R OFFSET(4) NUMBITS(1) [],
    /// Stop on MR1: the TC and PC will be stopped and TCR[0] will be set to 0 if MR1 ma
    MR1S OFFSET(5) NUMBITS(1) [],
    /// Interrupt on MR2: an interrupt is generated when MR2 matches the value in the TC
    MR2I OFFSET(6) NUMBITS(1) [],
    /// Reset on MR2: the TC will be reset if MR2 matches it.
    MR2R OFFSET(7) NUMBITS(1) [],
    /// Stop on MR2: the TC and PC will be stopped and TCR[0] will be set to 0 if MR2 ma
    MR2S OFFSET(8) NUMBITS(1) [],
    /// Interrupt on MR3: an interrupt is generated when MR3 matches the value in the TC
    MR3I OFFSET(9) NUMBITS(1) [],
    /// Reset on MR3: the TC will be reset if MR3 matches it.
    MR3R OFFSET(10) NUMBITS(1) [],
    /// Stop on MR3: the TC and PC will be stopped and TCR[0] will be set to 0 if MR3 ma
    MR3S OFFSET(11) NUMBITS(1) [],
    /// Reload MR0 with the contents of the Match 0 Shadow Register when the TC is reset
    MR0RL OFFSET(24) NUMBITS(1) [],
    /// Reload MR1 with the contents of the Match 1 Shadow Register when the TC is reset
    MR1RL OFFSET(25) NUMBITS(1) [],
    /// Reload MR2 with the contents of the Match 2 Shadow Register when the TC is reset
    MR2RL OFFSET(26) NUMBITS(1) [],
    /// Reload MR3 with the contents of the Match 3 Shadow Register when the TC is reset
    MR3RL OFFSET(27) NUMBITS(1) []
],
CCR [
    /// Rising edge of capture channel 0: a sequence of 0 then 1 causes CR0 to be loaded
    CAP0RE OFFSET(0) NUMBITS(1) [],
    /// Falling edge of capture channel 0: a sequence of 1 then 0 causes CR0 to be loade
    CAP0FE OFFSET(1) NUMBITS(1) [],
    /// Generate interrupt on channel 0 capture event: a CR0 load generates an interrupt
    CAP0I OFFSET(2) NUMBITS(1) [],
    /// Rising edge of capture channel 1: a sequence of 0 then 1 causes CR1 to be loaded
    CAP1RE OFFSET(3) NUMBITS(1) [],
    /// Falling edge of capture channel 1: a sequence of 1 then 0 causes CR1 to be loade
    CAP1FE OFFSET(4) NUMBITS(1) [],
    /// Generate interrupt on channel 1 capture event: a CR1 load generates an interrupt
    CAP1I OFFSET(5) NUMBITS(1) [],
    /// Rising edge of capture channel 2: a sequence of 0 then 1 causes CR2 to be loaded
    CAP2RE OFFSET(6) NUMBITS(1) [],
    /// Falling edge of capture channel 2: a sequence of 1 then 0 causes CR2 to be loade
    CAP2FE OFFSET(7) NUMBITS(1) [],
    /// Generate interrupt on channel 2 capture event: a CR2 load generates an interrupt
    CAP2I OFFSET(8) NUMBITS(1) [],
    /// Rising edge of capture channel 3: a sequence of 0 then 1 causes CR3 to be loaded
    CAP3RE OFFSET(9) NUMBITS(1) [],
    /// Falling edge of capture channel 3: a sequence of 1 then 0 causes CR3 to be loade
    CAP3FE OFFSET(10) NUMBITS(1) [],
    /// Generate interrupt on channel 3 capture event: a CR3 load generates an interrupt
    CAP3I OFFSET(11) NUMBITS(1) []
],
EMR [
    /// External Match 0. This bit reflects the state of output MAT0, whether or not thi
    EM0 OFFSET(0) NUMBITS(1) [],
    /// External Match 1. This bit reflects the state of output MAT1, whether or not thi
    EM1 OFFSET(1) NUMBITS(1) [],
    /// External Match 2. This bit reflects the state of output MAT2, whether or not thi
    EM2 OFFSET(2) NUMBITS(1) [],
    /// External Match 3. This bit reflects the state of output MAT3, whether or not thi
    EM3 OFFSET(3) NUMBITS(1) [],
    /// External Match Control 0. Determines the functionality of External Match 0.
    EMC0 OFFSET(4) NUMBITS(2) [
        /// Do Nothing.
        DoNothing = 0,
        /// Clear. Clear the corresponding External Match bit/output to 0 (MAT0 pin is LOW i
        ClearClearTheCorrespondingExternalMatchBitOutputTo0MAT0PinIsLOWIfPinnedOut = 1,
        /// Set. Set the corresponding External Match bit/output to 1 (MAT0 pin is HIGH if p
        SetSetTheCorrespondingExternalMatchBitOutputTo1MAT0PinIsHIGHIfPinnedOut = 2,
        /// Toggle. Toggle the corresponding External Match bit/output.
        ToggleToggleTheCorrespondingExternalMatchBitOutput = 3
    ],
    /// External Match Control 1. Determines the functionality of External Match 1.
    EMC1 OFFSET(6) NUMBITS(2) [
        /// Do Nothing.
        DoNothing = 0,
        /// Clear. Clear the corresponding External Match bit/output to 0 (MAT1 pin is LOW i
        ClearClearTheCorrespondingExternalMatchBitOutputTo0MAT1PinIsLOWIfPinnedOut = 1,
        /// Set. Set the corresponding External Match bit/output to 1 (MAT1 pin is HIGH if p
        SetSetTheCorrespondingExternalMatchBitOutputTo1MAT1PinIsHIGHIfPinnedOut = 2,
        /// Toggle. Toggle the corresponding External Match bit/output.
        ToggleToggleTheCorrespondingExternalMatchBitOutput = 3
    ],
    /// External Match Control 2. Determines the functionality of External Match 2.
    EMC2 OFFSET(8) NUMBITS(2) [
        /// Do Nothing.
        DoNothing = 0,
        /// Clear. Clear the corresponding External Match bit/output to 0 (MAT2 pin is LOW i
        ClearClearTheCorrespondingExternalMatchBitOutputTo0MAT2PinIsLOWIfPinnedOut = 1,
        /// Set. Set the corresponding External Match bit/output to 1 (MAT2 pin is HIGH if p
        SetSetTheCorrespondingExternalMatchBitOutputTo1MAT2PinIsHIGHIfPinnedOut = 2,
        /// Toggle. Toggle the corresponding External Match bit/output.
        ToggleToggleTheCorrespondingExternalMatchBitOutput = 3
    ],
    /// External Match Control 3. Determines the functionality of External Match 3.
    EMC3 OFFSET(10) NUMBITS(2) [
        /// Do Nothing.
        DoNothing = 0,
        /// Clear. Clear the corresponding External Match bit/output to 0 (MAT3 pin is LOW i
        ClearClearTheCorrespondingExternalMatchBitOutputTo0MAT3PinIsLOWIfPinnedOut = 1,
        /// Set. Set the corresponding External Match bit/output to 1 (MAT3 pin is HIGH if p
        SetSetTheCorrespondingExternalMatchBitOutputTo1MAT3PinIsHIGHIfPinnedOut = 2,
        /// Toggle. Toggle the corresponding External Match bit/output.
        ToggleToggleTheCorrespondingExternalMatchBitOutput = 3
    ]
],
CTCR [
    /// Counter/Timer Mode This field selects which rising APB bus clock edges can incre
    CTMODE OFFSET(0) NUMBITS(2) [
        /// Timer Mode. Incremented every rising APB bus clock edge.
        TimerModeIncrementedEveryRisingAPBBusClockEdge = 0,
        /// Counter Mode rising edge. TC is incremented on rising edges on the CAP input sel
        CounterModeRisingEdgeTCIsIncrementedOnRisingEdgesOnTheCAPInputSelectedByBits32 = 1,
        /// Counter Mode falling edge. TC is incremented on falling edges on the CAP input s
        COUNTER_FALLING_EDGE = 2,
        /// Counter Mode dual edge. TC is incremented on both edges on the CAP input selecte
        CounterModeDualEdgeTCIsIncrementedOnBothEdgesOnTheCAPInputSelectedByBits32 = 3
    ],
    /// Count Input Select When bits 1:0 in this register are not 00, these bits select
    CINSEL OFFSET(2) NUMBITS(2) [
        /// Channel 0. CAPn.0 for CTIMERn
        Channel0CAPn0ForCTIMERn = 0,
        /// Channel 1. CAPn.1 for CTIMERn
        Channel1CAPn1ForCTIMERn = 1,
        /// Channel 2. CAPn.2 for CTIMERn
        Channel2CAPn2ForCTIMERn = 2,
        /// Channel 3. CAPn.3 for CTIMERn
        Channel3CAPn3ForCTIMERn = 3
    ],
    /// Setting this bit to 1 enables clearing of the timer and the prescaler when the c
    ENCC OFFSET(4) NUMBITS(1) [],
    /// Edge select. When bit 4 is 1, these bits select which capture input edge will ca
    SELCC OFFSET(5) NUMBITS(3) [
        /// Channel 0 Rising Edge. Rising edge of the signal on capture channel 0 clears the
        CHANNEL_0_RISING = 0,
        /// Channel 0 Falling Edge. Falling edge of the signal on capture channel 0 clears t
        CHANNEL_0_FALLING = 1,
        /// Channel 1 Rising Edge. Rising edge of the signal on capture channel 1 clears the
        CHANNEL_1_RISING = 2,
        /// Channel 1 Falling Edge. Falling edge of the signal on capture channel 1 clears t
        CHANNEL_1_FALLING = 3,
        /// Channel 2 Rising Edge. Rising edge of the signal on capture channel 2 clears the
        CHANNEL_2_RISING = 4,
        /// Channel 2 Falling Edge. Falling edge of the signal on capture channel 2 clears t
        CHANNEL_2_FALLING = 5
    ]
],
PWMC [
    /// PWM mode enable for channel0.
    PWMEN0 OFFSET(0) NUMBITS(1) [
        /// Match. CTIMERn_MAT0 is controlled by EM0.
        MatchCTIMERn_MAT0IsControlledByEM0 = 0,
        /// PWM. PWM mode is enabled for CTIMERn_MAT0.
        PWMPWMModeIsEnabledForCTIMERn_MAT0 = 1
    ],
    /// PWM mode enable for channel1.
    PWMEN1 OFFSET(1) NUMBITS(1) [
        /// Match. CTIMERn_MAT01 is controlled by EM1.
        MatchCTIMERn_MAT01IsControlledByEM1 = 0,
        /// PWM. PWM mode is enabled for CTIMERn_MAT1.
        PWMPWMModeIsEnabledForCTIMERn_MAT1 = 1
    ],
    /// PWM mode enable for channel2.
    PWMEN2 OFFSET(2) NUMBITS(1) [
        /// Match. CTIMERn_MAT2 is controlled by EM2.
        MatchCTIMERn_MAT2IsControlledByEM2 = 0,
        /// PWM. PWM mode is enabled for CTIMERn_MAT2.
        PWMPWMModeIsEnabledForCTIMERn_MAT2 = 1
    ],
    /// PWM mode enable for channel3. Note: It is recommended to use match channel 3 to
    PWMEN3 OFFSET(3) NUMBITS(1) [
        /// Match. CTIMERn_MAT3 is controlled by EM3.
        MatchCTIMERn_MAT3IsControlledByEM3 = 0,
        /// PWM. PWM mode is enabled for CT132Bn_MAT3.
        PWMPWMModeIsEnabledForCT132Bn_MAT3 = 1
    ]
],
MR0 [
    /// Timer counter match value.
    MATCH OFFSET(0) NUMBITS(32) []
],
MR1 [
    /// Timer counter match value.
    MATCH OFFSET(0) NUMBITS(32) []
],
MR2 [
    /// Timer counter match value.
    MATCH OFFSET(0) NUMBITS(32) []
],
MR3 [
    /// Timer counter match value.
    MATCH OFFSET(0) NUMBITS(32) []
],
CR0 [
    /// Timer counter capture value.
    CAP OFFSET(0) NUMBITS(32) []
],
CR1 [
    /// Timer counter capture value.
    CAP OFFSET(0) NUMBITS(32) []
],
CR2 [
    /// Timer counter capture value.
    CAP OFFSET(0) NUMBITS(32) []
],
CR3 [
    /// Timer counter capture value.
    CAP OFFSET(0) NUMBITS(32) []
],
MSR0 [
    /// Timer counter match shadow value.
    SHADOW OFFSET(0) NUMBITS(32) []
],
MSR1 [
    /// Timer counter match shadow value.
    SHADOW OFFSET(0) NUMBITS(32) []
],
MSR2 [
    /// Timer counter match shadow value.
    SHADOW OFFSET(0) NUMBITS(32) []
],
MSR3 [
    /// Timer counter match shadow value.
    SHADOW OFFSET(0) NUMBITS(32) []
]
];

const CTIMER0_BASE: StaticRef<Ctimer0Registers> =
    unsafe { StaticRef::new(0x50008000 as *const Ctimer0Registers) };

pub struct LPCTimer<'a> {
    registers: StaticRef<Ctimer0Registers>,
    client: OptionalCell<&'a dyn hil::time::AlarmClient>,
    armed: VolatileCell<bool>,
}

impl<'a> LPCTimer<'a> {
    pub const fn new() -> LPCTimer<'a> {
        LPCTimer {
            registers: CTIMER0_BASE,
            client: OptionalCell::empty(),
            armed: VolatileCell::new(false),
        }
    }

    pub fn init(&self, pclk_hz: u32) {
        let pr = pclk_hz / 1_000_000 - 1;

        self.registers.tcr.modify(TCR::CRST::SET);
        self.registers.tcr.modify(TCR::CRST::CLEAR);

        self.registers.tcr.set(0x1);

        self.registers.ctcr.set(0);

        self.registers.pr.set(pr);

        self.registers.pc.set(0);

        self.registers.tcr.modify(TCR::CEN::SET);

        self.registers.mr_0.set(250_000);

        self.registers
            .mcr
            .modify(MCR::MR0R::CLEAR + MCR::MR0S::CLEAR + MCR::MR0I::CLEAR);

        self.registers.ir.modify(IR::MR0INT::SET);
    }

    fn enable_interrupt(&self) {
        self.registers.mcr.modify(MCR::MR0I::SET);
    }

    fn disable_interrupt(&self) {
        self.registers.mcr.modify(MCR::MR0I::CLEAR);
    }

    fn enable_timer_interrupt(&self) {
        unsafe {
            with_interrupts_disabled(|| {
                let n = cortexm33::nvic::Nvic::new(CTIMER0);
                n.enable();
            })
        }
    }

    #[allow(dead_code)]
    fn disable_timer_interrupt(&self) {
        unsafe {
            cortexm33::nvic::Nvic::new(CTIMER0).disable();
        }
    }

    pub fn handle_interrupt(&self) {
        self.registers.ir.modify(IR::MR0INT::SET);

        self.armed.set(false);

        self.client.map(|client| client.alarm());
    }

    pub fn get_pr(&self) -> u32 {
        self.registers.pr.get()
    }
}

impl Time for LPCTimer<'_> {
    type Frequency = hil::time::Freq1MHz;
    type Ticks = Ticks32;

    fn now(&self) -> Self::Ticks {
        self.registers.tcr.set(0x1);

        Self::Ticks::from(self.registers.tc.get())
    }
}

impl<'a> Alarm<'a> for LPCTimer<'a> {
    fn set_alarm_client(&self, client: &'a dyn hil::time::AlarmClient) {
        self.client.set(client)
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        let mut expire = reference.wrapping_add(dt);
        let now = self.now();

        if !now.within_range(reference, expire) {
            expire = now;
        }

        let min = self.minimum_dt();
        if expire.wrapping_sub(now) < min {
            expire = now.wrapping_add(min);
        }

        self.registers.mr_0.set(expire.into_u32());

        self.registers.ir.modify(IR::MR0INT::SET);

        self.enable_interrupt();
        self.enable_timer_interrupt();

        self.armed.set(true);
    }

    fn get_alarm(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.mr_0.get())
    }

    fn disarm(&self) -> Result<(), kernel::ErrorCode> {
        self.disable_interrupt();
        self.armed.set(false);

        unsafe {
            with_interrupts_disabled(|| {
                cortexm33::nvic::Nvic::new(CTIMER0).clear_pending();
            });
        }

        Ok(())
    }

    fn is_armed(&self) -> bool {
        self.armed.get()
    }

    fn minimum_dt(&self) -> Self::Ticks {
        Self::Ticks::from(50)
    }
}
