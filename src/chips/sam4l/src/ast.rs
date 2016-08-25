// chips::sam4l::ast -- Implementation of a single hardware timer.
//
// Author: Amit Levy <levya@cs.stanford.edu>
// Author: Philip Levis <pal@cs.stanford.edu>
// Date: July 16, 2015
//

use core::cell::Cell;
use core::intrinsics;
use hil::Controller;
use hil::alarm::{Alarm, AlarmClient, Freq16KHz};
use nvic;
use pm::{self, PBDClock};

#[repr(C, packed)]
#[allow(missing_copy_implementations)]
struct AstRegisters {
    cr: u32,
    cv: u32,
    sr: u32,
    scr: u32,
    ier: u32,
    idr: u32,
    imr: u32,
    wer: u32,
    // 0x20
    ar0: u32,
    ar1: u32,
    reserved0: [u32; 2],
    pir0: u32,
    pir1: u32,
    reserved1: [u32; 2],
    // 0x40
    clock: u32,
    dtr: u32,
    eve: u32,
    evd: u32,
    evm: u32,
    calv: u32, // we leave out parameter and version
}

pub const AST_BASE: isize = 0x400F0800;

#[allow(missing_copy_implementations)]
pub struct Ast {
    regs: *mut AstRegisters,
    callback: Cell<Option<&'static AlarmClient>>,
}

pub static mut AST: Ast = Ast {
    regs: AST_BASE as *mut AstRegisters,
    callback: Cell::new(None),
};

impl Controller for Ast {
    type Config = &'static AlarmClient;

    fn configure(&self, client: &'static AlarmClient) {
        self.callback.set(Some(client));

        unsafe {
            pm::enable_clock(pm::Clock::PBD(PBDClock::AST));
        }
        self.select_clock(Clock::ClockOsc32);
        self.set_prescalar(0); // 32KHz / (2^(0 + 1)) = 16KHz
        self.enable_alarm_wake();
        self.clear_alarm();
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

impl Ast {
    pub fn clock_busy(&self) -> bool {
        unsafe { intrinsics::volatile_load(&(*self.regs).sr) & (1 << 28) != 0 }
    }


    pub fn busy(&self) -> bool {
        unsafe { intrinsics::volatile_load(&(*self.regs).sr) & (1 << 24) != 0 }
    }

    // Clears the alarm bit in the status register (indicating the alarm value
    // has been reached).
    pub fn clear_alarm(&self) {
        while self.busy() {}
        unsafe {
            intrinsics::volatile_store(&mut (*self.regs).scr, 1 << 8);
            nvic::clear_pending(nvic::NvicIdx::ASTALARM);
        }
    }

    // Clears the per0 bit in the status register (indicating the alarm value
    // has been reached).
    pub fn clear_periodic(&mut self) {
        while self.busy() {}
        unsafe {
            intrinsics::volatile_store(&mut (*self.regs).scr, 1 << 16);
        }
    }


    pub fn select_clock(&self, clock: Clock) {
        unsafe {
            // Disable clock by setting first bit to zero
            while self.clock_busy() {}
            let enb = intrinsics::volatile_load(&(*self.regs).clock) & !1;
            intrinsics::volatile_store(&mut (*self.regs).clock, enb);
            while self.clock_busy() {}

            // Select clock
            intrinsics::volatile_store(&mut (*self.regs).clock, (clock as u32) << 8);
            while self.clock_busy() {}

            // Re-enable clock
            let enb = intrinsics::volatile_load(&(*self.regs).clock) | 1;
            intrinsics::volatile_store(&mut (*self.regs).clock, enb);
        }
    }

    pub fn enable(&self) {
        while self.busy() {}
        unsafe {
            let cr = intrinsics::volatile_load(&(*self.regs).cr) | 1;
            intrinsics::volatile_store(&mut (*self.regs).cr, cr);
        }
    }

    pub fn is_enabled(&self) -> bool {
        while self.busy() {}
        unsafe { intrinsics::volatile_load(&(*self.regs).cr) & 1 == 1 }
    }

    pub fn disable(&self) {
        while self.busy() {}
        unsafe {
            let cr = intrinsics::volatile_load(&(*self.regs).cr) & !1;
            intrinsics::volatile_store(&mut (*self.regs).cr, cr);
        }
    }

    pub fn set_prescalar(&self, val: u8) {
        while self.busy() {}
        unsafe {
            let cr = intrinsics::volatile_load(&(*self.regs).cr) | (val as u32) << 16;
            intrinsics::volatile_store(&mut (*self.regs).cr, cr);
        }
    }

    pub fn enable_alarm_irq(&self) {
        unsafe {
            nvic::enable(nvic::NvicIdx::ASTALARM);
            intrinsics::volatile_store(&mut (*self.regs).ier, 1 << 8);
        }
    }

    pub fn disable_alarm_irq(&self) {
        unsafe {
            intrinsics::volatile_store(&mut (*self.regs).idr, 1 << 8);
        }
    }

    pub fn enable_ovf_irq(&mut self) {
        unsafe {
            nvic::enable(nvic::NvicIdx::ASTOVF);
            intrinsics::volatile_store(&mut (*self.regs).ier, 1);
        }
    }

    pub fn disable_ovf_irq(&mut self) {
        unsafe {
            intrinsics::volatile_store(&mut (*self.regs).idr, 1);
        }
    }

    pub fn enable_periodic_irq(&mut self) {
        unsafe {
            nvic::enable(nvic::NvicIdx::ASTPER);
            intrinsics::volatile_store(&mut (*self.regs).ier, 1 << 16);
        }
    }

    pub fn disable_periodic_irq(&mut self) {
        unsafe {
            intrinsics::volatile_store(&mut (*self.regs).idr, 1 << 16);
        }
    }

    pub fn enable_alarm_wake(&self) {
        while self.busy() {}
        unsafe {
            let wer = intrinsics::volatile_load(&mut (*self.regs).wer) | 1 << 8;
            intrinsics::volatile_store(&mut (*self.regs).wer, wer);
        }
    }

    pub fn set_periodic_interval(&mut self, interval: u32) {
        while self.busy() {}
        unsafe {
            intrinsics::volatile_store(&mut (*self.regs).pir0, interval);
        }
    }

    pub fn get_counter(&self) -> u32 {
        while self.busy() {}
        unsafe { intrinsics::volatile_load(&(*self.regs).cv) }
    }


    pub fn set_counter(&self, value: u32) {
        while self.busy() {}
        unsafe {
            intrinsics::volatile_store(&mut (*self.regs).cv, value);
        }
    }

    pub fn handle_interrupt(&mut self) {
        self.clear_alarm();
        self.callback.get().map(|cb| {
            cb.fired();
        });
    }
}

impl Alarm for Ast {
    type Frequency = Freq16KHz;

    fn now(&self) -> u32 {
        while self.busy() {}
        unsafe { intrinsics::volatile_load(&(*self.regs).cv) }
    }

    fn disable_alarm(&self) {
        self.disable_alarm_irq();
    }

    fn is_armed(&self) -> bool {
        self.is_enabled()
    }

    fn set_alarm(&self, tics: u32) {
        self.disable();
        while self.busy() {}
        unsafe {
            intrinsics::volatile_store(&mut (*self.regs).ar0, tics);
        }
        self.clear_alarm();
        self.enable_alarm_irq();
        self.enable();
    }

    fn get_alarm(&self) -> u32 {
        while self.busy() {}
        unsafe { intrinsics::volatile_load(&(*self.regs).ar0) }
    }
}

interrupt_handler!(ast_alarm_handler, ASTALARM);
