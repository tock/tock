//! Implementation of the SAM4L hardware watchdog timer.

use core::cell::Cell;
use kernel::common::VolatileCell;
use kernel::hil;
use pm::{self, Clock, PBDClock};

#[repr(C, packed)]
pub struct WdtRegisters {
    cr: VolatileCell<u32>,
    clr: VolatileCell<u32>,
    sr: VolatileCell<u32>,
    ier: VolatileCell<u32>,
    idr: VolatileCell<u32>,
    imr: VolatileCell<u32>,
    isr: VolatileCell<u32>,
    icr: VolatileCell<u32>,
}

// Page 59 of SAM4L data sheet
const BASE_ADDRESS: *mut WdtRegisters = 0x400F0C00 as *mut WdtRegisters;

pub struct Wdt {
    registers: *mut WdtRegisters,
    enabled: Cell<bool>,
}

pub static mut WDT: Wdt = Wdt::new(BASE_ADDRESS);

impl Wdt {
    const fn new(base_address: *mut WdtRegisters) -> Wdt {
        Wdt {
            registers: base_address,
            enabled: Cell::new(false),
        }
    }

    fn start(&self, period: usize) {
        let regs: &WdtRegisters = unsafe { &*self.registers };

        self.enabled.set(true);

        unsafe {
            pm::enable_clock(Clock::PBD(PBDClock::WDT));
        }

        // Choose the best period setting based on what was passed to `start()`
        let scaler = match period {
            0...2 => 7,
            3...6 => 8,
            7...12 => 9,
            13...24 => 10,
            25...48 => 11,
            49...96 => 12,
            97...192 => 13,
            193...384 => 14,
            385...768 => 15,
            769...1536 => 16,
            1537...3072 => 17,
            3073...6144 => 18,
            6145...12288 => 19,
            12289...24576 => 20,
            24577...49152 => 21,
            49153...98304 => 22,
            98305...196608 => 23,
            196609...393216 => 24,
            393217...786432 => 25,
            786433...1572864 => 26,
            1572865...3145728 => 27,
            3145729...6291456 => 28,
            6291457...12582912 => 29,
            12582913...25165824 => 30,
            _ => 31,
        };

        let control = (1 << 16) |     // Clock enable
                      (scaler << 8) | // Set PSEL to based on period
                      (1 << 7)  |     // Flash calibration done (set to default)
                      (1 << 1)  |     // Disable after reset
                      (1 << 0); //...... Enable

        // Need to write twice for it to work
        regs.cr.set((0x55 << 24) | control);
        regs.cr.set((0xAA << 24) | control);
    }

    fn stop(&self) {
        let regs: &WdtRegisters = unsafe { &*self.registers };

        // Set enable bit (bit 0) to 0 to disable
        let control = regs.cr.get() & !0x01;

        // Need to write twice for it to work
        regs.cr.set((0x55 << 24) | control);
        regs.cr.set((0xAA << 24) | control);

        unsafe {
            pm::disable_clock(Clock::PBD(PBDClock::WDT));
        }

        self.enabled.set(false);
    }

    fn tickle(&self) {
        let regs: &WdtRegisters = unsafe { &*self.registers };

        // Need to write the WDTCLR bit twice for it to work
        regs.clr.set((0x55 << 24) | (1 << 0));
        regs.clr.set((0xAA << 24) | (1 << 0));
    }
}

impl hil::watchdog::Watchdog for Wdt {
    fn start(&self, period: usize) {
        self.start(period);
    }

    fn stop(&self) {
        self.stop();
    }

    fn tickle(&self) {
        self.tickle();
    }
}
