// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL.

//! STM32F4xx flash driver
//!
//! This driver provides basic functionalities for the entire STM32F4 series.
//!
//! # Features
//!
//! - [x] Configuring latency based on the system clock frequency
//!
//! # Missing features
//!
//! - [ ] Support for different power supplies
//! - [ ] Instruction prefetch
//! - [ ] Instruction and data cache
//!
//!
//! # Usage
//!
//! To use this driver, a reference to the Flash peripheral is required:
//!
//! ```rust,ignore
//! // Inside the board main.rs
//! let flash = &peripherals.stm32f4.flash;
//! ```
//!
//! ## Retrieve the current flash latency
//!
//! ```rust,ignore
//! let flash_latency = flash.get_latency() as usize;
//! debug!("Current flash latency is {}", flash_latency);
//! ```

use crate::chip_specific::flash::FlashChipSpecific as FlashChipSpecificTrait;
use crate::chip_specific::flash::FlashLatency16;
use crate::chip_specific::flash::RegisterToFlashLatency;

use kernel::debug;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::registers::{register_bitfields, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

use core::marker::PhantomData;

#[repr(C)]
struct FlashRegisters {
    /// Flash access control register
    acr: ReadWrite<u32, ACR::Register>,
    /// Flash key register
    keyr: WriteOnly<u32>,
    /// Flash option key register
    optkeyr: WriteOnly<u32>,
    /// Status register
    sr: ReadWrite<u32, SR::Register>,
    /// Control register
    cr: ReadWrite<u32, CR::Register>,
    /// Flash option control register
    optcr: ReadWrite<u32, OPTCR::Register>,
    /// Flash option control register 1
    #[cfg(feature = "stm32f429")]
    optcr1: ReadWrite<u32, OPTCR1::Register>,
}

register_bitfields![u32,
    ACR [
        /// Latency
        // NOTE: This bit field can be either 3 or 4 bits long
        LATENCY OFFSET(0) NUMBITS(4) [],
        /// Prefetch enable
        PRFTEN OFFSET(8) NUMBITS(1) [],
        /// Instruction cache enable
        ICEN OFFSET(9) NUMBITS(1) [],
        /// Data cache enable
        DCEN OFFSET(10) NUMBITS(1) [],
        /// Instruction cache reset
        ICRST OFFSET(11) NUMBITS(1) [],
        /// Data cache reset
        DCRST OFFSET(12) NUMBITS(1) []
    ],
    KEYR [
        /// FPEC key
        KEY OFFSET(0) NUMBITS(32) []
    ],
    OPTKEYR [
        /// Option byte key
        OPTKEY OFFSET(0) NUMBITS(32) []
    ],
    SR [
        /// End of operation
        EOP OFFSET(0) NUMBITS(1) [],
        /// Operation error
        OPERR OFFSET(1) NUMBITS(1) [],
        /// Write protection error
        WRPERR OFFSET(4) NUMBITS(1) [],
        /// Programming alignment error
        PGAERR OFFSET(5) NUMBITS(1) [],
        /// Programming parallelism error
        PGPERR OFFSET(6) NUMBITS(1) [],
        /// Programming sequence error
        PGSERR OFFSET(7) NUMBITS(1) [],
        /// Read protection error
        // NOTE: This bit field is not available on STM32F405, STM32F415, STM32F407 and STM32F417
        RDERR OFFSET(8) NUMBITS(1) [],
        /// Busy
        BSY OFFSET(16) NUMBITS(1) []
    ],
    CR [
        /// Programming
        PG OFFSET(0) NUMBITS(1) [],
        /// Sector Erase
        SER OFFSET(1) NUMBITS(1) [],
        /// Mass Erase of sectors 0 to 11
        MER OFFSET(2) NUMBITS(1) [],
        /// Sector number
        // NOTE: This bit field can be either 4 or 5 bits long depending on the chip model
        SNB OFFSET(3) NUMBITS(5) [],
        /// Program size
        PSIZE OFFSET(8) NUMBITS(2) [],
        /// Mass Erase of sectors 12 to 23
        // NOTE: This bit is not available on all chip models
        MER1 OFFSET(15) NUMBITS(1) [],
        /// Start
        STRT OFFSET(16) NUMBITS(1) [],
        /// End of operation interrupt enable
        EOPIE OFFSET(24) NUMBITS(1) [],
        /// Error interrupt enable
        ERRIE OFFSET(25) NUMBITS(1) [],
        /// Lock
        LOCK OFFSET(31) NUMBITS(1) []
    ],
    OPTCR [
        /// Option lock
        OPTLOCK OFFSET(0) NUMBITS(1) [],
        /// Option start
        OPTSTRT OFFSET(1) NUMBITS(1) [],
        /// BOR reset Level
        BOR_LEV OFFSET(2) NUMBITS(2) [],
        /// WDG_SW User option bytes
        WDG_SW OFFSET(5) NUMBITS(1) [],
        /// nRST_STOP User option bytes
        nRST_STOP OFFSET(6) NUMBITS(1) [],
        /// nRST_STDBY User option bytes
        nRST_STDBY OFFSET(7) NUMBITS(1) [],
        /// Read protect
        RDP OFFSET(8) NUMBITS(8) [],
        /// Not write protect
        // NOTE: The length of this bit field varies with the chip model
        nWRP OFFSET(16) NUMBITS(12) []
    ],
    OPTCR1 [
        /// Not write protect
        // NOTE: The length of this bit field varies with the chip model
        nWRP OFFSET(16) NUMBITS(12) []
    ]
];

// All chips models have the same FLASH_BASE
const FLASH_BASE: StaticRef<FlashRegisters> =
    unsafe { StaticRef::new(0x40023C00 as *const FlashRegisters) };

/// Main Flash struct
pub struct Flash<FlashChipSpecific> {
    registers: StaticRef<FlashRegisters>,
    _marker: PhantomData<FlashChipSpecific>,
}

impl<FlashChipSpecific: FlashChipSpecificTrait> Flash<FlashChipSpecific> {
    // Flash constructor. It should be called when creating Stm32f4xxDefaultPeripherals.
    pub(crate) fn new() -> Self {
        Self {
            registers: FLASH_BASE,
            _marker: PhantomData,
        }
    }

    fn read_latency_from_register(&self) -> u32 {
        self.registers.acr.read(ACR::LATENCY)
    }

    // TODO: Take into the account the power supply
    //
    // NOTE: This method is pub(crate) to prevent modifying the flash latency from board files.
    // Flash latency is dependent on the system clock frequency. Other peripherals will modify this
    // when appropriate.
    pub(crate) fn set_latency(&self, sys_clock_frequency: usize) -> Result<(), ErrorCode> {
        let flash_latency =
            FlashChipSpecific::get_number_wait_cycles_based_on_frequency(sys_clock_frequency);
        self.registers
            .acr
            .modify(ACR::LATENCY.val(flash_latency.into()));

        // Wait until the flash latency is set
        // The value 16 was chosen randomly, but it behaves well in tests. It can be tuned in a
        // future revision of the driver.
        for _ in 0..16 {
            if self.get_latency() == flash_latency {
                return Ok(());
            }
        }

        // Return BUSY if setting the frequency took too long. The caller can either:
        //
        // + recall this method
        // + or busy wait get_latency() until the flash latency has the desired value
        Err(ErrorCode::BUSY)
    }

    pub(crate) fn get_latency(&self) -> FlashChipSpecific::FlashLatency {
        FlashChipSpecific::FlashLatency::convert_register_to_enum(self.read_latency_from_register())
    }
}

/// Tests for the STM32F4xx flash driver.
///
/// If any contributions are made to this driver, it is highly recommended to run these tests to
/// ensure that everything still works as expected. The tests are chip agnostic. They can be run
/// on any STM32F4 chips.
///
/// # Usage
///
/// First, the flash module must be imported:
///
/// ```rust,ignore
/// // Change this line depending on the chip the board is using
/// use stm32f429zi::flash;
/// ```
///
/// Then, get a reference to the peripheral:
///
/// ```rust,ignore
/// // Inside the board main.rs
/// let flash = &peripherals.stm32f4.flash;
/// ```
///
/// To run all tests:
///
/// ```rust,ignore
/// flash::tests::run_all(flash);
/// ```
///
/// The following output should be printed:
///
/// ```text
/// ===============================================
/// Testing setting flash latency...
///
/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/// Testing number of wait cycles based on the system frequency...
/// Finished testing number of wait cycles based on the system clock frequency. Everything is alright!
/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
///
///
/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/// Testing setting flash latency...
/// Finished testing setting flash latency. Everything is alright!
/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
///
/// Finished testing flash. Everything is alright!
/// ===============================================
/// ```
///
/// To run individual tests, see the functions in this module.
///
/// # Errors
///
/// In case of any errors, open an issue ticket at <https://github.com/tock/tock>. Please provide
/// the output of the test execution.
pub mod tests {
    use super::*;
    use crate::clocks::hsi::HSI_FREQUENCY_MHZ;

    const AHB_ETHERNET_MINIMUM_FREQUENCY_MHZ: usize = 25;
    // Different chips have different maximum values for APB1
    const APB1_MAX_FREQUENCY_MHZ_1: usize = 42;
    const APB1_MAX_FREQUENCY_MHZ_2: usize = 45;
    const APB1_MAX_FREQUENCY_MHZ_3: usize = 50;
    // Different chips have different maximum values for APB2
    const APB2_MAX_FREQUENCY_MHZ_1: usize = 84;
    #[cfg(not(feature = "stm32f401"))] // Not needed for this chip model
    const APB2_MAX_FREQUENCY_MHZ_2: usize = 90;
    #[cfg(not(feature = "stm32f401"))] // Not needed for this chip model
    const APB2_MAX_FREQUENCY_MHZ_3: usize = 100;
    // Many STM32F4 chips allow a maximum frequency of 168MHz and some of them 180MHz if overdrive
    // is turned on
    #[cfg(not(any(feature = "stm32f401", feature = "stm32f412",)))] // Not needed for these chips
    const SYS_MAX_FREQUENCY_NO_OVERDRIVE_MHZ: usize = 168;
    #[cfg(not(any(feature = "stm32f401", feature = "stm32f412",)))] // Not needed for these chips
    const SYS_MAX_FREQUENCY_OVERDRIVE_MHZ: usize = 180;
    // Default PLL frequency
    #[cfg(not(feature = "stm32f401"))] // Not needed for this chip model
    const PLL_FREQUENCY_MHZ: usize = 96;

    //#[cfg(any(
    //feature = "stm32f401",
    //feature = "stm32f412",
    //feature = "stm32f429",
    //feature = "stm32f446"
    //))]
    /// Test for the mapping between the system clock frequency and flash latency
    ///
    /// It is highly recommended to run this test since everything else depends on it.
    pub fn test_get_number_wait_cycles_based_on_frequency<
        FlashChipSpecific: FlashChipSpecificTrait<FlashLatency = FlashLatency16>,
    >() {
        debug!("");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing number of wait cycles based on the system frequency...");

        assert_eq!(
            FlashChipSpecific::FlashLatency::Latency0,
            FlashChipSpecific::get_number_wait_cycles_based_on_frequency(HSI_FREQUENCY_MHZ)
        );

        assert_eq!(
            FlashChipSpecific::FlashLatency::Latency0,
            FlashChipSpecific::get_number_wait_cycles_based_on_frequency(
                AHB_ETHERNET_MINIMUM_FREQUENCY_MHZ
            )
        );

        assert_eq!(
            FlashChipSpecific::FlashLatency::Latency1,
            FlashChipSpecific::get_number_wait_cycles_based_on_frequency(APB1_MAX_FREQUENCY_MHZ_1)
        );
        assert_eq!(
            FlashChipSpecific::FlashLatency::Latency1,
            FlashChipSpecific::get_number_wait_cycles_based_on_frequency(APB1_MAX_FREQUENCY_MHZ_2)
        );
        assert_eq!(
            FlashChipSpecific::FlashLatency::Latency1,
            FlashChipSpecific::get_number_wait_cycles_based_on_frequency(APB1_MAX_FREQUENCY_MHZ_3)
        );

        assert_eq!(
            FlashChipSpecific::FlashLatency::Latency2,
            FlashChipSpecific::get_number_wait_cycles_based_on_frequency(APB2_MAX_FREQUENCY_MHZ_1)
        );

        // STM32F401 maximum clock frequency is 84MHz
        #[cfg(not(feature = "stm32f401"))]
        {
            assert_eq!(
                FlashChipSpecific::FlashLatency::Latency2,
                FlashChipSpecific::get_number_wait_cycles_based_on_frequency(
                    APB2_MAX_FREQUENCY_MHZ_2
                )
            );

            assert_eq!(
                FlashChipSpecific::FlashLatency::Latency3,
                FlashChipSpecific::get_number_wait_cycles_based_on_frequency(
                    APB2_MAX_FREQUENCY_MHZ_3
                )
            );

            assert_eq!(
                FlashChipSpecific::FlashLatency::Latency3,
                FlashChipSpecific::get_number_wait_cycles_based_on_frequency(PLL_FREQUENCY_MHZ)
            );
        }

        #[cfg(not(any(feature = "stm32f401", feature = "stm32f412",)))]
        // Not needed for these chips
        {
            assert_eq!(
                FlashChipSpecific::FlashLatency::Latency5,
                FlashChipSpecific::get_number_wait_cycles_based_on_frequency(
                    SYS_MAX_FREQUENCY_NO_OVERDRIVE_MHZ
                )
            );

            assert_eq!(
                FlashChipSpecific::FlashLatency::Latency5,
                FlashChipSpecific::get_number_wait_cycles_based_on_frequency(
                    SYS_MAX_FREQUENCY_OVERDRIVE_MHZ
                )
            );
        }

        debug!("Finished testing number of wait cycles based on the system clock frequency. Everything is alright!");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("");
    }

    /// Test for the set_flash() method
    ///
    /// If there is no error, the following output will be printed on the console:
    ///
    /// ```text
    /// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    /// Testing setting flash latency...
    /// Finished testing setting flash latency. Everything is alright!
    /// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    /// ```
    pub fn test_set_flash_latency<
        FlashChipSpecific: FlashChipSpecificTrait<FlashLatency = FlashLatency16>,
    >(
        flash: &Flash<FlashChipSpecific>,
    ) {
        debug!("");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing setting flash latency...");

        assert_eq!(Ok(()), flash.set_latency(HSI_FREQUENCY_MHZ));
        assert_eq!(
            FlashChipSpecific::FlashLatency::Latency0,
            flash.get_latency()
        );

        assert_eq!(
            Ok(()),
            flash.set_latency(AHB_ETHERNET_MINIMUM_FREQUENCY_MHZ)
        );
        assert_eq!(
            FlashChipSpecific::FlashLatency::Latency0,
            flash.get_latency()
        );

        assert_eq!(Ok(()), flash.set_latency(APB1_MAX_FREQUENCY_MHZ_1));
        assert_eq!(
            FlashChipSpecific::FlashLatency::Latency1,
            flash.get_latency()
        );

        assert_eq!(Ok(()), flash.set_latency(APB1_MAX_FREQUENCY_MHZ_2));
        assert_eq!(
            FlashChipSpecific::FlashLatency::Latency1,
            flash.get_latency()
        );

        assert_eq!(Ok(()), flash.set_latency(APB1_MAX_FREQUENCY_MHZ_3));
        assert_eq!(
            FlashChipSpecific::FlashLatency::Latency1,
            flash.get_latency()
        );

        assert_eq!(Ok(()), flash.set_latency(APB2_MAX_FREQUENCY_MHZ_1));

        // STM32F401 maximum system clock frequency is 84MHz
        #[cfg(not(feature = "stm32f401"))]
        {
            assert_eq!(Ok(()), flash.set_latency(APB2_MAX_FREQUENCY_MHZ_2));

            assert_eq!(Ok(()), flash.set_latency(APB2_MAX_FREQUENCY_MHZ_3));
            assert_eq!(
                FlashChipSpecific::FlashLatency::Latency3,
                flash.get_latency()
            );

            assert_eq!(Ok(()), flash.set_latency(PLL_FREQUENCY_MHZ));
            assert_eq!(
                FlashChipSpecific::FlashLatency::Latency3,
                flash.get_latency()
            );
        }

        // Low entries STM32F4 chips don't support frequencies higher than 100 MHz,
        // but the foundation and advanced ones support system clock frequencies up to
        // 180MHz
        #[cfg(not(any(feature = "stm32f401", feature = "stm32f412",)))]
        {
            assert_eq!(
                Ok(()),
                flash.set_latency(SYS_MAX_FREQUENCY_NO_OVERDRIVE_MHZ)
            );
            assert_eq!(
                FlashChipSpecific::FlashLatency::Latency5,
                flash.get_latency()
            );

            assert_eq!(Ok(()), flash.set_latency(SYS_MAX_FREQUENCY_OVERDRIVE_MHZ));
            assert_eq!(
                FlashChipSpecific::FlashLatency::Latency5,
                flash.get_latency()
            );
        }

        // Revert to default settings
        assert_eq!(Ok(()), flash.set_latency(HSI_FREQUENCY_MHZ));
        assert_eq!(
            FlashChipSpecific::FlashLatency::Latency0,
            flash.get_latency()
        );

        debug!("Finished testing setting flash latency. Everything is alright!");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("");
    }

    /// Run the entire test suite
    pub fn run_all<FlashChipSpecific: FlashChipSpecificTrait<FlashLatency = FlashLatency16>>(
        flash: &Flash<FlashChipSpecific>,
    ) {
        debug!("");
        debug!("===============================================");
        debug!("Testing setting flash latency...");

        test_get_number_wait_cycles_based_on_frequency::<FlashChipSpecific>();
        test_set_flash_latency::<FlashChipSpecific>(flash);

        debug!("Finished testing flash. Everything is alright!");
        debug!("===============================================");
        debug!("");
    }
}
