//! Watchdog registers.

use kernel::utilities::StaticRef;

use sifive::watchdog::WatchdogRegisters;

pub const WATCHDOG_BASE: StaticRef<WatchdogRegisters> =
    unsafe { StaticRef::new(0x1000_0000 as *const WatchdogRegisters) };
