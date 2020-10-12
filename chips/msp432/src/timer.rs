//! Timer (TIMER_Ax)

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil::time::{
    Alarm, AlarmClient, Counter, Frequency, OverflowClient, Ticks, Ticks16, Time,
};
use kernel::ReturnCode;

pub static mut TIMER_A0: TimerA = TimerA::new(TIMER_A0_BASE);
pub static mut TIMER_A1: TimerA = TimerA::new(TIMER_A1_BASE);
pub static mut TIMER_A2: TimerA = TimerA::new(TIMER_A2_BASE);
pub static mut TIMER_A3: TimerA = TimerA::new(TIMER_A3_BASE);

const TIMER_A0_BASE: StaticRef<TimerRegisters> =
    unsafe { StaticRef::new(0x4000_0000u32 as *const TimerRegisters) };

const TIMER_A1_BASE: StaticRef<TimerRegisters> =
    unsafe { StaticRef::new(0x4000_0400u32 as *const TimerRegisters) };

const TIMER_A2_BASE: StaticRef<TimerRegisters> =
    unsafe { StaticRef::new(0x4000_0800u32 as *const TimerRegisters) };

const TIMER_A3_BASE: StaticRef<TimerRegisters> =
    unsafe { StaticRef::new(0x4000_0C00u32 as *const TimerRegisters) };

register_structs! {
    /// Timer_Ax
    TimerRegisters {
        /// Timer_Ax Control
        (0x00 => ctl: ReadWrite<u16, TAxCTL::Register>),
        /// Timer_Ax Capture/Compare Control 0
        (0x02 => cctl0: ReadWrite<u16, TAxCCTLx::Register>),
        /// Timer_Ax Capture/Compare Control 1
        (0x04 => cctl1: ReadWrite<u16, TAxCCTLx::Register>),
        /// Timer_Ax Capture/Compare Control 2
        (0x06 => cctl2: ReadWrite<u16, TAxCCTLx::Register>),
        /// Timer_Ax Capture/Compare Control 3
        (0x08 => cctl3: ReadWrite<u16, TAxCCTLx::Register>),
        /// Timer_Ax Capture/Compare Control 4
        (0x0A => cctl4: ReadWrite<u16, TAxCCTLx::Register>),
        /// Timer_Ax Capture/Compare Control 5
        (0x0C => cctl5: ReadWrite<u16, TAxCCTLx::Register>),
        /// Timer_Ax Capture/Compare Control 6
        (0x0E => cctl6: ReadWrite<u16, TAxCCTLx::Register>),
        /// Timer_Ax Counter
        (0x10 => cnt: ReadWrite<u16>),
        /// Timer_Ax Capture/Compare 0
        (0x12 => ccr0: ReadWrite<u16>),
        /// Timer_Ax Capture/Compare 1
        (0x14 => ccr1: ReadWrite<u16>),
        /// Timer_Ax Capture/Compare 2
        (0x16 => ccr2: ReadWrite<u16>),
        /// Timer_Ax Capture/Compare 3
        (0x18 => ccr3: ReadWrite<u16>),
        /// Timer_Ax Capture/Compare 4
        (0x1A => ccr4: ReadWrite<u16>),
        /// Timer_Ax Capture/Compare 5
        (0x1C => ccr5: ReadWrite<u16>),
        /// Timer_Ax Capture/Compare 6
        (0x1E => ccr6: ReadWrite<u16>),
        /// Timer_Ax Expansion 0
        (0x20 => ex0: ReadWrite<u16, TAxEX0::Register>),
        (0x22 => _reserved),
        /// Timer_Ax Interrupt Vector
        (0x2E => iv: ReadWrite<u16, TAxIV::Register>),
        (0x30 => @END),
    }
}

register_bitfields! [u16,
    /// Timer_Ax Control Register
    TAxCTL [
        /// Timer_A interrupt flag
        TAIFG OFFSET(0) NUMBITS(1) [],
        /// Timer_A interrupt enable
        TAIE OFFSET(1) NUMBITS(1) [],
        /// TIMER_A clear. Setting this bit resets TAxR, the timer clock divider logic, and the count direction.
        TACLR OFFSET(2) NUMBITS(1) [],
        /// Mode control. Setting MCx=0x00 when Timer_A is not in use conserves power.
        MC OFFSET(4) NUMBITS(2) [
            /// Stop mode: Timer is halted
            StopMode = 0,
            /// Up mode: Timer counts up to TAxCCR0
            UpMode = 1,
            /// Continuous mode: Timer counts up to 0xFFFF
            ContinuousMode = 2,
            /// Up/Down mode: Timer counts up to TAxCCR0 then down to 0x0000
            UpDownMode = 3
        ],
        /// Input divider. These bits along with the TAIDEX bits select the divider for the input clock.
        ID OFFSET(6) NUMBITS(2) [
            /// Clock divided by 1
            DividedBy1 = 0,
            /// Clock divided by 2
            DividedBy2 = 1,
            /// Clock divided by 4
            DividedBy4 = 2,
            /// Clock divied by 8
            DividedBy8 = 3
        ],
        /// Timer_A clock source Select
        TASSEL OFFSET(8) NUMBITS(2) [
            /// TAxCLK
            TAxCLK = 0,
            /// ACLK
            ACLK = 1,
            /// SMCLK
            SMCLK = 2,
            /// INCLK
            INCLK = 3
        ]
    ],
    /// Timer_Ax Capture/Compare Control Register
    TAxCCTLx [
        /// Capture/compare interrupt flag
        CCIFG OFFSET(0) NUMBITS(1) [],
        /// Capture overflow. This bit indicates a capture overflow occured. COV must be reset with software.
        COV OFFSET(1) NUMBITS(1) [],
        /// Output. For output mode 0, this bit directly controls the state of the output
        OUT OFFSET(2) NUMBITS(1) [],
        /// Capture/compare input. The selected input signal can be read by this bit.
        CCI OFFSET(3) NUMBITS(1) [],
        /// Capture/compare interrupt enable. This bit enables the interrupt request of the corresponding CCIFG flag.
        CCIE OFFSET(4) NUMBITS(1) [],
        /// Output mode. Modes 2, 3, 6 and 7 are not useful for TAxCCR0 because EQUx=EQU0.
        OUTMOD OFFSET(5) NUMBITS(3) [
            /// OUT bit value
            OutBit = 0,
            /// Set
            Set = 1,
            /// Toggle/reset
            ToggleReset = 2,
            /// Set/reset
            SetReset = 3,
            /// Toggle
            Toggle = 4,
            /// Reset
            Reset = 5,
            /// Toggle/set
            ToggleSet = 6,
            /// Reset/set
            ResetSet = 7
        ],
        /// Capture mode
        CAP OFFSET(8) NUMBITS(1) [],
        /// Synchronized capture/compare input
        SCCI OFFSET(10) NUMBITS(1) [],
        /// Synchronize capture source. This bit is used to synchronize the capture input signal with the timer clock.
        SCS OFFSET(11) NUMBITS(1) [
            /// Asynchronous capture
            Asynchronous = 0,
            /// Synchronous capture
            Synchronous = 1
        ],
        /// Capture/compare input select. These bits select the TAxCCR0 input signal.
        CCIS OFFSET(12) NUMBITS(2) [
            /// CCIxA
            CCIxA = 0,
            /// CCIxB
            CCIxB = 1,
            /// GND
            GND = 2,
            /// VCC
            VCC = 3
        ],
        /// Capture mode
        CM OFFSET(14) NUMBITS(2) [
            /// No capture
            NoCapture = 0,
            /// Capture on rising edge
            CaptureRisingEdge = 1,
            /// Capture on falling edge
            CaptureFallingEdge = 2,
            /// Capture on bith rising and falling edges
            CaptureBothEdges = 3
        ]
    ],
    /// Timer_Ax Interrupt Vector Register
    TAxIV [
        TAIV OFFSET(0) NUMBITS(16) [
            /// No interrupt pending
            NoInterrupt = 0x00,
            /// Capture/compare: TAxCCR1 CCIFG
            InterruptCCR1 = 0x02,
            /// Capture/compare: TAxCCR2 CCIFG
            InterruptCCR2 = 0x04,
            /// Capture/compare: TAxCCR3 CCIFG
            InterruptCCR3 = 0x06,
            /// Capture/compare: TAxCCR4 CCIFG
            InterruptCCR4 = 0x08,
            /// Capture/compare: TAxCCR5 CCIFG
            InterruptCCR5 = 0x0A,
            /// Capture/compare: TAxCCR6 CCIFG
            InterruptCCR6 = 0x0C,
            /// Timer overflow: TAxCTL TAIFG
            InterruptTimer = 0x0E
        ]
    ],
    /// Timer_Ax Expansion Register
    TAxEX0 [
        /// Input divider expansion. These bits along with the ID bits select the divider for the input clock.
        TAIDEX OFFSET(0) NUMBITS(3) [
            /// Divide by 1
            DivideBy1 = 0,
            /// Divide by 2
            DivideBy2 = 1,
            /// Divide by 3
            DivideBy3 = 2,
            /// Divide by 4
            DivideBy4 = 3,
            /// Divide by 5
            DivideBy5 = 4,
            /// Divide by 6
            DivideBy6 = 5,
            /// Divide by 7
            DivideBy7 = 6,
            /// Divide by 8
            DivideBy8 = 7
        ]
    ]
];

/// Since this timer-modules will be used for other things than alarm too
/// (e.g. PWM, Timer, etc.) keep track for what it is used for.
#[derive(PartialEq, Copy, Clone)]
enum TimerMode {
    Disabled,
    Alarm,
    InternalTimer,
}

pub struct TimerAFrequency {}

impl Frequency for TimerAFrequency {
    fn frequency() -> u32 {
        crate::cs::ACLK_HZ / 16
    }
}

pub trait InternalTimer {
    /// Start timer in a given frequency. No interrupts are generated, the signal when the timer
    /// has elapsed is used directly by a hardware module.
    /// SUCCESS: timer started successfully
    /// EINVAL: frequency too high or too low
    /// EBUSY: timer already in use
    fn start(&self, frequency_hz: u32) -> kernel::ReturnCode;

    /// Stop the timer
    fn stop(&self);
}

pub struct TimerA<'a> {
    registers: StaticRef<TimerRegisters>,
    mode: Cell<TimerMode>,
    alarm_client: OptionalCell<&'a dyn AlarmClient>,
}

impl<'a> TimerA<'a> {
    const fn new(base: StaticRef<TimerRegisters>) -> TimerA<'a> {
        TimerA {
            registers: base,
            mode: Cell::new(TimerMode::Disabled),
            alarm_client: OptionalCell::empty(),
        }
    }

    // Setup the timer to use it for alarms
    fn setup_for_alarm(&self) {
        // Setup the timer to use the ACLK (32.768kHz) as clock source, configure it to continuous
        // mode, divide the clock down to 2048Hz:
        // 16bit at 2048Hz: granulation about 0.5ms, maximum interval about 30s.

        self.registers.ctl.modify(
            TAxCTL::TASSEL::ACLK         // Set ACLK as clock source
                + TAxCTL::ID::DividedBy8        // Divide the clock source by 8 -> 4096Hz
                + TAxCTL::MC::ContinuousMode    // Setup for contiuous mode    
                + TAxCTL::TAIE::CLEAR           // Disable interrupts
                + TAxCTL::TAIFG::CLEAR, // Clear any pending interrupts
        );

        // divide the 4096Hz by 2 to get 2048Hz
        self.registers.ex0.modify(TAxEX0::TAIDEX::DivideBy2);
        self.mode.set(TimerMode::Alarm);
    }

    // Stops the timer, no matter how it is configured
    fn stop_timer(&self) {
        self.registers.ctl.modify(TAxCTL::MC::StopMode);
        self.mode.set(TimerMode::Disabled);
    }

    fn handle_alarm_interrupt(&self) {
        // Disable the interrupt, since the alarm was fired
        self.registers.cctl0.modify(TAxCCTLx::CCIE::CLEAR);
        self.alarm_client.map(|client| client.alarm());
    }

    pub fn handle_interrupt(&self) {
        if self.registers.cctl0.is_set(TAxCCTLx::CCIFG) {
            if self.mode.get() == TimerMode::Alarm {
                self.handle_alarm_interrupt();
            }
            self.registers.cctl0.modify(TAxCCTLx::CCIFG::CLEAR);
        }
    }
}

impl<'a> Time for TimerA<'a> {
    type Frequency = TimerAFrequency;
    type Ticks = Ticks16;

    fn now(&self) -> Ticks16 {
        Self::Ticks::from(self.registers.cnt.get())
    }
}

impl<'a> Counter<'a> for TimerA<'a> {
    fn set_overflow_client(&'a self, _client: &'a dyn OverflowClient) {}

    fn start(&self) -> ReturnCode {
        self.setup_for_alarm();
        ReturnCode::SUCCESS
    }

    fn stop(&self) -> ReturnCode {
        self.stop_timer();
        ReturnCode::SUCCESS
    }

    fn reset(&self) -> ReturnCode {
        self.registers.cnt.set(0);
        ReturnCode::SUCCESS
    }

    fn is_running(&self) -> bool {
        self.registers.cctl0.is_set(TAxCCTLx::CCIE)
    }
}

impl<'a> Alarm<'a> for TimerA<'a> {
    fn set_alarm_client(&'a self, client: &'a dyn AlarmClient) {
        self.alarm_client.set(client);
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        if self.mode.get() != TimerMode::Alarm {
            self.setup_for_alarm();
        }
        let now = self.now();
        let mut expire = reference.wrapping_add(dt);
        if !now.within_range(reference, expire) {
            expire = now;
        }

        if expire.wrapping_sub(now) <= self.minimum_dt() {
            expire = now.wrapping_add(self.minimum_dt());
        }

        self.disarm();
        // Set compare register
        self.registers.ccr0.set(expire.into_u16());
        // Enable capture/compare interrupt
        self.registers.cctl0.modify(TAxCCTLx::CCIE::SET);
    }

    fn get_alarm(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.ccr0.get())
    }

    fn is_armed(&self) -> bool {
        let int_enabled = self.registers.cctl0.is_set(TAxCCTLx::CCIE);
        (self.mode.get() == TimerMode::Alarm) && int_enabled
    }

    fn disarm(&self) -> ReturnCode {
        // Disable the capture/compare interrupt
        self.registers.cctl0.modify(TAxCCTLx::CCIE::CLEAR);
        // Stop the timer completely
        //self.stop_timer();
        ReturnCode::SUCCESS
    }

    fn minimum_dt(&self) -> Self::Ticks {
        Self::Ticks::from(1 as u16)
    }
}

impl<'a> InternalTimer for TimerA<'a> {
    fn start(&self, frequency_hz: u32) -> kernel::ReturnCode {
        if self.mode.get() != TimerMode::Disabled && self.mode.get() != TimerMode::InternalTimer {
            return kernel::ReturnCode::EBUSY;
        }

        if frequency_hz > crate::cs::SMCLK_HZ {
            return kernel::ReturnCode::EINVAL;
        }

        // Stop timer if a different frequency was configured before
        self.stop_timer();

        let reg_val = if frequency_hz <= 100 {
            // Divide the SMCLK by 40 -> 1_500_000 / 40 = 37.5kHz
            self.registers.ctl.modify(TAxCTL::ID::DividedBy8);
            self.registers.ex0.modify(TAxEX0::TAIDEX::DivideBy5);
            (crate::cs::SMCLK_HZ / 40) / frequency_hz
        } else {
            self.registers.ctl.modify(TAxCTL::ID::DividedBy1);
            self.registers.ex0.modify(TAxEX0::TAIDEX::DivideBy1);
            crate::cs::SMCLK_HZ / frequency_hz
        };

        self.registers.ctl.modify(
            TAxCTL::TASSEL::SMCLK    // Set SMCLK as clock source
            + TAxCTL::MC::UpMode            // Setup for up-mode    
            + TAxCTL::TAIE::CLEAR           // Disable interrupts
            + TAxCTL::TAIFG::CLEAR, // Clear any pending interrupts
        );

        // Set timer value
        self.registers.ccr0.set((reg_val - 1) as u16);
        self.registers.cctl0.modify(TAxCCTLx::CCIE::CLEAR);
        // Set capture value to raise interrupt
        self.registers.ccr1.set((reg_val - 2) as u16);
        // Enable CCR interrupt to trigger the corresponding Hardware
        self.registers
            .cctl1
            .modify(TAxCCTLx::OUTMOD::SetReset + TAxCCTLx::OUT::CLEAR + TAxCCTLx::CCIE::CLEAR);

        self.mode.set(TimerMode::InternalTimer);
        kernel::ReturnCode::SUCCESS
    }

    fn stop(&self) {
        self.stop_timer();
    }
}
