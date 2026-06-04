// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Top-level S32G3 clock configuration driver.
//!
//! This module provides the [`Clocks`] struct that owns all clock sources and
//! the MC_CGM instances, offering a unified entry point for configuring the
//! entire clock tree.
//!
//! # Clock Turn-On Sequence (RM §24.5)
//!
//! 1. FIRC and SIRC are always on after reset.
//! 2. Enable FXOSC if a crystal is present.
//! 3. Enable PLLs in order: CORE → PERIPH → DDR → ACCEL (§24.5.3).
//! 4. Enable DFS blocks and configure ports.
//! 5. Switch MC_CGM muxes from FIRC to the desired PLL/DFS outputs.
//!
//! # MC_CGM_0 Mux Quick Reference (RM Table 79, Table 84)
//!
//! | Mux | Output           | Default (reset) | Typical Production Source        |
//! |-----|------------------|-----------------|----------------------------------|
//! |  0  | XBAR_2X_CLK     | FIRC            | CORE_DFS1_CLK                    |
//! |  3  | PER_CLK         | FIRC            | PERIPH_PLL_PHI1_CLK              |
//! |  4  | FTM_0_REF_CLK   | FIRC            | PERIPH_PLL_PHI1_CLK              |
//! |  5  | FTM_1_REF_CLK   | FIRC            | PERIPH_PLL_PHI1_CLK              |
//! |  6  | FLEXRAY_PE_CLK  | FIRC            | PERIPH_PLL_PHI1_CLK / FXOSC      |
//! |  7  | CAN_PE_CLK      | FIRC            | PERIPH_PLL_PHI2_CLK              |
//! |  8  | LIN_BAUD_CLK    | FIRC            | PERIPH_PLL_PHI3_CLK              |
//! | 12  | QSPI_2X_CLK    | FIRC            | PERIPH_DFS1_CLK                  |
//! | 14  | USDHC_CLK       | FIRC            | PERIPH_DFS3_CLK                  |
//! | 16  | SPI_CLK         | FIRC            | PERIPH_PLL_PHI7_CLK              |
//!
//! # MC_CGM_1 (RM Table 80)
//!
//! | Mux | Output        | Source               |
//! |-----|---------------|----------------------|
//! |  0  | A53_CORE_CLK  | CORE_PLL_PHI0_CLK    |
//!
//! # MC_CGM_5 (RM Table 82)
//!
//! | Mux | Output  | Source          |
//! |-----|---------|-----------------|
//! |  0  | DDR_CLK | DDR_PLL_PHI0    |
//!
//! # Usage
//!
//! ```rust,ignore
//! use nxp_s32g3::clocks::Clocks;
//!
//! let clocks = Clocks::new();
//!
//! // One-shot production clock configuration (FXOSC, PLLs, DFS, muxes)
//! clocks.setup_production_clocks().unwrap();
//!
//! // Query current clock frequencies
//! let lin_hz = clocks.get_lin_baud_clk_hz();
//! let can_hz = clocks.get_can_pe_clk_hz();
//!
//! // Runtime mux source switching (no raw indices needed)
//! use nxp_s32g3::clocks::mc_cgm::CgmClockSource;
//! clocks.set_lin_baud_clk_source(CgmClockSource::Firc).unwrap();
//! ```

use crate::clocks::dfs::{Dfs, DfsInstance, DfsPort, DfsPortConfig};
use crate::clocks::firc::Firc;
use crate::clocks::fxosc::Fxosc;
use crate::clocks::mc_cgm::{CgmClockSource, CgmInstance, McCgm};
use crate::clocks::pll::{PhiConfig, Pll, PllConfig, PllInstance};
use crate::clocks::sirc::Sirc;

use kernel::ErrorCode;

// ---------------------------------------------------------------------------
// Top-level Clocks struct
// ---------------------------------------------------------------------------

/// Top-level clock management structure for the S32G3.
///
/// Owns all clock source drivers and MC_CGM instances. Provides helper methods
/// for common clock configurations and enforces the correct power-up sequence.
#[allow(dead_code)]
pub struct Clocks {
    // --- Clock Sources (always available) ---
    /// Fast Internal RC Oscillator (48 MHz, always on)
    firc: Firc,
    /// Slow Internal RC Oscillator (32 kHz, always on)
    sirc: Sirc,
    /// Fast External Crystal Oscillator (20–40 MHz)
    fxosc: Fxosc,

    // --- PLLs ---
    /// CORE_PLL — drives A53, M7, HSE_H, interconnect
    core_pll: Pll,
    /// PERIPH_PLL — drives peripherals (CAN, LIN, SPI, GMAC, …)
    periph_pll: Pll,
    /// DDR_PLL — drives DRAM interface
    ddr_pll: Pll,
    /// ACCEL_PLL — drives hardware accelerators
    accel_pll: Pll,

    // --- DFS ---
    /// CORE_DFS — 6 outputs from CORE_PLL VCO
    core_dfs: Dfs,
    /// PERIPH_DFS — 6 outputs from PERIPH_PLL VCO
    periph_dfs: Dfs,

    // --- Clock Generation Modules ---
    /// MC_CGM_0 — main peripheral muxes (mux 0..16)
    mc_cgm0: McCgm,
    /// MC_CGM_1 — A53 core clock (mux 0)
    mc_cgm1: McCgm,
    /// MC_CGM_2 — PFE clocks (mux 0..9)
    mc_cgm2: McCgm,
    /// MC_CGM_5 — DDR clock (mux 0)
    mc_cgm5: McCgm,
    /// MC_CGM_6 — GMAC clocks (mux 0..3)
    mc_cgm6: McCgm,
}

// ---------------------------------------------------------------------------
// Private constants: eliminate magic numbers for mux indices and PHI outputs
// ---------------------------------------------------------------------------
// MC_CGM_0 mux indices
const MUX_XBAR_2X_CLK: usize = 0;
const MUX_PER_CLK: usize = 3;
const MUX_FLEXRAY_PE_CLK: usize = 6;
const MUX_CAN_PE_CLK: usize = 7;
const MUX_LIN_BAUD_CLK: usize = 8;
const MUX_QSPI_2X_CLK: usize = 12;
const MUX_USDHC_CLK: usize = 14;
const MUX_SPI_CLK: usize = 16;
// MC_CGM_1 mux indices
const MUX_A53_CORE_CLK: usize = 0;
// MC_CGM_5 mux indices
const MUX_DDR_CLK: usize = 0;
// PHI output indices
const PHI0: usize = 0;
const PHI1: usize = 1;
const PHI2: usize = 2;
const PHI3: usize = 3;
const PHI4: usize = 4;
const PHI5: usize = 5;
const PHI6: usize = 6;
const PHI7: usize = 7;

impl Clocks {
    /// Create a new `Clocks` instance with all sub-drivers initialized.
    ///
    /// This does not enable any clocks beyond their reset-default state
    /// (FIRC and SIRC are always on; everything else is powered down).
    pub const fn new() -> Self {
        Self {
            firc: Firc::new(),
            sirc: Sirc::new(),
            fxosc: Fxosc::new(),

            core_pll: Pll::new(PllInstance::Core),
            periph_pll: Pll::new(PllInstance::Periph),
            ddr_pll: Pll::new(PllInstance::Ddr),
            accel_pll: Pll::new(PllInstance::Accel),

            core_dfs: Dfs::new(DfsInstance::Core),
            periph_dfs: Dfs::new(DfsInstance::Periph),

            mc_cgm0: McCgm::new(CgmInstance::Cgm0),
            mc_cgm1: McCgm::new(CgmInstance::Cgm1),
            mc_cgm2: McCgm::new(CgmInstance::Cgm2),
            mc_cgm5: McCgm::new(CgmInstance::Cgm5),
            mc_cgm6: McCgm::new(CgmInstance::Cgm6),
        }
    }

    /// Execute the full PLL turn-on sequence (RM §24.5.3).
    ///
    /// PLLs must already be configured via their `configure()` methods before
    /// calling this. Skips PLLs that are already locked.
    ///
    /// Order: CORE → PERIPH → DDR → ACCEL
    ///
    /// # Errors
    /// Returns the first PLL that fails to lock.
    pub fn enable_all_plls(&self) -> Result<(), ErrorCode> {
        if self.core_pll.get_vco_frequency_hz() > 0 && !self.core_pll.is_locked() {
            self.core_pll.enable_pll()?;
        }
        if self.periph_pll.get_vco_frequency_hz() > 0 && !self.periph_pll.is_locked() {
            self.periph_pll.enable_pll()?;
        }
        if self.ddr_pll.get_vco_frequency_hz() > 0 && !self.ddr_pll.is_locked() {
            self.ddr_pll.enable_pll()?;
        }
        if self.accel_pll.get_vco_frequency_hz() > 0 && !self.accel_pll.is_locked() {
            self.accel_pll.enable_pll()?;
        }
        Ok(())
    }

    /// Enable the DFS blocks and propagate VCO frequencies from their
    /// respective parent PLLs.
    ///
    /// The parent PLLs must be locked before calling this.
    ///
    /// # Errors
    /// - [`ErrorCode::OFF`]: parent PLL is not locked
    pub fn enable_dfs_blocks(&self) -> Result<(), ErrorCode> {
        // CORE_DFS
        if self.core_pll.is_locked() {
            self.core_dfs
                .set_vco_frequency_hz(self.core_pll.get_vco_frequency_hz());
            self.core_dfs.enable_dfs();
        } else if self.core_dfs.is_enabled_dfs() {
            return Err(ErrorCode::OFF);
        }

        // PERIPH_DFS
        if self.periph_pll.is_locked() {
            self.periph_dfs
                .set_vco_frequency_hz(self.periph_pll.get_vco_frequency_hz());
            self.periph_dfs.enable_dfs();
        } else if self.periph_dfs.is_enabled_dfs() {
            return Err(ErrorCode::OFF);
        }

        Ok(())
    }

    /// Switch all software-resettable-domain-3 muxes to FIRC.
    ///
    /// Required before deasserting reset of software domain 3 (RM §24.4):
    /// - MC_CGM_0 mux 3 (PER_CLK)
    /// - MC_CGM_0 mux 6 (FLEXRAY_PE_CLK)
    /// - MC_CGM_0 mux 7 (CAN_PE_CLK)
    /// - MC_CGM_0 mux 8 (LIN_BAUD_CLK)
    pub fn force_domain3_safe_clocks(&self) -> Result<(), ErrorCode> {
        self.mc_cgm0.force_safe_clock(MUX_PER_CLK)?;
        self.mc_cgm0.force_safe_clock(MUX_FLEXRAY_PE_CLK)?;
        self.mc_cgm0.force_safe_clock(MUX_CAN_PE_CLK)?;
        self.mc_cgm0.force_safe_clock(MUX_LIN_BAUD_CLK)?;
        Ok(())
    }

    /// Switch MC_CGM_1 mux 0 (A53_CORE_CLK) to FIRC.
    ///
    /// Required before deasserting reset of software domain 1 (RM §24.4).
    pub fn force_domain1_safe_clock(&self) -> Result<(), ErrorCode> {
        self.mc_cgm1.force_safe_clock(MUX_A53_CORE_CLK)
    }

    /// Switch all MC_CGM_2 muxes (PFE) to FIRC.
    ///
    /// Required before deasserting reset of software domain 2 (RM §24.4).
    pub fn force_domain2_safe_clocks(&self) -> Result<(), ErrorCode> {
        for mux in 0..10 {
            self.mc_cgm2.force_safe_clock(mux)?;
        }
        Ok(())
    }

    /// Get the LIN_BAUD_CLK frequency in Hz.
    ///
    /// This is the clock fed to LINFlexD instances via MC_CGM_0 mux 8.
    /// Returns the frequency based on the currently selected mux source.
    pub fn get_lin_baud_clk_hz(&self) -> Option<u32> {
        match self.mc_cgm0.get_mux_source(MUX_LIN_BAUD_CLK) {
            Some(CgmClockSource::Firc) => Some(self.firc.get_frequency_hz()),
            Some(CgmClockSource::Fxosc) => self.fxosc.get_frequency_hz(),
            Some(CgmClockSource::PeriphPllPhi3) => self.periph_pll.get_phi_frequency_hz(PHI3),
            _ => None,
        }
    }

    /// Get the CAN_PE_CLK frequency in Hz.
    ///
    /// This is the clock fed to FlexCAN via MC_CGM_0 mux 7.
    pub fn get_can_pe_clk_hz(&self) -> Option<u32> {
        match self.mc_cgm0.get_mux_source(MUX_CAN_PE_CLK) {
            Some(CgmClockSource::Firc) => Some(self.firc.get_frequency_hz()),
            Some(CgmClockSource::Fxosc) => self.fxosc.get_frequency_hz(),
            Some(CgmClockSource::PeriphPllPhi2) => self.periph_pll.get_phi_frequency_hz(PHI2),
            _ => None,
        }
    }

    /// Get the SPI_CLK frequency in Hz.
    ///
    /// MC_CGM_0 mux 16.
    pub fn get_spi_clk_hz(&self) -> Option<u32> {
        match self.mc_cgm0.get_mux_source(MUX_SPI_CLK) {
            Some(CgmClockSource::Firc) => Some(self.firc.get_frequency_hz()),
            Some(CgmClockSource::PeriphPllPhi7) => self.periph_pll.get_phi_frequency_hz(PHI7),
            _ => None,
        }
    }

    /// Get the PER_CLK frequency in Hz.
    ///
    /// MC_CGM_0 mux 3.
    pub fn get_per_clk_hz(&self) -> Option<u32> {
        match self.mc_cgm0.get_mux_source(MUX_PER_CLK) {
            Some(CgmClockSource::Firc) => Some(self.firc.get_frequency_hz()),
            Some(CgmClockSource::PeriphPllPhi1) => self.periph_pll.get_phi_frequency_hz(PHI1),
            _ => None,
        }
    }

    /// Get the A53_CORE_CLK frequency in Hz.
    ///
    /// MC_CGM_1 mux 0.
    pub fn get_a53_core_clk_hz(&self) -> Option<u32> {
        match self.mc_cgm1.get_mux_source(MUX_A53_CORE_CLK) {
            Some(CgmClockSource::Firc) => Some(self.firc.get_frequency_hz()),
            Some(CgmClockSource::CorePllPhi0) => self.core_pll.get_phi_frequency_hz(PHI0),
            _ => None,
        }
    }

    /// Get the XBAR_2X_CLK frequency in Hz.
    ///
    /// MC_CGM_0 mux 0.
    pub fn get_xbar_2x_clk_hz(&self) -> Option<u32> {
        match self.mc_cgm0.get_mux_source(MUX_XBAR_2X_CLK) {
            Some(CgmClockSource::Firc) => Some(self.firc.get_frequency_hz()),
            Some(CgmClockSource::CoreDfs1) => self.core_dfs.get_port_frequency_hz(DfsPort::Port0),
            _ => None,
        }
    }

    /// Get the DDR_CLK frequency in Hz.
    ///
    /// MC_CGM_5 mux 0.
    pub fn get_ddr_clk_hz(&self) -> Option<u32> {
        match self.mc_cgm5.get_mux_source(MUX_DDR_CLK) {
            Some(CgmClockSource::Firc) => Some(self.firc.get_frequency_hz()),
            Some(CgmClockSource::DdrPllPhi0) => self.ddr_pll.get_phi_frequency_hz(PHI0),
            _ => None,
        }
    }

    /// Get the QSPI_2X_CLK frequency in Hz.
    ///
    /// MC_CGM_0 mux 12.
    pub fn get_qspi_2x_clk_hz(&self) -> Option<u32> {
        match self.mc_cgm0.get_mux_source(MUX_QSPI_2X_CLK) {
            Some(CgmClockSource::Firc) => Some(self.firc.get_frequency_hz()),
            Some(CgmClockSource::PeriphDfs1) => {
                self.periph_dfs.get_port_frequency_hz(DfsPort::Port0)
            }
            _ => None,
        }
    }

    /// Get the USDHC_CLK frequency in Hz.
    ///
    /// MC_CGM_0 mux 14.
    pub fn get_usdhc_clk_hz(&self) -> Option<u32> {
        match self.mc_cgm0.get_mux_source(MUX_USDHC_CLK) {
            Some(CgmClockSource::Firc) => Some(self.firc.get_frequency_hz()),
            Some(CgmClockSource::PeriphDfs3) => {
                self.periph_dfs.get_port_frequency_hz(DfsPort::Port2)
            }
            _ => None,
        }
    }

    // =======================================================================
    // Typed mux-source setters — no raw indices required
    // =======================================================================

    /// Select the clock source for LIN_BAUD_CLK (MC_CGM_0 mux 8).
    pub fn set_lin_baud_clk_source(&self, source: CgmClockSource) -> Result<(), ErrorCode> {
        self.mc_cgm0.set_mux_source(MUX_LIN_BAUD_CLK, source)
    }

    /// Select the clock source for CAN_PE_CLK (MC_CGM_0 mux 7).
    pub fn set_can_pe_clk_source(&self, source: CgmClockSource) -> Result<(), ErrorCode> {
        self.mc_cgm0.set_mux_source(MUX_CAN_PE_CLK, source)
    }

    /// Select the clock source for SPI_CLK (MC_CGM_0 mux 16).
    pub fn set_spi_clk_source(&self, source: CgmClockSource) -> Result<(), ErrorCode> {
        self.mc_cgm0.set_mux_source(MUX_SPI_CLK, source)
    }

    /// Select the clock source for PER_CLK (MC_CGM_0 mux 3).
    pub fn set_per_clk_source(&self, source: CgmClockSource) -> Result<(), ErrorCode> {
        self.mc_cgm0.set_mux_source(MUX_PER_CLK, source)
    }

    /// Select the clock source for XBAR_2X_CLK (MC_CGM_0 mux 0).
    pub fn set_xbar_2x_clk_source(&self, source: CgmClockSource) -> Result<(), ErrorCode> {
        self.mc_cgm0.set_mux_source(MUX_XBAR_2X_CLK, source)
    }

    /// Select the clock source for QSPI_2X_CLK (MC_CGM_0 mux 12).
    pub fn set_qspi_2x_clk_source(&self, source: CgmClockSource) -> Result<(), ErrorCode> {
        self.mc_cgm0.set_mux_source(MUX_QSPI_2X_CLK, source)
    }

    /// Select the clock source for USDHC_CLK (MC_CGM_0 mux 14).
    pub fn set_usdhc_clk_source(&self, source: CgmClockSource) -> Result<(), ErrorCode> {
        self.mc_cgm0.set_mux_source(MUX_USDHC_CLK, source)
    }

    /// Select the clock source for A53_CORE_CLK (MC_CGM_1 mux 0).
    pub fn set_a53_core_clk_source(&self, source: CgmClockSource) -> Result<(), ErrorCode> {
        self.mc_cgm1.set_mux_source(MUX_A53_CORE_CLK, source)
    }

    /// Select the clock source for DDR_CLK (MC_CGM_5 mux 0).
    pub fn set_ddr_clk_source(&self, source: CgmClockSource) -> Result<(), ErrorCode> {
        self.mc_cgm5.set_mux_source(MUX_DDR_CLK, source)
    }

    /// Emit a compact live clock-tree snapshot over the synchronous LF0 trace
    /// path. This reads hardware state; it does not switch sources.
    pub fn trace_clock_summary(&self) {
        let fxosc_hz = self.fxosc.get_frequency_hz().unwrap_or(40_000_000);
        let _core = self.core_pll.snapshot(fxosc_hz);
        let _periph = self.periph_pll.snapshot(fxosc_hz);
        let _ddr = self.ddr_pll.snapshot(fxosc_hz);
        let _accel = self.accel_pll.snapshot(fxosc_hz);
    }

    // =======================================================================
    // Board-level clock initialization
    // =======================================================================

    /// Configure and enable all production clocks for the S32G3 SAIL board.
    ///
    /// This follows the RM §24.5 clock-on procedure:
    ///
    /// 1. Enable FXOSC (40 MHz crystal on SAIL board)
    /// 2. Configure and enable PLLs (using FXOSC as reference):
    ///    - CORE_PLL: VCO = 2600 MHz (40/1 × 65)
    ///    - PERIPH_PLL: VCO = 2000 MHz (40/1 × 50)
    ///    - DDR_PLL: VCO = 1600 MHz (40/1 × 40)
    ///    - ACCEL_PLL: VCO = 2000 MHz (40/1 × 50)
    /// 3. Configure PHI output dividers
    /// 4. Enable DFS blocks and configure ports
    /// 5. Switch MC_CGM muxes to production sources
    ///
    /// # PLL Output Frequencies
    ///
    /// ## CORE_PLL (VCO = 2600 MHz)
    /// - PHI0: 2600 / (1+1) = 1300 MHz → A53_CORE_CLK (via MC_CGM_1 mux 0)
    /// - PHI1: 2600 / (9+1) = 260 MHz → reserved
    ///
    /// ## PERIPH_PLL (VCO = 2000 MHz)
    /// - PHI0: 2000 / (19+1) = 100 MHz → CLKOUT (MC_CGM_0 mux 1/2)
    /// - PHI1: 2000 / (19+1) = 100 MHz → PER_CLK (MC_CGM_0 mux 3)
    /// - PHI2: 2000 / (23+1) ≈ 83 MHz → CAN_PE_CLK (MC_CGM_0 mux 7)
    /// - PHI3: 2000 / (49+1) = 40 MHz → LIN_BAUD_CLK (MC_CGM_0 mux 8)
    /// - PHI4: 2000 / (15+1) = 125 MHz → GMAC_TS_CLK (MC_CGM_6 mux 0)
    /// - PHI5: 2000 / (15+1) = 125 MHz → GMAC/PFE TX (MC_CGM_6 mux 1)
    /// - PHI6: 2000 / (7+1) = 250 MHz → reserved
    /// - PHI7: 2000 / (19+1) = 100 MHz → SPI_CLK (MC_CGM_0 mux 16)
    ///
    /// ## DDR_PLL (VCO = 1600 MHz)
    /// - PHI0: 1600 / (1+1) = 800 MHz → DDR_CLK (MC_CGM_5 mux 0)
    ///
    /// ## CORE_DFS (source: CORE_PLL VCO = 2600 MHz)
    /// - Port 0 (CORE_DFS1): MFI=1, MFN=23 → 2600·18/(1·36+23) = 793.2 MHz →
    ///   XBAR_2X_CLK. Must match the prior boot stage's target (see configure_port below).
    ///
    /// ## PERIPH_DFS (source: PERIPH_PLL VCO = 2000 MHz)
    /// - Port 0 (PERIPH_DFS1): 2000 / (2×3) = ~333 MHz → QSPI_2X_CLK
    /// - Port 2 (PERIPH_DFS3): 2000 / (2×3) = ~333 MHz → USDHC_CLK
    /// - Port 4 (PERIPH_DFS5): 2000 / (2×5) = 200 MHz → CLKOUT
    ///
    /// # Errors
    ///
    /// Returns the first error encountered during initialization.
    pub fn setup_production_clocks(&self) -> Result<(), ErrorCode> {
        // Reference clock: FXOSC at 40 MHz (SAIL board crystal)
        const FXOSC_FREQ_MHZ: u32 = 40;
        const FXOSC_FREQ_HZ: u32 = FXOSC_FREQ_MHZ * 1_000_000;

        // --- Safe-Clock Parking ----------------------------------------------
        // Park A53_CORE_CLK (MC_CGM_1 Mux 0) and XBAR_2X_CLK (MC_CGM_0 Mux 0)
        // on the safe FIRC clock so we can safely reprogram CORE_PLL and CORE_DFS
        // without glitching or deadlocking the running A53 core!
        self.mc_cgm1
            .set_mux_source(MUX_A53_CORE_CLK, CgmClockSource::Firc)?;
        self.mc_cgm0
            .set_mux_source(MUX_XBAR_2X_CLK, CgmClockSource::Firc)?;

        // --- Step 1: Enable FXOSC -------------------------------------------
        self.fxosc.set_frequency_mhz(FXOSC_FREQ_MHZ);
        self.fxosc.enable_crystal()?;
        // --- Step 2: Configure PLLs -----------------------------------------
        // Disabling CORE_PLL clears its lock bit, forcing Tock to actively
        // reprogram it to the target 2.6 GHz instead of skipping it.
        self.core_pll.disable_pll()?;
        self.core_pll.select_reference_fxosc()?;
        self.core_pll.configure(
            FXOSC_FREQ_HZ,
            PllConfig::from_frequencies(FXOSC_FREQ_HZ, 2600000000, 1),
        )?;
        // PHI0 = 2600 / (1+1) = 1300 MHz → A53_CORE_CLK
        self.core_pll.configure_phi(PhiConfig {
            index: PHI0,
            div: 1,
            enabled: true,
        })?;
        // PHI1 = 2600 / (9+1) = 260 MHz (reserved; not consumed by A53)
        self.core_pll.configure_phi(PhiConfig {
            index: PHI1,
            div: 9,
            enabled: true,
        })?;

        // PERIPH_PLL: VCO = 40 MHz / 1 × 50 = 2000 MHz
        self.periph_pll.select_reference_fxosc()?;
        self.periph_pll.configure(
            FXOSC_FREQ_HZ,
            PllConfig {
                rdiv: 1,
                mfi: 50,
                mfn: 0,
            },
        )?;
        // PHI0 = 2000 / (19+1) = 100 MHz → CLKOUT
        self.periph_pll.configure_phi(PhiConfig {
            index: PHI0,
            div: 19,
            enabled: true,
        })?;
        // PHI1 = 2000 / (19+1) = 100 MHz → PER_CLK
        self.periph_pll.configure_phi(PhiConfig {
            index: PHI1,
            div: 19,
            enabled: true,
        })?;
        // PHI2 = 2000 / (23+1) ≈ 83 MHz → CAN_PE_CLK
        self.periph_pll.configure_phi(PhiConfig {
            index: PHI2,
            div: 23,
            enabled: true,
        })?;
        // PHI3 = 2000 / (49+1) = 40 MHz → LIN_BAUD_CLK
        self.periph_pll.configure_phi(PhiConfig {
            index: PHI3,
            div: 49,
            enabled: true,
        })?;
        // PHI4 = 2000 / (15+1) = 125 MHz → GMAC_TS_CLK
        self.periph_pll.configure_phi(PhiConfig {
            index: PHI4,
            div: 15,
            enabled: true,
        })?;
        // PHI5 = 2000 / (15+1) = 125 MHz → GMAC/PFE TX
        self.periph_pll.configure_phi(PhiConfig {
            index: PHI5,
            div: 15,
            enabled: true,
        })?;
        // PHI6 = 2000 / (7+1) = 250 MHz
        self.periph_pll.configure_phi(PhiConfig {
            index: PHI6,
            div: 7,
            enabled: true,
        })?;
        // PHI7 = 2000 / (19+1) = 100 MHz → SPI_CLK
        self.periph_pll.configure_phi(PhiConfig {
            index: PHI7,
            div: 19,
            enabled: true,
        })?;
        // DDR_PLL: VCO = 40 MHz / 1 × 40 = 1600 MHz
        self.ddr_pll.select_reference_fxosc()?;
        self.ddr_pll.configure(
            FXOSC_FREQ_HZ,
            PllConfig {
                rdiv: 1,
                mfi: 40,
                mfn: 0,
            },
        )?;
        // PHI0 = 1600 / (1+1) = 800 MHz → DDR_CLK
        self.ddr_pll.configure_phi(PhiConfig {
            index: PHI0,
            div: 1,
            enabled: true,
        })?;

        // ACCEL_PLL: VCO = 40 MHz / 1 × 50 = 2000 MHz
        self.accel_pll.select_reference_fxosc()?;
        self.accel_pll.configure(
            FXOSC_FREQ_HZ,
            PllConfig {
                rdiv: 1,
                mfi: 50,
                mfn: 0,
            },
        )?;
        // PHI0 = 2000 / (3+1) = 500 MHz
        self.accel_pll.configure_phi(PhiConfig {
            index: PHI0,
            div: 3,
            enabled: true,
        })?;
        // PHI1 = 2000 / (7+1) = 250 MHz → PFE_PE_CLK
        self.accel_pll.configure_phi(PhiConfig {
            index: PHI1,
            div: 7,
            enabled: true,
        })?;

        // --- Step 3: Enable PLLs in order (RM §24.5.3) ----------------------
        self.enable_all_plls()?;

        // --- Step 4: Enable DFS blocks and configure ports ------------------
        // We first force-disable the CORE_DFS block to clear port locks,
        // making sure configure_port actively programs the divider instead of skipping it.
        self.core_dfs.disable_dfs();
        self.core_dfs.enable_dfs();
        // Delay to allow the DFS block to exit reset and stabilize before configuring
        for _ in 0..10000 {
            core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        }
        self.enable_dfs_blocks()?;
        // CORE_DFS port 0 → CORE_DFS1_CLK → XBAR_2X_CLK (MC_CGM_0 mux 0).
        //
        // If a prior boot stage (e.g. the A53 boot ROM or secondary loader)
        // has already configured this DFS port and is actively fetching
        // instructions through XBAR_2X_CLK, reprogramming the port would
        // deadlock that core.  We therefore program the exact MFI/MFN that
        // matches the typical prior-stage configuration (MFI=1, MFN=23),
        // which makes the `init_dfs_port()` logic in the other stage see
        // live registers that already match its target and skip the reset.
        //
        // Target XBAR_2X ≈ 793 MHz
        // (f = 2600·18/(1·36+23) ≈ 793.2 MHz).
        // Adjust this if your board uses a different prior-stage divider.
        self.core_dfs.configure_port(DfsPortConfig {
            port: DfsPort::Port0,
            mfi: 1,
            mfn: 23,
        })?;

        // PERIPH_DFS port 0: f = 2000 / (2 × (3 + 0/36)) ≈ 333 MHz
        // → PERIPH_DFS1_CLK → QSPI_2X_CLK (MC_CGM_0 mux 12)
        self.periph_dfs.configure_port(DfsPortConfig {
            port: DfsPort::Port0,
            mfi: 3,
            mfn: 0,
        })?;

        // PERIPH_DFS port 2: f = 2000 / (2 × (3 + 0/36)) ≈ 333 MHz
        // → PERIPH_DFS3_CLK → USDHC_CLK (MC_CGM_0 mux 14)
        self.periph_dfs.configure_port(DfsPortConfig {
            port: DfsPort::Port2,
            mfi: 3,
            mfn: 0,
        })?;

        // PERIPH_DFS port 4: f = 2000 / (2 × (5 + 0/36)) = 200 MHz
        // → PERIPH_DFS5_CLK → CLKOUT (MC_CGM_0 mux 1/2)
        self.periph_dfs.configure_port(DfsPortConfig {
            port: DfsPort::Port4,
            mfi: 5,
            mfn: 0,
        })?;

        // --- Step 5: Switch MC_CGM muxes to production sources -------------
        self.mc_cgm0
            .set_mux_source(MUX_XBAR_2X_CLK, CgmClockSource::CoreDfs1)?;
        self.mc_cgm0
            .set_mux_source(MUX_PER_CLK, CgmClockSource::PeriphPllPhi1)?;
        self.mc_cgm0
            .set_mux_source(MUX_CAN_PE_CLK, CgmClockSource::PeriphPllPhi2)?;
        self.mc_cgm0
            .set_mux_source(MUX_LIN_BAUD_CLK, CgmClockSource::PeriphPllPhi3)?;
        self.mc_cgm0
            .set_mux_source(MUX_QSPI_2X_CLK, CgmClockSource::PeriphDfs1)?;
        self.mc_cgm0
            .set_mux_source(MUX_USDHC_CLK, CgmClockSource::PeriphDfs3)?;
        self.mc_cgm0
            .set_mux_source(MUX_SPI_CLK, CgmClockSource::PeriphPllPhi7)?;
        self.mc_cgm1
            // The A53 clock infrastructure must be programmed before releasing
            // CA53 from reset, otherwise the core has no clock.
            .set_mux_source(MUX_A53_CORE_CLK, CgmClockSource::CorePllPhi0)?;
        self.mc_cgm5
            .set_mux_source(MUX_DDR_CLK, CgmClockSource::DdrPllPhi0)?;

        Ok(())
    }
}
