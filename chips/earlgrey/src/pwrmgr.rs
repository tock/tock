// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use crate::registers::top_earlgrey::PWRMGR_AON_BASE_ADDR;
use kernel::utilities::StaticRef;
use lowrisc::pwrmgr::PwrMgrRegisters;

pub(crate) const PWRMGR_BASE: StaticRef<PwrMgrRegisters> =
    unsafe { StaticRef::new(PWRMGR_AON_BASE_ADDR as *const PwrMgrRegisters) };
