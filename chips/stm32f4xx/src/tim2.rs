// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use cortexm4f::support::atomic;
use kernel::hil::time::{
    Alarm, AlarmClient, Counter, Freq16KHz, Frequency, OverflowClient, Ticks, Ticks32, Time,
};
use kernel::platform::chip::ClockInterface;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

use crate::clocks::{phclk, Stm32f4Clocks};
use crate::nvic;

/// General purpose timers
#[repr(C)]
struct Tim2Registers {
    /// control register 1
    cr1: ReadWrite<u32, CR1::Register>,
    /// control register 2
    cr2: ReadWrite<u32, CR2::Register>,
    /// slave mode control register
    smcr: ReadWrite<u32, SMCR::Register>,
    /// DMA/Interrupt enable register
    dier: ReadWrite<u32, DIER::Register>,
    /// status register
    sr: ReadWrite<u32, SR::Register>,
    /// event generation register
    egr: WriteOnly<u32, EGR::Register>,
    /// capture/compare mode register 1 (output mode)
    ccmr1_output: ReadWrite<u32, CCMR1_Output::Register>,
    /// capture/compare mode register 2 (output mode)
    ccmr2_output: ReadWrite<u32, CCMR2_Output::Register>,
    /// capture/compare enable register
    ccer: ReadWrite<u32, CCER::Register>,
    /// counter
    cnt: ReadWrite<u32, CNT::Register>,
    /// prescaler
    psc: ReadWrite<u32>,
    /// auto-reload register
    arr: ReadWrite<u32, ARR::Register>,
    _reserved0: [u8; 4],
    /// capture/compare register 1
    ccr1: ReadWrite<u32, CCR1::Register>,
    /// capture/compare register 2
    ccr2: ReadWrite<u32, CCR2::Register>,
    /// capture/compare register 3
    ccr3: ReadWrite<u32, CCR3::Register>,
    /// capture/compare register 4
    ccr4: ReadWrite<u32, CCR4::Register>,
    _reserved1: [u8; 4],
    /// DMA control register
    dcr: ReadWrite<u32, DCR::Register>,
    /// DMA address for full transfer
    dmar: ReadWrite<u32>,
    /// TIM5 option register
    or_: ReadWrite<u32>,
}

register_bitfields![u32,
    CR1 [
        /// Clock division
        CKD OFFSET(8) NUMBITS(2) [],
        /// Auto-reload preload enable
        ARPE OFFSET(7) NUMBITS(1) [],
        /// Center-aligned mode selection
        CMS OFFSET(5) NUMBITS(2) [],
        /// Direction
        DIR OFFSET(4) NUMBITS(1) [],
        /// One-pulse mode
        OPM OFFSET(3) NUMBITS(1) [],
        /// Update request source
        URS OFFSET(2) NUMBITS(1) [],
        /// Update disable
        UDIS OFFSET(1) NUMBITS(1) [],
        /// Counter enable
        CEN OFFSET(0) NUMBITS(1) []
    ],
    CR2 [
        /// TI1 selection
        TI1S OFFSET(7) NUMBITS(1) [],
        /// Master mode selection
        MMS OFFSET(4) NUMBITS(3) [],
        /// Capture/compare DMA selection
        CCDS OFFSET(3) NUMBITS(1) []
    ],
    SMCR [
        /// External trigger polarity
        ETP OFFSET(15) NUMBITS(1) [],
        /// External clock enable
        ECE OFFSET(14) NUMBITS(1) [],
        /// External trigger prescaler
        ETPS OFFSET(12) NUMBITS(2) [],
        /// External trigger filter
        ETF OFFSET(8) NUMBITS(4) [],
        /// Master/Slave mode
        MSM OFFSET(7) NUMBITS(1) [],
        /// Trigger selection
        TS OFFSET(4) NUMBITS(3) [],
        /// Slave mode selection
        SMS OFFSET(0) NUMBITS(3) []
    ],
    DIER [
        /// Trigger DMA request enable
        TDE OFFSET(14) NUMBITS(1) [],
        /// Capture/Compare 4 DMA request enable
        CC4DE OFFSET(12) NUMBITS(1) [],
        /// Capture/Compare 3 DMA request enable
        CC3DE OFFSET(11) NUMBITS(1) [],
        /// Capture/Compare 2 DMA request enable
        CC2DE OFFSET(10) NUMBITS(1) [],
        /// Capture/Compare 1 DMA request enable
        CC1DE OFFSET(9) NUMBITS(1) [],
        /// Update DMA request enable
        UDE OFFSET(8) NUMBITS(1) [],
        /// Trigger interrupt enable
        TIE OFFSET(6) NUMBITS(1) [],
        /// Capture/Compare 4 interrupt enable
        CC4IE OFFSET(4) NUMBITS(1) [],
        /// Capture/Compare 3 interrupt enable
        CC3IE OFFSET(3) NUMBITS(1) [],
        /// Capture/Compare 2 interrupt enable
        CC2IE OFFSET(2) NUMBITS(1) [],
        /// Capture/Compare 1 interrupt enable
        CC1IE OFFSET(1) NUMBITS(1) [],
        /// Update interrupt enable
        UIE OFFSET(0) NUMBITS(1) []
    ],
    SR [
        /// Capture/Compare 4 overcapture flag
        CC4OF OFFSET(12) NUMBITS(1) [],
        /// Capture/Compare 3 overcapture flag
        CC3OF OFFSET(11) NUMBITS(1) [],
        /// Capture/compare 2 overcapture flag
        CC2OF OFFSET(10) NUMBITS(1) [],
        /// Capture/Compare 1 overcapture flag
        CC1OF OFFSET(9) NUMBITS(1) [],
        /// Trigger interrupt flag
        TIF OFFSET(6) NUMBITS(1) [],
        /// Capture/Compare 4 interrupt flag
        CC4IF OFFSET(4) NUMBITS(1) [],
        /// Capture/Compare 3 interrupt flag
        CC3IF OFFSET(3) NUMBITS(1) [],
        /// Capture/Compare 2 interrupt flag
        CC2IF OFFSET(2) NUMBITS(1) [],
        /// Capture/compare 1 interrupt flag
        CC1IF OFFSET(1) NUMBITS(1) [],
        /// Update interrupt flag
        UIF OFFSET(0) NUMBITS(1) []
    ],
    EGR [
        /// Trigger generation
        TG OFFSET(6) NUMBITS(1) [],
        /// Capture/compare 4 generation
        CC4G OFFSET(4) NUMBITS(1) [],
        /// Capture/compare 3 generation
        CC3G OFFSET(3) NUMBITS(1) [],
        /// Capture/compare 2 generation
        CC2G OFFSET(2) NUMBITS(1) [],
        /// Capture/compare 1 generation
        CC1G OFFSET(1) NUMBITS(1) [],
        /// Update generation
        UG OFFSET(0) NUMBITS(1) []
    ],
    CCMR1_Output [
        /// OC2CE
        OC2CE OFFSET(15) NUMBITS(1) [],
        /// OC2M
        OC2M OFFSET(12) NUMBITS(3) [],
        /// OC2PE
        OC2PE OFFSET(11) NUMBITS(1) [],
        /// OC2FE
        OC2FE OFFSET(10) NUMBITS(1) [],
        /// CC2S
        CC2S OFFSET(8) NUMBITS(2) [],
        /// OC1CE
        OC1CE OFFSET(7) NUMBITS(1) [],
        /// OC1M
        OC1M OFFSET(4) NUMBITS(3) [],
        /// OC1PE
        OC1PE OFFSET(3) NUMBITS(1) [],
        /// OC1FE
        OC1FE OFFSET(2) NUMBITS(1) [],
        /// CC1S
        CC1S OFFSET(0) NUMBITS(2) []
    ],
    CCMR1_Input [
        /// Input capture 2 filter
        IC2F OFFSET(12) NUMBITS(4) [],
        /// Input capture 2 prescaler
        IC2PCS OFFSET(10) NUMBITS(2) [],
        /// Capture/Compare 2 selection
        CC2S OFFSET(8) NUMBITS(2) [],
        /// Input capture 1 filter
        IC1F OFFSET(4) NUMBITS(4) [],
        /// Input capture 1 prescaler
        ICPCS OFFSET(2) NUMBITS(2) [],
        /// Capture/Compare 1 selection
        CC1S OFFSET(0) NUMBITS(2) []
    ],
    CCMR2_Output [
        /// O24CE
        O24CE OFFSET(15) NUMBITS(1) [],
        /// OC4M
        OC4M OFFSET(12) NUMBITS(3) [],
        /// OC4PE
        OC4PE OFFSET(11) NUMBITS(1) [],
        /// OC4FE
        OC4FE OFFSET(10) NUMBITS(1) [],
        /// CC4S
        CC4S OFFSET(8) NUMBITS(2) [],
        /// OC3CE
        OC3CE OFFSET(7) NUMBITS(1) [],
        /// OC3M
        OC3M OFFSET(4) NUMBITS(3) [],
        /// OC3PE
        OC3PE OFFSET(3) NUMBITS(1) [],
        /// OC3FE
        OC3FE OFFSET(2) NUMBITS(1) [],
        /// CC3S
        CC3S OFFSET(0) NUMBITS(2) []
    ],
    CCMR2_Input [
        /// Input capture 4 filter
        IC4F OFFSET(12) NUMBITS(4) [],
        /// Input capture 4 prescaler
        IC4PSC OFFSET(10) NUMBITS(2) [],
        /// Capture/Compare 4 selection
        CC4S OFFSET(8) NUMBITS(2) [],
        /// Input capture 3 filter
        IC3F OFFSET(4) NUMBITS(4) [],
        /// Input capture 3 prescaler
        IC3PSC OFFSET(2) NUMBITS(2) [],
        /// Capture/compare 3 selection
        CC3S OFFSET(0) NUMBITS(2) []
    ],
    CCER [
        /// Capture/Compare 4 output Polarity
        CC4NP OFFSET(15) NUMBITS(1) [],
        /// Capture/Compare 3 output Polarity
        CC4P OFFSET(13) NUMBITS(1) [],
        /// Capture/Compare 4 output enable
        CC4E OFFSET(12) NUMBITS(1) [],
        /// Capture/Compare 3 output Polarity
        CC3NP OFFSET(11) NUMBITS(1) [],
        /// Capture/Compare 3 output Polarity
        CC3P OFFSET(9) NUMBITS(1) [],
        /// Capture/Compare 3 output enable
        CC3E OFFSET(8) NUMBITS(1) [],
        /// Capture/Compare 2 output Polarity
        CC2NP OFFSET(7) NUMBITS(1) [],
        /// Capture/Compare 2 output Polarity
        CC2P OFFSET(5) NUMBITS(1) [],
        /// Capture/Compare 2 output enable
        CC2E OFFSET(4) NUMBITS(1) [],
        /// Capture/Compare 1 output Polarity
        CC1NP OFFSET(3) NUMBITS(1) [],
        /// Capture/Compare 1 output Polarity
        CC1P OFFSET(1) NUMBITS(1) [],
        /// Capture/Compare 1 output enable
        CC1E OFFSET(0) NUMBITS(1) []
    ],
    CNT [
        /// High counter value
        CNT_H OFFSET(16) NUMBITS(16) [],
        /// Low counter value
        CNT_L OFFSET(0) NUMBITS(16) []
    ],
    ARR [
        /// High Auto-reload value
        ARR_H OFFSET(16) NUMBITS(16) [],
        /// Low Auto-reload value
        ARR_L OFFSET(0) NUMBITS(16) []
    ],
    CCR1 [
        /// High Capture/Compare 1 value
        CCR1_H OFFSET(16) NUMBITS(16) [],
        /// Low Capture/Compare 1 value
        CCR1_L OFFSET(0) NUMBITS(16) []
    ],
    CCR2 [
        /// High Capture/Compare 2 value
        CCR2_H OFFSET(16) NUMBITS(16) [],
        /// Low Capture/Compare 2 value
        CCR2_L OFFSET(0) NUMBITS(16) []
    ],
    CCR3 [
        /// High Capture/Compare value
        CCR3_H OFFSET(16) NUMBITS(16) [],
        /// Low Capture/Compare value
        CCR3_L OFFSET(0) NUMBITS(16) []
    ],
    CCR4 [
        /// High Capture/Compare value
        CCR4_H OFFSET(16) NUMBITS(16) [],
        /// Low Capture/Compare value
        CCR4_L OFFSET(0) NUMBITS(16) []
    ],
    DCR [
        /// DMA burst length
        DBL OFFSET(8) NUMBITS(5) [],
        /// DMA base address
        DBA OFFSET(0) NUMBITS(5) []
    ]
];

const TIM2_BASE: StaticRef<Tim2Registers> =
    unsafe { StaticRef::new(0x40000000 as *const Tim2Registers) };

pub struct Tim2<'a> {
    registers: StaticRef<Tim2Registers>,
    clock: Tim2Clock<'a>,
    client: OptionalCell<&'a dyn AlarmClient>,
    irqn: u32,
}

impl<'a> Tim2<'a> {
    pub const fn new(clocks: &'a dyn Stm32f4Clocks) -> Self {
        Self {
            registers: TIM2_BASE,
            clock: Tim2Clock(phclk::PeripheralClock::new(
                phclk::PeripheralClockType::APB1(phclk::PCLK1::TIM2),
                clocks,
            )),
            client: OptionalCell::empty(),
            irqn: nvic::TIM2,
        }
    }

    pub fn is_enabled_clock(&self) -> bool {
        self.clock.is_enabled()
    }

    pub fn enable_clock(&self) {
        self.clock.enable();
    }

    pub fn disable_clock(&self) {
        self.clock.disable();
    }

    pub fn handle_interrupt(&self) {
        self.registers.sr.modify(SR::CC1IF::CLEAR);

        self.client.map(|client| client.alarm());
    }

    // starts the timer
    pub fn start(&self) {
        // Before calling set_alarm, we assume clock to TIM2 has been
        // enabled.

        self.registers.arr.set(0xFFFF_FFFF - 1);
        self.calibrate();
    }

    // set up the prescaler for the target frequency (16KHz)
    pub fn calibrate(&self) {
        let clk_freq = self.clock.0.get_frequency();

        // TIM2 uses PCLK1. Set the prescaler to the current PCLK1 frequency divided by the wanted
        // frequency (16KHz).
        // WARNING: When PCLK1 is not a multiple of 16KHz (e.g. PCLK1 == 25MHz), the prescaler is
        // the truncated division result, which would cause loss of timer precision
        // TODO: We could use a 1KHz or 1MHz frequency instead of 16KHz to cover most clock frequencies
        // or use a parametric frequency (generic/argument)
        let psc = clk_freq / Freq16KHz::frequency();
        self.registers.psc.set(psc - 1);

        // We need set EGR.UG in order for the prescale value to become active.
        self.registers.egr.write(EGR::UG::SET);
        self.registers.cr1.modify(CR1::CEN::SET);
    }

    // get the value of the cnt register
    pub fn get_timer_cnt(&self) -> u32 {
        self.registers.cnt.get()
    }

    // set the value of the cnt register
    pub fn set_timer_cnt(&self, value: u32) {
        self.registers.cnt.set(value);
    }
}

impl Time for Tim2<'_> {
    type Frequency = Freq16KHz;
    type Ticks = Ticks32;

    fn now(&self) -> Ticks32 {
        Ticks32::from(self.registers.cnt.get())
    }
}

impl<'a> Counter<'a> for Tim2<'a> {
    fn set_overflow_client(&self, _client: &'a dyn OverflowClient) {}

    // starts the timer
    fn start(&self) -> Result<(), ErrorCode> {
        self.start();
        Ok(())
    }

    fn stop(&self) -> Result<(), ErrorCode> {
        self.registers.cr1.modify(CR1::CEN::CLEAR);
        self.registers.sr.modify(SR::CC1IF::CLEAR);
        Ok(())
    }

    fn reset(&self) -> Result<(), ErrorCode> {
        self.registers.cnt.set(0);
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.registers.cr1.is_set(CR1::CEN)
    }
}

impl<'a> Alarm<'a> for Tim2<'a> {
    fn set_alarm_client(&self, client: &'a dyn AlarmClient) {
        self.client.set(client);
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

        let _ = self.disarm();
        self.registers.ccr1.set(expire.into_u32());
        self.registers.dier.modify(DIER::CC1IE::SET);
    }

    fn get_alarm(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.ccr1.get())
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        unsafe {
            atomic(|| {
                // Disable counter
                self.registers.dier.modify(DIER::CC1IE::CLEAR);
                cortexm4f::nvic::Nvic::new(self.irqn).clear_pending();
            });
        }
        Ok(())
    }

    fn is_armed(&self) -> bool {
        // If counter is enabled, then CC1IE is set
        self.registers.dier.is_set(DIER::CC1IE)
    }

    fn minimum_dt(&self) -> Self::Ticks {
        Self::Ticks::from(1)
    }
}

struct Tim2Clock<'a>(phclk::PeripheralClock<'a>);

impl ClockInterface for Tim2Clock<'_> {
    fn is_enabled(&self) -> bool {
        self.0.is_enabled()
    }

    fn enable(&self) {
        self.0.enable();
    }

    fn disable(&self) {
        self.0.disable();
    }
}
