// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! FXOSC (Fast External Crystal Oscillator) driver for NXP S32G3.
//!
//! The FXOSC accepts a 20–40 MHz crystal and provides FXOSC_CLK used as:
//!
//! - Reference for all PLLs (after boot switches from FIRC)
//! - Reference for FlexCAN, FlexRay
//! - CLKOUT source
//!
//! See RM §24.2.6 and the "FXOSC" chapter for register details.
//!
//! # Input Clock Modes
//!
//! - Crystal mode (default)
//! - Single-ended bypass
//! - Differential bypass
//!
//! # Usage
//!
//! ```rust,ignore
//! use nxp_s32g3::clocks::fxosc::Fxosc;
//!
//! let fxosc = Fxosc::new();
//! fxosc.set_frequency_mhz(40);
//! fxosc.enable_crystal().unwrap();
//! ```

use core::cell::Cell;

use kernel::platform::chip::ClockInterface;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

/// FXOSC base address (RM FXOSC chapter, register map).
const FXOSC_BASE_ADDR: u32 = 0x4005_0000;

/// FXOSC register base reference.
pub const FXOSC_BASE: StaticRef<FxoscRegisters> =
    unsafe { StaticRef::new(FXOSC_BASE_ADDR as *const FxoscRegisters) };

// Units: bare loop iterations (register read + compare + branch).
// At 48 MHz FIRC (~10 cycles/MMIO read) this caps the wait at ≈50 ms —
// crystal startup is typically 2–5 ms; 50 ms covers worst-case load
// capacitance and cold start (RM FXOSC chapter).
// Callers MUST propagate an error on expiry, not silently continue.
const HW_POLL_MAX: u32 = 240_000;

register_structs! {
    pub FxoscRegisters {
        /// FXOSC Control Register
        (0x000 => pub ctrl: ReadWrite<u32, FXOSC_CTRL::Register>),
        /// FXOSC Status Register (read-only; written for transient observation only)
        (0x004 => pub stat: ReadOnly<u32, FXOSC_STAT::Register>),
        (0x008 => @END),
    }
}

register_bitfields![u32,
    /// FXOSC Control Register (RM §28.2.2). Reset: 0x019D_00C0.
    FXOSC_CTRL [
        /// Oscillator Bypass — 1 = bypass internal osc, use EXTAL (bit 31).
        OSC_BYP OFFSET(31) NUMBITS(1) [],
        /// Comparator Enable — REQUIRED for Crystal mode (bit 24). Reset = 1.
        COMP_EN OFFSET(24) NUMBITS(1) [],
        /// End-of-Count Value (bits 23:16). Stab time = EOCV * 128 * 4 / f_xtal.
        EOCV OFFSET(16) NUMBITS(8) [],
        /// Crystal Overdrive Protection — transconductance level (bits 7:4).
        /// 4-bit field; see RM Table for code→transconductance mapping. Reset = 0xC.
        GM_SEL OFFSET(4) NUMBITS(4) [],
        /// Automatic Level Controller Disable (bit 2). 0 = ALC enabled. Reset = 0.
        ALC_D OFFSET(2) NUMBITS(1) [],
        /// Oscillator Power-Down Control (bit 0). 1 = enabled, 0 = disabled. Reset = 0.
        OSCON OFFSET(0) NUMBITS(1) []
    ],
    FXOSC_STAT [
        /// Oscillator Status — 1 = on and providing a stable clock (bit 31).
        OSC_STAT OFFSET(31) NUMBITS(1) []
    ]
];

/// FXOSC input mode selection.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FxoscMode {
    /// Crystal mode (default)
    Crystal,
    /// Single-ended bypass (external clock input)
    Bypass,
}

/// Fast External Crystal Oscillator driver.
pub struct Fxosc {
    registers: StaticRef<FxoscRegisters>,
    frequency_hz: OptionalCell<u32>,
    enabled: Cell<bool>,
}

impl Fxosc {
    /// Create a new FXOSC instance.
    pub const fn new() -> Self {
        Self {
            registers: FXOSC_BASE,
            frequency_hz: OptionalCell::empty(),
            enabled: Cell::new(false),
        }
    }

    /// Set the crystal/input frequency in MHz.
    ///
    /// Must be called before `enable()`.
    pub fn set_frequency_mhz(&self, freq_mhz: u32) {
        self.frequency_hz.set(freq_mhz * 1_000_000);
    }

    /// Get the FXOSC frequency in Hz, if enabled.
    pub fn get_frequency_hz(&self) -> Option<u32> {
        if self.enabled.get() {
            self.frequency_hz.get()
        } else {
            None
        }
    }

    /// Get the FXOSC frequency in MHz, if enabled.
    pub fn get_frequency_mhz(&self) -> Option<usize> {
        self.get_frequency_hz().map(|f| (f / 1_000_000) as usize)
    }

    /// Enable the FXOSC in the specified mode, following RM §28.5 init sequence.
    ///
    /// # INIT-ONLY
    /// Spin-waits up to `HW_POLL_MAX` iterations (WCET ≈ 50 ms at 48 MHz FIRC).
    /// **Must only be called during board initialisation, before `kernel_loop()`.**
    /// Runtime FXOSC reconfiguration is prohibited — see safety manual §CLOCK-INIT.
    ///
    /// Performs disable→reconfigure→enable as the RM mandates ("FXOSC must be
    /// disabled when the operation mode is modified").
    ///
    /// # Errors
    /// - [`ErrorCode::INVAL`]: frequency not configured
    /// - [`ErrorCode::BUSY`]: oscillator did not stabilize within `HW_POLL_MAX` iterations
    pub fn enable_mode(&self, mode: FxoscMode) -> Result<(), ErrorCode> {
        if self.frequency_hz.get().is_none() {
            return Err(ErrorCode::INVAL);
        }

        let regs = &*self.registers;

        // Short-circuit: if FXOSC is already running (e.g. left enabled by a
        // prior boot stage), preserve its configuration and return.
        if regs.stat.is_set(FXOSC_STAT::OSC_STAT) {
            self.enabled.set(true);
            return Ok(());
        }

        // ----- Step 0: disable FXOSC before touching operation mode --------
        // RM §28.5 NOTE: "FXOSC must be disabled when the operation mode is
        // modified."
        regs.ctrl.modify(FXOSC_CTRL::OSCON::CLEAR);

        // ----- Step 1: select operation mode (OSC_BYP + COMP_EN) ----------
        match mode {
            FxoscMode::Crystal => {
                regs.ctrl.write(
                    FXOSC_CTRL::OSC_BYP::CLEAR
                        + FXOSC_CTRL::COMP_EN::SET
                        + FXOSC_CTRL::EOCV.val(0x80)
                        + FXOSC_CTRL::GM_SEL.val(0xC)
                        + FXOSC_CTRL::ALC_D::CLEAR
                        + FXOSC_CTRL::OSCON::CLEAR,
                );
            }
            FxoscMode::Bypass => {
                regs.ctrl.write(
                    FXOSC_CTRL::OSC_BYP::SET
                        + FXOSC_CTRL::COMP_EN::CLEAR
                        + FXOSC_CTRL::EOCV.val(0x80)
                        + FXOSC_CTRL::GM_SEL.val(0)
                        + FXOSC_CTRL::ALC_D::CLEAR
                        + FXOSC_CTRL::OSCON::CLEAR,
                );
            }
        }

        // ----- Step 2: enable FXOSC (OSCON=1) ------------------------------
        regs.ctrl.modify(FXOSC_CTRL::OSCON::SET);

        // ----- Step 3: wait for OSC_STAT to assert -------------------------
        for _ in 0..HW_POLL_MAX {
            if regs.stat.is_set(FXOSC_STAT::OSC_STAT) {
                self.enabled.set(true);
                return Ok(());
            }
        }
        Err(ErrorCode::BUSY)
    }

    /// Enable the FXOSC in crystal mode (default).
    pub fn enable_crystal(&self) -> Result<(), ErrorCode> {
        self.enable_mode(FxoscMode::Crystal)
    }

    /// Disable the FXOSC.
    ///
    /// # INIT-ONLY
    /// Spin-waits up to `HW_POLL_MAX` iterations (WCET ≈ 50 ms at 48 MHz FIRC).
    /// **Must only be called during board initialisation, before `kernel_loop()`.**
    /// Runtime FXOSC reconfiguration is prohibited — see safety manual §CLOCK-INIT.
    ///
    /// # Errors
    ///
    /// - [`ErrorCode::BUSY`]: oscillator did not turn off in time
    pub fn disable_osc(&self) -> Result<(), ErrorCode> {
        let regs = &*self.registers;
        regs.ctrl.modify(FXOSC_CTRL::OSCON::CLEAR);

        for _ in 0..HW_POLL_MAX {
            if !regs.stat.is_set(FXOSC_STAT::OSC_STAT) {
                self.enabled.set(false);
                return Ok(());
            }
        }
        Err(ErrorCode::BUSY)
    }
}

impl ClockInterface for Fxosc {
    fn is_enabled(&self) -> bool {
        self.enabled.get()
    }

    fn enable(&self) {
        // Default: crystal mode. Ignore errors in the trait interface.
        let _ = self.enable_crystal();
    }

    fn disable(&self) {
        let _ = self.disable_osc();
    }
}
