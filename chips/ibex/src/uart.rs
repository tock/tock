use kernel::common::StaticRef;
use lowrisc::uart::{Uart, UartRegisters};

pub static mut UART0: Uart = Uart::new(UART0_BASE, 50_000_000);

const UART0_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x4000_0000 as *const UartRegisters) };
