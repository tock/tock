// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Watchdog registers.

use kernel::utilities::StaticRef;

use sifive::watchdog::WatchdogRegisters;

pub const WATCHDOG_BASE: StaticRef<WatchdogRegisters> =
    unsafe { StaticRef::new(0x1000_0000 as *const WatchdogRegisters) };
