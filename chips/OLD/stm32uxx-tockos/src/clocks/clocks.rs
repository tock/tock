// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL.
//
// Author: Ioan-Cristian CÎRSTEA <ioan.cirstea@oxidos.io>

//! STM32U5xx clock helper (minimal, HSI16-only).
//!
//! This module provides minimal accessors around the RCC for:
//! - AHB, APB1 and APB2 prescalers
//! - AHB, APB1 and APB2 frequencies (in MHz)
//! - System clock source and frequency (assuming HSI16-only for now).
//!
//! It is intentionally much simpler than the STM32F4 version:
//! - No PLL support
//! - No HSE support
//! - No MCO / tests
//!
//! Policy for “use HSI16 as SYSCLK” lives in [`crate::clocks::hsi::Hsi16`].

use crate::chip_specifics::ChipSpecs as ChipSpecsTrait;
use crate::clocks::hsi::Hsi16;
use crate::rcc::{AHBPrescaler, APBPrescaler, Rcc, SysClockSource};

use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

/// Main struct for querying / configuring on-chip clocks.
///
/// For this minimal STM32U5 bring-up:
/// - SYSCLK is assumed to be HSI16, configured by `Hsi16::configure_as_sysclk()`.
/// - We only expose AHB / APB prescalers and derived frequencies.
pub struct Clocks<'a, ChipSpecs> {
    rcc: &'a Rcc,
    //flash: OptionalCell<&'a crate::flash::Flash<ChipSpecs>>,
    /// Optional cache or hooks for future extensions (e.g. flash latency).
    _chip: OptionalCell<core::marker::PhantomData<ChipSpecs>>,
}

impl<'a, ChipSpecs: ChipSpecsTrait> Clocks<'a, ChipSpecs> {
    /// Constructor; should be called when the default peripherals are created.
    ///
    /// This does *not* touch the hardware; it just wraps the already initialized RCC.
    pub fn new(rcc: &'a Rcc) -> Self {
        Self {
            rcc,
            //flash: OptionalCell::empty(),
            _chip: OptionalCell::new(core::marker::PhantomData),
        }
    }

    //pub(crate) fn set_flash(&self, flash: &crate::flash::Flash<ChipSpecs>) {
    //    // For now, do nothing. Once you add more clock sources (PLL, HSE),
    //    // this method should store a reference to flash and adjust latency
    //    // as SYSCLK frequency changes.
    //    //self.flash.set(flash);
    //}

    /// Set the AHB prescaler.
    ///
    /// AHB bus, core, memory, DMA, SysTick and FCLK frequencies are equal to
    /// the system clock frequency divided by the AHB prescaler.
    ///
    /// # Errors
    ///
    /// - `Err(ErrorCode::FAIL)` if changing the AHB prescaler would violate
    ///   APB1 or APB2 maximum frequency limits.
    /// - `Err(ErrorCode::BUSY)` if the prescaler write did not latch after
    ///   a small number of retries.
    pub fn set_ahb_prescaler(&self, prescaler: AHBPrescaler) -> Result<(), ErrorCode> {
        let divider: usize = prescaler.into();
        let new_ahb_frequency = self.get_sys_clock_frequency_mhz() / divider;

        if !self.check_apb1_frequency_limit(new_ahb_frequency)
            || !self.check_apb2_frequency_limit(new_ahb_frequency)
        {
            return Err(ErrorCode::FAIL);
        }

        self.rcc.set_ahb_prescaler(prescaler);

        for _ in 0..16 {
            if self.get_ahb_prescaler() == prescaler {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    /// Get the currently configured AHB prescaler.
    pub fn get_ahb_prescaler(&self) -> AHBPrescaler {
        self.rcc.get_ahb_prescaler()
    }

    /// Get the AHB frequency (HCLK) in MHz.
    pub fn get_ahb_frequency_mhz(&self) -> usize {
        let ahb_divider: usize = self.get_ahb_prescaler().into();
        self.get_sys_clock_frequency_mhz() / ahb_divider
    }

    // APB1 frequency must not exceed the chip-specific maximum.
    // `ahb_frequency_mhz` is the *prospective* AHB frequency.
    fn check_apb1_frequency_limit(&self, ahb_frequency_mhz: usize) -> bool {
        ahb_frequency_mhz
            <= ChipSpecs::APB1_FREQUENCY_LIMIT_MHZ
                * Into::<usize>::into(self.rcc.get_apb1_prescaler())
    }

    /// Set the APB1 prescaler.
    ///
    /// The APB1 peripheral clock frequency is equal to the AHB frequency divided by
    /// the APB1 prescaler.
    ///
    /// # Errors
    ///
    /// - `Err(ErrorCode::FAIL)` if the desired prescaler would break the APB1 limit.
    /// - `Err(ErrorCode::BUSY)` if the prescaler write did not latch.
    pub fn set_apb1_prescaler(&self, prescaler: APBPrescaler) -> Result<(), ErrorCode> {
        let ahb_frequency = self.get_ahb_frequency_mhz();
        let divider: usize = prescaler.into();
        if ahb_frequency / divider > ChipSpecs::APB1_FREQUENCY_LIMIT_MHZ {
            return Err(ErrorCode::FAIL);
        }

        self.rcc.set_apb1_prescaler(prescaler);

        for _ in 0..16 {
            if self.rcc.get_apb1_prescaler() == prescaler {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    /// Get the currently configured APB1 prescaler.
    pub fn get_apb1_prescaler(&self) -> APBPrescaler {
        self.rcc.get_apb1_prescaler()
    }

    /// Get the APB1 frequency (PCLK1) in MHz.
    pub fn get_apb1_frequency_mhz(&self) -> usize {
        let divider: usize = self.rcc.get_apb1_prescaler().into();
        self.get_ahb_frequency_mhz() / divider
    }

    // Same deal as APB1: APB2 must not exceed its maximum.
    fn check_apb2_frequency_limit(&self, ahb_frequency_mhz: usize) -> bool {
        ahb_frequency_mhz
            <= ChipSpecs::APB2_FREQUENCY_LIMIT_MHZ
                * Into::<usize>::into(self.rcc.get_apb2_prescaler())
    }

    /// Set the APB2 prescaler.
    ///
    /// The APB2 peripheral clock frequency is equal to the AHB frequency divided by
    /// the APB2 prescaler.
    ///
    /// # Errors
    ///
    /// - `Err(ErrorCode::FAIL)` if the desired prescaler would break the APB2 limit.
    /// - `Err(ErrorCode::BUSY)` if the prescaler write did not latch.
    pub fn set_apb2_prescaler(&self, prescaler: APBPrescaler) -> Result<(), ErrorCode> {
        let current_ahb_frequency = self.get_ahb_frequency_mhz();
        let divider: usize = prescaler.into();
        if current_ahb_frequency / divider > ChipSpecs::APB2_FREQUENCY_LIMIT_MHZ {
            return Err(ErrorCode::FAIL);
        }

        self.rcc.set_apb2_prescaler(prescaler);

        for _ in 0..16 {
            if self.rcc.get_apb2_prescaler() == prescaler {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    /// Get the currently configured APB2 prescaler.
    pub fn get_apb2_prescaler(&self) -> APBPrescaler {
        self.rcc.get_apb2_prescaler()
    }

    /// Get the APB2 frequency (PCLK2) in MHz.
    pub fn get_apb2_frequency_mhz(&self) -> usize {
        let divider: usize = self.rcc.get_apb2_prescaler().into();
        self.get_ahb_frequency_mhz() / divider
    }

    /// Get the current system clock source as seen in RCC.
    pub fn get_sys_clock_source(&self) -> SysClockSource {
        self.rcc.get_sys_clock_source()
    }

    /// Get the current system clock frequency in MHz.
    ///
    /// For this minimal port, we assume:
    /// - `Hsi16::configure_as_sysclk()` was called once at startup.
    /// - SYSCLK is HSI16 for normal operation.
    pub fn get_sys_clock_frequency_mhz(&self) -> usize {
        match self.get_sys_clock_source() {
            SysClockSource::HSI16 => Hsi16::freq_mhz(),
            // For now, treat "Other" as HSI16 as well. Once you add PLL/HSE,
            // this must be extended.
            SysClockSource::Other(_) => {
                panic!("Unsupported SYSCLK source");
            }
        }
    }
}

/// Minimal STM32U5 clocks trait.
///
/// This lets peripherals depend only on this trait instead of the concrete
/// `Clocks` type, which makes mocking and testing easier later.
pub trait Stm32u5Clocks {
    /// Get RCC instance.
    fn get_rcc(&self) -> &Rcc;
    // CURRENT CLOCKS DEFAULT TO HSI16, ONLY HSI16 HAS BEEN IMPLEMENTED AS SYSCLOCK
    /// Get current AHB clock (HCLK) frequency in Hz.
    fn get_ahb_frequency(&self) -> usize;

    /// Get current APB1 clock (PCLK1) frequency in Hz.
    fn get_apb1_frequency(&self) -> usize;

    /// Get current APB2 clock (PCLK2) frequency in Hz.
    fn get_apb2_frequency(&self) -> usize;

    /// Get current system clock frequency in Hz.
    fn get_sys_frequency(&self) -> usize;
}

impl<'a, ChipSpecs: ChipSpecsTrait> Stm32u5Clocks for Clocks<'a, ChipSpecs> {
    fn get_rcc(&self) -> &'a Rcc {
        self.rcc
    }

    fn get_ahb_frequency(&self) -> usize {
        self.get_ahb_frequency_mhz() * 1_000_000
    }

    fn get_apb1_frequency(&self) -> usize {
        self.get_apb1_frequency_mhz() * 1_000_000
    }

    fn get_apb2_frequency(&self) -> usize {
        self.get_apb2_frequency_mhz() * 1_000_000
    }

    fn get_sys_frequency(&self) -> usize {
        self.get_sys_clock_frequency_mhz() * 1_000_000
    }
}
