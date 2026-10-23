// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Pad configuration shared by the RP2040 and the RP2350.
//!
//! The two chips' GPIO blocks diverge and stay in their own crates, but the
//! pad control register does not: `GPIO_PAD` places `DRIVE` at bits 5:4,
//! `SCHMITT` at bit 1 and `SLEWFAST` at bit 0 on both, and encodes drive
//! strength with the same four values. These two enums describe those fields,
//! so a driver that has to tune a pad can be written once.

/// Slew rate of an output
#[derive(Debug, Eq, PartialEq)]
pub enum SlewRate {
    /// Slow slew rate.
    Slow = 0,
    /// Fast slew rate.
    Fast = 1,
}

/// Drive Strength of a GPIO Pin
#[derive(Debug, Eq, PartialEq)]
pub enum DriveStrength {
    Drive2mA = 0,
    Drive4ma = 1,
    Drive8ma = 2,
    Drive12ma = 3,
}
