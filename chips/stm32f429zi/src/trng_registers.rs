// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! True random number generator

use kernel::utilities::StaticRef;
use stm32f4xx::trng::RngRegisters;

pub(crate) const RNG_BASE: StaticRef<RngRegisters> =
    unsafe { StaticRef::new(0x5006_0800 as *const RngRegisters) };
