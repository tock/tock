// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Standby SRAM Controller (SSRAMC) driver for NXP S32G3.
//!
//! The S32G3 has 32 KB of Standby SRAM at `0x2400_0000..0x2400_7FFF`.
//! Accessing this memory before its ECC bits are initialized triggers a fatal
//! multi-bit ECC error (Synchronous External Abort). This module triggers the
//! SSRAMC hardware initialization sequence to clear the entire block and write
//! valid ECC bits.
//!
//! Reference: S32G3 RM §36.
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

// ---------------------------------------------------------------------------
// Register Definitions
// ---------------------------------------------------------------------------

/// Base address of the SSRAMC configuration registers (RM §36.3).
pub const SSRAMC_BASE: StaticRef<SsramcRegisters> =
    unsafe { StaticRef::new(0x4402_8000 as *const SsramcRegisters) };

// RM §36.3.1 — SSRAMC register map.
register_structs! {
    pub SsramcRegisters {
        /// Platform RAM Control Register — requests initialization and sets
        /// wait cycles (RM §36.3.2).
        (0x00 => pub pramcr: ReadWrite<u32, PRAMCR::Register>),
        /// Platform RAM Initialization Address Register Start — lower bound of
        /// the local memory range to initialize (RM §36.3.3).
        (0x04 => pub pramias: ReadWrite<u32, PRAMIAS::Register>),
        /// Platform RAM Initialization Address Register End — upper bound of
        /// the local memory range to initialize (RM §36.3.4).
        (0x08 => pub pramiae: ReadWrite<u32, PRAMIAE::Register>),
        /// Platform RAM Status Register — reports initialization progress and
        /// ECC errors (RM §36.3.5).
        (0x0C => pub pramsr: ReadWrite<u32, PRAMSR::Register>),
        /// Platform RAM ECC Address — address associated with an ECC error
        /// (RM §36.3.6).
        (0x10 => pub pramecca: ReadOnly<u32, PRAMECCA::Register>),
        (0x14 => @END),
    }
}

register_bitfields![u32,
    /// Platform RAM Control Register (PRAMCR)
    PRAMCR [
        /// Initialization Wait Cycles
        IWS OFFSET(1) NUMBITS(2) [
            NoWait = 0,
            OneWait = 1,
            TwoWaits = 2,
            ThreeWaits = 3
        ],
        /// Initialization Request
        INITREQ OFFSET(0) NUMBITS(1) [
            NoRequest = 0,
            Request = 1
        ]
    ],

    /// Platform RAM Initialization Address Register Start (PRAMIAS)
    PRAMIAS [
        /// Initialization Start Address
        IAS OFFSET(0) NUMBITS(17) []
    ],

    /// Platform RAM Initialization Address Register End (PRAMIAE)
    PRAMIAE [
        /// Initialization End Address
        IAE OFFSET(0) NUMBITS(17) []
    ],

    PRAMSR [
        /// ECC Syndrome Value
        SYND OFFSET(8) NUMBITS(8) [],
        /// ECC Single-bit Error
        SGLERR OFFSET(7) NUMBITS(1) [],
        /// ECC Multi-bit Error
        MLTERR OFFSET(6) NUMBITS(1) [],
        /// ECC Address Error
        AERR OFFSET(5) NUMBITS(1) [],
        /// Address Source (local vs external)
        AEXT OFFSET(4) NUMBITS(1) [
            Local = 0,
            External = 1
        ],
        /// Initialization Progress Status
        IPEND OFFSET(2) NUMBITS(1) [
            NotInProgress = 0,
            InProgress = 1
        ],
        /// Initialization Error
        IERR OFFSET(1) NUMBITS(1) [],
        /// Initialization Done
        IDONE OFFSET(0) NUMBITS(1) []
    ],

    /// Platform RAM ECC Address Register (PRAMECCA)
    PRAMECCA [
        /// Controller ID of the reported error
        CTRLID OFFSET(21) NUMBITS(4) [],
        /// RAM bank with the ECC error
        EBNK OFFSET(20) NUMBITS(1) [
            Bank0 = 0,
            Bank1 = 1
        ],
        /// ECC Error Address
        EADR OFFSET(0) NUMBITS(17) []
    ]
];

// ---------------------------------------------------------------------------
// Busy-Wait Ceiling
// ---------------------------------------------------------------------------

/// Maximum iterations for the SSRAMC initialization spin-wait.
///
/// Units: bare loop iterations (register read + compare + branch).
/// At 48 MHz FIRC (~10 cycles/MMIO read) this caps the wait at ≈5 ms —
/// well above the hardware's typical sub-microsecond initialization time.
/// Callers MUST propagate an error on expiry, not silently continue.
const HW_POLL_MAX: u32 = 24_000;

// ---------------------------------------------------------------------------
// Driver
// ---------------------------------------------------------------------------

/// Standby SRAM Controller (SSRAMC) driver.
pub struct Ssramc {
    registers: StaticRef<SsramcRegisters>,
}

impl Ssramc {
    /// Create a new `Ssramc` instance.
    pub const fn new(registers: StaticRef<SsramcRegisters>) -> Self {
        Self { registers }
    }

    /// Initialize Standby SRAM ECC via the SSRAMC hardware block.
    ///
    /// This routine:
    /// 1. Clears any error status flags in PRAMSR.
    /// 2. Disables the controller.
    /// 3. Sets the start/end range offsets for the full 32 KB block.
    /// 4. Triggers hardware initialization.
    /// 5. Busy-waits until the `IDONE` bit is set.
    ///
    /// # INIT-ONLY
    /// Spin-waits up to `HW_POLL_MAX` iterations (WCET ≈ 5 ms at 48 MHz FIRC).
    /// **Must only be called during board initialisation, before `kernel_loop()`.**
    /// Runtime re-initialization is prohibited — see safety manual §SSRAMC-INIT.
    ///
    /// # Errors
    /// - [`ErrorCode::BUSY`]: initialization did not complete within `HW_POLL_MAX`
    pub fn init(&self) -> Result<(), ErrorCode> {
        // Step 1: Clear all status flags (write-1-to-clear).
        self.registers.pramsr.write(
            PRAMSR::SGLERR::SET
                + PRAMSR::MLTERR::SET
                + PRAMSR::AERR::SET
                + PRAMSR::IERR::SET
                + PRAMSR::IDONE::SET,
        );

        // Step 2: Disable controller before setting ranges.
        self.registers.pramcr.write(PRAMCR::INITREQ::NoRequest);

        // Step 3: 32 KB block — local memory offsets 0x0 to 0x7FF.
        // Per RM §36.1.6 and the platform reference (see docs/tock/releasing_a53.md),
        // the end offset for the full 32 KB Standby SRAM is 0x7FF.
        self.registers.pramias.write(PRAMIAS::IAS.val(0));
        // 32 KB ÷ 64 B/line − 1 = 0x7FF (RM §36.3.4).
        self.registers.pramiae.write(PRAMIAE::IAE.val(0x7FF));

        // Step 4: Trigger initialization.
        self.registers.pramcr.write(PRAMCR::INITREQ::Request);

        // Step 5: Poll for completion.
        for _ in 0..HW_POLL_MAX {
            if self.registers.pramsr.is_set(PRAMSR::IDONE) {
                // Clear IDONE flag.
                self.registers.pramsr.write(PRAMSR::IDONE::SET);
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }
}
