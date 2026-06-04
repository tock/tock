// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! FIRC (Fast Internal RC Oscillator) driver for NXP S32G3.
//!
//! The FIRC provides a 48 MHz clock that is always enabled and cannot be
//! turned off. It serves as:
//!
//! - Default system clock after reset
//! - PLL reference when FXOSC is unavailable
//! - Fallback clock on loss-of-lock/loss-of-clock detection
//! - Clock source for PMC, MC_RGM, FCCU, FOSU, SWT, SIUL2
//!
//! See RM §24.2.4.

use kernel::platform::chip::ClockInterface;

/// FIRC frequency in Hz (48 MHz).
pub const FIRC_FREQUENCY_HZ: u32 = 48_000_000;

/// FIRC frequency in MHz.
pub const FIRC_FREQUENCY_MHZ: usize = 48;

/// Fast Internal RC Oscillator.
///
/// The FIRC is always-on and cannot be disabled. Its frequency is fixed
/// at 48 MHz.
pub struct Firc {
    _private: (),
}

impl Firc {
    /// Create a new FIRC instance.
    pub const fn new() -> Self {
        Self { _private: () }
    }

    /// Get the FIRC frequency in Hz.
    ///
    /// Always returns `Some(48_000_000)` since FIRC cannot be disabled.
    pub fn get_frequency_hz(&self) -> u32 {
        FIRC_FREQUENCY_HZ
    }

    /// Get the FIRC frequency in MHz.
    pub fn get_frequency_mhz(&self) -> usize {
        FIRC_FREQUENCY_MHZ
    }
}

impl ClockInterface for Firc {
    fn is_enabled(&self) -> bool {
        // FIRC is always enabled (RM §24.2.4: "Always enabled")
        true
    }

    fn enable(&self) {
        // No-op: FIRC is always on
    }

    fn disable(&self) {
        // No-op: FIRC cannot be disabled
    }
}
