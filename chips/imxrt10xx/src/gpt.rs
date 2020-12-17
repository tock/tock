use core::sync::atomic::{AtomicU32, Ordering};
use cortexm7;
use cortexm7::support::atomic;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::hil::time::{Ticks, Ticks32, Time};
use kernel::ClockInterface;
use kernel::ReturnCode;

use crate::ccm;
use crate::nvic;

/// General purpose timers
#[repr(C)]
struct GptRegisters {
    /// GPT Control Register
    cr: ReadWrite<u32, CR::Register>,
    /// GPT Prescaler Register
    pr: ReadWrite<u32, PR::Register>,
    /// GPT Status Register
    sr: ReadWrite<u32, SR::Register>,
    /// GPT Interrupt Register
    ir: ReadWrite<u32, IR::Register>,
    /// GPT Output Compare Register 1
    ocr1: ReadWrite<u32, OCR1::Register>,
    /// GPT Output Compare Register 2
    ocr2: ReadWrite<u32, OCR2::Register>,
    /// GPT Output Compare Register 3
    ocr3: ReadWrite<u32, OCR3::Register>,
    /// GPT Input Capture Register 1
    icr1: ReadOnly<u32, ICR1::Register>,
    /// GPT Input Capture Register 2
    icr2: ReadOnly<u32, ICR2::Register>,
    /// GPT Counter Register
    cnt: ReadOnly<u32, CNT::Register>,
}

register_bitfields![u32,
    CR [
        /// Force Output Compare Channel 3
        FO3 OFFSET(31) NUMBITS(1) [],
        /// Force Output Compare Channel 2
        FO2 OFFSET(30) NUMBITS(1) [],
        /// Force Output Compare Channel 1
        FO1 OFFSET(29) NUMBITS(1) [],
        /// Controls the Output Compare Channel 3 operating mode
        OM3 OFFSET(26) NUMBITS(3) [],
        /// Controls the Output Compare Channel 2 operating mode
        OM2 OFFSET(23) NUMBITS(3) [],
        /// Controls the Output Compare Channel 2 operating mode
        OM1 OFFSET(20) NUMBITS(3) [],
        /// Input Capture Channel 2 operating mode
        IM2 OFFSET(18) NUMBITS(2) [],
        /// Input Capture Channel 1 operating mode
        IM1 OFFSET(16) NUMBITS(2) [],
        /// Software reset
        SWR OFFSET(15) NUMBITS(1) [],
        /// Enable 24 MHz clock input from crystal
        EN_24M OFFSET(10) NUMBITS(1) [],
        /// Free run or Restart mode
        FRR OFFSET(9) NUMBITS(1) [],
        /// Clock source select
        CLKSRC OFFSET(6) NUMBITS(3) [
            /// No clock
            NoClock = 0,
            /// Peripheral Clock (ipg_clk)
            PeripheralClock = 1,
            /// High Frequency Reference Clock (ipg_clk_highfreq)
            HighFrequencyReferenceClock = 2,
            /// External Clock
            ExternalClock = 3,
            /// Low Frequency Reference Clock (ipg_clk_32k)
            LowFrequencyReferenceClock = 4,
            /// Crystal oscillator as Reference Clock (ipg_clk_24M)
            CrystalOscillator = 5
        ],
        /// GPT Stop Mode enable
        STOPEN OFFSET(5) NUMBITS(1) [],
        /// GPT Doze Mode Enable
        DOZEEN OFFSET(4) NUMBITS(1) [],
        /// GPT Wait Mode enable
        WAITEN OFFSET(3) NUMBITS(1) [],
        /// GPT debug mode enable
        DBGEN OFFSET(2) NUMBITS(1) [],
        /// GPT Enable mode
        ENMOD OFFSET(1) NUMBITS(1) [],
        /// GPT Enable
        EN OFFSET(0) NUMBITS(1) []
    ],

    PR [
        /// Prescaler bits for 24M crystal clock
        PRESCALER24M OFFSET(12) NUMBITS(4),
        /// Prescaler bits
        PRESCALER OFFSET(0) NUMBITS(12)
    ],

    SR [
        /// Rollover Flag
        ROV OFFSET(5) NUMBITS(1),
        /// Input capture 2 Flag
        IF2 OFFSET(4) NUMBITS(1),
        /// Input capture 1 Flag
        IF1 OFFSET(3) NUMBITS(1),
        /// Output Compare 3 Flag
        OF3 OFFSET(2) NUMBITS(1),
        /// Output Compare 2 Flag
        OF2 OFFSET(1) NUMBITS(1),
        /// Output Compare 1 Flag
        OF1 OFFSET(0) NUMBITS(1)
    ],

    IR [
        /// Rollover Interrupt Enable
        ROVIE OFFSET(5) NUMBITS(1),
        /// Input capture 2 Interrupt Enable
        IF2IE OFFSET(4) NUMBITS(1),
        /// Input capture 1 Interrupt Enable
        IF1IE OFFSET(3) NUMBITS(1),
        /// Output Compare 3 Interrupt Enable
        OF3IE OFFSET(2) NUMBITS(1),
        /// Output Compare 2 Interrupt Enable
        OF2IE OFFSET(1) NUMBITS(1),
        /// Output Compare 1 Interrupt Enable
        OF1IE OFFSET(0) NUMBITS(1)
    ],

    OCR1 [
        COMP OFFSET(0) NUMBITS(32)
    ],

    OCR2 [
        COMP OFFSET(0) NUMBITS(32)
    ],

    OCR3 [
        COMP OFFSET(0) NUMBITS(32)
    ],

    ICR1 [
        CAPT OFFSET(0) NUMBITS(32)
    ],

    ICR2 [
        CAPT OFFSET(0) NUMBITS(32)
    ],

    CNT [
        COUNT OFFSET(0) NUMBITS(32)
    ]
];

const GPT1_BASE: StaticRef<GptRegisters> =
    unsafe { StaticRef::new(0x401EC000 as *const GptRegisters) };
const GPT2_BASE: StaticRef<GptRegisters> =
    unsafe { StaticRef::new(0x401F0000 as *const GptRegisters) };

pub struct Gpt<'a, S> {
    registers: StaticRef<GptRegisters>,
    clock: Gpt1Clock,
    client: OptionalCell<&'a dyn hil::time::AlarmClient>,
    irqn: u32,
    _selection: core::marker::PhantomData<S>,
}

pub type Gpt1<'a> = Gpt<'static, _1>;
pub type Gpt2<'a> = Gpt<'static, _2>;

pub static mut GPT1: Gpt1<'static> = Gpt::new(GPT1_BASE, nvic::GPT1);
pub static mut GPT2: Gpt2<'static> = Gpt::new(GPT2_BASE, nvic::GPT2);

impl<'a, S> Gpt<'a, S> {
    const fn new(registers: StaticRef<GptRegisters>, irqn: u32) -> Self {
        Gpt {
            registers,
            clock: Gpt1Clock(ccm::PeripheralClock::CCGR1(ccm::HCLK1::GPT1)),
            client: OptionalCell::empty(),
            irqn,
            _selection: core::marker::PhantomData,
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
        self.registers.sr.modify(SR::OF1::SET);
        self.registers.ir.modify(IR::OF1IE::CLEAR);

        self.client.map(|client| client.alarm());
    }

    /// Start the GPT, specifying the peripheral clock selection and the peripheral clock divider
    ///
    /// If you select the crystal oscillator as the periodic clock root, the GPT will divide the
    /// input clock by 3.
    ///
    /// `divider` must be non-zero.
    pub fn start(&self, selection: ccm::PerclkClockSel, divider: u8) {
        // Disable GPT and the GPT interrupt register first
        self.registers.cr.modify(CR::EN::CLEAR);

        self.registers.ir.modify(IR::ROVIE::CLEAR);
        self.registers.ir.modify(IR::IF1IE::CLEAR);
        self.registers.ir.modify(IR::IF2IE::CLEAR);
        self.registers.ir.modify(IR::OF1IE::CLEAR);
        self.registers.ir.modify(IR::OF2IE::CLEAR);
        self.registers.ir.modify(IR::OF3IE::CLEAR);

        // Clear Output mode to disconnected
        self.registers.cr.modify(CR::OM1::CLEAR);
        self.registers.cr.modify(CR::OM2::CLEAR);
        self.registers.cr.modify(CR::OM3::CLEAR);

        // Disable Input Capture Mode
        self.registers.cr.modify(CR::IM1::CLEAR);
        self.registers.cr.modify(CR::IM2::CLEAR);

        // Reset all the registers to the their default values, except EN,
        // ENMOD, STOPEN, DOZEEN, WAITEN, and DBGEN bits in the CR
        self.registers.cr.modify(CR::SWR::SET);

        // wait until registers are cleared
        while self.registers.cr.is_set(CR::SWR) {}

        // Clear the GPT status register
        self.registers.sr.set(31 as u32);

        // Enable free run mode
        self.registers.cr.modify(CR::FRR::SET);

        // Enable run in wait mode
        self.registers.cr.modify(CR::WAITEN::SET);

        // Enable run in stop mode
        self.registers.cr.modify(CR::STOPEN::SET);

        // Bring GPT counter to 0x00000000
        self.registers.cr.modify(CR::ENMOD::SET);

        // Set the value of the Output Compare Register
        self.registers.ocr1.set(0xFFFF_FFFF - 1);

        match selection {
            ccm::PerclkClockSel::IPG => {
                // Disable 24Mhz clock input from crystal
                self.registers.cr.modify(CR::EN_24M::CLEAR);

                // We will use the ipg_clk_highfreq provided by perclk_clk_root,
                // which runs at 24.75 MHz. Before calling set_alarm, we assume clock
                // to GPT1 has been enabled.
                self.registers.cr.modify(CR::CLKSRC.val(0x2 as u32));

                // We do not prescale the value for the moment. We will do so
                // after we will set the ARM_PLL1 CLK accordingly.
                self.registers.pr.modify(PR::PRESCALER.val(0 as u32));

                self.set_frequency(IMXRT1050_IPG_CLOCK_HZ / divider as u32);
            }
            ccm::PerclkClockSel::Oscillator => {
                // Enable 24MHz clock input
                self.registers
                    .cr
                    .modify(CR::EN_24M::SET + CR::CLKSRC::CrystalOscillator);

                // Funknown reasons, the 24HMz prescaler must be non-zero, even
                // though zero is a valid value according to the reference manual.
                // If it's not set, the counter doesn't count! Thanks to the se4L
                // project for adding a comment to their code.
                //
                // I'm also finding that it can't be too large; a prescaler of 8
                // for the 24MHz clock doesn't work!
                const DEFAULT_PRESCALER: u32 = 3;
                self.registers
                    .pr
                    .write(PR::PRESCALER24M.val(DEFAULT_PRESCALER - 1));
                self.set_frequency(OSCILLATOR_HZ / DEFAULT_PRESCALER / divider as u32);
            }
        }

        // Enable the GPT
        self.registers.cr.modify(CR::EN::SET);

        // Enable the Output Compare 1 Interrupt Enable
        self.registers.ir.modify(IR::OF1IE::SET);
    }

    fn set_frequency(&self, hz: u32) {
        let idx = match self.irqn {
            nvic::GPT1 => 0,
            nvic::GPT2 => 1,
            _ => unreachable!(),
        };
        GPT_FREQUENCIES[idx].store(hz, Ordering::Release);
    }
}

/// Assumed IPG clock frequency for the iMXRT1050 processor family.
///
/// TODO this is not a constant value; it changes when setting the ARM clock
/// frequency. Change this after correctly configuring ARM frequency.
const IMXRT1050_IPG_CLOCK_HZ: u32 = 24_750_000;
/// Crystal oscillator frequency
const OSCILLATOR_HZ: u32 = 24_000_000;

/// GPT selection tags
pub enum _1 {}
pub enum _2 {}

static GPT_FREQUENCIES: [AtomicU32; 2] = [AtomicU32::new(0), AtomicU32::new(0)];

impl hil::time::Frequency for _1 {
    fn frequency() -> u32 {
        GPT_FREQUENCIES[0].load(Ordering::Acquire)
    }
}

impl hil::time::Frequency for _2 {
    fn frequency() -> u32 {
        GPT_FREQUENCIES[1].load(Ordering::Acquire)
    }
}

impl<F: hil::time::Frequency> hil::time::Time for Gpt<'_, F> {
    type Frequency = F;
    type Ticks = Ticks32;

    fn now(&self) -> Ticks32 {
        Ticks32::from(self.registers.cnt.get())
    }
}

impl<'a, F: hil::time::Frequency> hil::time::Alarm<'a> for Gpt<'a, F> {
    fn set_alarm_client(&self, client: &'a dyn hil::time::AlarmClient) {
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

        self.disarm();
        self.registers.ocr1.set(expire.into_u32());
        self.registers.ir.modify(IR::OF1IE::SET);
    }

    fn get_alarm(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.ocr1.get())
    }

    fn disarm(&self) -> ReturnCode {
        unsafe {
            atomic(|| {
                // Disable counter
                self.registers.ir.modify(IR::OF1IE::CLEAR);
                cortexm7::nvic::Nvic::new(self.irqn).clear_pending();
            });
        }
        ReturnCode::SUCCESS
    }

    fn is_armed(&self) -> bool {
        // If alarm is enabled, then OF1IE is set
        self.registers.ir.is_set(IR::OF1IE)
    }

    fn minimum_dt(&self) -> Self::Ticks {
        Self::Ticks::from(1)
    }
}

struct Gpt1Clock(ccm::PeripheralClock);

impl ClockInterface for Gpt1Clock {
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
