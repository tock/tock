// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL.
//
// Author: Ioan-Cristian CÎRSTEA <ioan.cirstea@oxidos.io>

//! HSI (high-speed internal) clock driver for the STM32F4xx family. [^doc_ref]
//!
//! # Usage
//!
//! For the purposes of brevity, any error checking has been removed. In real applications, always
//! check the return values of the [Hsi] methods.
//!
//! First, get a reference to the [Hsi] struct:
//! ```rust,ignore
//! let hsi = &peripherals.stm32f4.clocks.hsi;
//! ```
//!
//! ## Start the clock
//!
//! ```rust,ignore
//! hsi.enable();
//! ```
//!
//! ## Stop the clock
//!
//! ```rust,ignore
//! hsi.disable();
//! ```
//!
//! ## Check if the clock is enabled
//! ```rust,ignore
//! if hsi.is_enabled() {
//!     /* Do something */
//! } else {
//!     /* Do something */
//! }
//! ```
//!
//! ## Get the frequency of the clock
//! ```rust,ignore
//! let hsi_frequency_mhz = hsi.get_frequency().unwrap();
//! ```
//!
//! [^doc_ref]: See 6.2.2 in the documentation.

use crate::rcc::Rcc;

use kernel::debug;
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

/// Tests for the HSI clock
///
/// This module ensures that the HSI clock works as expected. If changes are brought to the HSI
/// clock, ensure to run all the tests to see if anything is broken.
///
/// # Usage
///
/// First, import the [crate::clocks::hsi] module in the desired board main file:
///
/// ```rust,ignore
/// use stm32f429zi::clocks::hsi;
/// ```
///
/// Then, to run the tests, put the following line before [kernel::process::load_processes]:
///
/// ```rust,ignore
/// hsi::tests::run(&peripherals.stm32f4.clocks.hsi);
/// ```
///
/// If everything works as expected, the following message should be printed on the kernel console:
///
/// ```text
/// ===============================================
/// Testing HSI...
/// Finished testing HSI. Everything is alright!
/// ===============================================
/// ```
///
/// **NOTE:** All these tests assume default boot configuration.
pub mod tests {
    use super::*;

    /// Run the entire test suite.
    pub fn run(hsi: &Hsi) {
        debug!("");
        debug!("===============================================");
        debug!("Testing HSI...");

        // By default, the HSI clock is enabled
        assert!(hsi.is_enabled());

        // HSI frequency is 16MHz
        assert_eq!(Some(HSI_FREQUENCY_MHZ), hsi.get_frequency_mhz());

        // Nothing should happen if the HSI clock is being enabled when already running
        assert_eq!(Ok(()), hsi.enable());

        // Impossible to disable the HSI clock since it is the system clock source
        assert_eq!(Err(ErrorCode::FAIL), hsi.disable());

        debug!("Finished testing HSI. Everything is alright!");
        debug!("===============================================");
        debug!("");
    }
}
