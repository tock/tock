//! Clock peripheral driver, nRF5X-family
//!
//! The clock peripheral of the nRF51 series (chapter 13 of
//! the nRF51 reference manual v3.0), which manages the
//! low frequency and high frequency clocks. The low frequency
//! clock drives the real time clock (RTC), while the
//! high frequency clocks drive the timer system.
//!
//! Author
//! ---------
//! * Philip Levis
//! * Date: August 18, 2016

use kernel::common::cells::OptionalCell;
use kernel::common::registers::{ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;

pub static mut CLOCK: Clock = Clock::new();

#[repr(C)]
struct ClockRegisters {
    hfclkstart: WriteOnly<u32, Task::Register>,      // 0x000
    hfclkstop: WriteOnly<u32, Task::Register>,       // 0x004
    lfclkstart: WriteOnly<u32, Task::Register>,      // 0x008
    lfclkstop: WriteOnly<u32, Task::Register>,       // 0x00c
    cal: WriteOnly<u32, Task::Register>,             // 0x010
    cstart: WriteOnly<u32, Task::Register>,          // 0x014
    cstop: WriteOnly<u32, Task::Register>,           // 0x018
    _reserved1: [u32; 57],                           // 0x01c - 0x100
    hfclkstarted: ReadWrite<u32, Event::Register>,   // 0x100
    lfclkstarted: ReadWrite<u32, Event::Register>,   // 0x104
    _reserved2: [u32; 1],                            // 0x108
    done: ReadWrite<u32, Event::Register>,           // 0x10c
    ctto: ReadWrite<u32, Event::Register>,           // 0x110
    _reserved3: [u32; 124],                          // 0x110 - 0x304
    intenset: ReadWrite<u32, Interrupt::Register>,   // 0x304
    intenclr: ReadWrite<u32, Interrupt::Register>,   // 0x308
    _reserved4: [u32; 63],                           // 0x308 - 0x408
    hfclkrun: ReadOnly<u32, ClkRun::Register>,       // 0x408
    hfclkstat: ReadOnly<u32, HfClkStat::Register>,   // 0x40c
    _reserved5: [u32; 1],                            // 0x410
    lfclkrun: ReadOnly<u32, ClkRun::Register>,       // 0x414
    lfclkstat: ReadOnly<u32, LfClkStat::Register>,   // 0x418
    lfclksrccopy: ReadOnly<u32, LfClkSrc::Register>, // 0x41c
    _reserved6: [u32; 62],                           // 0x420 - 0x518
    lfclksrc: ReadWrite<u32, LfClkSrc::Register>,    // 0x518
    _reserved7: [u32; 7],                            // 0x51c - 0x538
    ctiv: ReadWrite<u32, CalibrationTimerInterval::Register>, // 0x538
    _reserved8: [u32; 5],                            // 0x53c - 0x550
    xtalfreq: ReadWrite<u32, CrystalFrequency::Register>, // 0x550
}

register_bitfields![u32,
    /// Tasks
    Task [
        EXECUTE 0
    ],

    /// Events.
    Event [
        READY 0
    ],

    /// Interrupts.
    ///
    /// Write '0' has no effect. When read this register will return the value of INTEN.
    Interrupt [
        HFCLKSTARTED 0,
        LFCLKSTARTED 1,
        DONE 3,
        CTTO 4
    ],

    /// Is this clock running?
    ClkRun [
        STATUS 0
    ],

    HfClkStat [
        /// Active clock source.
        SRC OFFSET(0) NUMBITS(1) [
            /// 16 MHz RC oscillator running and generating the HFCLK.
            RC = 0,
            /// 16 MHz HFCLK crystal oscillator running and generating the HFCLK.
            Xtal = 1
        ],
        /// HFCLK State.
        STATE OFFSET(16) NUMBITS(1) [
            NotRunning = 0,
            Running = 1
        ]
    ],

    LfClkStat [
        /// Active clock source.
        SRC OFFSET(0) NUMBITS(2) [
            /// 32.768 kHz RC oscillator running and generating the LFCLK.
            RC = 0,
            /// 32.768 kHz crystal oscillator running and generating the LFCLK.
            Xtal = 1,
            /// 32.768 kHz synthesizer synthesizing 32.768 kHz (from HFCLK) and generating the LFCLK.
            Synth = 2
        ],
        /// LFCLK State.
        STATE OFFSET(16) NUMBITS(1) [
            NotRunning = 0,
            Running = 1
        ]
    ],

    LfClkSrc [
        /// Clock source.
        SRC OFFSET(0) NUMBITS(2) [
            /// 32.768 kHz RC oscillator.
            RC = 0,
            /// 32.768 kHz crystal oscillator.
            Xtal = 1,
            /// 32.768 kHz synthesized from HFCLK.
            Synth = 2
        ]
    ],

    CalibrationTimerInterval [
        /// Calibration timer interval in multiple of 0.25 seconds.
        /// Range: 0.25 seconds to 31.75 seconds.
        CTIV OFFSET(0) NUMBITS(7)
    ],

    CrystalFrequency [
        /// Select nominal frequency of external crystal for HFCLK. This register
        /// has to match the actual crystal used in design to enable correct behaviour.
        XTALFREQ OFFSET(0) NUMBITS(8) [
            SixteenMHz = 0xff,
            ThirtyTwoMHz = 0x00
        ]
    ]
];

const CLOCK_BASE: StaticRef<ClockRegisters> =
    unsafe { StaticRef::new(0x40000000 as *const ClockRegisters) };

pub enum InterruptField {
    HFCLKSTARTED = (1 << 0),
    LFCLKSTARTED = (1 << 1),
    DONE = (1 << 3),
    CTTO = (1 << 4),
}

pub enum ClockTaskTriggered {
    NO = 0,
    YES = 1,
}

pub enum ClockRunning {
    NORUN = 0,
    RUN = (1 << 16),
}

pub enum LowClockSource {
    RC = 0,
    XTAL = 1,
    SYNTH = 2,
    MASK = 0x3,
}

pub enum HighClockSource {
    RC = 0,
    XTAL = 1,
}

pub enum XtalFreq {
    F16MHz = 0xFF,
    F32MHz = 0,
}

pub trait ClockClient {
    /// All clock interrupts are control signals, e.g., when
    /// a clock has started etc. We don't actually handle any
    /// of them for now, but keep this trait in place for if we
    /// do need to in the future.
    fn event(&self);
}

/// Clock struct
pub struct Clock {
    registers: StaticRef<ClockRegisters>,
    client: OptionalCell<&'static ClockClient>,
}

impl Clock {
    pub const fn new() -> Clock {
        Clock {
            registers: CLOCK_BASE,
            client: OptionalCell::empty(),
        }
    }

    pub fn set_client(&self, client: &'static ClockClient) {
        self.client.set(client);
    }

    pub fn interrupt_enable(&self, interrupt: InterruptField) {
        let regs = &*self.registers;
        regs.intenset.set(interrupt as u32);
    }

    pub fn interrupt_disable(&self, interrupt: InterruptField) {
        let regs = &*self.registers;
        regs.intenclr.set(interrupt as u32);
    }

    pub fn high_start(&self) {
        let regs = &*self.registers;
        regs.hfclkstart.write(Task::EXECUTE::SET);
    }

    pub fn high_stop(&self) {
        let regs = &*self.registers;
        regs.hfclkstop.write(Task::EXECUTE::SET);
    }

    pub fn high_started(&self) -> bool {
        let regs = &*self.registers;
        regs.hfclkstarted.is_set(Event::READY)
    }

    pub fn high_source(&self) -> HighClockSource {
        let regs = &*self.registers;
        match regs.hfclkstat.read_as_enum(HfClkStat::SRC) {
            Some(HfClkStat::SRC::Value::RC) => HighClockSource::RC,
            Some(HfClkStat::SRC::Value::Xtal) => HighClockSource::XTAL,
            None => unreachable!("invalid value"),
        }
    }

    pub fn high_freq(&self) -> XtalFreq {
        let regs = &*self.registers;
        match regs.xtalfreq.get() {
            0xff => XtalFreq::F16MHz,
            _ => XtalFreq::F32MHz,
        }
    }

    pub fn high_set_freq(&self, freq: XtalFreq) {
        let regs = &*self.registers;
        regs.xtalfreq.set(freq as u32);
    }

    pub fn high_running(&self) -> bool {
        let regs = &*self.registers;
        (regs.hfclkstat.get() & ClockRunning::RUN as u32) == ClockRunning::RUN as u32
    }

    #[no_mangle]
    #[inline(never)]
    pub fn low_start(&self) {
        let regs = &*self.registers;
        regs.lfclkstart.write(Task::EXECUTE::SET);
    }

    pub fn low_stop(&self) {
        let regs = &*self.registers;
        regs.lfclkstop.write(Task::EXECUTE::SET);
    }

    pub fn low_started(&self) -> bool {
        let regs = &*self.registers;
        regs.lfclkstarted.is_set(Event::READY)
    }

    pub fn low_source(&self) -> LowClockSource {
        let regs = &*self.registers;
        match regs.lfclkstat.get() & (LowClockSource::MASK as u32) {
            0b1 => LowClockSource::XTAL,
            0b10 => LowClockSource::SYNTH,
            _ => LowClockSource::RC,
        }
    }

    pub fn low_running(&self) -> bool {
        let regs = &*self.registers;
        (regs.lfclkstat.get() & ClockRunning::RUN as u32) == ClockRunning::RUN as u32
    }

    pub fn low_set_source(&self, src: LowClockSource) {
        let regs = &*self.registers;
        regs.lfclksrc.set(src as u32);
    }
}
