// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::utilities::StaticRef;
use lowrisc::registers::uart_regs::UartRegisters;
pub use lowrisc::uart::Uart;

use crate::chip_config::CONFIG;
use crate::registers::top_earlgrey::TOP_EARLGREY_UART0_BASE_ADDR;

pub const UART0_BAUDRATE: u32 = CONFIG.uart_baudrate;

pub const UART0_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(TOP_EARLGREY_UART0_BASE_ADDR as *const UartRegisters) };
