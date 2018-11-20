//! Watchdog

use kernel::common::StaticRef;

use sifive::watchdog::{Watchdog, WatchdogRegisters};

pub static mut WATCHDOG: Watchdog = Watchdog::new(WATCHDOG_BASE);

const WATCHDOG_BASE: StaticRef<WatchdogRegisters> =
    unsafe { StaticRef::new(0x1000_0000 as *const WatchdogRegisters) };
