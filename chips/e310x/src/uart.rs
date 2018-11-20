use core::cell::Cell;

use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::common::registers::{ReadWrite, ReadOnly};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ReturnCode;
use gpio;
use sifive::uart::Uart;

pub static mut UART0: Uart = Uart::new(UART0_BASE);

const UART0_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x1001_3000 as *const UartRegisters) };
