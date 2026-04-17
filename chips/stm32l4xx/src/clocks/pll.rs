// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Author: Kamil Duljas <kamil.duljas@gmail.com>

use crate::chip_specific::clock_constants;
use crate::clocks::msi::MSI_FREQUENCY_MHZ;
use crate::rcc::PllSource;
use crate::rcc::Rcc;
use crate::rcc::SysClockSource;
use crate::rcc::{DEFAULT_PLLM_VALUE, DEFAULT_PLLN_VALUE, DEFAULT_PLLR_VALUE};

use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

use core::marker::PhantomData;

/// Main PLL clock structure.
pub struct Pll<'a, PllConstants> {
    rcc: &'a Rcc,
    frequency_mhz: OptionalCell<usize>,
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
        let pllm: usize = Into::into(DEFAULT_PLLM_VALUE);
        let pllr: usize = Into::into(DEFAULT_PLLR_VALUE);
        Self {
            rcc,
            frequency_mhz: OptionalCell::new(MSI_FREQUENCY_MHZ / pllm * DEFAULT_PLLN_VALUE / pllr),
            _marker: PhantomData,
        }
    }

    /// Set the PLL source clock
    fn set_pll_source_clock(&self, source: PllSource) -> Result<(), ErrorCode> {
        if self.is_enabled() {
            Err(ErrorCode::FAIL)
        } else {
            self.rcc.set_pll_clock_source(source);
            Ok(())
        }
    }

    /// Start the PLL clock.
    ///
    /// # Errors
    ///
    /// + [Err]\([ErrorCode::FAIL]\): if enabling the PLL clock has set NoClock. First set pll clock source
    /// + [Err]\([ErrorCode::BUSY]\): if enabling the PLL clock took too long. Recall this method to
    /// ensure the PLL clock is running.
    pub fn enable(&self) -> Result<(), ErrorCode> {
        if self.rcc.get_pll_clock_source() == PllSource::NoClock {
            return Err(ErrorCode::FAIL);
        }
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
        self.rcc.set_pll_clock_source(PllSource::NoClock);

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
    /// Missing configuration for f_in_khz = 48000 target = 59 Mhz
    /// Missing configuration for f_in_khz = 48000 target = 61 Mhz
    /// Missing configuration for f_in_khz = 48000 target = 65 Mhz
    /// Missing configuration for f_in_khz = 48000 target = 67 Mhz
    /// Missing configuration for f_in_khz = 48000 target = 71 Mhz
    /// Missing configuration for f_in_khz = 48000 target = 73 Mhz
    /// Missing configuration for f_in_khz = 48000 target = 77 Mhz
    /// Missing configuration for f_in_khz = 48000 target = 79 Mhz
    ///
    /// # Parameters
    ///
    /// + pll_source: PLL source clock (HSI or HSE)
    ///
    /// + source_frequency: the frequency of the PLL source clock in MHz. For the HSI the frequency
    /// is fixed to 16MHz. For the HSE, the frequency is hardware-dependent
    ///
    /// + desired_frequency_mhz: the desired frequency in MHz. Supported values: 8 - 80 MHz
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
        // Source frequency / PLLM = VCO input frequency (must range from 4 MHz to 16 MHz)
        // VCO output frequency = VCO input frequency * PLLN (must range from 64 MHz to 344 MHz)
        // PLL output frequency = VCO output frequency / PLLR (must range from 8 Mhz to 80 Mhz)

        // Set PLL source (MSI or HSI)
        if self.set_pll_source_clock(pll_source) != Ok(()) {
            return Err(ErrorCode::FAIL);
        }

        if source_frequency < 4000 {
            return Err(ErrorCode::FAIL);
        }
        if let Some((m, n, r, _sysclk)) = Self::find_pll(source_frequency, desired_frequency_mhz) {
            self.rcc.set_pll_clock_m_divider(m.into());
            self.rcc.set_pll_clock_n_multiplier(n);
            self.rcc.set_pll_clock_r_divider(r.into());
            Ok(())
        } else {
            return Err(ErrorCode::FAIL);
        }
    }

    fn find_pll(f_in_khz: usize, target_mhz: usize) -> Option<(usize, usize, usize, usize)> {
        let target_khz = target_mhz * 1000;

        for m in 1..=8 {
            let vco_in = f_in_khz / m;
            if !(4_000..=16_000).contains(&vco_in) {
                continue;
            }

            for n in 8..=86 {
                let vco_out = vco_in * n;
                if !(64_000..=344_000).contains(&vco_out) {
                    continue;
                }

                for &r in &[2, 4, 6, 8] {
                    // Sprawdź, czy vco_out dzieli się równo przez R
                    if vco_out % r != 0 {
                        continue; // SYSCLK nie byłby całkowity
                    }

                    let sys = vco_out / r;

                    if !(8_000..=80_000).contains(&sys) {
                        continue;
                    }

                    let err = if sys > target_khz {
                        sys - target_khz
                    } else {
                        target_khz - sys
                    };

                    if err == 0 {
                        return Some((m, n, r, sys / 1000)); // w MHz
                    }
                }
            }
        }
        None
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
}
