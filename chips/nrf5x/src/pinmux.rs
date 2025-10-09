// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! An abstraction over the pin multiplexer, nRF5X-family
//!
//! Controller drivers should use the `Pinmux` type (instead of a `u32`) for
//! fields that determine which pins are used by the hardware. The board
//! configuration should create `Pinmux`s and pass them into controller drivers
//! during initialization.

/// An opaque wrapper around a configurable pin.
#[derive(Copy, Clone)]
pub struct Pinmux(Pin);

impl Pinmux {
    /// Creates a new `Pinmux` wrapping the numbered pin.
    pub unsafe fn from_pin(pin: crate::gpio::Pin) -> Pinmux {
        Pinmux(pin)
    }
}

impl From<Pinmux> for crate::gpio::Pin {
    fn from(val: Pinmux) -> Self {
        val.0
    }
}

impl From<Pinmux> for u32 {
    fn from(val: Pinmux) -> Self {
        val.0 as _
    }
}
