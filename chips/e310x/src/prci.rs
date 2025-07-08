// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Power Reset Clock Interrupt controller instantiation.

use kernel::utilities::StaticRef;
use sifive::prci::PrciRegisters;

pub const PRCI_BASE: StaticRef<PrciRegisters> =
    unsafe { StaticRef::new(0x1000_8000 as *const PrciRegisters) };
