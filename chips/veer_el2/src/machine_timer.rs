// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2024 Antmicro <www.antmicro.com>

use kernel::hil::time::Freq32KHz;
use kernel::utilities::StaticRef;
use sifive::clint::ClintRegisters;

pub const CLINT_BASE: StaticRef<ClintRegisters> =
    unsafe { StaticRef::new(0x0200_0000 as *const ClintRegisters) };

pub type Clint<'a> = sifive::clint::Clint<'a, Freq32KHz>;
