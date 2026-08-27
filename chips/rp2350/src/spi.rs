// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! SPI0 and SPI1 on the RP2350.
//!
//! The driver itself lives in the `rp2xxx` crate, shared with the other RP2
//! chip: both fit the same Arm PL022 PrimeCell with the same register layout.
//! What is specific to this chip is here -- the base addresses, and, in
//! `clocks.rs`, the `PeripheralClock` impl the driver uses to read `clk_peri`.
//!
//! Ref: 12.3 "SPI" in the RP2350 datasheet.

use crate::clocks::Clocks;
use crate::gpio::RPGpioPin;
use kernel::utilities::StaticRef;
use rp2xxx::spi::SpiRegisters;

const SPI0_BASE: StaticRef<SpiRegisters> =
    unsafe { StaticRef::new(0x40080000 as *const SpiRegisters) };

const SPI1_BASE: StaticRef<SpiRegisters> =
    unsafe { StaticRef::new(0x40088000 as *const SpiRegisters) };

/// The shared PL022 driver, with this chip's clocks and GPIO pins filled in.
pub type Spi<'a> = rp2xxx::spi::Spi<'a, Clocks, RPGpioPin<'a>>;

/// Create a driver for SPI0.
pub fn new_spi0(clocks: &Clocks) -> Spi<'_> {
    Spi::new(SPI0_BASE, clocks)
}

/// Create a driver for SPI1.
pub fn new_spi1(clocks: &Clocks) -> Spi<'_> {
    Spi::new(SPI1_BASE, clocks)
}
