//! Implementation of a single hardware timer.
//!
//! - Author: Amit Levy <levya@cs.stanford.edu>
//! - Author: Philip Levis <pal@cs.stanford.edu>
//! - Date: July 16, 2015

use core::cell::Cell;
use kernel::common::VolatileCell;
use kernel::hil::Controller;
use kernel::hil::time::{self, Alarm, Time, Freq16KHz};
use pm::{self, PBDClock};

/// Minimum number of clock tics to make sure ALARM0 register is synchronized
///
/// The datasheet has the following ominous language (Section 19.5.3.2):
///
/// > Because of synchronization, the transfer of the alarm value will not
/// > happen immediately. When changing/setting the alarm value, the user must
/// > make sure that the counter will not count the selected alarm value before
/// > the value is transferred to the register. In that case, the first alarm
/// > interrupt after the change will not be triggered.
///
/// In practice, we've observed that when the alarm is set for a counter value
/// less than or equal to four tics ahead of the current counter value, the
/// alarm interrupt doesn't fire. Thus, we simply round up to at least eight
/// tics. Seems safe enough and in practice has seemed to work.
const ALARM0_SYNC_TICS: u32 = 8;

#[repr(C, packed)]
struct AstRegisters {
    cr: VolatileCell<u32>,
    cv: VolatileCell<u32>,
    sr: VolatileCell<u32>,
    scr: VolatileCell<u32>,
    ier: VolatileCell<u32>,
    idr: VolatileCell<u32>,
    imr: VolatileCell<u32>,
    wer: VolatileCell<u32>,
    // 0x20
    ar0: VolatileCell<u32>,
    ar1: VolatileCell<u32>,
    _reserved0: [u32; 2],
    pir0: VolatileCell<u32>,
    pir1: VolatileCell<u32>,
    _reserved1: [u32; 2],
    // 0x40
    clock: VolatileCell<u32>,
    dtr: VolatileCell<u32>,
    eve: VolatileCell<u32>,
    evd: VolatileCell<u32>,
    evm: VolatileCell<u32>,
    calv: VolatileCell<u32>, // we leave out parameter and version
}

const AST_BASE: usize = 0x400F0800;

pub struct Ast<'a> {
    regs: *const AstRegisters,
    callback: Cell<Option<&'a time::Client>>,
}

pub static mut AST: Ast<'static> = Ast {
    regs: AST_BASE as *const AstRegisters,
    callback: Cell::new(None),
};

impl<'a> Controller for Ast<'a> {
    type Config = &'static time::Client;

    fn configure(&self, client: &'a time::Client) {
        self.callback.set(Some(client));

        unsafe {
            pm::enable_clock(pm::Clock::PBD(PBDClock::AST));
        }
        self.select_clock(Clock::ClockOsc32);
        self.set_prescalar(0); // 32KHz / (2^(0 + 1)) = 16KHz
        self.enable_alarm_wake();
        self.clear_alarm();
        self.enable();
    }
}

#[repr(usize)]
pub enum Clock {
    ClockRCSys = 0,
    ClockOsc32 = 1,
    ClockAPB = 2,
    ClockGclk2 = 3,
    Clock1K = 4,
}

impl<'a> Ast<'a> {
    pub fn clock_busy(&self) -> bool {
        unsafe { (*self.regs).sr.get() & (1 << 28) != 0 }
    }

    pub fn set_client(&self, client: &'a time::Client) {
        self.callback.set(Some(client));
    }

    pub fn busy(&self) -> bool {
        unsafe { (*self.regs).sr.get() & (1 << 24) != 0 }
    }

    // Clears the alarm bit in the status register (indicating the alarm value
    // has been reached).
    pub fn clear_alarm(&self) {
        while self.busy() {}
        unsafe {
            (*self.regs).scr.set(1 << 8);
        }
    }

    // Clears the per0 bit in the status register (indicating the alarm value
    // has been reached).
    pub fn clear_periodic(&mut self) {
        while self.busy() {}
        unsafe {
            // ptr::write_volatile(&mut (*self.regs).scr, 1 << 16);
            (*self.regs).scr.set(1 << 16);
        }
    }

    pub fn select_clock(&self, clock: Clock) {
        unsafe {
            // Disable clock by setting first bit to zero
            while self.clock_busy() {}
            let enb = (*self.regs).clock.get() & !1;
            (*self.regs).clock.set(enb);
            while self.clock_busy() {}

            // Select clock
            (*self.regs).clock.set((clock as u32) << 8);
            while self.clock_busy() {}

            // Re-enable clock
            let enb = (*self.regs).clock.get() | 1;
            (*self.regs).clock.set(enb);
        }
    }

    pub fn enable(&self) {
        while self.busy() {}
        unsafe {
            let cr = (*self.regs).cr.get() | 1;
            (*self.regs).cr.set(cr);
        }
    }

    pub fn is_enabled(&self) -> bool {
        while self.busy() {}
        unsafe { (*self.regs).cr.get() & 1 == 1 }
    }

    pub fn disable(&self) {
        while self.busy() {}
        unsafe {
            let cr = (*self.regs).cr.get() & !1;
            (*self.regs).cr.set(cr);
        }
    }

    pub fn set_prescalar(&self, val: u8) {
        while self.busy() {}
        unsafe {
            let cr = (*self.regs).cr.get() | (val as u32) << 16;
            (*self.regs).cr.set(cr);
        }
    }

    pub fn enable_alarm_irq(&self) {
        unsafe {
            (*self.regs).ier.set(1 << 8);
        }
    }

    pub fn disable_alarm_irq(&self) {
        unsafe {
            (*self.regs).idr.set(1 << 8);
        }
    }

    pub fn enable_ovf_irq(&mut self) {
        unsafe {
            (*self.regs).ier.set(1);
        }
    }

    pub fn disable_ovf_irq(&mut self) {
        unsafe {
            (*self.regs).idr.set(1);
        }
    }

    pub fn enable_periodic_irq(&mut self) {
        unsafe {
            (*self.regs).ier.set(1 << 16);
        }
    }

    pub fn disable_periodic_irq(&mut self) {
        unsafe {
            (*self.regs).idr.set(1 << 16);
        }
    }

    pub fn enable_alarm_wake(&self) {
        while self.busy() {}
        unsafe {
            let wer = (*self.regs).wer.get() | 1 << 8;
            (*self.regs).wer.set(wer);
        }
    }

    pub fn set_periodic_interval(&mut self, interval: u32) {
        while self.busy() {}
        unsafe {
            (*self.regs).pir0.set(interval);
        }
    }

    pub fn get_counter(&self) -> u32 {
        while self.busy() {}
        unsafe { (*self.regs).cv.get() }
    }


    pub fn set_counter(&self, value: u32) {
        while self.busy() {}
        unsafe {
            (*self.regs).cv.set(value);
        }
    }

    pub fn handle_interrupt(&mut self) {
        self.clear_alarm();
        self.callback.get().map(|cb| { cb.fired(); });
    }
}

impl<'a> Time for Ast<'a> {
    type Frequency = Freq16KHz;

    fn disable(&self) {
        self.disable_alarm_irq();
    }

    fn is_armed(&self) -> bool {
        self.is_enabled()
    }
}

impl<'a> Alarm for Ast<'a> {
    fn now(&self) -> u32 {
        while self.busy() {}
        unsafe { (*self.regs).cv.get() }
    }

    fn set_alarm(&self, mut tics: u32) {
        while self.busy() {}
        unsafe {
            let now = (*self.regs).cv.get();
            if tics.wrapping_sub(now) <= ALARM0_SYNC_TICS {
                tics = now.wrapping_add(ALARM0_SYNC_TICS);
            }
            (*self.regs).ar0.set(tics);
        }
        self.clear_alarm();
        self.enable_alarm_irq();
    }

    fn get_alarm(&self) -> u32 {
        while self.busy() {}
        unsafe { (*self.regs).ar0.get() }
    }
}

interrupt_handler!(ast_alarm_handler, ASTALARM);
