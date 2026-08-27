// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Peripherals shared by the RP2040 and the RP2350.
//!
//! Raspberry Pi's two microcontrollers share several Arm PrimeCell peripherals
//! verbatim. Drivers for those live here, and the `rp2040` and `rp2350` crates
//! wrap them with the parts that genuinely differ between the chips -- base
//! addresses, clocks, and GPIO pin types.
//!
//! Peripherals that only look alike are *not* here. `clocks`, `gpio` and `uart`
//! each diverge by hundreds of lines between the two chips and stay in their
//! own crates.

#![no_std]

pub mod spi;

/// Access to the peripheral clock, `clk_peri`.
///
/// Both chips expose it as a `Clock::Peripheral` variant, but the two `Clock`
/// enums diverge either side of that variant -- the RP2350 inserts `Hstx` --
/// so the two `Clocks` types are unrelated. Shared drivers that need the
/// peripheral clock frequency take this instead of a concrete `Clocks`.
pub trait PeripheralClock {
    /// The frequency of `clk_peri`, in Hz.
    fn peripheral_frequency(&self) -> u32;
}
