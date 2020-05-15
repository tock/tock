// Watchdog Timer (WDT)

use kernel::common::registers::{register_bitfields, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil::watchdog;

pub static mut WATCHDOG: Watchdog = Watchdog::new();

const WATCHDOG_BASE: StaticRef<WatchdogRegisters> =
    unsafe { StaticRef::new(0x4000_480Cu32 as *const WatchdogRegisters) };

/// every write access has to set this 'password' in the upper 8 bit of the
/// register, otherwise the watchdog resets the whole system
const PASSWORD: u16 = 0x5A;

#[repr(C)]
struct WatchdogRegisters {
    ctl: ReadWrite<u16, WDTCTL::Register>,
}

register_bitfields! [u16,
    WDTCTL [
        // Watchdog timer interval selection
        WDTIS OFFSET(0) NUMBITS(3),
        // Watchdog timer counter clear
        WDTCNTCL OFFSET(3) NUMBITS(1),
        // Watchdog timer mode select
        WDTTMSEL OFFSET(4) NUMBITS(1),
        // Watchdog timer clock source select
        WDTSSEL OFFSET(5) NUMBITS(2),
        // Watchdog timer hold -> enable/disable
        WDTHOLD OFFSET(7) NUMBITS(1),
        // Watchdog timer password
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
        self.registers
            .ctl
            .modify(WDTCTL::WDTPW.val(PASSWORD) + WDTCTL::WDTHOLD::CLEAR);
    }

    fn stop(&self) {
        self.registers
            .ctl
            .modify(WDTCTL::WDTPW.val(PASSWORD) + WDTCTL::WDTHOLD::SET);
    }

    fn tickle(&self) {
        self.registers
            .ctl
            .modify(WDTCTL::WDTPW.val(PASSWORD) + WDTCTL::WDTCNTCL::SET);
    }
}
