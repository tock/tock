// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Instantiation of the sifive Platform Level Interrupt Controller

use kernel::utilities::StaticRef;
pub use sifive::plic::Plic;
use sifive::plic::PlicRegisters;

pub const PLIC_BASE: StaticRef<PlicRegisters> =
    unsafe { StaticRef::new(0x0c00_0000 as *const PlicRegisters) };
