// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Peripherals shared by the RP2040 and the RP2350.
//!
//! Raspberry Pi's two microcontrollers carry several peripherals one driver
//! can serve. Some are Arm PrimeCells the two chips take verbatim, such as the
//! PL022 in `spi`. Others are Raspberry Pi's own and differ only in where
//! things sit rather than in how they are laid out, such as `pio`, whose
//! interrupt registers begin at a different offset on each chip. Both belong
//! here, and the `rp2040` and `rp2350` crates wrap them with what genuinely
//! differs: base addresses, clocks, and GPIO pin types.
//!
//! The test is whether the register layout and the behaviour are the same, not
//! whether the names match. `clocks`, `gpio` and `uart` each diverge by
//! hundreds of lines between the two chips and stay in their own crates.
//!
//! Not everything here is a driver. `dma` and `pads` hold no registers at all,
//! only the traits a shared driver needs of a chip. DMA is the case that makes
//! them necessary: its registers differ too much for one driver, so each chip
//! keeps its own, and `dma` describes what `pio_gspi` needs of either. A
//! peripheral can fail the test above and still have an interface worth
//! stating once.

#![no_std]

pub mod dma;
pub mod pads;
pub mod pio;
pub mod pio_gspi;
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
