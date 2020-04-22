use kernel::common::StaticRef;
use sifive::uart::{Uart, UartRegisters};

pub static mut UART0: Uart = Uart::new(UART0_BASE, 32_000_000);

const UART0_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x2000_0000 as *const UartRegisters) };
