// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Author: Kamil Duljas <kamil.duljas@gmail.com>

//! STM32L4xx flash driver
//!
//! This driver provides basic functionalities for the entire STM32L4 series.
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
//! let flash = &peripherals.stm32l4.flash;
//! ```
//!
//! ## Retrieve the current flash latency
//!
//! ```rust,ignore
//! let flash_latency = flash.get_latency() as usize;
//! debug!("Current flash latency is {}", flash_latency);
//! ```

use crate::chip_specific::flash::FlashChipSpecific as FlashChipSpecificTrait;
use crate::chip_specific::flash::RegisterToFlashLatency;
use crate::pwr::Pwr;

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
    pdkeyr: WriteOnly<u32>,
    /// Flash option key register
    keyr: WriteOnly<u32>,
    /// Status register
    sr: ReadWrite<u32, SR::Register>,
    /// Control register
    cr: ReadWrite<u32, CR::Register>,
    /// Flash option control register
    optcr: ReadWrite<u32, OPTCR::Register>,
}

register_bitfields![u32,
    // FLASH access control register (ACR)
    ACR [
        /// Latency (number of wait states). STM32L4x6: 3-bit field.
        LATENCY OFFSET(0) NUMBITS(3) [
            WS0 = 0,
            WS1 = 1,
            WS2 = 2,
            WS3 = 3,
            WS4 = 4,
            WS5 = 5,
            WS6 = 6,
            WS7 = 7
        ],
        /// Prefetch enable
        PRFTEN OFFSET(8) NUMBITS(1) [],
        /// Instruction cache enable
        ICEN OFFSET(9) NUMBITS(1) [],
        /// Data cache enable
        DCEN OFFSET(10) NUMBITS(1) [],
        /// Instruction cache reset
        ICRST OFFSET(11) NUMBITS(1) [],
        /// Data cache reset
        DCRST OFFSET(12) NUMBITS(1) [],
        /// Flash power-down mode during run
        RUN_PD OFFSET(13) NUMBITS(1) [],
        /// Flash power-down mode during sleep
        SLEEP_PD OFFSET(14) NUMBITS(1) []
    ],

    // FLASH status register (SR)
    SR [
        /// End of operation
        EOP OFFSET(0) NUMBITS(1) [],
        /// Operation error
        OPERR OFFSET(1) NUMBITS(1) [],
        /// Programming error
        PROGERR OFFSET(3) NUMBITS(1) [],
        /// Write protection error
        WRPERR OFFSET(4) NUMBITS(1) [],
        /// Programming alignment error
        PGAERR OFFSET(5) NUMBITS(1) [],
        /// Size error
        SIZERR OFFSET(6) NUMBITS(1) [],
        /// Programming sequence error
        PGSERR OFFSET(7) NUMBITS(1) [],
        /// Fast programming data miss error
        MISERR OFFSET(8) NUMBITS(1) [],
        /// Fast programming error
        FASTERR OFFSET(9) NUMBITS(1) [],
        /// PCROP read error
        RDERR OFFSET(14) NUMBITS(1) [],
        /// Option validity error
        OPTVERR OFFSET(15) NUMBITS(1) [],
        /// Busy
        BSY OFFSET(16) NUMBITS(1) []
    ],

    // FLASH control register (CR)
    CR [
        /// Programming
        PG OFFSET(0) NUMBITS(1) [],
        /// Page erase
        PER OFFSET(1) NUMBITS(1) [],
        /// Mass erase bank 1
        MER1 OFFSET(2) NUMBITS(1) [],
        /// Page number (for page erase)
        PNB OFFSET(3) NUMBITS(8) [],
        /// Bank selection for page erase (0: Bank1, 1: Bank2)
        BKER OFFSET(11) NUMBITS(1) [],
        /// Mass erase bank 2
        MER2 OFFSET(15) NUMBITS(1) [],
        /// Start erase operation
        START OFFSET(16) NUMBITS(1) [],
        /// Options modification start
        OPTSTRT OFFSET(17) NUMBITS(1) [],
        /// Fast programming
        FSTPG OFFSET(18) NUMBITS(1) [],
        /// End of operation interrupt enable
        EOPIE OFFSET(24) NUMBITS(1) [],
        /// Error interrupt enable
        ERRIE OFFSET(25) NUMBITS(1) [],
        /// PCROP read error interrupt enable
        RDERRIE OFFSET(26) NUMBITS(1) [],
        /// Force option byte loading
        OBL_LAUNCH OFFSET(27) NUMBITS(1) [],
        /// Option bytes lock
        OPTLOCK OFFSET(30) NUMBITS(1) [],
        /// FLASH control register lock
        LOCK OFFSET(31) NUMBITS(1) []
    ],

    // FLASH option register (OPTR) — named OPTCR here to keep code stable
    OPTCR [
        /// Read protection level
        RDP OFFSET(0) NUMBITS(8) [],
        /// BOR reset level
        BOR_LEV OFFSET(8) NUMBITS(2) [],
        /// Reset generated when entering Stop mode (active low)
        nRST_STOP OFFSET(12) NUMBITS(1) [],
        /// Reset generated when entering Standby mode (active low)
        nRST_STDBY OFFSET(13) NUMBITS(1) [],
        /// Reset generated when entering Shutdown mode (active low)
        nRST_SHDW OFFSET(14) NUMBITS(1) [],
        /// Independent watchdog selection
        IWDG_SW OFFSET(16) NUMBITS(1) [],
        /// Independent watchdog counter freeze in Stop mode
        IWDG_STOP OFFSET(17) NUMBITS(1) [],
        /// Independent watchdog counter freeze in Standby mode
        IWDG_STDBY OFFSET(18) NUMBITS(1) [],
        /// Window watchdog selection
        WWDG_SW OFFSET(19) NUMBITS(1) [],
        /// Boot from Bank 2 (if dual-bank supported)
        BFB2 OFFSET(20) NUMBITS(1) [],
        /// Dual-bank configuration (if available)
        DUALBANK OFFSET(21) NUMBITS(1) []
    ],
];

// STM32L4xx FLASH base address
const FLASH_BASE: StaticRef<FlashRegisters> =
    unsafe { StaticRef::new(0x40022000 as *const FlashRegisters) };

/// Main Flash struct
pub struct Flash<FlashChipSpecific> {
    registers: StaticRef<FlashRegisters>,
    pwr: Pwr,
    _marker: PhantomData<FlashChipSpecific>,
}

impl<FlashChipSpecific: FlashChipSpecificTrait> Flash<FlashChipSpecific> {
    // Flash constructor. It should be called when creating Stm32l4xxDefaultPeripherals.
    pub(crate) fn new(pwr: Pwr) -> Self {
        Self {
            registers: FLASH_BASE,
            pwr,
            _marker: PhantomData,
        }
    }

    fn read_latency_from_register(&self) -> u32 {
        self.registers.acr.read(ACR::LATENCY)
    }

    // NOTE: This method is pub(crate) to prevent modifying the flash latency from board files.
    // Flash latency is dependent on the system clock frequency. Other peripherals will modify this
    // when appropriate.
    pub(crate) fn set_latency(&self, sys_clock_frequency: usize) -> Result<(), ErrorCode> {
        if !self.pwr.is_vos_ready() {
            return Err(ErrorCode::BUSY);
        }

        let flash_latency =
            FlashChipSpecific::get_number_wait_cycles_based_on_frequency_and_voltage(
                sys_clock_frequency,
                self.pwr.get_vos() as usize,
            );

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
