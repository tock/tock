// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

use kernel::ErrorCode;
use kernel::hil::time::Time;
use kernel::hil::time::{self, Ticks, Ticks32};
use kernel::hil::{self};
use kernel::utilities::StaticRef;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{ReadWrite, WriteOnly, register_bitfields, register_structs};

use crate::gpio::{Mode, Pin};

register_structs! {
    pub TimRegisters {
        /// control register 1
        (0x000 => cr1: ReadWrite<u32, CR1::Register>),
        /// control register 2
        (0x004 => cr2: ReadWrite<u32, CR2::Register>),
        /// slave mode control register
        (0x008 => smcr: ReadWrite<u32, SMCR::Register>),
        /// DMA/Interrupt enable register
        (0x00C => dier: ReadWrite<u32, DIER::Register>),
        /// status register
        (0x010 => sr: ReadWrite<u32, SR::Register>),
        /// event generation register
        (0x014 => egr: WriteOnly<u32, EGR::Register>),
        /// capture/compare mode register 1 (output mode)
        (0x018 => ccmr1_output: ReadWrite<u32, CCMR1_Output::Register>),
        /// capture/compare mode register 2 (output mode)
        (0x01C => ccmr2_output: ReadWrite<u32, CCMR2_Output::Register>),
        /// capture/compare enable register
        (0x020 => ccer: ReadWrite<u32, CCER::Register>),
        /// counter
        (0x024 => cnt: ReadWrite<u32, CNT::Register>),
        /// prescaler
        (0x028 => psc: ReadWrite<u32,PSC::Register>),
        /// auto-reload register
        (0x02C => arr: ReadWrite<u32, ARR::Register>),
        (0x030 => _reserved0),
        /// capture/compare register 1
        (0x034 => ccr1: ReadWrite<u32, CCR1::Register>),
        /// capture/compare register 2
        (0x038 => ccr2: ReadWrite<u32, CCR2::Register>),
        /// capture/compare register 3
        (0x03C => ccr3: ReadWrite<u32, CCR3::Register>),
        /// capture/compare register 4
        (0x040 => ccr4: ReadWrite<u32, CCR4::Register>),
        (0x044 => _reserved1),
        /// DMA address for full transfer
        (0x058 => ecr: ReadWrite<u32, ECR::Register>),
        /// timer input selection register
        (0x05C => tisel: ReadWrite<u32, TISEL::Register>),
        /// alternate function register 1
        (0x060 => af1: ReadWrite<u32, AF1::Register>),
        /// alternate function register 2
        (0x064 => af2: ReadWrite<u32, AF2::Register>),
        (0x068 => _reserved2),
        /// DMA control register
        (0x3DC => dcr: ReadWrite<u32, DCR::Register>),
        /// DMA address for full transfer
        (0x3E0 => dmar: ReadWrite<u32,DMAR::Register>),
        (0x3E4 => @END),
    }
}

pub const TIM2_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x50000000 as *const TimRegisters) };

register_bitfields![u32,
    CR1 [
        /// Dithering Enable
        DITHEN OFFSET(12) NUMBITS(1) [],
        /// UIF status bit remapping
        UIFREMAP OFFSET(11) NUMBITS(1) [],
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
        /// Master mode selection
        MMS_3 OFFSET(25) NUMBITS(1) [],
        /// TI1 selection
        TI1S OFFSET(7) NUMBITS(1) [],
        /// Master mode selection
        MMS OFFSET(4) NUMBITS(3) [],
        /// Capture/compare DMA selection
        CCDS OFFSET(3) NUMBITS(1) []
    ],
    SMCR [
        /// SMS preload source
        SMSPS OFFSET(25) NUMBITS(1) [],
        /// SMS preload enable
        SMSPE OFFSET(24) NUMBITS(1) [],
        /// Trigger selection
        TS_4_3 OFFSET(20) NUMBITS(2) [],
        /// Slave mode selection - bit 3
        SMS_bit3 OFFSET(16) NUMBITS(1) [],
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
        TS_2_0 OFFSET(4) NUMBITS(3) [],
        /// OCREF clear selection
        OCCS OFFSET(3) NUMBITS(1) [],
        /// Slave mode selection
        SMS OFFSET(0) NUMBITS(3) []
    ],
    DIER [
        /// Transition error interrupt enable
        TERRIE OFFSET(23) NUMBITS(1) [],
        /// Index error interrupt enable
        IERRIE OFFSET(22) NUMBITS(1) [],
        /// Direction change interrupt enable
        DIRIE OFFSET(21) NUMBITS(1) [],
        /// Index interrupt enable
        IDXIE OFFSET(20) NUMBITS(1) [],
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
        /// Transition error interrupt flag
        TERRF OFFSET(23) NUMBITS(1) [],
        /// Index error interrupt flag
        IERRF OFFSET(22) NUMBITS(1) [],
        /// Direction change interrupt flag
        DIRF OFFSET(21) NUMBITS(1) [],
        /// Index interrupt flag
        IDXF OFFSET(20) NUMBITS(1) [],
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
        /// Output Compare 2 mode - bit 3
        OC2M_bit3 OFFSET(24) NUMBITS(1) [],
        /// Output Compare 1 mode - bit 3
        OC1M_bit3 OFFSET(16) NUMBITS(1) [],
        /// Output compare 2 clear enable
        OC2CE OFFSET(15) NUMBITS(1) [],
        /// Output compare 2 mode
        OC2M OFFSET(12) NUMBITS(3) [
            PwmMode1 = 0b110,
        ],
        /// Output compare 2 preload enable
        OC2PE OFFSET(11) NUMBITS(1) [],
        /// Output compare 2 fast enable
        OC2FE OFFSET(10) NUMBITS(1) [],
        /// Capture/Compare 2 selection
        CC2S OFFSET(8) NUMBITS(2) [],
        /// Output compare 1 clear enable
        OC1CE OFFSET(7) NUMBITS(1) [],
        /// Output compare 1 mode
        OC1M OFFSET(4) NUMBITS(3) [
        PwmMode1 = 0b110,
        ],
        /// Output compare 1 preload enable
        OC1PE OFFSET(3) NUMBITS(1) [],
        /// Output compare 1 fast enable
        OC1FE OFFSET(2) NUMBITS(1) [],
        /// Capture/Compare 1 selection
        CC1S OFFSET(0) NUMBITS(2) []
    ],
    CCMR1_Input [
        /// Input capture 2 filter
        IC2F OFFSET(12) NUMBITS(4) [],
        /// Input capture 2 prescaler
        IC2PSC OFFSET(10) NUMBITS(2) [],
        /// Capture/compare 2 selection
        CC2S OFFSET(8) NUMBITS(2) [],
        /// Input capture 1 filter
        IC1F OFFSET(4) NUMBITS(4) [],
        /// Input capture 1 prescaler
        IC1PSC OFFSET(2) NUMBITS(2) [],
        /// Capture/Compare 1 selection
        CC1S OFFSET(0) NUMBITS(2) []
    ],
    CCMR2_Output [
        /// Output Compare 2 mode - bit 3
        OC4M_bit3 OFFSET(24) NUMBITS(1) [],
        /// Output Compare 1 mode - bit 3
        OC3M_bit3 OFFSET(16) NUMBITS(1) [],
        /// Output compare 4 clear enable
        OC4CE OFFSET(15) NUMBITS(1) [],
        /// Output compare 4 mode
        OC4M OFFSET(12) NUMBITS(3) [
            PwmMode1 = 0b110,
        ],
        /// Output compare 4 preload enable
        OC4PE OFFSET(11) NUMBITS(1) [],
        /// Output compare 4 fast enable
        OC4FE OFFSET(10) NUMBITS(1) [],
        /// Capture/Compare 4 selection
        CC4S OFFSET(8) NUMBITS(2) [],
        /// Output compare 3 clear enable
        OC3CE OFFSET(7) NUMBITS(1) [],
        /// Output compare 3 mode
        OC3M OFFSET(4) NUMBITS(3) [
            PwmMode1 = 0b110,
        ],
        /// Output compare 3 preload enable
        OC3PE OFFSET(3) NUMBITS(1) [],
        /// Output compare 3 fast enable
        OC3FE OFFSET(2) NUMBITS(1) [],
        /// Capture/Compare 3 selection
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
        /// Capture/Compare 3 selection
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
        CNT OFFSET(16) NUMBITS(32) []
    ],
    PSC [
        /// Prescaler value
        PSC OFFSET(0) NUMBITS(16) []
    ],
    ARR [
        ARR OFFSET(0) NUMBITS(32) []
    ],
    CCR1 [
        CCR1 OFFSET(0) NUMBITS(32) []
    ],
    CCR2 [
        /// High Capture/Compare 2 value (TIM2 only)
        CCR2_H OFFSET(16) NUMBITS(16) [],
        /// Low Capture/Compare 2 value
        CCR2_L OFFSET(0) NUMBITS(16) []
    ],
    CCR3 [
        /// High Capture/Compare value (TIM2 only)
        CCR3_H OFFSET(16) NUMBITS(16) [],
        /// Low Capture/Compare value
        CCR3_L OFFSET(0) NUMBITS(16) []
    ],
    CCR4 [
        /// High Capture/Compare value (TIM2 only)
        CCR4_H OFFSET(16) NUMBITS(16) [],
        /// Low Capture/Compare value
        CCR4_L OFFSET(0) NUMBITS(16) []
    ],
    ECR [
        /// Pulse width prescaler
        PWPRSC OFFSET(24) NUMBITS(3) [],
        /// Pulse width
        PW OFFSET(16) NUMBITS(8) [],
        /// Index positioning
        IPOS OFFSET(6) NUMBITS(2) [],
        /// First index
        FIDX OFFSET(5) NUMBITS(1) [],
        /// Index direction
        IDIR OFFSET(1) NUMBITS(2) [],
        /// Index enable
        IE OFFSET(0) NUMBITS(1) []
    ],
    TISEL [
        /// Selects tim_ti4[0..15] input
        TI4SEL OFFSET(24) NUMBITS(4) [],
        /// Selects tim_ti3[0..15] input
        TI3SEL OFFSET(16) NUMBITS(4) [],
        /// Selects tim_ti2[0..15] input
        TI2SEL OFFSET(8) NUMBITS(4) [],
        /// Selects tim_ti1[0..15] input
        TI1SEL OFFSET(0) NUMBITS(4) []
    ],
    AF1 [
        /// etr_in source selection
        ETRSEL OFFSET(14) NUMBITS(4) []
    ],
    AF2 [
        /// ocref_clr source selection
        OCRSEL OFFSET(16) NUMBITS(3) []
    ],
    DCR [
        /// DMA burst source selection
        DBSS OFFSET(16) NUMBITS(4) [],
        /// DMA burst length
        DBL OFFSET(8) NUMBITS(5) [],
        /// DMA base address
        DBA OFFSET(0) NUMBITS(5) []
    ],
    DMAR [
        /// DMA register for burst accesses
        ETRSEL OFFSET(0) NUMBITS(32) []
    ]
];

/// TIM2 hardware driver for the STM32U5.
///
/// This driver implements the Tock Alarm HIL using the 32-bit general-purpose
/// TIM2 timer. It is configured to run at 32kHz to provide high-resolution
/// timing while remaining power-efficient.
pub struct Tim2<'a> {
    registers: StaticRef<TimRegisters>,
    enable_clock: fn(),
    client: OptionalCell<&'a dyn time::AlarmClient>,
}

impl<'a> Tim2<'a> {
    /// Creates a new instance of the driver.
    ///
    /// - `base`: The StaticRef pointing to the MMIO base address of the peripheral.
    /// - `enable_clock`: (For Timers) A callback function to power on the peripheral via RCC.
    pub const fn new(base: StaticRef<TimRegisters>, enable_clock: fn()) -> Tim2<'a> {
        Tim2 {
            registers: base,
            enable_clock,
            client: OptionalCell::empty(),
        }
    }

    fn enable_clock(&self) {
        (self.enable_clock)();
    }

    /// Core interrupt handler for the peripheral.
    ///
    /// This function must be called from the chip's main interrupt service routine
    /// (located in `chip.rs`) whenever the corresponding IRQ fires. It
    /// identifies the cause of the interrupt, clears the relevant hardware
    /// pending flags, and notifies any registered clients.
    pub fn handle_interrupt(&self) {
        // Clear interrupt flag
        self.registers.sr.modify(SR::CC1IF::CLEAR);

        self.client.map(|client| {
            client.alarm();
        });
    }

    /// Initializes and starts the timer hardware.
    ///
    /// This sets the prescaler to 124 (converting the 4MHz clock to 32kHz)
    /// and enables the 32-bit free-running counter.
    pub fn start(&self) {
        self.enable_clock();

        // 1. Set the value
        self.registers.psc.write(PSC::PSC.val(124));

        // 2. Force the hardware to load the value NOW
        // On STM32, the PSC is buffered. By setting the UG bit in EGR,
        self.registers.egr.write(EGR::UG::SET);

        // 3. Clear the status flag caused by the manual update
        self.registers.sr.modify(SR::UIF::CLEAR);

        self.registers.arr.write(ARR::ARR.val(0xFFFFFFFF));
        self.registers.cr1.modify(CR1::CEN::SET);
    }
}

impl time::Time for Tim2<'_> {
    type Frequency = time::Freq32KHz;
    type Ticks = Ticks32;

    fn now(&self) -> Ticks32 {
        Ticks32::from(self.registers.cnt.read(CNT::CNT))
    }
}

impl<'a> time::Alarm<'a> for Tim2<'a> {
    fn set_alarm_client(&self, client: &'a dyn time::AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, reference: Ticks32, dt: Ticks32) {
        // 1. Calculate the raw target time
        let mut expire = reference.wrapping_add(dt);
        let now = self.now();

        // 2. The "Past Check": If the target is behind us, clamp it to 'now'
        if !now.within_range(reference, expire) {
            expire = now;
        }

        // 3. The "Minimum Delay": If the alarm is too close to now,
        // push it forward slightly to give the CPU time to finish this function.
        if expire.wrapping_sub(now) < self.minimum_dt() {
            expire = now.wrapping_add(self.minimum_dt());
        }

        // 4. DISARM and CLEAR FIRST
        // This stops old alarms from firing while we are setting the new one.
        let _ = self.disarm();
        self.registers.sr.modify(SR::CC1IF::CLEAR);

        // 5. Program the hardware
        self.registers.ccr1.write(CCR1::CCR1.val(expire.into_u32()));
        self.registers.dier.modify(DIER::CC1IE::SET);
    }

    fn get_alarm(&self) -> Ticks32 {
        Ticks32::from(self.registers.ccr1.read(CCR1::CCR1))
    }

    fn is_armed(&self) -> bool {
        self.registers.dier.is_set(DIER::CC1IE)
    }

    fn disarm(&self) -> Result<(), kernel::ErrorCode> {
        self.registers.dier.modify(DIER::CC1IE::CLEAR);
        Ok(())
    }

    fn minimum_dt(&self) -> Ticks32 {
        Ticks32::from(2)
    }
}

// PWM driver for TIM3.
// This driver implements the Tock PWM HIL using the 16-bit general-purpose TIM3 timer.
// It works by making the timer count from 0 to a specified value and toggling the pin high/low

pub enum ClockSource {
    /// high-speed internal 16 MHz RC oscillator clock
    Hsi16,
    /// multi-speed internal RC oscillator clock , composed of four
    /// base oscillators (48 MHz, 4 MHz, 3.072 MHz, 400 kHz)
    Msis(usize),
    /// high-speed external crystal or clock, from 4 to 50 MHz
    Hse(usize),
    /// PLL1 output, depends on PLL configuration (input, multiplier,dividers)
    Pll1(usize),
}

impl ClockSource {
    /// MSIS is selected as the system clock on startup after a reset. Configured at 4MHz
    pub const RESET_DEFAULT: ClockSource = ClockSource::Msis(4_000_000);

    pub fn as_hz(&self) -> usize {
        match self {
            ClockSource::Hsi16 => 16_000_000,
            ClockSource::Msis(hz) => *hz,
            ClockSource::Hse(hz) => *hz,
            ClockSource::Pll1(hz) => *hz,
        }
    }
}

pub const TIM3_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x50000400 as *const TimRegisters) };

pub struct Pwm<'a> {
    // Base address for the TIM3 registers
    registers: StaticRef<TimRegisters>,
    // The clock source
    timer_clock: ClockSource,
    // Function to enable the clock for the timer
    enable_clock: fn(),
    // Needed so the struct can carry 'a lifetime because of type Pin = Pin<'a>
    _phantom: core::marker::PhantomData<&'a ()>,
}

impl<'a> Pwm<'a> {
    pub const fn new(
        base: StaticRef<TimRegisters>,
        enable_clock: fn(),
        timer_clock: ClockSource,
    ) -> Pwm<'a> {
        Pwm {
            registers: base,
            timer_clock,
            enable_clock,
            _phantom: core::marker::PhantomData,
        }
    }
    fn enable_clock(&self) {
        (self.enable_clock)();
    }
    // Switches the pin to alternate function mode and sets the alternate function to AF2 (TIM3_CH1)
    fn configure_pin(&self, pin: &Pin) {
        pin.set_mode(Mode::AlternateFunction);
        pin.set_alternate_function(2);
    }

    fn start_pwm(
        &self,
        pin: &Pin,
        frequency_hz: usize,
        duty_cycle: usize,
        max_duty_cycle: usize,
    ) -> Result<(), ErrorCode> {
        self.enable_clock();

        if frequency_hz == 0 {
            return self.stop_pwm(pin);
        }

        if frequency_hz > self.timer_clock.as_hz() {
            return Err(ErrorCode::INVAL);
        }

        if duty_cycle > max_duty_cycle {
            return Err(ErrorCode::INVAL);
        }
        // Prevent overflow in the ARR register
        if self.timer_clock.as_hz() / frequency_hz > 65535 {
            return Err(ErrorCode::INVAL);
        }

        self.configure_pin(pin);

        //We keep the prescaler 0 for maximum resolution
        let prescaler = 0;
        // Arr_value is the value that the timer counts to before resetting , it determines the frequency of the signal.
        // We derive it from Frequency = TimerClock / ((PSC + 1) * (ARR + 1))
        let arr_value = (self.timer_clock.as_hz() / frequency_hz) - 1;
        // CCR = ARR * (duty_cycle / max_duty_cycle)
        // This is the value at which the output pin will be toggled. This dictates the duty cycle of the signal.
        // For example if Arr = 100 and Ccr = 25 , the output will be high for 25 ticks and low for 75 ticks, giving a duty cycle of 25%
        let ccr_val = (arr_value * duty_cycle) / max_duty_cycle;

        // Enable auto-reload preload to buffer the ARR value and avoid glitches when changing frequency
        self.registers.cr1.modify(CR1::ARPE::SET);
        // Set frequency and prescaler
        self.registers.psc.write(PSC::PSC.val(prescaler as u32));
        self.registers.arr.write(ARR::ARR.val(arr_value as u32));

        self.registers
            .ccmr1_output
            .modify(CCMR1_Output::OC1M::PwmMode1 + CCMR1_Output::OC1PE::SET);
        self.registers.ccr1.write(CCR1::CCR1.val(ccr_val as u32));
        self.registers.ccer.modify(CCER::CC1E::SET);

        // Force an update event to load the prescaler and ARR
        self.registers.egr.write(EGR::UG::SET);
        // Start counter
        self.registers.cr1.modify(CR1::CEN::SET);

        Ok(())
    }

    fn stop_pwm(&self, pin: &Pin) -> Result<(), ErrorCode> {
        //Stop the counter and disable the output compare channel, then set the pin to analog mode instead of keeping it claimed by TIM3
        self.registers.cr1.modify(CR1::CEN::CLEAR);
        self.registers.ccer.modify(CCER::CC1E::CLEAR);
        pin.set_mode(Mode::Analog);

        Ok(())
    }
}
impl<'a> hil::pwm::Pwm for Pwm<'a> {
    type Pin = Pin<'a>;

    fn start(&self, pin: &Self::Pin, frequency: usize, duty_cycle: usize) -> Result<(), ErrorCode> {
        self.start_pwm(pin, frequency, duty_cycle, self.get_maximum_duty_cycle())
    }

    fn stop(&self, pin: &Self::Pin) -> Result<(), ErrorCode> {
        self.stop_pwm(pin)
    }

    fn get_maximum_frequency_hz(&self) -> usize {
        self.timer_clock.as_hz()
    }

    fn get_maximum_duty_cycle(&self) -> usize {
        u16::MAX as usize + 1
    }
}

pub struct PwmPin<'a> {
    pwm: &'a Pwm<'a>,
    pin: &'a Pin<'a>,
}

impl<'a> PwmPin<'a> {
    pub const fn new(pwm: &'a Pwm<'a>, pin: &'a Pin<'a>) -> Self {
        PwmPin { pwm, pin }
    }
}
impl hil::pwm::PwmPin for PwmPin<'_> {
    fn start(&self, frequency_hz: usize, duty_cycle: usize) -> Result<(), ErrorCode> {
        self.pwm.start_pwm(
            self.pin,
            frequency_hz,
            duty_cycle,
            self.get_maximum_duty_cycle(),
        )
    }
    fn stop(&self) -> Result<(), ErrorCode> {
        self.pwm.stop_pwm(self.pin)
    }
    fn get_maximum_frequency_hz(&self) -> usize {
        self.pwm.timer_clock.as_hz()
    }
    fn get_maximum_duty_cycle(&self) -> usize {
        u16::MAX as usize + 1
    }
}
