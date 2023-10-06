// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! STM32F401 Flash

use crate::chip_specs::Stm32f401Specs;

/// STM32F401 Flash
pub type Flash = stm32f4xx::flash::Flash<Stm32f401Specs>;
