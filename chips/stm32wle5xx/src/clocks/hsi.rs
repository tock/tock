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

use crate::rcc::Rcc;

use kernel::ErrorCode;

/// HSI frequency in MHz
pub const HSI_FREQUENCY_MHZ: usize = 16;

/// Main HSI clock structure
pub struct Hsi<'a> {
    rcc: &'a Rcc,
}

impl<'a> Hsi<'a> {
    /// Create a new instance of the HSI clock.
    ///
    /// # Parameters
    ///
    /// + rcc: an instance of [crate::rcc]
    ///
    /// # Returns
    ///
    /// An instance of the HSI clock.
    pub(in crate::clocks) fn new(rcc: &'a Rcc) -> Self {
        Self { rcc }
    }

    /// Start the HSI clock.
    ///
    /// # Errors
    ///
    /// + [Err]\([ErrorCode::BUSY]\): if enabling the HSI clock took too long. Recall this method to
    /// ensure the HSI clock is running.
    pub fn enable(&self) -> Result<(), ErrorCode> {
        self.rcc.enable_hsi_clock();

        for _ in 0..100 {
            if self.rcc.is_ready_hsi_clock() {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    /// Stop the HSI clock.
    ///
    /// # Errors
    ///
    /// + [Err]\([ErrorCode::FAIL]\): if the HSI clock is configured as the system clock.
    /// + [Err]\([ErrorCode::BUSY]\): disabling the HSI clock took to long. Retry to ensure it is
    /// not running.
    pub fn disable(&self) -> Result<(), ErrorCode> {
        if self.rcc.is_hsi_clock_system_clock() {
            return Err(ErrorCode::FAIL);
        }

        self.rcc.disable_hsi_clock();

        for _ in 0..10 {
            if !self.rcc.is_ready_hsi_clock() {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    /// Check whether the HSI clock is enabled or not.
    ///
    /// # Returns
    ///
    /// + [false]: the HSI clock is not enabled
    /// + [true]: the HSI clock is enabled
    pub fn is_enabled(&self) -> bool {
        self.rcc.is_enabled_hsi_clock()
    }

    /// Get the frequency in MHz of the HSI clock.
    ///
    /// # Returns
    ///
    /// + [Some]\(frequency_mhz\): if the HSI clock is enabled.
    /// + [None]: if the HSI clock is disabled.
    pub fn get_frequency_mhz(&self) -> Option<usize> {
        if self.is_enabled() {
            Some(HSI_FREQUENCY_MHZ)
        } else {
            None
        }
    }
}
