//! UART instantiation.

use kernel::common::StaticRef;
use sifive::uart::UartRegisters;

pub const UART0_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x1001_3000 as *const UartRegisters) };
