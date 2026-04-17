// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Author: Kamil Duljas <kamil.duljas@gmail.com>

use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

use crate::chip_specific::ChipSpecs as ChipSpecsTrait;
use crate::clocks::hsi::Hsi;
use crate::clocks::msi::Msi;
use crate::clocks::pll::Pll;
use crate::flash::Flash;
use crate::rcc::{AHBPrescaler, APBPrescaler, PllSource, Rcc, SysClockSource};

/// Main struct for configuring on-board clocks.
pub struct Clocks<'a, ChipSpecs> {
    rcc: &'a Rcc,
    flash: OptionalCell<&'a Flash<ChipSpecs>>,
    /// MSI (multispeed internal) RC oscillator clock
    pub msi: Msi<'a>,
    /// High speed internal clock
    pub hsi: Hsi<'a>,
    /// Main phase loop-lock clock
    pub pll: Pll<'a, ChipSpecs>,
}

impl<'a, ChipSpecs: ChipSpecsTrait> Clocks<'a, ChipSpecs> {
    // The constructor must be called when the default peripherals are created
    pub fn new(rcc: &'a Rcc) -> Self {
        Self {
            rcc,
            flash: OptionalCell::empty(),
            msi: Msi::new(rcc),
            hsi: Hsi::new(rcc),
            pll: Pll::new(rcc),
        }
    }

    // This method should be called when the dependencies are resolved
    pub(crate) fn set_flash(&self, flash: &'a Flash<ChipSpecs>) {
        self.flash.set(flash);
    }

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
            SysClockSource::PLL => self.pll.is_enabled(),
        }) {
            return Err(ErrorCode::FAIL);
        }

        let current_frequency = self.get_sys_clock_frequency_mhz();

        // Get the frequency of the source to be configured
        let alternate_frequency = match source {
            // The unwrap can't fail because the source clock status was checked before
            SysClockSource::MSI => self.msi.get_frequency_mhz().unwrap(),
            SysClockSource::HSI => self.hsi.get_frequency_mhz().unwrap(),
            SysClockSource::PLL => self.pll.get_frequency_mhz().unwrap(),
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

        // The documentation recommends the following sequence when changing the system clock
        // frequency:
        //
        // + if the desired frequency is higher than the current frequency, first change flash
        // latency, then set the new system clock source.
        // + if the desired frequency is lower than the current frequency, first change the system
        // clock source, then set the flash latency
        if alternate_frequency > current_frequency {
            self.flash
                .unwrap_or_panic()
                .set_latency(alternate_frequency)?;
        }
        self.rcc.set_sys_clock_source(source);
        if alternate_frequency < current_frequency {
            self.flash
                .unwrap_or_panic()
                .set_latency(alternate_frequency)?;
        }

        // If this point is reached, everything worked as expected
        Ok(())
    }

    /// Get the current system clock source
    pub fn get_sys_clock_source(&self) -> SysClockSource {
        self.rcc.get_sys_clock_source()
    }

    /// Get the current system clock frequency in MHz
    pub fn get_sys_clock_frequency_mhz(&self) -> usize {
        match self.get_sys_clock_source() {
            SysClockSource::MSI => self.msi.get_frequency_mhz().unwrap(),
            SysClockSource::HSI => self.hsi.get_frequency_mhz().unwrap(),
            SysClockSource::PLL => self.pll.get_frequency_mhz().unwrap(),
        }
    }

    /// Set the frequency of the PLL clock.
    ///
    /// # Parameters
    ///
    /// + pll_source: PLL source clock (MSI or HSI)
    ///
    /// + desired_frequency_mhz: the desired frequency in MHz. Supported values: 8 - 80 MHz
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
            PllSource::HSI => self.hsi.get_frequency_mhz().unwrap(),
            PllSource::NoClock => todo!(),
        };
        self.pll
            .set_frequency_mhz(pll_source, source_frequency, desired_frequency_mhz)
    }
}

/// Stm32l4Clocks trait
///
/// This can be used to control clocks without the need to keep a reference of the chip specific
/// Clocks struct, for instance by peripherals
pub trait Stm32l4Clocks {
    /// Get RCC instance
    fn get_rcc(&self) -> &Rcc;

    /// Get current AHB clock (HCLK) frequency in Hz
    fn get_ahb_frequency(&self) -> usize;

    // Extend this to expose additional clock resources
}

impl<'a, ChipSpecs: ChipSpecsTrait> Stm32l4Clocks for Clocks<'a, ChipSpecs> {
    fn get_rcc(&self) -> &'a Rcc {
        self.rcc
    }

    fn get_ahb_frequency(&self) -> usize {
        self.get_ahb_frequency_mhz() * 1_000_000
    }
}
