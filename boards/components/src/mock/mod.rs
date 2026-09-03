// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Components for mock (fake) hardware devices.
//!
//! These wire up the capsules in `capsules_extra::mock`, which pretend to be
//! real peripherals so a board can be exercised without the actual hardware.

pub mod i2c_bus;
pub mod sht4x;
