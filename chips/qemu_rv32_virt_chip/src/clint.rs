// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Machine Timer instantiation.

use kernel::utilities::StaticRef;
use sifive::clint::ClintRegisters;

pub const CLINT_BASE: StaticRef<ClintRegisters> =
    unsafe { StaticRef::new(0x0200_0000 as *const ClintRegisters) };
