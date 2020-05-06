use core;
use core::cell::Cell;
use core::cmp::min;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::debug;
use kernel::hil::watchdog;
use kernel::ReturnCode;

const WATCHDOG_BASE: StaticRef<WatchdogRegisters> =
    unsafe { StaticRef::new(0x4000_480C as *const WatchdogRegisters) };

pub static mut WATCHDOG: Watchdog = Watchdog::new();

#[repr(C)]
struct WatchdogRegisters {
    ctl: ReadWrite<u32, WDTCTL::Register>,
}

register_bitfields! [u32,
    WDTCTL [
        WDTIS OFFSET(0) NUMBITS(3),
        WDTCNTCL OFFSET(3) NUMBITS(1),
        WDTTMSEL OFFSET(4) NUMBITS(1),
        WDTSSEL OFFSET(5) NUMBITS(2),
        WDTHOLD OFFSET(7) NUMBITS(1),
        WDTPW OFFSET(8) NUMBITS(8)
    ]
];

pub struct Watchdog {
    registers: StaticRef<WatchdogRegisters>,
}

impl Watchdog {
    pub const fn new() -> Watchdog {
        Watchdog {
            registers: WATCHDOG_BASE,
        }
    }
}

impl watchdog::Watchdog for Watchdog {
    fn start(&self, period: usize) {
        // TODO: implement the period for the watchdog
        let regs = &*self.registers;
        regs.ctl.modify(WDTCTL::WDTTMSEL.val(0)); // set to watchdog mode
        regs.ctl.modify(WDTCTL::WDTHOLD.val(0)); // enable watchdog
    }

    fn stop(&self) {
        let regs = &*self.registers;
        regs.ctl.modify(WDTCTL::WDTHOLD.val(1));
    }

    fn tickle(&self) {
        let regs = &*self.registers;
        regs.ctl.modify(WDTCTL::WDTCNTCL.val(1));
    }
}
