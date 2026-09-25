// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Half duplex SPI over PIO, as the CYW43439 radio wants it.
//!
//! The transport lives in `rp2xxx::pio_gspi`, shared with the other RP2 chip.
//! This chip supplies its DMA channel and its GPIO pin type.

use crate::dma::DmaChannel;
use crate::gpio::RPGpioPin;

/// The shared gSPI transport, with this chip's DMA and pins filled in.
pub type PioGSpi<'a> = rp2xxx::pio_gspi::PioGSpi<'a, DmaChannel<'a>, RPGpioPin<'a>>;
