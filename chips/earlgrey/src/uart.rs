use kernel::common::StaticRef;
pub use lowrisc::uart::Uart;
use lowrisc::uart::UartRegisters;

use crate::chip_config::CONFIG;

pub const UART0_BAUDRATE: u32 = CONFIG.uart_baudrate;

pub const UART0_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x4000_0000 as *const UartRegisters) };
