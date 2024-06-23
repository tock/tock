// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! HSE (high-speed external) clock driver for the STM32F4xx family. [^doc_ref]
//!
//! # Usage
//!
//! For the purposes of brevity, any error checking has been removed. In real applications, always
//! check the return values of the [Hse] methods.
//!
//! First, get a reference to the [Hse] struct:
//! ```rust,ignore
//! let hse = &base_peripherals.clocks.hse;
//! ```
//!
//! ## Start the clock
//!
//! ```rust,ignore
//! hse.enable(stm32f429zi::rcc::HseMode::BYPASS);
//! ```
//!
//! ## Set the clock frequency
//! ```rust,ignore
//! hse.set_frequency_mhz(8);
//! ```
//!
//! ## Stop the clock
//!
//! ```rust,ignore
//! hse.disable();
//! ```
//!
//! ## Check if the clock is enabled
//! ```rust,ignore
//! if hse.is_enabled() {
//!     /* Do something */
//! } else {
//!     /* Do something */
//! }
//! ```
//!
//! ## Get the frequency of the clock
//! ```rust,ignore
//! let hse_frequency_mhz = hse.get_frequency_mhz().unwrap();
//! ```
//!
//! [^doc_ref]: See 6.2.1 in the documentation.

use crate::rcc::HseMode;
use crate::rcc::Rcc;

use kernel::debug;
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

/// Tests for the HSE clock
///
/// This module ensures that the HSE clock works as expected. If changes are brought to the HSE
/// clock, ensure to run all the tests to see if anything is broken.
///
/// # Usage
///
/// First, import the [crate::clocks::hse] module in the desired board main file:
///
/// ```rust,ignore
/// use stm32f429zi::clocks::hse;
/// ```
///
/// Then, to run the tests, put the following line before [kernel::process::load_processes]:
///
/// ```rust,ignore
/// hse::tests::run(&peripherals.stm32f4.clocks.hse);
/// ```
///
/// If everything works as expected, the following message should be printed on the kernel console:
///
/// ```text
/// ===============================================
/// Testing HSE...
/// Finished testing HSE. Everything is alright!
/// ===============================================
/// ```
///
/// **NOTE:** All these tests assume default boot configuration.
pub mod tests {
    use super::*;

    /// Run the entire test suite.
    pub fn run(hse: &Hse) {
        debug!("");
        debug!("===============================================");
        debug!("Testing HSE...");

        // By default, the HSE clock is disabled
        assert!(!hse.is_enabled());

        // HSE frequency is None
        assert_eq!(None, hse.get_frequency_mhz());

        // HSE should be enabled
        assert_eq!(Ok(()), hse.enable(HseMode::BYPASS));

        // HSE frequency is 8MHz
        assert_eq!(Some(8), hse.get_frequency_mhz());

        // Nothing should happen if the HSE clock is being enabled when already running
        assert_eq!(Ok(()), hse.enable(HseMode::BYPASS));

        // It is possible to disable the HSE clock since it is not the system clock source
        assert_eq!(Ok(()), hse.disable());

        debug!("Finished testing HSE. Everything is alright!");
        debug!("===============================================");
        debug!("");
    }
}
