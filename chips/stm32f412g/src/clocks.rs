// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! STM32F412 Clocks

use crate::chip_specs::Stm32f412Specs;

/// STM32F412 Clocks
pub type Clocks<'a> = stm32f4xx::clocks::Clocks<'a, Stm32f412Specs>;
