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
//! The clock module for STM32WLE5xx chips is highly similar to the
//! one for STM32L4xx chips. This clock implementation provides the
//! minimal functionality required to enable peripherals and configure
//! speeds (as tested for I2C and UART). This is still highly a work
//! in progress and documentation comments here describing the usage
//! will be updated as development continues.

use crate::chip_specific::ChipSpecs as ChipSpecsTrait;
use crate::clocks::hse::Hse;
use crate::clocks::hsi::Hsi;
use crate::clocks::hsi::HSI_FREQUENCY_MHZ;
use crate::clocks::msi::Msi;
use crate::clocks::pll::Pll;

use crate::rcc::AHBPrescaler;
use crate::rcc::APBPrescaler;
use crate::rcc::MCODivider;
use crate::rcc::MCOSource;
use crate::rcc::PllSource;
use crate::rcc::Rcc;
use crate::rcc::SysClockSource;

use kernel::ErrorCode;

/// Main struct for configuring on-board clocks.
pub struct Clocks<'a, ChipSpecs> {
    rcc: &'a Rcc,
    /// MSI
    pub msi: Msi<'a>,
    /// High speed internal clock
    pub hsi: Hsi<'a>,
    /// High speed external clock
    pub hse: Hse<'a>,
    /// Main phase loop-lock clock
    pub pll: Pll<'a, ChipSpecs>,
}

impl<'a, ChipSpecs: ChipSpecsTrait> Clocks<'a, ChipSpecs> {
    // The constructor must be called when the default peripherals are created
    pub fn new(rcc: &'a Rcc) -> Self {
        Self {
            rcc,
            msi: Msi::new(rcc),
            hsi: Hsi::new(rcc),
            hse: Hse::new(rcc),
            pll: Pll::new(rcc),
        }
    }

    // This method should be called when the dependencies are resolved
    // pub(crate) fn set_flash(&self, flash: &'a Flash<ChipSpecs>) {
    //     self.flash.set(flash);
    // }

    /// Set the AHB prescaler
    ///
    /// AHB bus, core, memory, DMA, Cortex System timer and FCLK Cortex free-running clock
    /// frequencies are equal to the system clock frequency divided by the AHB prescaler.
    ///
    /// # Errors:
    ///
    /// + [Err]\([ErrorCode::FAIL]\) if changing the AHB prescaler doesn't preserve APB frequency
    /// constraints
    /// + [Err]\([ErrorCode::BUSY]\) if changing the AHB prescaler took too long. Retry.
    pub fn set_ahb_prescaler(&self, prescaler: AHBPrescaler) -> Result<(), ErrorCode> {
        // Changing the AHB prescaler affects the APB frequencies. A check must be done to ensure
        // that the constraints are still valid
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

    /// Get the current configured AHB prescaler
    pub fn get_ahb_prescaler(&self) -> AHBPrescaler {
        self.rcc.get_ahb_prescaler()
    }

    /// Get the frequency of the AHB
    pub fn get_ahb_frequency_mhz(&self) -> usize {
        let ahb_divider: usize = self.get_ahb_prescaler().into();
        self.get_sys_clock_frequency_mhz() / ahb_divider
    }

    // APB1 frequency must not be higher than the maximum allowable frequency. This method is
    // called when the system clock source is changed. The ahb_frequency_mhz is the
    // hypothetical future frequency.
    fn check_apb1_frequency_limit(&self, ahb_frequency_mhz: usize) -> bool {
        ahb_frequency_mhz
            <= ChipSpecs::APB1_FREQUENCY_LIMIT_MHZ
                * Into::<usize>::into(self.rcc.get_apb1_prescaler())
    }

    /// Set the APB1 prescaler.
    ///
    /// The APB1 peripheral clock frequency is equal to the AHB frequency divided by the APB1
    /// prescaler.
    ///
    /// # Errors:
    ///
    /// + [Err]\([ErrorCode::FAIL]\) if the desired prescaler would break the APB1 frequency limit
    /// + [Err]\([ErrorCode::BUSY]\) if setting the prescaler took too long. Retry.
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

    /// Get the current configured APB1 prescaler
    pub fn get_apb1_prescaler(&self) -> APBPrescaler {
        self.rcc.get_apb1_prescaler()
    }

    /// Get the current APB1 frequency
    pub fn get_apb1_frequency_mhz(&self) -> usize {
        // Every enum variant can be converted into a usize
        let divider: usize = self.rcc.get_apb1_prescaler().into();
        self.get_ahb_frequency_mhz() / divider
    }

    // Same as for APB1, APB2 has a frequency limit that must be enforced by software
    fn check_apb2_frequency_limit(&self, ahb_frequency_mhz: usize) -> bool {
        ahb_frequency_mhz
            <= ChipSpecs::APB2_FREQUENCY_LIMIT_MHZ
                * Into::<usize>::into(self.rcc.get_apb2_prescaler())
    }

    /// Set the APB2 prescaler.
    ///
    /// The APB2 peripheral clock frequency is equal to the AHB frequency divided by the APB2
    /// prescaler.
    ///
    /// # Errors:
    ///
    /// + [Err]\([ErrorCode::FAIL]\) if the desired prescaler would break the APB2 frequency limit
    /// + [Err]\([ErrorCode::BUSY]\) if setting the prescaler took too long. Retry.
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

    /// Get the current configured APB2 prescaler
    pub fn get_apb2_prescaler(&self) -> APBPrescaler {
        self.rcc.get_apb2_prescaler()
    }

    /// Get the current APB2 frequency
    pub fn get_apb2_frequency_mhz(&self) -> usize {
        // Every enum variant can be converted into a usize
        let divider: usize = self.rcc.get_apb2_prescaler().into();
        self.get_ahb_frequency_mhz() / divider
    }

    /// Set the system clock source
    ///
    /// # Errors:
    ///
    /// + [Err]\([ErrorCode::FAIL]\) if the source is not enabled.
    /// + [Err]\([ErrorCode::SIZE]\) if the source frequency surpasses the system clock frequency
    /// limit, or the APB1 and APB2 limits are not satisfied.
    /// + [Err]\([ErrorCode::BUSY]\) if the source switching took too long. Retry.
    pub fn set_sys_clock_source(&self, source: SysClockSource) -> Result<(), ErrorCode> {
        // Immediately return if the required source is already configured as the system clock
        // source. Should this maybe be Err(ErrorCode::ALREADY)?
        if source == self.get_sys_clock_source() {
            return Ok(());
        }

        // Ensure the source is enabled before configuring it as the system clock source
        if !(match source {
            SysClockSource::MSI => self.msi.is_enabled(),
            SysClockSource::HSI => self.hsi.is_enabled(),
            SysClockSource::HSE => self.hse.is_enabled(),
            SysClockSource::PLLR => self.pll.is_enabled(),
        }) {
            return Err(ErrorCode::FAIL);
        }

        // Get the frequency of the source to be configured
        let alternate_frequency = match source {
            // The unwrap can't fail because the source clock status was checked before
            SysClockSource::MSI => self.msi.get_frequency_mhz().unwrap(),
            SysClockSource::HSI => self.hsi.get_frequency_mhz().unwrap(),
            SysClockSource::HSE => self.hse.get_frequency_mhz().unwrap(),
            SysClockSource::PLLR => self.pll.get_frequency_mhz().unwrap(),
        };

        // Check the alternate frequency is not higher than the system clock limit
        if alternate_frequency > ChipSpecs::SYS_CLOCK_FREQUENCY_LIMIT_MHZ {
            return Err(ErrorCode::SIZE);
        }

        // Retrieve the currently configured AHB prescaler
        let ahb_divider: usize = self.get_ahb_prescaler().into();
        // Compute the possible future AHB frequency
        let ahb_frequency = alternate_frequency / ahb_divider;

        // APB1 frequency must not exceed APB1_FREQUENCY_LIMIT_MHZ
        if !self.check_apb1_frequency_limit(ahb_frequency) {
            return Err(ErrorCode::SIZE);
        }

        // APB2 frequency must not exceed APB2_FREQUENCY_LIMIT_MHZ
        if !self.check_apb2_frequency_limit(ahb_frequency) {
            return Err(ErrorCode::SIZE);
        }

        self.rcc.set_sys_clock_source(source);
        // This method is currently a nop.
        Ok(())
    }

    /// Get the current system clock source
    pub fn get_sys_clock_source(&self) -> SysClockSource {
        self.rcc.get_sys_clock_source()
    }

    /// Get the current system clock frequency in MHz
    pub fn get_sys_clock_frequency_mhz(&self) -> usize {
        match self.get_sys_clock_source() {
            // These unwraps can't panic because set_sys_clock_frequency ensures that the source is
            // enabled. Also, Hsi and Pll structs ensure that the clocks can't be disabled when
            // they are configured as the system clock
            SysClockSource::MSI => self.msi.get_frequency_mhz().unwrap(),
            SysClockSource::HSI => self.hsi.get_frequency_mhz().unwrap(),
            SysClockSource::HSE => self.hse.get_frequency_mhz().unwrap(),
            SysClockSource::PLLR => self.pll.get_frequency_mhz().unwrap(),
        }
    }

    /// Get the current system clock frequency in MHz from RCC registers instead of the cached
    /// value. Used for debug only.
    pub fn _get_sys_clock_frequency_mhz_no_cache(&self) -> usize {
        match self.get_sys_clock_source() {
            // These unwraps can't panic because set_sys_clock_frequency ensures that the source is
            // enabled. Also, Hsi and Pll structs ensure that the clocks can't be disabled when
            // they are configured as the system clock
            SysClockSource::MSI => self.msi.get_frequency_mhz().unwrap(),
            SysClockSource::HSI => self.hsi.get_frequency_mhz().unwrap(),
            SysClockSource::HSE => self.hse.get_frequency_mhz().unwrap(),
            SysClockSource::PLLR => {
                let pll_source_frequency = match self.rcc.get_pll_clocks_source() {
                    PllSource::MSI => self.msi.get_frequency_mhz().unwrap(),
                    PllSource::HSI => self.hsi.get_frequency_mhz().unwrap(),
                    PllSource::HSE => self.hse.get_frequency_mhz().unwrap(),
                    _ => unimplemented!(),
                };
                self.pll
                    .get_frequency_mhz_no_cache(pll_source_frequency)
                    .unwrap()
            }
        }
    }

    /// Set the frequency of the PLL clock.
    ///
    /// # Parameters
    ///
    /// + pll_source: PLL source clock (HSI or HSE or MSI)
    ///
    /// + desired_frequency_mhz: the desired frequency in MHz.
    ///
    /// # Errors
    ///
    /// + [Err]\([ErrorCode::INVAL]\): if the desired frequency can't be achieved
    /// + [Err]\([ErrorCode::FAIL]\): if the PLL clock is already enabled. It must be disabled before
    pub fn set_pll_frequency_mhz(
        &self,
        pll_source: PllSource,
        desired_frequency_mhz: usize,
    ) -> Result<(), ErrorCode> {
        let source_frequency = match pll_source {
            PllSource::MSI => self.msi.get_frequency_mhz().unwrap(),
            PllSource::HSI => HSI_FREQUENCY_MHZ,
            PllSource::HSE => self.hse.get_frequency_mhz().unwrap(),
            _ => unimplemented!(),
        };
        self.pll
            .set_frequency_mhz(pll_source, source_frequency, desired_frequency_mhz)
    }

    /// Set the clock source for the microcontroller clock output 1 (MCO1)
    ///
    /// # Errors:
    ///
    /// + [Err]\([ErrorCode::FAIL]\) if the source apart from HSI is already enabled.
    pub fn set_mco1_clock_source(&self, source: MCOSource) -> Result<(), ErrorCode> {
        match source {
            MCOSource::HSE => {
                if !self.hse.is_enabled() {
                    return Err(ErrorCode::FAIL);
                }
            }
            MCOSource::PLLR => {
                if self.pll.is_enabled() {
                    return Err(ErrorCode::FAIL);
                }
            }
            _ => (),
        }

        self.rcc.set_mco1_clock_source(source);

        Ok(())
    }

    /// Get the clock source of the MCO
    pub fn get_mco1_clock_source(&self) -> MCOSource {
        self.rcc.get_mco1_clock_source()
    }

    /// Set MCO1 divider
    ///
    /// # Errors:
    ///
    /// + [Err]\([ErrorCode::FAIL]\) if the configured source apart from HSI is already enabled.
    pub fn set_mco1_clock_divider(&self, divider: MCODivider) -> Result<(), ErrorCode> {
        match self.get_mco1_clock_source() {
            MCOSource::PLLR => {
                if self.pll.is_enabled() {
                    return Err(ErrorCode::FAIL);
                }
            }
            MCOSource::HSI => (),
            MCOSource::HSE => (),
            _ => unimplemented!(),
        }

        self.rcc.set_mco_clock_divider(divider);

        Ok(())
    }

    /// Get MCO1 divider
    pub fn get_mco1_clock_divider(&self) -> MCODivider {
        self.rcc.get_mco_clock_divider()
    }
}

/// Stm32WLE5xx Clocks trait
///
/// This can be used to control clocks without the need to keep a reference of the chip specific
/// Clocks struct, for instance by peripherals
pub trait Stm32wle5xxClocks {
    /// Get RCC instance
    fn get_rcc(&self) -> &Rcc;

    /// Get current AHB clock (HCLK) frequency in Hz
    fn get_ahb_frequency(&self) -> usize;

    // Extend this to expose additional clock resources
}

impl<'a, ChipSpecs: ChipSpecsTrait> Stm32wle5xxClocks for Clocks<'a, ChipSpecs> {
    fn get_rcc(&self) -> &'a Rcc {
        self.rcc
    }

    fn get_ahb_frequency(&self) -> usize {
        self.get_ahb_frequency_mhz() * 1_000_000
    }
}
