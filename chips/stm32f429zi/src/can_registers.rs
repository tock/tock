// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! CAN

use kernel::utilities::StaticRef;
use stm32f4xx::can::Registers;

pub(crate) const CAN1_BASE: StaticRef<Registers> =
    unsafe { StaticRef::new(0x40006400 as *const Registers) };
