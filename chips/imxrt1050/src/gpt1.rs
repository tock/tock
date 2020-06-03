use cortexm7;
use cortexm7::support::atomic;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadWrite, ReadOnly};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ClockInterface;

use crate::nvic;
use crate::ccm;

/// General purpose timers
#[repr(C)]
struct Gpt1Registers {
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
        CLKSRC OFFSET(6) NUMBITS(3) [],

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

const GPT1_BASE: StaticRef<Gpt1Registers> =
    unsafe { StaticRef::new(0x401EC000 as *const Gpt1Registers) };

pub struct Gpt1<'a> {
    registers: StaticRef<Gpt1Registers>,
    clock: Gpt1Clock,
    client: OptionalCell<&'a dyn hil::time::AlarmClient>,
    irqn: u32,
}

pub static mut GPT1: Gpt1<'static> = Gpt1::new();

impl Gpt1<'a> {
    const fn new() -> Gpt1<'a> {
        Gpt1 {
            registers: GPT1_BASE,
            clock: Gpt1Clock(ccm::PeripheralClock::CCGR1(ccm::HCLK1::GPT1)),
            client: OptionalCell::empty(),
            irqn: nvic::GPT1,
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
        self.registers.cr.modify(CR::EN::CLEAR);

        self.client.map(|client| client.fired());
    }

    pub fn start(&self) {
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
        self.registers.cr.modify(CR::FRR::CLEAR);

        // Enable run in wait mode
        self.registers.cr.modify(CR::WAITEN::SET);

        // Enable run in stop mode
        self.registers.cr.modify(CR::STOPEN::SET);

        // Bring GPT counter to 0x00000000
        self.registers.cr.modify(CR::ENMOD::SET);

        // Set the value of the Output Compare Register
        self.registers.ocr1.set(0xFFFF_FFFF - 1);

        // Disable 24Mhz clock input from crystal
        self.registers.cr.modify(CR::EN_24M::CLEAR);

        // We will use the ipg_clk_highfreq provided by perclk_clk_root,
        // which runs at 6 MHz. Before calling set_alarm, we assume clock 
        // to GPT1 has been enabled. 
        self.registers.cr.modify(CR::CLKSRC.val(0x2 as u32));

        // Prescale 6Mhz to 16Khz, by dividing it by 375. The change in the
        // prescaler value immediately affects the output clock frequency
        self.registers.pr.modify(PR::PRESCALER.val(0 as u32));

        // Enable the GPT 
        self.registers.cr.modify(CR::EN::SET);

        // Enable the Output Compare 1 Interrupt Enable
        self.registers.ir.modify(IR::OF1IE::SET);
    }
}

impl hil::time::Alarm<'a> for Gpt1<'a> {
    fn set_client(&self, client: &'a dyn hil::time::AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, tics: u32) {
        self.registers.cr.modify(CR::EN::CLEAR);
        self.registers.cr.modify(CR::SWR::SET);

        // wait until registers are cleared
        while self.registers.cr.is_set(CR::SWR) {}

        self.registers.cr.modify(CR::FRR::CLEAR);
        self.registers.cr.modify(CR::CLKSRC.val(0x2 as u32));
        self.registers.pr.modify(PR::PRESCALER.val(0 as u32));

        self.registers.ocr1.set(tics);
        self.registers.ir.modify(IR::OF1IE::SET);
        self.registers.cr.modify(CR::EN::SET);
        // self.registers.cr.modify(CR::OM1::SET);
    }

    fn get_alarm(&self) -> u32 {
        self.registers.ocr1.get()
    }

    fn disable(&self) {
        unsafe {
            atomic(|| {
                // Disable counter
                self.registers.ir.modify(IR::OF1IE::CLEAR);
                cortexm7::nvic::Nvic::new(self.irqn).clear_pending();
            });
        }
    }

    fn is_enabled(&self) -> bool {
        // If alarm is enabled, then OF1IE is set
        self.registers.ir.is_set(IR::OF1IE)
    }
}

/// The frequency is dependent on the ARM_PLL1 frequency.
/// In our case, we get a 24.75 MHz frequency for the timer.
/// The frequency will be fixed when the ARM_PLL1 CLK will
/// be correctly configured.
impl hil::time::Time for Gpt1<'a> {
    type Frequency = hil::time::Freq2475MHz;

    fn now(&self) -> u32 {
        self.registers.cnt.get()
    }

    fn max_tics(&self) -> u32 {
        core::u32::MAX
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