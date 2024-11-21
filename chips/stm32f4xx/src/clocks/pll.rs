// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL.
//
// Author: Ioan-Cristian CÎRSTEA <ioan.cirstea@oxidos.io>

//! Main phase-locked loop (PLL) clock driver for the STM32F4xx family. [^doc_ref]
//!
//! Many boards of the STM32F4xx family provide several PLL clocks. However, all of them have a
//! main PLL clock. This driver is designed for the main PLL clock. It will be simply referred as
//! the PLL clock.
//!
//! The PLL clock is composed of two outputs:
//!
//! + the main one used for the system clock
//! + the PLL48CLK used for USB OTG FS, the random number generator and SDIO clocks
//!
//! # Implemented features
//!
//! - [x] Default configuration of 96MHz with reduced PLL jitter
//! - [x] 1MHz frequency precision
//! - [x] Support for 13-216MHz frequency range
//! - [x] Support for PLL48CLK output
//!
//! # Missing features
//!
//! - [ ] Precision higher than 1MHz
//! - [ ] Source selection
//! - [ ] Precise control over the PLL48CLK frequency
//!
//! # Usage
//!
//! For the purposes of brevity, any error checking has been removed. In real applications, always
//! check the return values of the [Pll] methods.
//!
//! First, get a reference to the [Pll] struct:
//! ```rust,ignore
//! let pll = &peripherals.stm32f4.clocks.pll;
//! ```
//!
//! ## Start the clock with a given frequency
//!
//! ```rust,ignore
//! pll.set_frequency_mhz(PllSource::HSI, HSI_FREQUENCY_MHZ, 100); // 100MHz
//! pll.enable();
//! ```
//!
//! ## Stop the clock
//!
//! ```rust,ignore
//! pll.disable();
//! ```
//!
//! ## Check whether the PLL clock is running or not
//! ```rust,ignore
//! if pll.is_enabled() {
//!     // do something...
//! } else {
//!     // do something...
//! }
//! ```
//!
//! ## Check the clock frequency
//!
//! ```rust,ignore
//! let optional_pll_frequency = pll.get_frequency_mhz();
//! if let None = optional_pll_frequency {
//!     /* Clock stopped */
//! }
//! let pll_frequency = optional_pll_frequency.unwrap();
//! /* Computations based on the PLL frequency */
//! ```
//!
//! ## Reconfigure the clock once started
//!
//! ```rust,ignore
//! pll.disable(); // The PLL clock can't be configured while running
//! pll.set_frequency_mhz(PllSource::HSI, HSI_FREQUENCY_MHZ, 50); // 50MHz
//! pll.enable();
//! ```
//!
//! ## Configure the PLL clock so that PLL48CLK output is correctly calibrated
//! ```rust,ignore
//! // The frequency of the PLL clock must be 1, 1.5, 2, 2.5, 3, 3.5 or 4 x 48MHz in order to get
//! // 48MHz output. Otherwise, the driver will attempt to get the closest frequency lower than 48MHz
//! pll.set_frequency_mhz(PllSource::HSI, HSI_FREQUENCY_MHZ, 72); // 72MHz = 48MHz * 1.5
//! pll.enable();
//! ```
//!
//! ## Check if the PLL48CLK output is calibrated.
//! ```rust,ignore
//! if !pll.is_pll48_calibrated() {
//!     /* Handle the case when it is not calibrated */
//! }
//! ```
//!
//! ## Get the frequency of the PLL48CLK output
//!
//! ```rust,ignore
//! let optional_pll48_frequency = pll.get_frequency_mhz();
//! if let None = optional_pll48_frequency {
//!     /* Clock stopped */
//! }
//! let pll48_frequency = optional_pll48_frequency.unwrap();
//! ```
//!
//! [^doc_ref]: See 6.2.3 in the documentation.

use crate::chip_specific::clock_constants;
use crate::clocks::hsi::HSI_FREQUENCY_MHZ;
use crate::rcc::Rcc;
use crate::rcc::SysClockSource;
use crate::rcc::{PllSource, PLLM, PLLP, PLLQ};
use crate::rcc::{DEFAULT_PLLM_VALUE, DEFAULT_PLLN_VALUE, DEFAULT_PLLP_VALUE, DEFAULT_PLLQ_VALUE};

use kernel::debug;
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

use core::cell::Cell;
use core::marker::PhantomData;

/// Main PLL clock structure.
pub struct Pll<'a, PllConstants> {
    rcc: &'a Rcc,
    frequency_mhz: OptionalCell<usize>,
    pll48_frequency_mhz: OptionalCell<usize>,
    pll48_calibrated: Cell<bool>,
    _marker: PhantomData<PllConstants>,
}

impl<'a, PllConstants: clock_constants::PllConstants> Pll<'a, PllConstants> {
    // Create a new instance of the PLL clock.
    //
    // The instance of the PLL clock is configured to run at 96MHz and with minimal PLL jitter
    // effects.
    //
    // # Parameters
    //
    // + rcc: an instance of [crate::rcc]
    //
    // # Returns
    //
    // An instance of the PLL clock.
    pub(in crate::clocks) fn new(rcc: &'a Rcc) -> Self {
        const PLLP: usize = match DEFAULT_PLLP_VALUE {
            PLLP::DivideBy2 => 2,
            PLLP::DivideBy4 => 4,
            PLLP::DivideBy6 => 6,
            PLLP::DivideBy8 => 8,
        };
        const PLLM: usize = DEFAULT_PLLM_VALUE as usize;
        const PLLQ: usize = DEFAULT_PLLQ_VALUE as usize;
        Self {
            rcc,
            frequency_mhz: OptionalCell::new(HSI_FREQUENCY_MHZ / PLLM * DEFAULT_PLLN_VALUE / PLLP),
            pll48_frequency_mhz: OptionalCell::new(
                HSI_FREQUENCY_MHZ / PLLM * DEFAULT_PLLN_VALUE / PLLQ,
            ),
            pll48_calibrated: Cell::new(true),
            _marker: PhantomData,
        }
    }

    // The caller must ensure the desired frequency lies between MIN_FREQ_MHZ and
    // MAX_FREQ_MHZ.  Otherwise, the return value makes no sense.
    fn compute_pllp(desired_frequency_mhz: usize) -> PLLP {
        if desired_frequency_mhz < 55 {
            PLLP::DivideBy8
        } else if desired_frequency_mhz < 73 {
            PLLP::DivideBy6
        } else if desired_frequency_mhz < 109 {
            PLLP::DivideBy4
        } else {
            PLLP::DivideBy2
        }
    }

    // The caller must ensure the desired frequency lies between MIN_FREQ_MHZ and
    // MAX_FREQ_MHZ. Otherwise, the return value makes no sense.
    fn compute_plln(
        desired_frequency_mhz: usize,
        pll_source_clock_freq: usize,
        pllp: PLLP,
    ) -> usize {
        let vco_input_frequency: usize = pll_source_clock_freq / DEFAULT_PLLM_VALUE as usize;
        desired_frequency_mhz * Into::<usize>::into(pllp) / vco_input_frequency
    }

    // The caller must ensure the VCO output frequency lies between 100 and 432MHz. Otherwise, the
    // return value makes no sense.
    fn compute_pllq(vco_output_frequency_mhz: usize) -> PLLQ {
        for pllq in 3..10 {
            if 48 * pllq >= vco_output_frequency_mhz {
                return match pllq {
                    3 => PLLQ::DivideBy3,
                    4 => PLLQ::DivideBy4,
                    5 => PLLQ::DivideBy5,
                    6 => PLLQ::DivideBy6,
                    7 => PLLQ::DivideBy7,
                    8 => PLLQ::DivideBy8,
                    _ => PLLQ::DivideBy9,
                };
            }
        }
        unreachable!("The previous for loop should always return");
    }

    /// Set the PLL source clock
    fn set_pll_source_clock(&self, source: PllSource) -> Result<(), ErrorCode> {
        if self.is_enabled() {
            Err(ErrorCode::FAIL)
        } else {
            self.rcc.set_pll_clocks_source(source);
            Ok(())
        }
    }

    /// Start the PLL clock.
    ///
    /// # Errors
    ///
    /// + [Err]\([ErrorCode::BUSY]\): if enabling the PLL clock took too long. Recall this method to
    /// ensure the PLL clock is running.
    pub fn enable(&self) -> Result<(), ErrorCode> {
        // Enable the PLL clock
        self.rcc.enable_pll_clock();

        // Wait until the PLL clock is locked.
        // 200 was obtained by running tests in release mode
        for _ in 0..200 {
            if self.rcc.is_locked_pll_clock() {
                return Ok(());
            }
        }

        // If waiting for the PLL clock took too long, return ErrorCode::BUSY
        Err(ErrorCode::BUSY)
    }

    /// Stop the PLL clock.
    ///
    /// # Errors
    ///
    /// + [Err]\([ErrorCode::FAIL]\): if the PLL clock is configured as the system clock.
    /// + [Err]\([ErrorCode::BUSY]\): disabling the PLL clock took to long. Retry to ensure it is
    /// not running.
    pub fn disable(&self) -> Result<(), ErrorCode> {
        // Can't disable the PLL clock when it is used as the system clock
        if self.rcc.get_sys_clock_source() == SysClockSource::PLL {
            return Err(ErrorCode::FAIL);
        }

        // Disable the PLL clock
        self.rcc.disable_pll_clock();

        // Wait to unlock the PLL clock
        // 10 was obtained by testing in release mode
        for _ in 0..10 {
            if !self.rcc.is_locked_pll_clock() {
                return Ok(());
            }
        }

        // If the waiting was too long, return ErrorCode::BUSY
        Err(ErrorCode::BUSY)
    }

    /// Check whether the PLL clock is enabled or not.
    ///
    /// # Returns
    ///
    /// + [false]: the PLL clock is not enabled
    /// + [true]: the PLL clock is enabled
    pub fn is_enabled(&self) -> bool {
        self.rcc.is_enabled_pll_clock()
    }

    /// Set the frequency of the PLL clock.
    ///
    /// The PLL clock has two outputs:
    ///
    /// + main output used for configuring the system clock
    /// + a second output called PLL48CLK used by OTG USB FS (48MHz), the random number generator
    /// (≤ 48MHz) and the SDIO (≤ 48MHz) clocks.
    ///
    /// When calling this method, the given frequency is set for the main output. The method will
    /// attempt to configure the PLL48CLK output to 48MHz, or to the highest value less than 48MHz
    /// if it is not possible to get a precise 48MHz. In order to obtain a precise 48MHz frequency
    /// (for the OTG USB FS peripheral), one should call this method with a frequency of 1, 1.5, 2,
    /// 2.5 ... 4 x 48MHz.
    ///
    /// # Parameters
    ///
    /// + pll_source: PLL source clock (HSI or HSE)
    ///
    /// + source_frequency: the frequency of the PLL source clock in MHz. For the HSI the frequency
    /// is fixed to 16MHz. For the HSE, the frequency is hardware-dependent
    ///
    /// + desired_frequency_mhz: the desired frequency in MHz. Supported values: 24-216MHz for
    /// STM32F401 and 13-216MHz for all the other chips
    ///
    /// # Errors
    ///
    /// + [Err]\([ErrorCode::INVAL]\): if the desired frequency can't be achieved
    /// + [Err]\([ErrorCode::FAIL]\): if the PLL clock is already enabled. It must be disabled before
    /// configuring it.
    pub(super) fn set_frequency_mhz(
        &self,
        pll_source: PllSource,
        source_frequency: usize,
        desired_frequency_mhz: usize,
    ) -> Result<(), ErrorCode> {
        // Check for errors:
        // + PLL clock running
        // + invalid frequency
        if self.rcc.is_enabled_pll_clock() {
            return Err(ErrorCode::FAIL);
        } else if desired_frequency_mhz < PllConstants::MIN_FREQ_MHZ
            || desired_frequency_mhz > PllConstants::MAX_FREQ_MHZ
        {
            return Err(ErrorCode::INVAL);
        }

        // The output frequencies for the PLL clock is computed as following:
        // Source frequency / PLLM = VCO input frequency (must range from 1MHz to 2MHz)
        // VCO output frequency = VCO input frequency * PLLN (must range from 100MHz to 432MHz)
        // PLL output frequency = VCO output frequency / PLLP
        // PLL48CLK = VCO output frequency / PLLQ

        // Set PLL source (HSI or HSE)
        if self.set_pll_source_clock(pll_source) != Ok(()) {
            return Err(ErrorCode::FAIL);
        }

        // Compute PLLP
        let pllp = Self::compute_pllp(desired_frequency_mhz);
        self.rcc.set_pll_clock_p_divider(pllp);

        // Compute PLLN
        let plln = Self::compute_plln(desired_frequency_mhz, source_frequency, pllp);
        self.rcc.set_pll_clock_n_multiplier(plln);

        // Compute PLLQ
        let vco_output_frequency = source_frequency / DEFAULT_PLLM_VALUE as usize * plln;
        let pllq = Self::compute_pllq(vco_output_frequency);
        self.rcc.set_pll_clock_q_divider(pllq);

        // Check if PLL48CLK is calibrated, e.g. its frequency is exactly 48MHz
        let pll48_frequency = vco_output_frequency / pllq as usize;
        self.pll48_calibrated
            .set(pll48_frequency == 48 && vco_output_frequency % pllq as usize == 0);

        // Cache the frequency so it is not computed every time a get method is called
        self.frequency_mhz.set(desired_frequency_mhz);
        self.pll48_frequency_mhz.set(pll48_frequency);

        Ok(())
    }

    /// Get the frequency in MHz of the PLL clock.
    ///
    /// # Returns
    ///
    /// + [Some]\(frequency_mhz\): if the PLL clock is enabled.
    /// + [None]: if the PLL clock is disabled.
    pub fn get_frequency_mhz(&self) -> Option<usize> {
        if self.is_enabled() {
            self.frequency_mhz.get()
        } else {
            None
        }
    }

    /// Get the frequency in MHz of the PLL clock from RCC registers instead of using the cached
    /// value.
    ///
    /// # Returns
    ///
    /// + [Some]\(frequency_mhz\): if the PLL clock is enabled.
    /// + [None]: if the PLL clock is disabled.
    pub fn get_frequency_mhz_no_cache(&self, source_frequency: usize) -> Option<usize> {
        if self.is_enabled() {
            let pllm = self.rcc.get_pll_clocks_m_divider() as usize;
            let plln = self.rcc.get_pll_clock_n_multiplier();
            let pllp: usize = self.rcc.get_pll_clock_p_divider().into();
            Some(source_frequency / pllm * plln / pllp)
        } else {
            None
        }
    }

    /// Get the frequency in MHz of the PLL48 clock.
    ///
    /// **NOTE:** If the PLL clock was not configured with a frequency multiple of 48MHz, the
    /// returned value is inaccurate.
    ///
    /// # Returns
    ///
    /// + [Some]\(frequency_mhz\): if the PLL clock is enabled.
    /// + [None]: if the PLL clock is disabled.
    pub fn get_frequency_mhz_pll48(&self) -> Option<usize> {
        if self.is_enabled() {
            self.pll48_frequency_mhz.get()
        } else {
            None
        }
    }

    /// Check if the PLL48 clock is calibrated (its output is exactly 48MHz).
    ///
    /// A frequency of 48MHz is required for USB OTG FS.
    ///
    /// # Returns
    ///
    /// + [true]: the PLL48 clock frequency is exactly 48MHz.
    /// + [false]: the PLL48 clock is not exactly 48MHz.
    pub fn is_pll48_calibrated(&self) -> bool {
        self.pll48_calibrated.get()
    }
}

/// Tests for the PLL clock
///
/// This module ensures that the PLL clock works as expected. If changes are brought to the PLL
/// clock, ensure to run all the tests to see if anything is broken.
///
/// # Usage
///
/// First, import the [crate::clocks::pll] module inside the board main file:
///
/// ```rust,ignore
/// use stm32f429zi::pll;
/// ```
/// To run all the available tests, add this line before **kernel::process::load_processes()**:
///
/// ```rust,ignore
/// pll::tests::run(&peripherals.stm32f4.clocks.pll);
/// ```
///
/// If everything works as expected, the following message should be printed on the kernel console:
///
/// ```text
/// ===============================================
/// Testing PLL...
/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/// Testing PLL configuration...
/// Finished testing PLL configuration.
/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/// Testing PLL struct...
/// Finished testing PLL struct.
/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/// Finished testing PLL. Everything is alright!
/// ===============================================
/// ```
///
/// There is also the possibility to run a part of the test suite. Check the functions present in
/// this module for more details.
///
/// # Errors
///
/// If there are any errors, open an issue ticket at <https://github.com/tock/tock>. Please provide the
/// output of the test execution.
pub mod tests {
    use super::{
        clock_constants, debug, ErrorCode, Pll, PllSource, DEFAULT_PLLM_VALUE, HSI_FREQUENCY_MHZ,
        PLLM, PLLP, PLLQ,
    };

    // Depending on the default PLLM value, the computed PLLN value changes.
    const MULTIPLIER: usize = match DEFAULT_PLLM_VALUE {
        PLLM::DivideBy8 => 1,
        PLLM::DivideBy16 => 2,
    };

    /// Test if the configuration parameters are correctly computed for a given frequency.
    ///
    /// # Usage
    ///
    /// ```rust,ignore
    /// use stm32f429zi::pll; // Import the pll module
    /// /* Code goes here */
    /// pll::test::test_pll_config(&peripherals.stm32f4.pll); // Run the tests
    /// ```
    pub fn test_pll_config<PllConstants: clock_constants::PllConstants>() {
        debug!("Testing PLL configuration...");

        // 13 or 24MHz --> minimum value
        let mut pllp = Pll::<PllConstants>::compute_pllp(PllConstants::MIN_FREQ_MHZ);
        assert_eq!(PLLP::DivideBy8, pllp);
        let mut plln =
            Pll::<PllConstants>::compute_plln(PllConstants::MIN_FREQ_MHZ, HSI_FREQUENCY_MHZ, pllp);

        #[cfg(not(feature = "stm32f401"))]
        assert_eq!(52 * MULTIPLIER, plln);
        #[cfg(feature = "stm32f401")]
        assert_eq!(96 * MULTIPLIER, plln);

        let mut vco_output_frequency_mhz = HSI_FREQUENCY_MHZ / DEFAULT_PLLM_VALUE as usize * plln;
        let mut pllq = Pll::<PllConstants>::compute_pllq(vco_output_frequency_mhz);

        #[cfg(not(feature = "stm32f401"))]
        assert_eq!(PLLQ::DivideBy3, pllq);
        #[cfg(feature = "stm32f401")]
        assert_eq!(PLLQ::DivideBy4, pllq);

        // 25MHz --> minimum required value for Ethernet devices
        pllp = Pll::<PllConstants>::compute_pllp(25);
        assert_eq!(PLLP::DivideBy8, pllp);
        plln = Pll::<PllConstants>::compute_plln(25, HSI_FREQUENCY_MHZ, pllp);
        assert_eq!(100 * MULTIPLIER, plln);
        vco_output_frequency_mhz = HSI_FREQUENCY_MHZ / DEFAULT_PLLM_VALUE as usize * plln;
        pllq = Pll::<PllConstants>::compute_pllq(vco_output_frequency_mhz);
        assert_eq!(PLLQ::DivideBy5, pllq);

        // 54MHz --> last frequency before PLLP becomes DivideBy6
        pllp = Pll::<PllConstants>::compute_pllp(54);
        assert_eq!(PLLP::DivideBy8, pllp);
        plln = Pll::<PllConstants>::compute_plln(54, HSI_FREQUENCY_MHZ, pllp);
        assert_eq!(216 * MULTIPLIER, plln);
        vco_output_frequency_mhz = HSI_FREQUENCY_MHZ / DEFAULT_PLLM_VALUE as usize * plln;
        pllq = Pll::<PllConstants>::compute_pllq(vco_output_frequency_mhz);
        assert_eq!(PLLQ::DivideBy9, pllq);

        // 55MHz --> PLLP becomes DivideBy6
        pllp = Pll::<PllConstants>::compute_pllp(55);
        assert_eq!(PLLP::DivideBy6, pllp);
        plln = Pll::<PllConstants>::compute_plln(55, HSI_FREQUENCY_MHZ, pllp);
        assert_eq!(165 * MULTIPLIER, plln);
        vco_output_frequency_mhz = HSI_FREQUENCY_MHZ / DEFAULT_PLLM_VALUE as usize * plln;
        pllq = Pll::<PllConstants>::compute_pllq(vco_output_frequency_mhz);
        assert_eq!(PLLQ::DivideBy7, pllq);

        // 70MHz --> Another value for PLLP::DivideBy6
        pllp = Pll::<PllConstants>::compute_pllp(70);
        assert_eq!(PLLP::DivideBy6, pllp);
        plln = Pll::<PllConstants>::compute_plln(70, HSI_FREQUENCY_MHZ, pllp);
        assert_eq!(210 * MULTIPLIER, plln);
        vco_output_frequency_mhz = HSI_FREQUENCY_MHZ / DEFAULT_PLLM_VALUE as usize * plln;
        pllq = Pll::<PllConstants>::compute_pllq(vco_output_frequency_mhz);
        assert_eq!(PLLQ::DivideBy9, pllq);

        // 72MHz --> last frequency before PLLP becomes DivideBy4
        pllp = Pll::<PllConstants>::compute_pllp(72);
        assert_eq!(PLLP::DivideBy6, pllp);
        plln = Pll::<PllConstants>::compute_plln(72, HSI_FREQUENCY_MHZ, pllp);
        assert_eq!(216 * MULTIPLIER, plln);
        vco_output_frequency_mhz = HSI_FREQUENCY_MHZ / DEFAULT_PLLM_VALUE as usize * plln;
        pllq = Pll::<PllConstants>::compute_pllq(vco_output_frequency_mhz);
        assert_eq!(PLLQ::DivideBy9, pllq);

        // 73MHz --> PLLP becomes DivideBy4
        pllp = Pll::<PllConstants>::compute_pllp(73);
        assert_eq!(PLLP::DivideBy4, pllp);
        plln = Pll::<PllConstants>::compute_plln(73, HSI_FREQUENCY_MHZ, pllp);
        assert_eq!(146 * MULTIPLIER, plln);
        vco_output_frequency_mhz = HSI_FREQUENCY_MHZ / DEFAULT_PLLM_VALUE as usize * plln;
        pllq = Pll::<PllConstants>::compute_pllq(vco_output_frequency_mhz);
        assert_eq!(PLLQ::DivideBy7, pllq);

        // 100MHz --> Another value for PLLP::DivideBy4
        pllp = Pll::<PllConstants>::compute_pllp(100);
        assert_eq!(PLLP::DivideBy4, pllp);
        plln = Pll::<PllConstants>::compute_plln(100, HSI_FREQUENCY_MHZ, pllp);
        assert_eq!(200 * MULTIPLIER, plln);
        vco_output_frequency_mhz = HSI_FREQUENCY_MHZ / DEFAULT_PLLM_VALUE as usize * plln;
        pllq = Pll::<PllConstants>::compute_pllq(vco_output_frequency_mhz);
        assert_eq!(PLLQ::DivideBy9, pllq);

        // 108MHz --> last frequency before PLLP becomes DivideBy2
        pllp = Pll::<PllConstants>::compute_pllp(108);
        assert_eq!(PLLP::DivideBy4, pllp);
        plln = Pll::<PllConstants>::compute_plln(108, HSI_FREQUENCY_MHZ, pllp);
        assert_eq!(216 * MULTIPLIER, plln);
        vco_output_frequency_mhz = HSI_FREQUENCY_MHZ / DEFAULT_PLLM_VALUE as usize * plln;
        pllq = Pll::<PllConstants>::compute_pllq(vco_output_frequency_mhz);
        assert_eq!(PLLQ::DivideBy9, pllq);

        // 109MHz --> PLLP becomes DivideBy2
        pllp = Pll::<PllConstants>::compute_pllp(109);
        assert_eq!(PLLP::DivideBy2, pllp);
        plln = Pll::<PllConstants>::compute_plln(109, HSI_FREQUENCY_MHZ, pllp);
        assert_eq!(109 * MULTIPLIER, plln);
        vco_output_frequency_mhz = HSI_FREQUENCY_MHZ / DEFAULT_PLLM_VALUE as usize * plln;
        pllq = Pll::<PllConstants>::compute_pllq(vco_output_frequency_mhz);
        assert_eq!(PLLQ::DivideBy5, pllq);

        // 125MHz --> Another value for PLLP::DivideBy2
        pllp = Pll::<PllConstants>::compute_pllp(125);
        assert_eq!(PLLP::DivideBy2, pllp);
        plln = Pll::<PllConstants>::compute_plln(125, HSI_FREQUENCY_MHZ, pllp);
        assert_eq!(125 * MULTIPLIER, plln);
        vco_output_frequency_mhz = HSI_FREQUENCY_MHZ / DEFAULT_PLLM_VALUE as usize * plln;
        pllq = Pll::<PllConstants>::compute_pllq(vco_output_frequency_mhz);
        assert_eq!(PLLQ::DivideBy6, pllq);

        // 180MHz --> Max frequency for the CPU
        pllp = Pll::<PllConstants>::compute_pllp(180);
        assert_eq!(PLLP::DivideBy2, pllp);
        plln = Pll::<PllConstants>::compute_plln(180, HSI_FREQUENCY_MHZ, pllp);
        assert_eq!(180 * MULTIPLIER, plln);
        vco_output_frequency_mhz = HSI_FREQUENCY_MHZ / DEFAULT_PLLM_VALUE as usize * plln;
        pllq = Pll::<PllConstants>::compute_pllq(vco_output_frequency_mhz);
        assert_eq!(PLLQ::DivideBy8, pllq);

        // 216MHz --> Max frequency for the PLL due to the VCO output frequency limit
        pllp = Pll::<PllConstants>::compute_pllp(216);
        assert_eq!(PLLP::DivideBy2, pllp);
        plln = Pll::<PllConstants>::compute_plln(216, HSI_FREQUENCY_MHZ, pllp);
        assert_eq!(216 * MULTIPLIER, plln);
        vco_output_frequency_mhz = HSI_FREQUENCY_MHZ / DEFAULT_PLLM_VALUE as usize * plln;
        pllq = Pll::<PllConstants>::compute_pllq(vco_output_frequency_mhz);
        assert_eq!(PLLQ::DivideBy9, pllq);

        debug!("Finished testing PLL configuration.");
    }

    /// Check if the PLL works as expected.
    ///
    /// **NOTE:** it is highly recommended to call [test_pll_config]
    /// first to check whether the configuration parameters are correctly computed.
    ///
    /// # Usage
    ///
    /// ```rust,ignore
    /// use stm32f429zi::pll; // Import the PLL module
    /// /* Code goes here */
    /// pll::test::test_pll_struct(&peripherals.stm32f4.pll); // Run the tests
    /// ```
    pub fn test_pll_struct<'a, PllConstants: clock_constants::PllConstants>(
        pll: &'a Pll<'a, PllConstants>,
    ) {
        debug!("Testing PLL struct...");
        // Make sure the PLL clock is disabled
        assert_eq!(Ok(()), pll.disable());
        assert!(!pll.is_enabled());

        // Attempting to configure the PLL with either too high or too low frequency
        assert_eq!(
            Err(ErrorCode::INVAL),
            pll.set_frequency_mhz(PllSource::HSI, HSI_FREQUENCY_MHZ, 12)
        );
        assert_eq!(
            Err(ErrorCode::INVAL),
            pll.set_frequency_mhz(PllSource::HSI, HSI_FREQUENCY_MHZ, 217)
        );

        // Start the PLL with the default configuration.
        assert_eq!(Ok(()), pll.enable());

        // Make sure the PLL is enabled.
        assert!(pll.is_enabled());

        // By default, the PLL clock is set to 96MHz
        assert_eq!(Some(96), pll.get_frequency_mhz());

        // By default, the PLL48 clock is correctly calibrated
        assert!(pll.is_pll48_calibrated());

        // Impossible to configure the PLL clock once it is enabled.
        assert_eq!(
            Err(ErrorCode::FAIL),
            pll.set_frequency_mhz(PllSource::HSI, HSI_FREQUENCY_MHZ, 50)
        );

        // Stop the PLL in order to reconfigure it.
        assert_eq!(Ok(()), pll.disable());

        // Configure the PLL clock to run at 25MHz
        assert_eq!(
            Ok(()),
            pll.set_frequency_mhz(PllSource::HSI, HSI_FREQUENCY_MHZ, 25)
        );

        // Start the PLL with the new configuration
        assert_eq!(Ok(()), pll.enable());

        // get_frequency() method should reflect the new change
        assert_eq!(Some(25), pll.get_frequency_mhz());

        // Since 25 is not a multiple of 48, the PLL48 clock is not correctly calibrated
        assert!(!pll.is_pll48_calibrated());

        // The expected PLL48 clock value in this case should be approximately 40 MHz.
        // It is actually exactly 40MHz in this particular case.
        assert_eq!(Some(40), pll.get_frequency_mhz_pll48());

        // Stop the PLL clock
        assert_eq!(Ok(()), pll.disable());

        // Attempting to get the frequency of the PLL clock when it is disabled should return None.
        assert_eq!(None, pll.get_frequency_mhz());
        // Same for PLL48 clock
        assert_eq!(None, pll.get_frequency_mhz_pll48());

        // Attempting to configure the PLL clock with a frequency multiple of 48MHz
        assert_eq!(
            Ok(()),
            pll.set_frequency_mhz(PllSource::HSI, HSI_FREQUENCY_MHZ, 144)
        );
        assert_eq!(Ok(()), pll.enable());
        assert_eq!(Some(144), pll.get_frequency_mhz());

        // PLL48 clock output should be correctly calibrated
        assert!(pll.is_pll48_calibrated());
        assert_eq!(Some(48), pll.get_frequency_mhz_pll48());

        // Reconfigure the clock for 100MHz
        assert_eq!(Ok(()), pll.disable());
        assert_eq!(
            Ok(()),
            pll.set_frequency_mhz(PllSource::HSI, HSI_FREQUENCY_MHZ, 100)
        );
        assert_eq!(Ok(()), pll.enable());
        assert_eq!(Some(100), pll.get_frequency_mhz());

        // In this case, the PLL48 clock is not correctly calibrated. Its frequency is
        // approximately 44MHz.
        assert!(!pll.is_pll48_calibrated());
        assert_eq!(Some(44), pll.get_frequency_mhz_pll48());

        // Configure the clock to 72MHz = 48MHz * 1.5
        assert_eq!(Ok(()), pll.disable());
        assert_eq!(
            Ok(()),
            pll.set_frequency_mhz(PllSource::HSI, HSI_FREQUENCY_MHZ, 72)
        );
        assert_eq!(Ok(()), pll.enable());
        assert_eq!(Some(72), pll.get_frequency_mhz());

        // In this case, the PLL48 clock is correctly calibrated
        assert!(pll.is_pll48_calibrated());
        assert_eq!(Some(48), pll.get_frequency_mhz_pll48());

        // Turn off the PLL clock
        assert_eq!(Ok(()), pll.disable());
        assert!(!pll.is_enabled());

        debug!("Finished testing PLL struct.");
    }

    /// Run the entire test suite.
    ///
    /// # Usage
    ///
    /// ```rust,ignore
    /// use stm32f429zi::pll; // Import the PLL module
    /// /* Code goes here */
    /// pll::test::run(&peripherals.stm32f4.pll); // Run the tests
    /// ```
    pub fn run<'a, PllConstants: clock_constants::PllConstants>(pll: &'a Pll<'a, PllConstants>) {
        debug!("");
        debug!("===============================================");
        debug!("Testing PLL...");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        test_pll_config::<PllConstants>();
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        test_pll_struct(pll);
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Finished testing PLL. Everything is alright!");
        debug!("===============================================");
        debug!("");
    }
}
