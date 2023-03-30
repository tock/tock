// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::utilities::StaticRef;
use lowrisc::otbn::OtbnRegisters;

pub const OTBN_BASE: StaticRef<OtbnRegisters> =
    unsafe { StaticRef::new(0x4113_0000 as *const OtbnRegisters) };
