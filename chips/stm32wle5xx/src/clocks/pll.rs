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

use crate::chip_specific::clock_constants;
use crate::clocks::hsi::HSI_FREQUENCY_MHZ;
use crate::rcc::Rcc;
use crate::rcc::SysClockSource;
use crate::rcc::{PLLPDivider, PLLQDivider, PllSource};
use crate::rcc::{DEFAULT_PLLM_VALUE, DEFAULT_PLLN_VALUE, DEFAULT_PLLP_VALUE, DEFAULT_PLLQ_VALUE};

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
            PLLPDivider::DivideBy2 => 2,
            PLLPDivider::DivideBy4 => 4,
            PLLPDivider::DivideBy6 => 6,
            PLLPDivider::DivideBy8 => 8,
            _ => unimplemented!(),
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
    fn compute_pllp(desired_frequency_mhz: usize) -> PLLPDivider {
        if desired_frequency_mhz < 55 {
            PLLPDivider::DivideBy8
        } else if desired_frequency_mhz < 73 {
            PLLPDivider::DivideBy6
        } else if desired_frequency_mhz < 109 {
            PLLPDivider::DivideBy4
        } else {
            PLLPDivider::DivideBy2
        }
    }

    // The caller must ensure the desired frequency lies between MIN_FREQ_MHZ and
    // MAX_FREQ_MHZ. Otherwise, the return value makes no sense.
    fn compute_plln(
        desired_frequency_mhz: usize,
        pll_source_clock_freq: usize,
        pllp: PLLPDivider,
    ) -> usize {
        let vco_input_frequency: usize = pll_source_clock_freq / DEFAULT_PLLM_VALUE as usize;
        desired_frequency_mhz * Into::<usize>::into(pllp) / vco_input_frequency
    }

    // The caller must ensure the VCO output frequency lies between 100 and 432MHz. Otherwise, the
    // return value makes no sense.
    fn compute_pllq(vco_output_frequency_mhz: usize) -> PLLQDivider {
        for pllq in 3..10 {
            if 48 * pllq >= vco_output_frequency_mhz {
                return match pllq {
                    3 => PLLQDivider::DivideBy3,
                    4 => PLLQDivider::DivideBy4,
                    5 => PLLQDivider::DivideBy5,
                    6 => PLLQDivider::DivideBy6,
                    7 => PLLQDivider::DivideBy7,
                    8 => PLLQDivider::DivideBy8,
                    _ => unimplemented!(),
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
        if self.rcc.get_sys_clock_source() == SysClockSource::PLLR {
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
    /// + desired_frequency_mhz: the desired frequency in MHz.
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
            .set(pll48_frequency == 48 && vco_output_frequency.is_multiple_of(pllq as usize));

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
