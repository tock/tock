// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL.
//
// Author: Ioan-Cristian CÎRSTEA <ioan.cirstea@oxidos.io>

//! STM32F4xx clock driver
//!
//! This crate provides drivers for various clocks: HSI, PLL, system, AHB, APB1 and APB2.
//! This documentation applies to the system clock, AHB, APB1 and APB2. For in-detail documentation
//! for HSI and PLL, check their documentation.
//!
//! # Features
//!
//! - [x] Dynamic system source
//! - [x] Hardware limits verification for AHB, APB1 and APB2.
//! - [x] Prescaler configuration for AHB, APB1 and APB2.
//! - [x] Support for MCO1
//!
//! # Limitations
//!
//! - [ ] Precision of 1MHz
//! - [ ] No support for MCO2
//!
//! # Usage [^usage_note]
//!
//! First, import the following enums:
//!
//! ```rust,ignore
//! // Assuming a STM32F429 chip. Change this to correspond to the chip model.
//! use stm32f429zi::rcc::APBPrescaler;
//! use stm32f429zi::rcc::AHBPrescaler;
//! use stm32f429zi::rcc::SysClockSource;
//! ```
//!
//! A reference to the [crate::clocks::Clocks] is needed:
//!
//! ```rust,ignore
//! // Add this in board main.rs
//! let clocks = &peripherals.stm32f4.clocks;
//! ```
//!
//! ## Retrieve the AHB frequency:
//!
//! ```rust,ignore
//! let ahb_frequency = clocks.get_ahb_frequency_mhz();
//! debug!("Current AHB frequency is {}MHz", ahb_frequency);
//! ```
//!
//! ## Retrieve the AHB prescaler:
//!
//! ```rust,ignore
//! let ahb_prescaler = clocks.get_ahb_prescaler();
//! debug!("Current AHB prescaler is {:?}", ahb_prescaler);
//! ```
//!
//! NOTE: If one wishes to get the usize equivalent value of [crate::clocks::Clocks::get_ahb_prescaler], to use in
//! computations for example, they must use [crate::rcc::AHBPrescaler].into() method:
//!
//! ```rust,ignore
//! let ahb_prescaler_usize: usize = clocks.get_ahb_prescaler().into();
//! if ahb_prescaler_usize > 8 {
//!     /* Do something */
//! }
//! ```
//!
//! ## Set the AHB prescaler
//!
//! ```rust,ignore
//! clocks.set_ahb_prescaler(AHBPrescaler::DivideBy4);
//! ```
//!
//! ## APB1 and APB2 prescalers
//!
//! APB1 and APB2 prescalers are configured in a similar way as AHB prescaler, except that the
//! corresponding enum is APBPrescaler.
//!
//! ## Retrieve the system clock frequency:
//!
//! ```rust,ignore
//! let sys_frequency = clocks.get_sys_clock_frequency_mhz();
//! debug!("Current system clock frequency is {}MHz", sys_frequency);
//! ```
//!
//! ## Retrieve the system clock source:
//!
//! ```rust,ignore
//! let sys_source = clocks.get_sys_clock_source();
//! debug!("Current system clock source is {:?}", sys_source);
//! ```
//!
//! ## Change the system clock source to PLL:
//!
//! Changing the system clock source is a fastidious task because of AHB, APB1 and APB2 limits,
//! which are chip-dependent. This example assumes a STM32F429 chip.
//!
//! First, get a reference to the PLL
//!
//! ```rust,ignore
//! let pll = &peripherals.stm32f4.clocks.pll;
//! ```
//!
//! Then, configure its frequency and enable it
//! ```rust,ignore
//! pll.set_frequency_mhz(50);
//! pll.enable();
//! ```
//!
//! STM32F429 maximum APB1 frequency is 45MHz, which is computed as following:
//! freq_APB1 = freq_sys / AHB_prescaler / APB1_prescaler
//! Default prescaler values are 1, which gives an frequency of 50MHz without modifying the
//! APB1 prescaler. As such, the APB1 prescaler must be changed.
//!
//! ```rust,ignore
//! clocks.set_apb1_prescaler(APBPrescaler::DivideBy2);
//! ```
//!
//! Since the APB1 frequency limit is satisfied now, the system clock source can be safely changed.
//!
//! ```rust,ignore
//! clocks.set_sys_clock_source(SysClockSource::PLL);
//! ```
//!
//! ## Another example of changing the system clock to PLL for STM32F429:
//!
//! As before, Pll clock is configured and enabled.
//!
//! ```rust,ignore
//! pll.set_frequency_mhz(100);
//! pll.enable();
//! ```
//!
//! Because of the high frequency of the PLL clock, both APB1 and APB2 prescalers must be
//! configured.
//!
//! ```rust,ignore
//! clocks.set_apb1_prescaler(APBPrescaler::DivideBy4);
//! clocks.set_apb2_prescaler(APBPrescaler::DivideBy2);
//! ```
//!
//! As an alternative, the AHB prescaler could be configured to change both APB1 and APB2
//! frequencies.
//!
//! ```rust,ignore
//! // Changing it to 2 wouldn't work, because it would give a frequency of 50MHz for the APB1.
//! clocks.set_ahb_prescaler(APBPrescaler::DivideBy4);
//! ```
//!
//! Now, it's safe to change the system clock source:
//!
//! ```rust,ignore
//! clocks.set_sys_clock_source(SysClockSource::PLL);
//! ```
//!
//! [^usage_note]: For the purpose of brevity, any error checking has been removed.

use crate::chip_specific::ChipSpecs as ChipSpecsTrait;
use crate::clocks::hse::Hse;
use crate::clocks::hsi::Hsi;
use crate::clocks::hsi::HSI_FREQUENCY_MHZ;
use crate::clocks::pll::Pll;
use crate::flash::Flash;
use crate::rcc::AHBPrescaler;
use crate::rcc::APBPrescaler;
use crate::rcc::MCO1Divider;
use crate::rcc::MCO1Source;
use crate::rcc::PllSource;
use crate::rcc::Rcc;
use crate::rcc::SysClockSource;

use kernel::debug;
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

/// Main struct for configuring on-board clocks.
pub struct Clocks<'a, ChipSpecs> {
    rcc: &'a Rcc,
    flash: OptionalCell<&'a Flash<ChipSpecs>>,
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
            flash: OptionalCell::empty(),
            hsi: Hsi::new(rcc),
            hse: Hse::new(rcc),
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
        if let false = match source {
            SysClockSource::HSI => self.hsi.is_enabled(),
            SysClockSource::HSE => self.hse.is_enabled(),
            SysClockSource::PLL => self.pll.is_enabled(),
        } {
            return Err(ErrorCode::FAIL);
        }

        let current_frequency = self.get_sys_clock_frequency_mhz();

        // Get the frequency of the source to be configured
        let alternate_frequency = match source {
            // The unwrap can't fail because the source clock status was checked before
            SysClockSource::HSI => self.hsi.get_frequency_mhz().unwrap(),
            SysClockSource::HSE => self.hse.get_frequency_mhz().unwrap(),
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
            // These unwraps can't panic because set_sys_clock_frequency ensures that the source is
            // enabled. Also, Hsi and Pll structs ensure that the clocks can't be disabled when
            // they are configured as the system clock
            SysClockSource::HSI => self.hsi.get_frequency_mhz().unwrap(),
            SysClockSource::HSE => self.hse.get_frequency_mhz().unwrap(),
            SysClockSource::PLL => self.pll.get_frequency_mhz().unwrap(),
        }
    }

    /// Get the current system clock frequency in MHz from RCC registers instead of the cached
    /// value. Used for debug only.
    pub fn _get_sys_clock_frequency_mhz_no_cache(&self) -> usize {
        match self.get_sys_clock_source() {
            // These unwraps can't panic because set_sys_clock_frequency ensures that the source is
            // enabled. Also, Hsi and Pll structs ensure that the clocks can't be disabled when
            // they are configured as the system clock
            SysClockSource::HSI => self.hsi.get_frequency_mhz().unwrap(),
            SysClockSource::HSE => self.hse.get_frequency_mhz().unwrap(),
            SysClockSource::PLL => {
                let pll_source_frequency = match self.rcc.get_pll_clocks_source() {
                    PllSource::HSI => self.hsi.get_frequency_mhz().unwrap(),
                    PllSource::HSE => self.hse.get_frequency_mhz().unwrap(),
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
    /// + pll_source: PLL source clock (HSI or HSE)
    ///
    /// + desired_frequency_mhz: the desired frequency in MHz. Supported values: 24-216MHz for
    /// STM32F401 and 13-216MHz for all the other chips
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
            PllSource::HSI => HSI_FREQUENCY_MHZ,
            PllSource::HSE => self.hse.get_frequency_mhz().unwrap(),
        };
        self.pll
            .set_frequency_mhz(pll_source, source_frequency, desired_frequency_mhz)
    }

    /// Set the clock source for the microcontroller clock output 1 (MCO1)
    ///
    /// # Errors:
    ///
    /// + [Err]\([ErrorCode::FAIL]\) if the source apart from HSI is already enabled.
    pub fn set_mco1_clock_source(&self, source: MCO1Source) -> Result<(), ErrorCode> {
        match source {
            MCO1Source::HSE => {
                if !self.hse.is_enabled() {
                    return Err(ErrorCode::FAIL);
                }
            }
            MCO1Source::PLL => {
                if self.pll.is_enabled() {
                    return Err(ErrorCode::FAIL);
                }
            }
            _ => (),
        }

        self.rcc.set_mco1_clock_source(source);

        Ok(())
    }

    /// Get the clock source of the MCO1
    pub fn get_mco1_clock_source(&self) -> MCO1Source {
        self.rcc.get_mco1_clock_source()
    }

    /// Set MCO1 divider
    ///
    /// # Errors:
    ///
    /// + [Err]\([ErrorCode::FAIL]\) if the configured source apart from HSI is already enabled.
    pub fn set_mco1_clock_divider(&self, divider: MCO1Divider) -> Result<(), ErrorCode> {
        match self.get_mco1_clock_source() {
            MCO1Source::PLL => {
                if self.pll.is_enabled() {
                    return Err(ErrorCode::FAIL);
                }
            }
            MCO1Source::HSI => (),
            MCO1Source::HSE => (),
        }

        self.rcc.set_mco1_clock_divider(divider);

        Ok(())
    }

    /// Get MCO1 divider
    pub fn get_mco1_clock_divider(&self) -> MCO1Divider {
        self.rcc.get_mco1_clock_divider()
    }
}

/// Stm32f4Clocks trait
///
/// This can be used to control clocks without the need to keep a reference of the chip specific
/// Clocks struct, for instance by peripherals
pub trait Stm32f4Clocks {
    /// Get RCC instance
    fn get_rcc(&self) -> &Rcc;

    /// Get current AHB clock (HCLK) frequency in Hz
    fn get_ahb_frequency(&self) -> usize;

    // Extend this to expose additional clock resources
}

impl<'a, ChipSpecs: ChipSpecsTrait> Stm32f4Clocks for Clocks<'a, ChipSpecs> {
    fn get_rcc(&self) -> &'a Rcc {
        self.rcc
    }

    fn get_ahb_frequency(&self) -> usize {
        self.get_ahb_frequency_mhz() * 1_000_000
    }
}

/// Tests for clocks functionalities
///
/// These tests ensure the clocks are properly working. If any changes are made to the clock
/// module, make sure to run these tests.
///
/// # Usage
///
/// First, import the [crate::clocks] module inside the board main file:
///
/// ```rust,ignore
/// // This example assumes a STM32F429 chip
/// use stm32f429zi::clocks;
/// ```
///
/// To run all the available tests, add this line before **kernel::process::load_processes()**:
///
/// ```rust,ignore
/// clocks::tests::run_all(&peripherals.stm32f4.clocks);
/// ```
///
/// If everything works as expected, the following message should be printed on the kernel console:
///
/// ```text
/// ===============================================
/// Testing clocks...
///
/// ===============================================
/// Testing HSI...
/// Finished testing HSI. Everything is alright!
/// ===============================================
///
///
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
///
///
/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/// Testing clocks struct...
/// Finished testing clocks struct. Everything is alright!
/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
///
/// Finished testing clocks. Everything is alright!
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
    use super::*;

    const LOW_FREQUENCY: usize = 25;
    #[cfg(not(any(
        feature = "stm32f401",
        feature = "stm32f410",
        feature = "stm32f411",
        feature = "stm32f412",
        feature = "stm32f413",
        feature = "stm32f423"
    )))]
    const HIGH_FREQUENCY: usize = 112;
    #[cfg(any(
        feature = "stm32f401",
        feature = "stm32f410",
        feature = "stm32f411",
        feature = "stm32f412",
        feature = "stm32f413",
        feature = "stm32f423"
    ))]
    const HIGH_FREQUENCY: usize = 80;

    fn set_default_configuration<ChipSpecs: ChipSpecsTrait>(clocks: &Clocks<ChipSpecs>) {
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::HSI));
        assert_eq!(Ok(()), clocks.pll.disable());
        assert_eq!(Ok(()), clocks.set_ahb_prescaler(AHBPrescaler::DivideBy1));
        assert_eq!(Ok(()), clocks.set_apb1_prescaler(APBPrescaler::DivideBy1));
        assert_eq!(Ok(()), clocks.set_apb2_prescaler(APBPrescaler::DivideBy1));
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_sys_clock_frequency_mhz());
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_apb1_frequency_mhz());
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_apb2_frequency_mhz());
    }

    // This macro ensure that the system clock frequency goes back to the default value to prevent
    // changing the UART baud rate
    macro_rules! check_and_panic {
        ($left:expr, $right:expr, $clocks: ident) => {
            match (&$left, &$right) {
                (left_val, right_val) => {
                    if *left_val != *right_val {
                        set_default_configuration($clocks);
                        assert_eq!($left, $right);
                    }
                }
            };
        };
    }

    /// Test for the AHB and APB prescalers
    ///
    /// # Usage
    ///
    /// First, import the clock module:
    ///
    /// ```rust,ignore
    /// // This test assumes a STM32F429 chip
    /// use stm32f429zi::clocks;
    /// ```
    ///
    /// Then run the test:
    ///
    /// ```rust,ignore
    /// clocks::test::test_prescalers(&peripherals.stm32f4.clocks);
    /// ```
    pub fn test_prescalers<ChipSpecs: ChipSpecsTrait>(clocks: &Clocks<ChipSpecs>) {
        // This test requires a bit of setup. A system clock running at HIGH_FREQUENCY is configured.
        check_and_panic!(
            Ok(()),
            clocks
                .pll
                .set_frequency_mhz(PllSource::HSI, HSI_FREQUENCY_MHZ, HIGH_FREQUENCY),
            clocks
        );
        check_and_panic!(Ok(()), clocks.pll.enable(), clocks);
        check_and_panic!(
            Ok(()),
            clocks.set_apb1_prescaler(APBPrescaler::DivideBy4),
            clocks
        );
        check_and_panic!(
            Ok(()),
            clocks.set_apb2_prescaler(APBPrescaler::DivideBy2),
            clocks
        );
        check_and_panic!(
            Ok(()),
            clocks.set_sys_clock_source(SysClockSource::PLL),
            clocks
        );

        // Trying to reduce the APB scaler to an invalid value should fail
        check_and_panic!(
            Err(ErrorCode::FAIL),
            clocks.set_apb1_prescaler(APBPrescaler::DivideBy1),
            clocks
        );
        // The following assert will pass on these models because of the low system clock
        // frequency limit
        #[cfg(not(any(
            feature = "stm32f401",
            feature = "stm32f410",
            feature = "stm32f411",
            feature = "stm32f412",
            feature = "stm32f413",
            feature = "stm32f423"
        )))]
        check_and_panic!(
            Err(ErrorCode::FAIL),
            clocks.set_apb2_prescaler(APBPrescaler::DivideBy1),
            clocks
        );
        // Any failure in changing the APB prescalers must preserve their values
        check_and_panic!(APBPrescaler::DivideBy4, clocks.get_apb1_prescaler(), clocks);
        check_and_panic!(APBPrescaler::DivideBy2, clocks.get_apb2_prescaler(), clocks);

        // Increasing the AHB prescaler should allow decreasing APB prescalers
        check_and_panic!(
            Ok(()),
            clocks.set_ahb_prescaler(AHBPrescaler::DivideBy4),
            clocks
        );
        check_and_panic!(
            Ok(()),
            clocks.set_apb1_prescaler(APBPrescaler::DivideBy1),
            clocks
        );
        check_and_panic!(
            Ok(()),
            clocks.set_apb2_prescaler(APBPrescaler::DivideBy1),
            clocks
        );

        // Now, decreasing the AHB prescaler would result in the violation of APB constraints
        check_and_panic!(
            Err(ErrorCode::FAIL),
            clocks.set_ahb_prescaler(AHBPrescaler::DivideBy1),
            clocks
        );
        // Any failure in changing the AHB prescaler must preserve its value
        check_and_panic!(AHBPrescaler::DivideBy4, clocks.get_ahb_prescaler(), clocks);

        // Revert to default configuration
        set_default_configuration(clocks);
    }

    /// Test for the [crate::clocks::Clocks] struct
    ///
    /// # Usage
    ///
    /// First, import the clock module:
    ///
    /// ```rust,ignore
    /// // This test assumes a STM32F429 chip
    /// use stm32f429zi::clocks;
    /// ```
    ///
    /// Then run the test:
    ///
    /// ```rust,ignore
    /// clocks::test::test_clocks_struct(&peripherals.stm32f4.clocks);
    /// ```
    pub fn test_clocks_struct<ChipSpecs: ChipSpecsTrait>(clocks: &Clocks<ChipSpecs>) {
        debug!("");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing clocks struct...");

        // By default, the HSI clock is the system clock
        check_and_panic!(SysClockSource::HSI, clocks.get_sys_clock_source(), clocks);

        // HSI frequency is 16MHz
        check_and_panic!(
            HSI_FREQUENCY_MHZ,
            clocks.get_sys_clock_frequency_mhz(),
            clocks
        );

        // APB1 default prescaler is 1
        check_and_panic!(APBPrescaler::DivideBy1, clocks.get_apb1_prescaler(), clocks);

        // APB1 default frequency is 16MHz
        check_and_panic!(HSI_FREQUENCY_MHZ, clocks.get_apb1_frequency_mhz(), clocks);

        // APB2 default prescaler is 1
        check_and_panic!(APBPrescaler::DivideBy1, clocks.get_apb1_prescaler(), clocks);

        // APB2 default frequency is 16MHz
        check_and_panic!(HSI_FREQUENCY_MHZ, clocks.get_apb2_frequency_mhz(), clocks);

        // Attempting to change the system clock source with a disabled source
        check_and_panic!(
            Err(ErrorCode::FAIL),
            clocks.set_sys_clock_source(SysClockSource::PLL),
            clocks
        );

        // Attempting to set twice the same system clock source is fine
        check_and_panic!(
            Ok(()),
            clocks.set_sys_clock_source(SysClockSource::HSI),
            clocks
        );

        // Change the system clock source to a low frequency so that APB prescalers don't need to be
        // changed
        check_and_panic!(
            Ok(()),
            clocks
                .pll
                .set_frequency_mhz(PllSource::HSI, HSI_FREQUENCY_MHZ, LOW_FREQUENCY),
            clocks
        );
        check_and_panic!(Ok(()), clocks.pll.enable(), clocks);
        check_and_panic!(
            Ok(()),
            clocks.set_sys_clock_source(SysClockSource::PLL),
            clocks
        );
        check_and_panic!(SysClockSource::PLL, clocks.get_sys_clock_source(), clocks);

        // Now the system clock frequency is equal to 25MHz
        check_and_panic!(LOW_FREQUENCY, clocks.get_sys_clock_frequency_mhz(), clocks);

        // APB1 and APB2 frequencies must also be 25MHz
        check_and_panic!(LOW_FREQUENCY, clocks.get_apb1_frequency_mhz(), clocks);
        check_and_panic!(LOW_FREQUENCY, clocks.get_apb2_frequency_mhz(), clocks);

        // Attempting to disable PLL when it is configured as the system clock must fail
        check_and_panic!(Err(ErrorCode::FAIL), clocks.pll.disable(), clocks);
        // Same for the HSI since it is used indirectly as a system clock through PLL
        check_and_panic!(Err(ErrorCode::FAIL), clocks.hsi.disable(), clocks);

        // Revert to default system clock configuration
        set_default_configuration(clocks);

        // Attempting to change the system clock frequency without correctly configuring the APB1
        // prescaler (freq_APB1 <= APB1_FREQUENCY_LIMIT_MHZ) and APB2 prescaler
        // (freq_APB2 <= APB2_FREQUENCY_LIMIT_MHZ) must fail
        check_and_panic!(Ok(()), clocks.pll.disable(), clocks);
        check_and_panic!(
            Ok(()),
            clocks
                .pll
                .set_frequency_mhz(PllSource::HSI, HSI_FREQUENCY_MHZ, HIGH_FREQUENCY),
            clocks
        );
        check_and_panic!(Ok(()), clocks.pll.enable(), clocks);
        check_and_panic!(
            Err(ErrorCode::SIZE),
            clocks.set_sys_clock_source(SysClockSource::PLL),
            clocks
        );

        // Even if the APB1 prescaler is changed to 2, it must fail
        // (HIGH_FREQUENCY / 2 > APB1_FREQUENCY_LIMIT_MHZ)
        check_and_panic!(
            Ok(()),
            clocks.set_apb1_prescaler(APBPrescaler::DivideBy2),
            clocks
        );
        #[cfg(not(any(
            feature = "stm32f401",
            feature = "stm32f410",
            feature = "stm32f411",
            feature = "stm32f412",
            feature = "stm32f413",
            feature = "stm32f423"
        )))]
        check_and_panic!(
            Err(ErrorCode::SIZE),
            clocks.set_sys_clock_source(SysClockSource::PLL),
            clocks
        );

        // Configuring APB1 prescaler to 4 is fine, but APB2 prescaler is still wrong
        check_and_panic!(
            Ok(()),
            clocks.set_apb1_prescaler(APBPrescaler::DivideBy4),
            clocks
        );
        #[cfg(not(any(
            feature = "stm32f401",
            feature = "stm32f410",
            feature = "stm32f411",
            feature = "stm32f412",
            feature = "stm32f413",
            feature = "stm32f423"
        )))]
        check_and_panic!(
            Err(ErrorCode::SIZE),
            clocks.set_sys_clock_source(SysClockSource::PLL),
            clocks
        );

        // Configuring APB2 prescaler to 2
        check_and_panic!(
            Ok(()),
            clocks.set_apb2_prescaler(APBPrescaler::DivideBy2),
            clocks
        );

        // Now the system clock source can be changed
        check_and_panic!(
            Ok(()),
            clocks.set_sys_clock_source(SysClockSource::PLL),
            clocks
        );
        check_and_panic!(HIGH_FREQUENCY / 4, clocks.get_apb1_frequency_mhz(), clocks);
        check_and_panic!(HIGH_FREQUENCY / 2, clocks.get_apb2_frequency_mhz(), clocks);

        // Revert to default system clock configuration
        set_default_configuration(clocks);

        // This time, configure the AHB prescaler instead of APB prescalers
        check_and_panic!(
            Ok(()),
            clocks.set_ahb_prescaler(AHBPrescaler::DivideBy4),
            clocks
        );
        check_and_panic!(Ok(()), clocks.pll.enable(), clocks);
        check_and_panic!(
            Ok(()),
            clocks.set_sys_clock_source(SysClockSource::PLL),
            clocks
        );
        check_and_panic!(HIGH_FREQUENCY / 4, clocks.get_ahb_frequency_mhz(), clocks);
        check_and_panic!(HIGH_FREQUENCY / 4, clocks.get_apb1_frequency_mhz(), clocks);
        check_and_panic!(HIGH_FREQUENCY / 4, clocks.get_apb2_frequency_mhz(), clocks);

        // Revert to default configuration
        set_default_configuration(clocks);

        debug!("Finished testing clocks struct. Everything is alright!");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("");
    }

    /// Test for the microcontroller clock outputs
    ///
    /// # Usage
    ///
    /// First, import the clock module:
    ///
    /// ```rust,ignore
    /// // This test assumes a STM32F429 chip
    /// use stm32f429zi::clocks;
    /// ```
    ///
    /// Then run the test:
    ///
    /// ```rust,ignore
    /// clocks::test::test_mco(&peripherals.stm32f4.clocks);
    /// ```
    pub fn test_mco<ChipSpecs: ChipSpecsTrait>(clocks: &Clocks<ChipSpecs>) {
        debug!("");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing MCOs...");

        // Set MCO1 source to PLL
        assert_eq!(Ok(()), clocks.set_mco1_clock_source(MCO1Source::PLL));

        // Set MCO1 divider to 3
        assert_eq!(
            Ok(()),
            clocks.set_mco1_clock_divider(MCO1Divider::DivideBy3)
        );

        // Enable PLL
        assert_eq!(Ok(()), clocks.pll.enable());

        // Attempting to change the divider while the PLL is running must fail
        assert_eq!(
            Err(ErrorCode::FAIL),
            clocks.set_mco1_clock_divider(MCO1Divider::DivideBy2)
        );

        // Switch back to HSI
        assert_eq!(Ok(()), clocks.set_mco1_clock_source(MCO1Source::HSI));

        // Attempting to change the source to PLL when it is already enabled must fail
        assert_eq!(
            Err(ErrorCode::FAIL),
            clocks.set_mco1_clock_source(MCO1Source::PLL)
        );

        debug!("Finished testing MCOs. Everything is alright!");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("");
    }

    /// Run the entire test suite for all clocks
    pub fn run_all<ChipSpecs: ChipSpecsTrait>(clocks: &Clocks<ChipSpecs>) {
        debug!("");
        debug!("===============================================");
        debug!("Testing clocks...");

        crate::clocks::hsi::tests::run(&clocks.hsi);
        crate::clocks::pll::tests::run(&clocks.pll);
        test_prescalers(clocks);
        test_clocks_struct(clocks);
        test_mco(clocks);

        debug!("Finished testing clocks. Everything is alright!");
        debug!("===============================================");
        debug!("");
    }
}
