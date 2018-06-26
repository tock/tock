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

use core::cell::Cell;
use kernel::common::cells::VolatileCell;
use kernel::common::StaticRef;

pub static mut CLOCK: Clock = Clock::new();

#[repr(C)]
struct ClockRegisters {
    tasks_hfclkstart: VolatileCell<u32>,    // 0x000
    tasks_hfclkstop: VolatileCell<u32>,     // 0x004
    tasks_lfclkstart: VolatileCell<u32>,    // 0x008
    tasks_lfclkstop: VolatileCell<u32>,     // 0x00c
    tasks_cal: VolatileCell<u32>,           // 0x010
    tasks_cstart: VolatileCell<u32>,        // 0x014
    tasks_cstop: VolatileCell<u32>,         // 0x018
    _reserved1: [VolatileCell<u32>; 57],    // 0x01c - 0x100
    events_hfclkstarted: VolatileCell<u32>, // 0x100
    events_lfclkstarted: VolatileCell<u32>, // 0x104
    _reserved2: VolatileCell<u32>,          // 0x108
    events_done: VolatileCell<u32>,         // 0x10c
    events_ctto: VolatileCell<u32>,         // 0x110
    _reserved3: [VolatileCell<u32>; 124],   // 0x110 - 0x304
    intenset: VolatileCell<u32>,            // 0x304
    intenclr: VolatileCell<u32>,            // 0x308
    _reserved4: [VolatileCell<u32>; 63],    // 0x308 - 0x408
    hfclkrun: VolatileCell<u32>,            // 0x408
    hfclkstat: VolatileCell<u32>,           // 0x40c
    _reserved5: [VolatileCell<u32>; 1],     // 0x410
    lfclkrun: VolatileCell<u32>,            // 0x414
    lfclkstat: VolatileCell<u32>,           // 0x418
    lfclksrccopy: VolatileCell<u32>,        // 0x41c
    _reserved6: [VolatileCell<u32>; 62],    // 0x420 - 0x518
    lfclksrc: VolatileCell<u32>,            // 0x518
    _reserved7: [VolatileCell<u32>; 7],     // 0x51c - 0x538
    ctiv: VolatileCell<u32>,                // 0x538
    _reserved8: [VolatileCell<u32>; 5],     // 0x53c - 0x550
    xtalfreq: VolatileCell<u32>,            // 0x550
}

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
    client: Cell<Option<&'static ClockClient>>,
}

impl Clock {
    pub const fn new() -> Clock {
        Clock {
            registers: CLOCK_BASE,
            client: Cell::new(None),
        }
    }

    pub fn set_client(&self, client: &'static ClockClient) {
        self.client.set(Some(client));
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
        regs.tasks_hfclkstart.set(1);
    }

    pub fn high_stop(&self) {
        let regs = &*self.registers;
        regs.tasks_hfclkstop.set(1);
    }

    pub fn high_started(&self) -> bool {
        let regs = &*self.registers;
        regs.events_hfclkstarted.get() == 1
    }

    pub fn high_source(&self) -> HighClockSource {
        let regs = &*self.registers;
        match regs.hfclkstat.get() & 1 {
            0b0 => HighClockSource::RC,
            _ => HighClockSource::XTAL,
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
        regs.tasks_lfclkstart.set(1);
    }

    pub fn low_stop(&self) {
        let regs = &*self.registers;
        regs.tasks_lfclkstop.set(1);
    }

    pub fn low_started(&self) -> bool {
        let regs = &*self.registers;
        regs.events_lfclkstarted.get() == 1
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
