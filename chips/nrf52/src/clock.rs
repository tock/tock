//! Clock peripheral driver, nRF52
//!
//! Based on Phil Levis clock driver for nRF51
//!
//! HFCLK - High Frequency Clock:
//!
//!     * 64 MHz internal oscillator (HFINT)
//!     * 64 MHz crystal oscillator, using 32 MHz external crystal (HFXO)
//!     * The HFXO must be running to use the RADIO, NFC module or the calibration mechanism
//!       associated with the 32.768 kHz RC oscillator.
//!
//! LFCLK - Low Frequency Clock Source:
//!
//!     * 32.768 kHz RC oscillator (LFRC)
//!     * 32.768 kHz crystal oscillator (LFXO)
//!     * 32.768 kHz synthesized from HFCLK (LFSYNT)
//!

use core::cell::Cell;
use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;

#[repr(C)]
struct ClockRegisters {
    tasks_hfclkstart: WriteOnly<u32, Control::Register>,
    tasks_hfclkstop: WriteOnly<u32, Control::Register>,
    tasks_lfclkstart: ReadWrite<u32, Control::Register>,
    tasks_lfclkstop: WriteOnly<u32, Control::Register>,
    tasks_cal: WriteOnly<u32, Control::Register>,
    tasks_ctstart: WriteOnly<u32, Control::Register>,
    tasks_ctstop: WriteOnly<u32, Control::Register>,
    _reserved1: [u32; 57],
    events_hfclkstarted: ReadOnly<u32, Status::Register>,
    events_lfclkstarted: ReadOnly<u32, Status::Register>,
    _reserverd2: u32,
    events_done: ReadOnly<u32, Status::Register>,
    events_ctto: ReadOnly<u32, Status::Register>,
    _reserved3: [u32; 124],
    intenset: ReadWrite<u32, Interrupt::Register>,
    intenclr: ReadWrite<u32, Interrupt::Register>,
    _reserved4: [u32; 63],
    hfclkrun: ReadOnly<u32, Status::Register>,
    hfclkstat: ReadWrite<u32, HfClkStat::Register>,
    _reserved5: [u32; 1],
    lfclkrun: ReadOnly<u32, Control::Register>,
    lfclkstat: ReadWrite<u32, LfClkStat::Register>,
    lfclksrccopy: ReadOnly<u32, LfClkSrcCopy::Register>,
    _reserved6: [u32; 62],
    lfclksrc: ReadWrite<u32, LfClkSrc::Register>,
    _reserved7: [u32; 7],
    ctiv: ReadWrite<u32, Ctiv::Register>,
    _reserved8: [u32; 8],
    traceconfig: ReadWrite<u32, TraceConfig::Register>,
}

register_bitfields! [u32,
    Control [
        ENABLE OFFSET(0) NUMBITS(1)
    ],
    Status [
        READY OFFSET(0) NUMBITS(1)
    ],
    Interrupt [
        HFCLKSTARTED OFFSET(0) NUMBITS(1),
        LFCLKSTARTED OFFSET(1) NUMBITS(1),
        DONE OFFSET(3) NUMBITS(1),
        CTTO OFFSET(4) NUMBITS(1)
    ],
    HfClkStat [
        SRC OFFSET(0) NUMBITS(1) [
            RC = 0,
            XTAL = 1
        ],
        STATE OFFSET(16) NUMBITS(1) [
            RUNNING = 1
        ]
    ],
    LfClkStat [
        SRC OFFSET(0) NUMBITS(2) [
            RC = 0,
            XTAL = 1,
            SYNTH = 2
        ],
        STATE OFFSET(16) NUMBITS(1) [
            RUNNING = 1
        ]
    ],
    LfClkSrcCopy [
        SRC OFFSET(0) NUMBITS(2) [
            RC = 0,
            XTAL = 1,
            SYNTH = 2
        ]
    ],
    LfClkSrc [
        SRC OFFSET(0) NUMBITS(2) [
            RC = 0,
            XTAL = 1,
            SYNTH = 2
        ]
    ],
    Ctiv [
        CTIV OFFSET(0) NUMBITS(7) []
    ],
    TraceConfig [
        TracePortSpeed OFFSET(0) NUMBITS(2) [
            THIRTYTWO = 0,
            SIXTEEN = 1,
            EIGHT = 2,
            FOUR = 3
        ],
        TraceMux OFFSET(16) NUMBITS(2) [
            GPIO = 0,
            SERIAL = 1,
            PARALELL = 2
        ]
    ]
];

const CLOCK_BASE: StaticRef<ClockRegisters> =
    unsafe { StaticRef::new(0x40000000 as *const ClockRegisters) };

/// Interrupt sources
pub enum InterruptField {
    HFCLKSTARTED = (1 << 0),
    LFCLKSTARTED = (1 << 1),
    DONE = (1 << 3),
    CTTO = (1 << 4),
}

/// Low frequency clock source
pub enum LowClockSource {
    RC = 0,
    XTAL = 1,
    SYNTH = 2,
    MASK = 3,
}

/// High frequency clock source
pub enum HighClockSource {
    RC = 0,
    XTAL = 1,
}

/// Clock struct
pub struct Clock {
    registers: StaticRef<ClockRegisters>,
    client: Cell<Option<&'static ClockClient>>,
}

pub trait ClockClient {
    /// All clock interrupts are control signals, e.g., when
    /// a clock has started etc. We don't actually handle any
    /// of them for now, but keep this trait in place for if we
    /// do need to in the future.
    fn event(&self);
}

pub static mut CLOCK: Clock = Clock::new();

impl Clock {
    /// Constructor
    pub const fn new() -> Clock {
        Clock {
            registers: CLOCK_BASE,
            client: Cell::new(None),
        }
    }

    /// Client for callbacks
    pub fn set_client(&self, client: &'static ClockClient) {
        self.client.set(Some(client));
    }

    /// Enable interrupt
    pub fn interrupt_enable(&self, interrupt: InterruptField) {
        let regs = &*self.registers;
        // this is a little too verbose
        match interrupt {
            InterruptField::CTTO => regs.intenset.write(Interrupt::CTTO::SET),
            InterruptField::DONE => regs.intenset.write(Interrupt::DONE::SET),
            InterruptField::HFCLKSTARTED => regs.intenset.write(Interrupt::HFCLKSTARTED::SET),
            InterruptField::LFCLKSTARTED => regs.intenset.write(Interrupt::LFCLKSTARTED::SET),
        }
    }

    /// Disable interrupt
    pub fn interrupt_disable(&self, interrupt: InterruptField) {
        let regs = &*self.registers;
        // this is a little too verbose
        match interrupt {
            InterruptField::CTTO => regs.intenset.write(Interrupt::CTTO::SET),
            InterruptField::DONE => regs.intenset.write(Interrupt::DONE::SET),
            InterruptField::HFCLKSTARTED => regs.intenset.write(Interrupt::HFCLKSTARTED::SET),
            InterruptField::LFCLKSTARTED => regs.intenset.write(Interrupt::LFCLKSTARTED::SET),
        }
    }

    /// Start the high frequency clock
    pub fn high_start(&self) {
        let regs = &*self.registers;
        regs.tasks_hfclkstart.write(Control::ENABLE::SET);
    }

    /// Stop the high frequency clock
    pub fn high_stop(&self) {
        let regs = &*self.registers;
        regs.tasks_hfclkstop.write(Control::ENABLE::SET);
    }

    /// Check if the high frequency clock has started
    pub fn high_started(&self) -> bool {
        let regs = &*self.registers;
        regs.events_hfclkstarted.matches_all(Status::READY.val(1))
    }

    /// Read clock source from the high frequency clock
    pub fn high_source(&self) -> HighClockSource {
        let regs = &*self.registers;
        match regs.hfclkstat.read(HfClkStat::SRC) {
            0 => HighClockSource::RC,
            _ => HighClockSource::XTAL,
        }
    }

    /// Check if the high frequency clock is running
    pub fn high_running(&self) -> bool {
        let regs = &*self.registers;
        regs.hfclkstat.matches_all(HfClkStat::STATE::RUNNING)
    }

    /// Start the low frequency clock
    pub fn low_start(&self) {
        let regs = &*self.registers;
        regs.tasks_lfclkstart.write(Control::ENABLE::SET);
    }

    /// Stop the low frequency clock
    pub fn low_stop(&self) {
        let regs = &*self.registers;
        regs.tasks_lfclkstop.write(Control::ENABLE::SET);
    }

    /// Check if the low frequency clock has started
    pub fn low_started(&self) -> bool {
        let regs = &*self.registers;
        regs.events_lfclkstarted.matches_all(Status::READY::SET)
    }

    /// Read clock source from the low frequency clock
    pub fn low_source(&self) -> LowClockSource {
        let regs = &*self.registers;
        match regs.lfclkstat.read(LfClkStat::SRC) {
            0b1 => LowClockSource::XTAL,
            0b10 => LowClockSource::SYNTH,
            _ => LowClockSource::RC,
        }
    }

    /// Check if the low frequency clock is running
    pub fn low_running(&self) -> bool {
        let regs = &*self.registers;
        regs.lfclkstat.matches_all(LfClkStat::STATE::RUNNING)
    }

    /// Set low frequency clock source
    pub fn low_set_source(&self, clock_source: LowClockSource) {
        let regs = &*self.registers;
        regs.lfclksrc.write(LfClkSrc::SRC.val(clock_source as u32));
    }

    /// Set high frequency clock source
    pub fn high_set_source(&self, clock_source: HighClockSource) {
        let regs = &*self.registers;
        regs.hfclkstat
            .write(HfClkStat::SRC.val(clock_source as u32));
    }
}
