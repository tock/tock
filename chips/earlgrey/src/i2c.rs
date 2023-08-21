// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use crate::registers::top_earlgrey::TOP_EARLGREY_I2C0_BASE_ADDR;
use kernel::utilities::StaticRef;
use lowrisc::i2c::I2cRegisters;

pub const I2C0_BASE: StaticRef<I2cRegisters> =
    unsafe { StaticRef::new(TOP_EARLGREY_I2C0_BASE_ADDR as *const I2cRegisters) };
