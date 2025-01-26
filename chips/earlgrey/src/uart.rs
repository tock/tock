// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::utilities::StaticRef;
use lowrisc::registers::uart_regs::UartRegisters;
pub use lowrisc::uart::Uart;

use crate::registers::top_earlgrey::UART0_BASE_ADDR;

pub const UART0_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(UART0_BASE_ADDR as *const UartRegisters) };
