// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL.
//
// Author: Ioan-Cristian CÎRSTEA <ioan.cirstea@oxidos.io>

//! This module contains all chip-specific code.
//!
//! Some models in the STM32F4 family may have additional features, while others not. Or they can
//! operate internally in different ways for the same feature. This module provides all the
//! chip-specific types and traits to be used by others modules in this crate or by other crates.

pub mod chip_specs;
pub mod clock_constants;
pub mod flash;

pub use chip_specs::ChipSpecs;
