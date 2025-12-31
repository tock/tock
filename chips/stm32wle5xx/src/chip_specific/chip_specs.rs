// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Trait that encompasses chip specifications
//!
//! The main use of this trait is to be passed as a bound for the type parameter for chip
//! peripherals in crates such as `stm32wle5xx`.

use crate::chip_specific::clock_constants::ClockConstants;

pub trait ChipSpecs: ClockConstants {}

impl<T: ClockConstants> ChipSpecs for T {}
