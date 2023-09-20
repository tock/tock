// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL.
//
// Author: Ioan-Cristian CÎRSTEA <ioan.cirstea@oxidos.io>

//! This module contains specification for different chips in the STM32F4 family

pub mod chip_specs;
pub mod stm32f401;
pub mod stm32f412;
pub mod stm32f429;
pub mod stm32f446;

pub use chip_specs::ChipSpecs;
