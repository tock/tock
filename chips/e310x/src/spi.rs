//! SPI instantiation.
//!
//! This chip has three SPI master controllers:
//!
//! - QSPI0
//! - (Q)SPI1
//! - (Q)SPI2
//!
//! The naming of the latter two does not appear to be uniform across
//! manuals.
//!
//! Currently, only SPI2 is exposed as it is the only one with a
//! cs_width of 1 and no flash controller support, making it the best
//! initial target for development.

use kernel::common::StaticRef;
use sifive::spi::{Spi, SpiRegisters};

/// SPI 2 controller
///
/// - Address: `0x10034000`
/// - Flash controller: N
/// - cs_width: 1
/// - div_width: 12
/// - TX buffer depth: 8 (important to prevent race conditions)
///
/// Bus clock 18MHz (taken from uart.rs)
pub static mut QSPI2: Spi = Spi::new(QSPI2_BASE, 8, 18_000_000);

const QSPI2_BASE: StaticRef<SpiRegisters> =
    unsafe { StaticRef::new(0x1003_4000 as *const SpiRegisters) };
