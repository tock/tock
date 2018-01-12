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
use core::mem;
use kernel::common::VolatileCell;

#[repr(C)]
struct Registers {
    pub tasks_hfclkstart: VolatileCell<u32>,
    pub tasks_hfclkstop: VolatileCell<u32>,
    pub tasks_lfclkstart: VolatileCell<u32>,
    pub tasks_lfclkstop: VolatileCell<u32>,
    pub tasks_cal: VolatileCell<u32>,
    pub tasks_cstart: VolatileCell<u32>,
    pub tasks_cstop: VolatileCell<u32>,
    _reserved1: [VolatileCell<u32>; 57],
    pub events_hfclkstarted: VolatileCell<u32>,
    pub events_lfclkstarted: VolatileCell<u32>,
    pub done: VolatileCell<u32>,
    pub ctto: VolatileCell<u32>,
    _reserved2: [VolatileCell<u32>; 125],
    pub intenset: VolatileCell<u32>,
    pub intenclr: VolatileCell<u32>,
    _reserved3: [VolatileCell<u32>; 63],
    pub hfclkrun: VolatileCell<u32>,
    pub hfclkstat: VolatileCell<u32>,
    _reserved4: [VolatileCell<u32>; 1],
    pub lfclkrun: VolatileCell<u32>,
    pub lfclkstat: VolatileCell<u32>,
    pub lfclksrccopy: VolatileCell<u32>,
    _reserved5: [VolatileCell<u32>; 62],
    pub lfclksrc: VolatileCell<u32>,
    _reserved6: [VolatileCell<u32>; 7],
    pub ctiv: VolatileCell<u32>,
    _reserved7: [VolatileCell<u32>; 5],
    pub xtalfreq: VolatileCell<u32>,
}

const CLOCK_BASE: usize = 0x40000000;

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

pub struct Clock {
    client: Cell<Option<&'static ClockClient>>,
}

pub trait ClockClient {
    /// All clock interrupts are control signals, e.g., when
    /// a clock has started etc. We don't actually handle any
    /// of them for now, but keep this trait in place for if we
    /// do need to in the future.
    fn event(&self);
}

pub static mut CLOCK: Clock = Clock {
    client: Cell::new(None),
};

#[allow(non_snake_case)]
fn CLK() -> &'static Registers {
    unsafe { mem::transmute(CLOCK_BASE as usize) }
}

impl Clock {
    pub fn set_client(&self, client: &'static ClockClient) {
        self.client.set(Some(client));
    }

    pub fn interrupt_enable(&self, interrupt: InterruptField) {
        CLK().intenset.set(interrupt as u32);
    }

    pub fn interrupt_disable(&self, interrupt: InterruptField) {
        CLK().intenclr.set(interrupt as u32);
    }

    pub fn high_start(&self) {
        CLK().tasks_hfclkstart.set(1);
    }

    pub fn high_stop(&self) {
        CLK().tasks_hfclkstop.set(1);
    }

    pub fn high_started(&self) -> bool {
        CLK().events_hfclkstarted.get() == 1
    }

    pub fn high_source(&self) -> HighClockSource {
        match CLK().hfclkstat.get() & 1 {
            0b0 => HighClockSource::RC,
            _ => HighClockSource::XTAL,
        }
    }

    pub fn high_freq(&self) -> XtalFreq {
        match CLK().xtalfreq.get() {
            0xff => XtalFreq::F16MHz,
            _ => XtalFreq::F32MHz,
        }
    }

    pub fn high_set_freq(&self, freq: XtalFreq) {
        CLK().xtalfreq.set(freq as u32);
    }

    pub fn high_running(&self) -> bool {
        (CLK().hfclkstat.get() & ClockRunning::RUN as u32) == ClockRunning::RUN as u32
    }

    pub fn low_start(&self) {
        CLK().tasks_lfclkstart.set(1);
    }

    pub fn low_stop(&self) {
        CLK().tasks_lfclkstop.set(1);
    }

    pub fn low_started(&self) -> bool {
        CLK().events_lfclkstarted.get() == 1
    }

    pub fn low_source(&self) -> LowClockSource {
        match CLK().lfclkstat.get() & (LowClockSource::MASK as u32) {
            0b1 => LowClockSource::XTAL,
            0b10 => LowClockSource::SYNTH,
            _ => LowClockSource::RC,
        }
    }

    pub fn low_running(&self) -> bool {
        (CLK().lfclkstat.get() & ClockRunning::RUN as u32) == ClockRunning::RUN as u32
    }

    pub fn low_set_source(&self, src: LowClockSource) {
        CLK().lfclksrc.set(src as u32);
    }
}
