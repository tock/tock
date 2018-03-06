//! Clock peripheral driver, nRF52
//!
//! Forked off Phil Levis Clock Driver for nRF51
//!
//! LFCLK - Low Frequency Clock Source:
//!     * 32.768 kHz RC oscillator (LFRC)
//!     * 32.768 kHz crystal oscillator (LFXO)
//!     * 32.768 kHz synthesized from HFCLK (LFSYNT)
//!
//! HFCLK - High Frequency Clock
//!     * 64 MHz internal oscillator (HFINT)
//!     * 64 MHz crystal oscillator (HFXO)
//!     * HFXO must be running the run the RADIO, NFC and calibration
//!

use core::cell::Cell;
use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};

struct ClockRegisters {
    pub tasks_hfclkstart: WriteOnly<u32, Control::Register>, // 0x000
    pub tasks_hfclkstop: WriteOnly<u32, Control::Register>,  // 0x004
    pub tasks_lfclkstart: ReadWrite<u32, Control::Register>, // 0x008
    pub tasks_lfclkstop: WriteOnly<u32, Control::Register>,  // 0x00c
    pub tasks_cal: WriteOnly<u32, Control::Register>,        // 0x010
    pub tasks_ctstart: WriteOnly<u32, Control::Register>,    // 0x014
    pub tasks_ctstop: WriteOnly<u32, Control::Register>,     // 0x018
    _reserved1: [u32; 57],                                   // 0x018 - 0x100
    pub events_hfclkstarted: ReadOnly<u32, Status::Register>, // 0x100
    pub events_lfclkstarted: ReadOnly<u32, Status::Register>, // 0x104
    _reserverd2: u32,                                        // 0x108
    pub events_done: ReadOnly<u32, Status::Register>,        // 0x10c
    pub events_ctto: ReadOnly<u32, Status::Register>,        // 0x110
    _reserved3: [u32; 124],                                  // 0x114 - 0x304
    pub intenset: ReadWrite<u32, Interrupt::Register>,       // 0x304
    pub intenclr: ReadWrite<u32, Interrupt::Register>,       // 0x308
    _reserved4: [u32; 63],                                   // 0x30c - 0x408
    pub hfclkrun: ReadOnly<u32, Status::Register>,           // 0x408
    pub hfclkstat: ReadWrite<u32, HfClkStat::Register>,      // 0x40c
    _reserved5: [u32; 1],                                    // 0x410
    pub lfclkrun: ReadOnly<u32, Control::Register>,          // 0x414
    pub lfclkstat: ReadWrite<u32, LfClkStat::Register>,      // 0x418
    pub lfclksrccopy: ReadOnly<u32, LfClkSrcCopy::Register>, // 0x41c
    _reserved6: [u32; 62],                                   // 0x420 - 0x518
    pub lfclksrc: ReadWrite<u32, LfClkSrc::Register>,        // 0x518
    _reserved7: [u32; 7],                                    // 0x51c - 0x538
    pub ctiv: ReadWrite<u32, Ctiv::Register>,                // 0x538
    _reserved8: [u32; 8],                                    // 0x53c - 0x55c
    pub traceconfig: ReadWrite<u32, TraceConfig::Register>,  // 0x55c
}

register_bitfields! [u32,
    Control [
        DISABLE 0,
        ENABLE 1
    ],
    Status [
        NOTREADY 0,
        READY 1
    ],
    Interrupt [
        HFCLKSTARTED 1,
        LFCLKSTARTED 2,
        DONE 8,
        CTTO 16
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

const CLOCK_BASE: usize = 0x40000000;

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
    registers: *const ClockRegisters,
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
    pub const fn new() -> Clock {
        Clock {
            registers: CLOCK_BASE as *const ClockRegisters,
            client: Cell::new(None),
        }
    }

    pub fn set_client(&self, client: &'static ClockClient) {
        self.client.set(Some(client));
    }

    // pub fn interrupt_enable(&self, interrupt: InterruptField) {
    //     let regs = unsafe { &*self.registers };
    //     regs.intenset.write(Interrupt::val(interrupt as u32);
    // }
    //
    // pub fn interrupt_disable(&self, interrupt: InterruptField) {
    //     let regs = &*self.registers;
    //     //regs.intenclr.set(interrupt as u32);
    // }

    pub fn high_start(&self) {
        let regs = unsafe { &*self.registers };
        regs.tasks_hfclkstart.write(Control::ENABLE::SET);
    }

    pub fn high_stop(&self) {
        let regs = unsafe { &*self.registers };
        regs.tasks_hfclkstop.write(Control::ENABLE::SET);
    }

    pub fn high_started(&self) -> bool {
        let regs = unsafe { &*self.registers };
        regs.events_hfclkstarted.matches(Status::READY.val(1))
    }

    pub fn high_source(&self) -> HighClockSource {
        let regs = unsafe { &*self.registers };
        match regs.hfclkstat.read(HfClkStat::SRC) {
            0b0 => HighClockSource::RC,
            _ => HighClockSource::XTAL,
        }
    }

    pub fn high_running(&self) -> bool {
        let regs = unsafe { &*self.registers };
        regs.hfclkstat.matches(HfClkStat::STATE::RUNNING)
    }

    // for debugging
    #[no_mangle]
    #[inline(never)]
    pub fn low_start(&self) {
        let regs = unsafe { &*self.registers };
        regs.tasks_lfclkstart.write(Control::ENABLE::SET);
    }

    pub fn low_stop(&self) {
        let regs = unsafe { &*self.registers };
        regs.tasks_lfclkstop.write(Control::ENABLE::SET);
    }

    pub fn low_started(&self) -> bool {
        let regs = unsafe { &*self.registers };
        regs.events_lfclkstarted.matches(Status::READY::SET)
    }

    pub fn low_source(&self) -> LowClockSource {
        let regs = unsafe { &*self.registers };
        match regs.lfclkstat.read(LfClkStat::SRC) {
            0b1 => LowClockSource::XTAL,
            0b10 => LowClockSource::SYNTH,
            _ => LowClockSource::RC,
        }
    }

    pub fn low_running(&self) -> bool {
        let regs = unsafe { &*self.registers };
        regs.lfclkstat.matches(LfClkStat::STATE::RUNNING)
    }

    pub fn low_set_source(&self, clock_source: LowClockSource) {
        let regs = unsafe { &*self.registers };
        regs.lfclksrc.write(LfClkSrc::SRC.val(clock_source as u32));
    }

    pub fn high_set_source(&self, clock_source: HighClockSource) {
        let regs = unsafe { &*self.registers };
        regs.hfclkstat
            .write(HfClkStat::SRC.val(clock_source as u32));
    }
}
