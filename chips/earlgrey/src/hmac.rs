// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use crate::registers::top_earlgrey::TOP_EARLGREY_HMAC_BASE_ADDR;
use kernel::utilities::StaticRef;
use lowrisc::hmac::HmacRegisters;

pub const HMAC0_BASE: StaticRef<HmacRegisters> =
    unsafe { StaticRef::new(TOP_EARLGREY_HMAC_BASE_ADDR as *const HmacRegisters) };
