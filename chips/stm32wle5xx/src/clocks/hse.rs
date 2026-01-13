// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! The clock module for STM32WLE5xx chips.
//!
//! The clock module for STM32WLE5xx chips is highly similar to the
//! one for STM32L4xx chips. This clock implementation provides the
//! minimal functionality required to enable peripherals and configure
//! speeds (as tested for I2C and UART). This is still highly a work
//! in progress and documentation comments here describing the usage
//! will be updated as development continues.

use crate::rcc::HseMode;
use crate::rcc::Rcc;

use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

/// Main HSE clock structure
pub struct Hse<'a> {
    rcc: &'a Rcc,
    hse_frequency_mhz: OptionalCell<usize>,
}

impl<'a> Hse<'a> {
    /// Create a new instance of the HSE clock.
    ///
    /// # Parameters
    ///
    /// + rcc: an instance of [crate::rcc]
    ///
    /// # Returns
    ///
    /// An instance of the HSE clock.
    pub(in crate::clocks) fn new(rcc: &'a Rcc) -> Self {
        Self {
            rcc,
            hse_frequency_mhz: OptionalCell::empty(),
        }
    }

    /// Start the HSE clock.
    ///
    /// # Errors
    ///
    /// + [Err]\([ErrorCode::BUSY]\): disabling the HSE clock took to long. Retry to ensure it is running
    pub fn enable(&self, source: HseMode) -> Result<(), ErrorCode> {
        if source == HseMode::BYPASS {
            self.rcc.enable_hse_clock_bypass();
        }

        self.rcc.enable_hse_clock();

        for _ in 0..100 {
            if self.rcc.is_ready_hse_clock() {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    /// Stop the HSE clock.
    ///
    /// # Errors
    ///
    /// + [Err]\([ErrorCode::FAIL]\): if the HSE clock is configured as the system clock.
    /// + [Err]\([ErrorCode::BUSY]\): disabling the HSE clock took to long. Retry to ensure it is
    /// not running.
    pub fn disable(&self) -> Result<(), ErrorCode> {
        if self.rcc.is_hse_clock_system_clock() {
            return Err(ErrorCode::FAIL);
        }

        self.rcc.disable_hse_clock();

        for _ in 0..10 {
            if !self.rcc.is_ready_hse_clock() {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    /// Check whether the HSE clock is enabled or not.
    ///
    /// # Returns
    ///
    /// + [false]: the HSE clock is not enabled
    /// + [true]: the HSE clock is enabled
    pub fn is_enabled(&self) -> bool {
        self.rcc.is_enabled_hse_clock()
    }

    /// Get the frequency in MHz of the HSE clock.
    ///
    /// # Returns
    ///
    /// + [Some]\(frequency_mhz\): if the HSE clock is enabled.
    /// + [None]: if the HSE clock is disabled.
    pub fn get_frequency_mhz(&self) -> Option<usize> {
        if self.is_enabled() {
            self.hse_frequency_mhz.get()
        } else {
            None
        }
    }

    /// Set the frequency in MHz of the HSE clock.
    ///
    /// # Parameters
    ///
    /// + frequency: HSE frequency in MHz
    pub fn set_frequency_mhz(&self, frequency: usize) {
        self.hse_frequency_mhz.set(frequency);
    }
}
