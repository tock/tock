// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! PLL (Phase-Locked Loop) driver for NXP S32G3.
//!
//! The S32G3 contains five PLLs (RM §24.2.7):
//!
//! - **CORE_PLL** — FM (SSCG), drives A53 clusters, M7, HSE_H, interconnect
//! - **PERIPH_PLL** — Non-FM, drives peripherals (GMAC, FlexCAN, FlexRay, LINFlexD, SPI, …)
//! - **DDR_PLL** — FM, drives DRAM interface
//! - **ACCEL_PLL** — FM, drives hardware accelerators
//!
//! Each PLL accepts FIRC (48 MHz) or FXOSC (20–40 MHz) as reference and
//! produces a VCO output:
//!
//! ```text
//! f_VCO = f_ref / RDIV × (MFI + MFN / 18432)
//! f_PHIn = f_VCO / (ODIVn + 1)
//!
//! The PLLs expose multiple PHI outputs (divided from VCO) that feed into
//! MC_CGM muxes.
//!
//! # PLL Turn-On Sequence (RM §24.5.3)
//!
//! 1. CORE_PLL → 2. PERIPH_PLL → 3. DDR_PLL → 4. ACCEL_PLL
//!
//! # Register Interface
//!
//! See RM "PLL digital interface (PLLDIG)" chapter for register definitions.
//! Each PLL instance has the same register layout at a different base address.

use core::cell::Cell;

use kernel::platform::chip::ClockInterface;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

// ---------------------------------------------------------------------------
// PLL Base Addresses (S32G3 RM §27.2.1, §27.3.1, §27.4.1, §27.5.1)
// ---------------------------------------------------------------------------

/// CORE_PLL (a.k.a. ARM_PLL) base — RM §27.3.1.
pub const CORE_PLL_BASE_ADDR: u32 = 0x4003_8000;
/// PERIPH_PLL base — RM §27.5.1.
pub const PERIPH_PLL_BASE_ADDR: u32 = 0x4003_C000;
/// ACCEL_PLL base — RM §27.2.1.
pub const ACCEL_PLL_BASE_ADDR: u32 = 0x4004_0000;
/// DDR_PLL base — RM §27.4.1.
pub const DDR_PLL_BASE_ADDR: u32 = 0x4004_4000;

/// Maximum number of PHI output dividers per PLL.
pub const MAX_PHI_OUTPUTS: usize = 8;

// Units: bare loop iterations (register read + compare + branch).
// At 48 MHz FIRC (~10 cycles/MMIO read) this caps the wait at ≈5 ms —
// per RM §24.5.3 PLL lock time is typically < 100 µs; 5 ms is a
// generous bound for worst-case silicon and voltage/temperature.
// Callers MUST propagate an error on expiry, not silently continue.
const HW_POLL_MAX: u32 = 24_000;
/// Fractional PLL denominator used by NXP's S32CC clock driver.
///
/// The field is 15 bits wide, but the S32CC PLL FRAC-N formula scales MFN by
/// 18432, not by the full 2^15 range.
const PLL_MFN_DENOMINATOR: u64 = 18_432;

// ---------------------------------------------------------------------------
// Register Definitions (RM "PLL digital interface (PLLDIG)")
// ---------------------------------------------------------------------------

register_structs! {
    /// PLLDIG register block.
    pub PllRegisters {
        /// PLL Control Register (PLLCR)
        /// RM §27.2.2.
        (0x000 => pub pllcr: ReadWrite<u32, PLLCR::Register>),
        /// PLL Status Register (PLLSR)
        /// RM §27.2.3.
        (0x004 => pub pllsr: ReadOnly<u32, PLLSR::Register>),
        /// PLL Divider Register (PLLDV)
        /// RM §27.2.4.
        (0x008 => pub plldv: ReadWrite<u32, PLLDV::Register>),
        /// PLL Frequency Modulation Register (PLLFM)
        /// RM §27.2.5.
        (0x00C => pub pllfm: ReadWrite<u32, PLLFM::Register>),
        /// PLL Fractional Divider Register (PLLFD)
        /// RM §27.2.6.
        (0x010 => pub pllfd: ReadWrite<u32, PLLFD::Register>),
        (0x014 => _reserved0),
        /// PLL Calibration Register 1 (PLLCAL1)
        (0x018 => pub pllcal1: ReadWrite<u32>),
        (0x01C => _reserved1),
        /// PLL Clock Mux Register (PLLCLKMUX) — selects reference clock.
        /// RM §27.2.7.
        (0x020 => pub pllclkmux: ReadWrite<u32, PLLCLKMUX::Register>),
        (0x024 => _reserved2),
        /// PLL Output Divider registers (PLLODIV0..7)
        /// RM §27.2.8.
        (0x080 => pub pllodiv: [ReadWrite<u32, PLLODIV::Register>; MAX_PHI_OUTPUTS]),
        (0x0A0 => @END),
    }
}

register_bitfields![u32,
    // RM §27.2.2
    /// PLL Control Register
    PLLCR [
        /// Power Down: 1 = PLL powered down
        PLLPD OFFSET(31) NUMBITS(1) []
    ],

    // RM §27.2.3
    /// PLL Status Register
    PLLSR [
        /// Lock status: 1 = PLL is locked
        LOCK OFFSET(2) NUMBITS(1) []
    ],

    // RM §27.2.4
    /// PLL Divider Register
    PLLDV [
        /// Reference Division Factor (1..7)
        RDIV OFFSET(12) NUMBITS(3) [],
        /// Multiplication Factor Integer part (16..255)
        MFI  OFFSET(0)  NUMBITS(8) []
    ],
    // RM §27.2.5
    /// PLL Frequency Modulation Register
    PLLFM [
        /// Spread-Spectrum Clock Generation Bypass: 0 = SSCG enabled
        SSCGBYP OFFSET(30) NUMBITS(1) [],
        /// Spread Control: 0 = center-spread
        SPREADCTL OFFSET(29) NUMBITS(1) [],
        /// Modulation Period (step count)
        STEPNO OFFSET(16) NUMBITS(8) [],
        /// Modulation Depth
        STEPSIZE OFFSET(0) NUMBITS(10) []
    ],

    // RM §27.2.6
    /// PLL Fractional Divider Register
    PLLFD [
        /// Sigma-Delta Modulation Enable
        SDMEN OFFSET(30) NUMBITS(1) [],
        /// Multiplication Factor Numerator (fractional part, 15 bits)
        MFN   OFFSET(0)  NUMBITS(15) []
    ],
    // RM §27.2.8
    /// PLL Output Divider Register (per PHI output)
    PLLODIV [
        /// Divider Enable
        DE   OFFSET(31) NUMBITS(1) [],
        /// Divider Value (actual division = DIV + 1; valid: 0..255)
        DIV  OFFSET(16) NUMBITS(8) []
    ],

    // RM §27.2.7
    /// PLL Clock Mux Register — selects the reference clock source
    PLLCLKMUX [
        /// Reference Clock Select: 0 = FIRC (48 MHz), 1 = FXOSC
        REFCLKSEL OFFSET(0) NUMBITS(1) [
            /// FIRC (48 MHz, default after reset)
            Firc = 0,
            /// FXOSC (external crystal, 20–40 MHz)
            Fxosc = 1
        ]
    ]
];

// ---------------------------------------------------------------------------
// PLL Instance Identifier
// ---------------------------------------------------------------------------

/// Identifies which PLL instance this driver controls.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PllInstance {
    Core,
    Periph,
    Ddr,
    Accel,
}

impl PllInstance {
    /// Number of PHI outputs for this PLL instance.
    pub fn num_phi_outputs(self) -> usize {
        match self {
            PllInstance::Core => 2,   // PHI0, PHI1
            PllInstance::Periph => 8, // PHI0..PHI7
            PllInstance::Ddr => 1,    // PHI0
            PllInstance::Accel => 2,  // PHI0, PHI1
        }
    }

    /// Whether frequency modulation (SSCG) is supported.
    pub fn supports_fm(self) -> bool {
        matches!(
            self,
            PllInstance::Core | PllInstance::Ddr | PllInstance::Accel
        )
    }

    /// Short uppercase name for log messages.
    pub fn name(self) -> &'static str {
        match self {
            PllInstance::Core => "CORE_PLL",
            PllInstance::Periph => "PERIPH_PLL",
            PllInstance::Ddr => "DDR_PLL",
            PllInstance::Accel => "ACCEL_PLL",
        }
    }
}

// ---------------------------------------------------------------------------
// PLL Configuration
// ---------------------------------------------------------------------------

/// Configuration parameters for a PLL.
///
/// VCO frequency: `f_vco = f_ref / rdiv * (mfi + mfn / 18432)`
#[derive(Copy, Clone, Debug)]
pub struct PllConfig {
    /// Reference clock divider (RDIV). Valid range: 1..7 (3-bit field).
    pub rdiv: u8,
    /// Multiplication factor integer (MFI). Valid range: 16..255 (8-bit field).
    pub mfi: u16,
    /// Multiplication factor numerator (fractional part). Valid values are
    /// less than 18432 per RM `PLLFD[MFN]` and the S32CC clock driver.
    pub mfn: u32,
}
impl PllConfig {
    /// Create a new PLL configuration.
    pub const fn new(rdiv: u8, mfi: u16, mfn: u32) -> Self {
        Self { rdiv, mfi, mfn }
    }
    /// Create a PLL configuration from frequency parameters.
    ///
    /// Solves `f_vco = f_ref / rdiv * (mfi + mfn / 18432)` using checked
    /// integer arithmetic and rejects values that cannot fit the hardware.
    pub fn from_frequencies(f_ref: u32, f_vco: u32, rdiv: u8) -> Result<Self, ErrorCode> {
        if f_ref == 0 || !(1..=7).contains(&rdiv) {
            return Err(ErrorCode::INVAL);
        }
        let ratio = u64::from(f_vco)
            .checked_mul(u64::from(rdiv))
            .and_then(|value| value.checked_mul(PLL_MFN_DENOMINATOR))
            .ok_or(ErrorCode::INVAL)?
            / u64::from(f_ref);
        let mfi = ratio / PLL_MFN_DENOMINATOR;
        let mfn = ratio % PLL_MFN_DENOMINATOR;
        if !(16..=255).contains(&mfi) || mfn >= PLL_MFN_DENOMINATOR {
            return Err(ErrorCode::INVAL);
        }
        Ok(Self {
            rdiv,
            mfi: u16::try_from(mfi).map_err(|_| ErrorCode::INVAL)?,
            mfn: u32::try_from(mfn).map_err(|_| ErrorCode::INVAL)?,
        })
    }
}
/// Configuration for a single PHI output divider.
///
/// Output frequency: `f_phi = f_vco / (div + 1)`.
#[derive(Copy, Clone, Debug)]
pub struct PhiConfig {
    /// PHI output index (0-based).
    pub index: usize,
    /// Divider value. Actual division is `div + 1`. Range: 0..255.
    pub div: u8,
    /// Whether this output is enabled.
    pub enabled: bool,
}

/// Decoded live PLL state read from hardware registers.
#[derive(Copy, Clone, Debug)]
pub struct PllSnapshot {
    pub pllclkmux: u32,
    pub plldv: u32,
    pub pllfd: u32,
    pub rdiv: u32,
    pub mfi: u32,
    pub mfn: u32,
    pub fractional: bool,
    pub ref_clock_hz: u32,
    pub vco_hz: u32,
}

// ---------------------------------------------------------------------------
// PLL Driver
// ---------------------------------------------------------------------------

/// PLL driver for a single S32G3 PLL instance.
pub struct Pll {
    registers: StaticRef<PllRegisters>,
    instance: PllInstance,
    /// Cached VCO frequency in Hz (set after configure + enable).
    vco_freq_hz: Cell<u32>,
    /// Whether the PLL is currently enabled and locked.
    locked: Cell<bool>,
}

impl Pll {
    /// Create a PLL driver for the given instance.
    pub const fn new(instance: PllInstance) -> Self {
        let base = match instance {
            PllInstance::Core => CORE_PLL_BASE_ADDR,
            PllInstance::Periph => PERIPH_PLL_BASE_ADDR,
            PllInstance::Ddr => DDR_PLL_BASE_ADDR,
            PllInstance::Accel => ACCEL_PLL_BASE_ADDR,
        };
        Self {
            registers: unsafe { StaticRef::new(base as *const PllRegisters) },
            instance,
            vco_freq_hz: Cell::new(0),
            locked: Cell::new(false),
        }
    }

    #[cfg(test)]
    pub(crate) const fn new_with_registers(
        instance: PllInstance,
        registers: StaticRef<PllRegisters>,
    ) -> Self {
        Self {
            registers,
            instance,
            vco_freq_hz: Cell::new(0),
            locked: Cell::new(false),
        }
    }

    /// Get the PLL instance identifier.
    pub fn instance(&self) -> PllInstance {
        self.instance
    }

    /// Select FXOSC as the PLL reference clock.
    ///
    /// By default after reset, PLLs use FIRC (48 MHz). This switches to the
    /// external crystal oscillator. Must be called while the PLL is powered
    /// down (PLLPD=1).
    ///
    /// If already locked, succeeds only when hardware already selects FXOSC;
    /// a locked PLL is never rewritten.
    ///
    /// # Errors
    /// - [`ErrorCode::BUSY`]: PLL is locked with a non-FXOSC reference
    pub fn select_reference_fxosc(&self) -> Result<(), ErrorCode> {
        let regs = &*self.registers;
        if regs.pllsr.is_set(PLLSR::LOCK) {
            self.locked.set(true);
            return if regs.pllclkmux.matches_all(PLLCLKMUX::REFCLKSEL::Fxosc) {
                Ok(())
            } else {
                Err(ErrorCode::BUSY)
            };
        }
        regs.pllclkmux.write(PLLCLKMUX::REFCLKSEL::Fxosc);
        Ok(())
    }

    /// Configure the PLL dividers.
    ///
    /// The PLL must be powered down (disabled) before calling this.
    ///
    /// # Parameters
    /// - `ref_freq_hz`: reference clock frequency (FIRC or FXOSC) in Hz
    /// - `config`: PLL multiplier/divider configuration
    ///
    /// # Errors
    /// - [`ErrorCode::BUSY`]: PLL is still running (must disable first)
    /// - [`ErrorCode::INVAL`]: configuration parameters out of range
    pub fn configure(&self, ref_freq_hz: u32, config: PllConfig) -> Result<(), ErrorCode> {
        let regs = &*self.registers;

        if regs.pllsr.is_set(PLLSR::LOCK) {
            self.locked.set(true);
            let snapshot = self.snapshot(ref_freq_hz)?;
            let matches = snapshot.ref_clock_hz == ref_freq_hz
                && snapshot.rdiv == u32::from(config.rdiv)
                && snapshot.mfi == u32::from(config.mfi)
                && snapshot.mfn == config.mfn
                && snapshot.fractional == (config.mfn != 0);
            if !matches {
                return Err(ErrorCode::BUSY);
            }
            self.vco_freq_hz.set(snapshot.vco_hz);
            return Ok(());
        }

        if config.rdiv == 0 || config.rdiv > 7 {
            return Err(ErrorCode::INVAL);
        }
        if config.mfi < 16 || config.mfi > 255 {
            return Err(ErrorCode::INVAL);
        }
        if config.mfn >= PLL_MFN_DENOMINATOR as u32 {
            return Err(ErrorCode::INVAL);
        }

        // Compute before writing hardware so invalid arithmetic cannot leave
        // a partially updated PLL configuration behind.
        let vco = Self::compute_vco_hz(
            ref_freq_hz,
            u32::from(config.rdiv),
            u32::from(config.mfi),
            config.mfn,
        )?;

        regs.plldv
            .write(PLLDV::RDIV.val(u32::from(config.rdiv)) + PLLDV::MFI.val(u32::from(config.mfi)));
        if config.mfn > 0 {
            regs.pllfd
                .write(PLLFD::SDMEN::SET + PLLFD::MFN.val(config.mfn));
        } else {
            regs.pllfd.write(PLLFD::SDMEN::CLEAR + PLLFD::MFN.val(0));
        }
        self.vco_freq_hz.set(vco);

        Ok(())
    }

    /// Configure a PHI output divider.
    ///
    /// # Parameters
    /// - `phi`: PHI output configuration
    ///
    /// # Errors
    /// - [`ErrorCode::INVAL`]: index out of range for this PLL
    pub fn configure_phi(&self, phi: PhiConfig) -> Result<(), ErrorCode> {
        if phi.index >= self.instance.num_phi_outputs() {
            return Err(ErrorCode::INVAL);
        }

        let regs = &*self.registers;
        if regs.pllsr.is_set(PLLSR::LOCK) {
            self.locked.set(true);
            let current = regs.pllodiv[phi.index].extract();
            let matches = current.is_set(PLLODIV::DE) == phi.enabled
                && current.read(PLLODIV::DIV) == u32::from(phi.div);
            return if matches {
                Ok(())
            } else {
                Err(ErrorCode::BUSY)
            };
        }

        if phi.enabled {
            regs.pllodiv[phi.index].write(PLLODIV::DE::SET + PLLODIV::DIV.val(phi.div as u32));
        } else {
            regs.pllodiv[phi.index].write(PLLODIV::DE::CLEAR + PLLODIV::DIV.val(0));
        }
        Ok(())
    }

    fn compute_vco_hz(ref_freq_hz: u32, rdiv: u32, mfi: u32, mfn: u32) -> Result<u32, ErrorCode> {
        if ref_freq_hz == 0 || rdiv == 0 {
            return Err(ErrorCode::INVAL);
        }
        let ref_div = u64::from(ref_freq_hz) / u64::from(rdiv);
        let integer = ref_div
            .checked_mul(u64::from(mfi))
            .ok_or(ErrorCode::INVAL)?;
        let fractional = ref_div
            .checked_mul(u64::from(mfn))
            .ok_or(ErrorCode::INVAL)?
            / PLL_MFN_DENOMINATOR;
        let vco = integer.checked_add(fractional).ok_or(ErrorCode::INVAL)?;
        u32::try_from(vco).map_err(|_| ErrorCode::INVAL)
    }

    /// Decode the live PLL registers without assuming the requested
    /// configuration was applied. `fxosc_freq_hz` is used only when the live
    /// PLLCLKMUX selects FXOSC; FIRC is fixed at 48 MHz.
    pub fn snapshot(&self, fxosc_freq_hz: u32) -> Result<PllSnapshot, ErrorCode> {
        let regs = &*self.registers;
        let pllclkmux = regs.pllclkmux.get();
        let plldv = regs.plldv.get();
        let pllfd = regs.pllfd.get();
        let rdiv = regs.plldv.read(PLLDV::RDIV);
        let mfi = regs.plldv.read(PLLDV::MFI);
        let fractional = regs.pllfd.is_set(PLLFD::SDMEN);
        let mfn = if fractional {
            regs.pllfd.read(PLLFD::MFN)
        } else {
            0
        };
        let ref_clock_hz = if regs.pllclkmux.matches_all(PLLCLKMUX::REFCLKSEL::Firc) {
            48_000_000
        } else {
            fxosc_freq_hz
        };
        let vco_hz = Self::compute_vco_hz(ref_clock_hz, rdiv, mfi, mfn)?;
        Ok(PllSnapshot {
            pllclkmux,
            plldv,
            pllfd,
            rdiv,
            mfi,
            mfn,
            fractional,
            ref_clock_hz,
            vco_hz,
        })
    }

    /// Return the raw PLLODIV register for a PHI output.
    pub fn odiv_raw(&self, phi_index: usize) -> Option<u32> {
        if phi_index >= self.instance.num_phi_outputs() {
            return None;
        }

        Some(self.registers.pllodiv[phi_index].get())
    }

    /// Enable (power up) the PLL and wait for lock.
    ///
    /// # INIT-ONLY
    /// Spin-waits up to `HW_POLL_MAX` iterations (WCET ≈ 5 ms at 48 MHz FIRC).
    /// **Must only be called during board initialisation, before `kernel_loop()`.**
    /// Runtime PLL reconfiguration is prohibited — see safety manual §CLOCK-INIT.
    ///
    /// If the PLL is already locked (e.g. configured by a prior boot stage),
    /// returns immediately.
    ///
    /// # Errors
    /// - [`ErrorCode::BUSY`]: PLL did not lock within `HW_POLL_MAX` iterations
    pub fn enable_pll(&self) -> Result<(), ErrorCode> {
        let regs = &*self.registers;
        let _name = self.instance.name();

        if regs.pllsr.is_set(PLLSR::LOCK) {
            self.locked.set(true);
            return Ok(());
        }

        regs.pllcr.write(PLLCR::PLLPD::CLEAR);
        for _ in 0..HW_POLL_MAX {
            if regs.pllsr.is_set(PLLSR::LOCK) {
                self.locked.set(true);
                return Ok(());
            }
        }
        regs.pllcr.write(PLLCR::PLLPD::SET);
        Err(ErrorCode::BUSY)
    }

    /// Disable (power down) the PLL.
    ///
    /// # INIT-ONLY
    /// Spin-waits up to `HW_POLL_MAX` iterations (WCET ≈ 5 ms at 48 MHz FIRC).
    /// **Must only be called during board initialisation, before `kernel_loop()`.**
    /// Runtime PLL reconfiguration is prohibited — see safety manual §CLOCK-INIT.
    ///
    /// # Errors
    /// - [`ErrorCode::BUSY`]: PLL did not power down within `HW_POLL_MAX` iterations
    pub fn disable_pll(&self) -> Result<(), ErrorCode> {
        let regs = &*self.registers;

        regs.pllcr.write(PLLCR::PLLPD::SET);
        for _ in 0..HW_POLL_MAX {
            if !regs.pllsr.is_set(PLLSR::LOCK) {
                self.locked.set(false);
                return Ok(());
            }
        }
        Err(ErrorCode::BUSY)
    }

    /// Check whether the PLL is locked (enabled and stable).
    pub fn is_locked(&self) -> bool {
        self.locked.get()
    }

    /// Get the VCO frequency in Hz (0 if not configured).
    pub fn get_vco_frequency_hz(&self) -> u32 {
        self.vco_freq_hz.get()
    }

    /// Get the frequency of a PHI output in Hz.
    ///
    /// Returns `None` if the PLL is not locked or the output is disabled.
    pub fn get_phi_frequency_hz(&self, phi_index: usize) -> Option<u32> {
        if !self.locked.get() || phi_index >= self.instance.num_phi_outputs() {
            return None;
        }

        let regs = &*self.registers;
        let odiv = regs.pllodiv[phi_index].extract();
        if !odiv.is_set(PLLODIV::DE) {
            return None;
        }

        let div_val = odiv.read(PLLODIV::DIV).checked_add(1)?;
        self.vco_freq_hz.get().checked_div(div_val)
    }

    /// Enable SSCG (frequency modulation) on PLLs that support it.
    ///
    /// Must be called before `enable()`. Uses center-spread modulation.
    ///
    /// # Errors
    /// - [`ErrorCode::NOSUPPORT`]: this PLL does not support FM
    pub fn enable_sscg(&self, step_no: u8, step_size: u16) -> Result<(), ErrorCode> {
        if !self.instance.supports_fm() {
            return Err(ErrorCode::NOSUPPORT);
        }

        let regs = &*self.registers;
        regs.pllfm.write(
            PLLFM::SSCGBYP::CLEAR
                + PLLFM::SPREADCTL::CLEAR
                + PLLFM::STEPNO.val(step_no as u32)
                + PLLFM::STEPSIZE.val(step_size as u32),
        );
        Ok(())
    }

    /// Disable SSCG (frequency modulation).
    pub fn disable_sscg(&self) {
        let regs = &*self.registers;
        regs.pllfm.write(PLLFM::SSCGBYP::SET);
    }
}

impl ClockInterface for Pll {
    fn is_enabled(&self) -> bool {
        self.locked.get()
    }

    fn enable(&self) {
        let _ = self.enable_pll();
    }

    fn disable(&self) {
        let _ = self.disable_pll();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLL_WORDS: usize = 0xA0 / 4;
    const PLLSR: usize = 1;
    const PLLDV: usize = 2;
    const PLLFD: usize = 4;
    const PLLCLKMUX: usize = 8;
    const PLLODIV: usize = 32;
    const LOCK: u32 = 1 << 2;
    const SDMEN: u32 = 1 << 30;
    const DE: u32 = 1 << 31;

    fn pll(backing: &mut [u32; PLL_WORDS], instance: PllInstance) -> Pll {
        let registers = unsafe { StaticRef::new(backing.as_mut_ptr() as *const PllRegisters) };
        Pll::new_with_registers(instance, registers)
    }

    fn live_config(backing: &mut [u32; PLL_WORDS], rdiv: u8, mfi: u8, mfn: u32, fxosc: bool) {
        backing[PLLSR] = LOCK;
        backing[PLLDV] = (u32::from(rdiv) << 12) | u32::from(mfi);
        backing[PLLFD] = if mfn == 0 { 0 } else { SDMEN | mfn };
        backing[PLLCLKMUX] = u32::from(fxosc);
    }

    #[test]
    fn unlocked_configure_writes_dividers_and_caches_vco() {
        let mut backing = [0; PLL_WORDS];
        let pll = pll(&mut backing, PllInstance::Core);
        let config = PllConfig::new(2, 40, 9_216);

        assert_eq!(pll.configure(48_000_000, config), Ok(()));
        assert_eq!(backing[PLLDV], (2 << 12) | 40);
        assert_eq!(backing[PLLFD], SDMEN | 9_216);
        assert_eq!(pll.get_vco_frequency_hz(), 972_000_000);
    }

    #[test]
    fn locked_reference_accepts_only_fxosc_without_writing() {
        let mut backing = [0; PLL_WORDS];
        backing[PLLSR] = LOCK;
        backing[PLLCLKMUX] = 1;
        let pll = pll(&mut backing, PllInstance::Core);
        let before = backing;
        assert_eq!(pll.select_reference_fxosc(), Ok(()));
        assert_eq!(backing, before);

        backing[PLLCLKMUX] = 0;
        let before = backing;
        assert_eq!(pll.select_reference_fxosc(), Err(ErrorCode::BUSY));
        assert_eq!(backing, before);
    }

    #[test]
    fn locked_configure_requires_exact_live_state_without_writing() {
        let config = PllConfig::new(2, 40, 1_024);
        let cases = [
            (2, 40, 1_024, true, 24_000_000, Ok(())),
            (2, 40, 1_024, false, 24_000_000, Err(ErrorCode::BUSY)),
            (3, 40, 1_024, true, 24_000_000, Err(ErrorCode::BUSY)),
            (2, 41, 1_024, true, 24_000_000, Err(ErrorCode::BUSY)),
            (2, 40, 1_025, true, 24_000_000, Err(ErrorCode::BUSY)),
            (2, 40, 0, true, 24_000_000, Err(ErrorCode::BUSY)),
        ];

        for (rdiv, mfi, mfn, fxosc, reference, expected) in cases {
            let mut backing = [0; PLL_WORDS];
            live_config(&mut backing, rdiv, mfi, mfn, fxosc);
            let pll = pll(&mut backing, PllInstance::Core);
            let before = backing;
            assert_eq!(pll.configure(reference, config), expected);
            assert_eq!(backing, before);
        }
    }

    #[test]
    fn locked_phi_requires_exact_enable_and_divider_without_writing() {
        let requested = PhiConfig {
            index: 1,
            div: 7,
            enabled: true,
        };
        let cases = [
            (DE | (7 << 16), Ok(())),
            (7 << 16, Err(ErrorCode::BUSY)),
            (DE | (8 << 16), Err(ErrorCode::BUSY)),
        ];

        for (odiv, expected) in cases {
            let mut backing = [0; PLL_WORDS];
            backing[PLLSR] = LOCK;
            backing[PLLODIV + requested.index] = odiv;
            let pll = pll(&mut backing, PllInstance::Core);
            let before = backing;
            assert_eq!(pll.configure_phi(requested), expected);
            assert_eq!(backing, before);
        }
    }

    #[test]
    fn from_frequencies_rejects_invalid_and_accepts_hardware_boundaries() {
        for (reference, target, rdiv) in [
            (0, 384_000_000, 1),
            (24_000_000, 384_000_000, 0),
            (24_000_000, 384_000_000, 8),
            (24_000_000, 360_000_000, 1),
            (1, u32::MAX, 7),
        ] {
            assert!(matches!(
                PllConfig::from_frequencies(reference, target, rdiv),
                Err(ErrorCode::INVAL)
            ));
        }

        let minimum = PllConfig::from_frequencies(24_000_000, 384_000_000, 1).unwrap();
        assert_eq!((minimum.rdiv, minimum.mfi, minimum.mfn), (1, 16, 0));
        let maximum = PllConfig::from_frequencies(16_000_000, 4_080_000_000, 1).unwrap();
        assert_eq!((maximum.rdiv, maximum.mfi, maximum.mfn), (1, 255, 0));

        let mut backing = [0; PLL_WORDS];
        let pll = pll(&mut backing, PllInstance::Core);
        assert_eq!(pll.configure(16_843_009, PllConfig::new(1, 255, 0)), Ok(()));
        assert_eq!(pll.get_vco_frequency_hz(), u32::MAX);
    }

    #[test]
    fn configure_rejects_invalid_fields_and_unrepresentable_vco_without_writes() {
        let cases = [
            (0, PllConfig::new(1, 16, 0)),
            (48_000_000, PllConfig::new(0, 16, 0)),
            (48_000_000, PllConfig::new(8, 16, 0)),
            (48_000_000, PllConfig::new(1, 15, 0)),
            (48_000_000, PllConfig::new(1, 256, 0)),
            (48_000_000, PllConfig::new(1, 16, 18_432)),
            (u32::MAX, PllConfig::new(1, 255, 0)),
        ];

        for (reference, config) in cases {
            let mut backing = [0xA5A5_A5A5; PLL_WORDS];
            backing[PLLSR] = 0;
            let pll = pll(&mut backing, PllInstance::Core);
            let before = backing;
            assert_eq!(pll.configure(reference, config), Err(ErrorCode::INVAL));
            assert_eq!(backing, before);
        }
    }

    #[test]
    fn snapshot_and_phi_getter_reject_invalid_or_unavailable_frequency() {
        let mut backing = [0; PLL_WORDS];
        let pll = pll(&mut backing, PllInstance::Core);
        assert!(matches!(pll.snapshot(24_000_000), Err(ErrorCode::INVAL)));
        assert_eq!(pll.get_phi_frequency_hz(0), None);

        live_config(&mut backing, 1, 16, 0, true);
        assert_eq!(pll.configure(24_000_000, PllConfig::new(1, 16, 0)), Ok(()));
        assert_eq!(pll.get_phi_frequency_hz(0), None);
        assert_eq!(pll.get_phi_frequency_hz(2), None);
    }
}
