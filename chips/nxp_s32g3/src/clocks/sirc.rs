// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! SIRC (Slow Internal RC Oscillator) driver for NXP S32G3.
//!
//! The SIRC provides a 32 kHz clock that is always enabled and cannot be
//! turned off. It is used as clock source for:
//!
//! - Real-time clock (RTC)
//! - POR_WDOG (power-on-reset watchdog)
//!
//! See RM §24.2.5.

use kernel::platform::chip::ClockInterface;

/// SIRC frequency in Hz (32 kHz).
pub const SIRC_FREQUENCY_HZ: u32 = 32_000;

/// Slow Internal RC Oscillator.
///
/// The SIRC is always-on and cannot be disabled. Its frequency is fixed
/// at 32 kHz.
pub struct Sirc {
    _private: (),
}

impl Sirc {
    /// Create a new SIRC instance.
    pub const fn new() -> Self {
        Self { _private: () }
    }

    /// Get the SIRC frequency in Hz.
    pub fn get_frequency_hz(&self) -> u32 {
        SIRC_FREQUENCY_HZ
    }
}

impl ClockInterface for Sirc {
    fn is_enabled(&self) -> bool {
        // SIRC is always enabled (RM §24.2.5: "Always enabled")
        true
    }

    fn enable(&self) {
        // No-op: SIRC is always on
    }

    fn disable(&self) {
        // No-op: SIRC cannot be disabled
    }
}
