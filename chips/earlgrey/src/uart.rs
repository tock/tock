use kernel::common::StaticRef;
use lowrisc::uart::{Uart, UartRegisters};

use crate::chip;
use crate::chip_config::CONFIG;

pub const UART0_BAUDRATE: u32 = CONFIG.uart_baudrate;

pub static mut UART0: Uart = Uart::new(UART0_BASE, chip::CHIP_FREQ);

const UART0_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x4000_0000 as *const UartRegisters) };
