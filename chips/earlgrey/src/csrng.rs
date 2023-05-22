// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::utilities::StaticRef;
use lowrisc::csrng::CsRngRegisters;

pub const CSRNG_BASE: StaticRef<CsRngRegisters> =
    unsafe { StaticRef::new(0x4115_0000 as *const CsRngRegisters) };
