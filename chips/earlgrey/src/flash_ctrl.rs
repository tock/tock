// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use crate::registers::top_earlgrey::FLASH_CTRL_CORE_BASE_ADDR;
use kernel::utilities::StaticRef;
use lowrisc::flash_ctrl::FlashCtrlRegisters;

pub const FLASH_CTRL_BASE: StaticRef<FlashCtrlRegisters> =
    unsafe { StaticRef::new(FLASH_CTRL_CORE_BASE_ADDR as *const FlashCtrlRegisters) };
