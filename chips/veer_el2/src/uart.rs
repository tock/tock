// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2024 Antmicro <www.antmicro.com>

use kernel::utilities::StaticRef;
use lowrisc::registers::uart_regs::UartRegisters;

pub const UART0_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x90002000 as *const UartRegisters) };

pub type UartType<'a> = lowrisc::uart::Uart<'a>;
pub type SimUartType<'a> = crate::io::SemihostUart;
