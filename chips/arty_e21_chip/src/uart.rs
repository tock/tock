// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::utilities::StaticRef;
use sifive::uart::UartRegisters;

pub const UART0_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x2000_0000 as *const UartRegisters) };
