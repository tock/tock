// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! The clock module for STM32WLE5xx chips.
//!
//! This is highly similar to the one for STM32L4xx chips. This clock
//! implementation provides the minimal functionality required to enable
//! peripherals and configure speeds (as tested for I2C and UART). This
//! is still highly a work in progress and documentation comments here
//! describing the usage will be updated as development continues.

use crate::rcc::Rcc;

use kernel::ErrorCode;

/// MSI frequency in MHz
pub const MSI_FREQUENCY_MHZ: usize = 4;

/// Main MSI clock structure
pub struct Msi<'a> {
    rcc: &'a Rcc,
}

impl<'a> Msi<'a> {
    /// Create a new instance of the MSI clock.
    ///
    /// # Parameters
    ///
    /// + rcc: an instance of [crate::rcc]
    ///
    /// # Returns
    ///
    /// An instance of the MSI clock.
    pub(in crate::clocks) fn new(rcc: &'a Rcc) -> Self {
        Self { rcc }
    }

    /// Start the MSI clock.
    ///
    /// # Errors
    ///
    /// + [Err]\([ErrorCode::BUSY]\): if enabling the MSI clock took too long. Recall this method to
    /// ensure the MSI clock is running.
    pub fn enable(&self) -> Result<(), ErrorCode> {
        self.rcc.enable_msi_clock();

        for _ in 0..100 {
            if self.rcc.is_ready_msi_clock() {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    /// Stop the MSI clock.
    ///
    /// # Errors
    ///
    /// + [Err]\([ErrorCode::FAIL]\): if the MSI clock is configured as the system clock.
    /// + [Err]\([ErrorCode::BUSY]\): disabling the MSI clock took to long. Retry to ensure it is
    /// not running.
    pub fn disable(&self) -> Result<(), ErrorCode> {
        if self.rcc.is_msi_clock_system_clock() {
            return Err(ErrorCode::FAIL);
        }

        self.rcc.disable_msi_clock();

        for _ in 0..10 {
            if !self.rcc.is_ready_msi_clock() {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    /// Check whether the MSI clock is enabled or not.
    ///
    /// # Returns
    ///
    /// + [false]: the MSI clock is not enabled
    /// + [true]: the MSI clock is enabled
    pub fn is_enabled(&self) -> bool {
        self.rcc.is_enabled_msi_clock()
    }

    /// Get the frequency in MHz of the MSI clock.
    ///
    /// # Returns
    ///
    /// + [Some]\(frequency_mhz\): if the MSI clock is enabled.
    /// + [None]: if the MSI clock is disabled.
    pub fn get_frequency_mhz(&self) -> Option<usize> {
        if self.is_enabled() {
            Some(MSI_FREQUENCY_MHZ)
        } else {
            None
        }
    }
}
