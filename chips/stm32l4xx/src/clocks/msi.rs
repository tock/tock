// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Author: Kamil Duljas <kamil.duljas@gmail.com>

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

/// Tests for the MSI clock
///
/// This module ensures that the MSI clock works as expected. If changes are brought to the MSI
/// clock, ensure to run all the tests to see if anything is broken.
///
/// # Usage
///
/// First, import the [crate::clocks::msi] module in the desired board main file:
///
/// ```rust,ignore
/// use stm32l476rg::clocks::msi;
/// ```
///
/// Then, to run the tests, put the following line before [kernel::process::load_processes]:
///
/// ```rust,ignore
/// msi::tests::run(&peripherals.stm32l4.clocks.msi);
/// ```
///
/// If everything works as expected, the following message should be printed on the kernel console:
///
/// ```text
/// ===============================================
/// Testing MSI...
/// Finished testing MSI. Everything is alright!
/// ===============================================
/// ```
///
/// **NOTE:** All these tests assume default boot configuration.
pub mod tests {
    use super::{ErrorCode, Msi, MSI_FREQUENCY_MHZ};
    use kernel::debug;

    /// Run the entire test suite.
    pub fn run(msi: &Msi) {
        debug!("");
        debug!("===============================================");
        debug!("Testing MSI...");

        // By default, the MSI clock is enabled
        assert!(msi.is_enabled());

        // MSI frequency is 4MHz
        assert_eq!(Some(MSI_FREQUENCY_MHZ), msi.get_frequency_mhz());

        // MSI is used as system clock, so disable should fail
        assert_eq!(Err(ErrorCode::FAIL), msi.disable());

        debug!("Finished testing MSI. Everything is alright!");
        debug!("===============================================");
        debug!("");
    }
}
