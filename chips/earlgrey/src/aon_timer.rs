// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use crate::registers::top_earlgrey::AON_TIMER_AON_BASE_ADDR;
use kernel::utilities::StaticRef;
use lowrisc::aon_timer::AonTimerRegisters;

pub const AON_TIMER_BASE: StaticRef<AonTimerRegisters> =
    unsafe { StaticRef::new(AON_TIMER_AON_BASE_ADDR as *const AonTimerRegisters) };
