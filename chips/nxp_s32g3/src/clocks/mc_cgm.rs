// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! MC_CGM (Clock Generation Module) driver for NXP S32G3.
//!
//! The MC_CGM modules contain clock mux selectors and dividers that route
//! PLL/oscillator outputs to peripheral and core clocks. RM §24.3.
//!
//! The S32G3 has multiple MC_CGM instances:
//!
//! - **MC_CGM_0** — Main peripheral clock muxes (XBAR, PER, CAN, LIN, SPI, QSPI, …)
//! - **MC_CGM_1** — A53 core clock
//! - **MC_CGM_2** — PFE clocks
//! - **MC_CGM_5** — DDR clock
//! - **MC_CGM_6** — GMAC clocks
//!
//! Each mux has:
//! - A Clock Select Control register (CSC) — selects the source
//! - A Clock Select Status register (CSS) — confirms the active source
//! - Zero or more Divider Control registers (DC_n) — enables and configures dividers
//!
//! # Clock Source Mapping (RM §24.3.1, Table 78)
//!
//! The `CgmClockSource` enum maps the clock selector index to the corresponding
//! source clock signal.
//!
//! # Key MC_CGM_0 Mux Assignments (RM §24.3.2.1, Table 79)
//!
//! | Mux | Selector Output | Sources |
//! |-----|-----------------|---------|
//! | 0   | XBAR_2X_CLK    | CORE_DFS1_CLK, FIRC_CLK |
//! | 3   | PER_CLK        | PERIPH_PLL_PHI1_CLK, FIRC_CLK |
//! | 7   | CAN_PE_CLK     | PERIPH_PLL_PHI2_CLK, FXOSC_CLK, FIRC_CLK |
//! | 8   | LIN_BAUD_CLK   | PERIPH_PLL_PHI3_CLK, FXOSC_CLK, FIRC_CLK |
//! | 12  | QSPI_2X_CLK   | PERIPH_DFS1_CLK, FIRC_CLK |
//! | 14  | USDHC_CLK      | PERIPH_DFS3_CLK, FIRC_CLK |
//! | 16  | SPI_CLK        | PERIPH_PLL_PHI7_CLK, FIRC_CLK |

use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

// ---------------------------------------------------------------------------
// MC_CGM Base Addresses
// ---------------------------------------------------------------------------

/// MC_CGM_0 base address (RM §25.x).
pub const MC_CGM_0_BASE_ADDR: u32 = 0x4003_0000;
/// MC_CGM_1 base address.
pub const MC_CGM_1_BASE_ADDR: u32 = 0x4003_4000;
/// MC_CGM_2 base address.
pub const MC_CGM_2_BASE_ADDR: u32 = 0x4401_8000;
/// MC_CGM_5 base address.
pub const MC_CGM_5_BASE_ADDR: u32 = 0x4006_8000;
/// MC_CGM_6 base address.
pub const MC_CGM_6_BASE_ADDR: u32 = 0x4053_C000;

/// Maximum mux count per MC_CGM instance.
const MAX_MUX_COUNT: usize = 17;

/// Maximum dividers per mux.
const MAX_DIV_PER_MUX: usize = 2;

// Units: bare loop iterations (register read + compare + branch).
// At 48 MHz FIRC (~10 cycles/MMIO read) this caps the wait at ≈5 ms —
// mux clock switches typically complete in < 1 µs; 5 ms is a generous
// safety bound (RM §25).
// Callers MUST propagate an error on expiry, not silently continue.
const HW_POLL_MAX: u32 = 24_000;

// ---------------------------------------------------------------------------
// Register Definitions
// ---------------------------------------------------------------------------

register_structs! {
    /// Per-mux register set (CSC + CSS + up to 2 dividers + DIV_UPD_STAT).
    ///
    /// Each mux occupies a 0x40-byte slot in the MC_CGM address space.
    /// Offset within MC_CGM: 0x300 + mux_index * 0x40.
    /// Layout per RM §25.x.x: CSC=0x00, CSS=0x04, DC_0=0x08, DC_1=0x0C,
    /// DIV_UPD_STAT=0x3C.
    pub CgmMuxRegisters {
        /// Clock Select Control: selects the clock source
        (0x00 => pub csc: ReadWrite<u32, MUX_CSC::Register>),
        /// Clock Select Status: shows the current active source
        (0x04 => pub css: ReadOnly<u32, MUX_CSS::Register>),
        /// Divider Control 0
        (0x08 => pub dc0: ReadWrite<u32, MUX_DC::Register>),
        /// Divider Control 1
        (0x0C => pub dc1: ReadWrite<u32, MUX_DC::Register>),
        (0x10 => _pad0),
        /// Divider Update Status (RM: offset 0x3C within mux slot)
        (0x3C => pub div_upd_stat: ReadOnly<u32, MUX_DIV_UPD_STAT::Register>),
        (0x40 => @END),
    }
}

register_structs! {
    /// MC_CGM register block. The mux array starts at offset 0x300.
    pub McCgmRegisters {
        (0x000 => _reserved_head),
        /// Per-mux register arrays (mux 0..16 for MC_CGM_0)
        (0x300 => pub mux: [CgmMuxRegisters; MAX_MUX_COUNT]),
        (0x300 + MAX_MUX_COUNT * 0x40 => @END),
    }
}

register_bitfields![u32,
    /// Mux Clock Select Control Register
    MUX_CSC [
        /// Clock source selector index (see CgmClockSource / RM Table 78)
        SELCTL OFFSET(24) NUMBITS(6) [],
        /// Clock switch request trigger (write 1 to initiate switch)
        CLK_SW OFFSET(2) NUMBITS(1) [],
        /// Safe Clock Select: force FIRC as source (for failover)
        SAFE_SW OFFSET(3) NUMBITS(1) [],
        /// Rampup/Rampdown enable (for PCFS)
        RAMPUP OFFSET(0) NUMBITS(1) [],
        RAMPDOWN OFFSET(1) NUMBITS(1) []
    ],

    /// Mux Clock Select Status Register
    MUX_CSS [
        /// Currently active clock source selector index
        SELSTAT OFFSET(24) NUMBITS(6) [],
        /// Clock switch in progress
        CLK_SW OFFSET(2) NUMBITS(1) [],
        /// Safe clock is active
        SAFE_SW OFFSET(3) NUMBITS(1) [],
        /// Switch was completed successfully
        SWIP OFFSET(16) NUMBITS(1) [],
        /// Switch trigger status
        SWTRG OFFSET(17) NUMBITS(3) []
    ],

    /// Mux Divider Control Register
    MUX_DC [
        /// Divider Enable
        DE  OFFSET(31) NUMBITS(1) [],
        /// Divider value. Actual division = DIV + 1.
        DIV OFFSET(16) NUMBITS(10) []
    ],

    /// Mux Divider Update Status
    MUX_DIV_UPD_STAT [
        /// Divider update in progress
        DIV_UPD_STAT OFFSET(0) NUMBITS(1) []
    ]
];

// ---------------------------------------------------------------------------
// Clock Source Enumeration (RM §24.3.1, Table 78)
// ---------------------------------------------------------------------------

/// Clock source index values for MC_CGM mux selectors.
///
/// These are the selector indices written to MUX_CSC[SELCTL] and read from
/// MUX_CSS[SELSTAT]. Only sources relevant to the main application clocks are
/// enumerated. See RM Table 78 for the complete mapping.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum CgmClockSource {
    /// FIRC_CLK (48 MHz)
    Firc = 0,
    /// SIRC_CLK (32 kHz)
    Sirc = 1,
    /// FXOSC_CLK (20–40 MHz)
    Fxosc = 2,
    /// CORE_PLL PHI0
    CorePllPhi0 = 4,
    /// CORE_PLL PHI1
    CorePllPhi1 = 5,
    /// CORE_DFS1_CLK (CORE_DFS port 0)
    CoreDfs1 = 12,
    /// CORE_DFS2_CLK (CORE_DFS port 1)
    CoreDfs2 = 13,
    /// CORE_DFS3_CLK (CORE_DFS port 2)
    CoreDfs3 = 14,
    /// CORE_DFS4_CLK (CORE_DFS port 3)
    CoreDfs4 = 15,
    /// CORE_DFS5_CLK (CORE_DFS port 4)
    CoreDfs5 = 16,
    /// CORE_DFS6_CLK (CORE_DFS port 5)
    CoreDfs6 = 17,
    /// PERIPH_PLL PHI0
    PeriphPllPhi0 = 18,
    /// PERIPH_PLL PHI1
    PeriphPllPhi1 = 19,
    /// PERIPH_PLL PHI2
    PeriphPllPhi2 = 20,
    /// PERIPH_PLL PHI3
    PeriphPllPhi3 = 21,
    /// PERIPH_PLL PHI4
    PeriphPllPhi4 = 22,
    /// PERIPH_PLL PHI5
    PeriphPllPhi5 = 23,
    /// PERIPH_PLL PHI6
    PeriphPllPhi6 = 24,
    /// PERIPH_PLL PHI7
    PeriphPllPhi7 = 25,
    /// PERIPH_DFS1_CLK (PERIPH_DFS port 0)
    PeriphDfs1 = 26,
    /// PERIPH_DFS2_CLK (PERIPH_DFS port 1)
    PeriphDfs2 = 27,
    /// PERIPH_DFS3_CLK (PERIPH_DFS port 2)
    PeriphDfs3 = 28,
    /// PERIPH_DFS4_CLK (PERIPH_DFS port 3)
    PeriphDfs4 = 29,
    /// PERIPH_DFS5_CLK (PERIPH_DFS port 4)
    PeriphDfs5 = 30,
    /// PERIPH_DFS6_CLK (PERIPH_DFS port 5)
    PeriphDfs6 = 31,
    /// ACCEL_PLL PHI0
    AccelPllPhi0 = 32,
    /// ACCEL_PLL PHI1
    AccelPllPhi1 = 33,
    /// DDR_PLL PHI0
    DdrPllPhi0 = 36,
}
impl CgmClockSource {
    /// Attempt to construct a `CgmClockSource` from a raw u8 selector index.
    ///
    /// Returns `None` if the index does not map to a known source.
    fn from_u8(val: u8) -> Option<Self> {
        match val {
            v if v == Self::Firc as u8 => Some(Self::Firc),
            v if v == Self::Sirc as u8 => Some(Self::Sirc),
            v if v == Self::Fxosc as u8 => Some(Self::Fxosc),
            v if v == Self::CorePllPhi0 as u8 => Some(Self::CorePllPhi0),
            v if v == Self::CorePllPhi1 as u8 => Some(Self::CorePllPhi1),
            v if v == Self::CoreDfs1 as u8 => Some(Self::CoreDfs1),
            v if v == Self::CoreDfs2 as u8 => Some(Self::CoreDfs2),
            v if v == Self::CoreDfs3 as u8 => Some(Self::CoreDfs3),
            v if v == Self::CoreDfs4 as u8 => Some(Self::CoreDfs4),
            v if v == Self::CoreDfs5 as u8 => Some(Self::CoreDfs5),
            v if v == Self::CoreDfs6 as u8 => Some(Self::CoreDfs6),
            v if v == Self::PeriphPllPhi0 as u8 => Some(Self::PeriphPllPhi0),
            v if v == Self::PeriphPllPhi1 as u8 => Some(Self::PeriphPllPhi1),
            v if v == Self::PeriphPllPhi2 as u8 => Some(Self::PeriphPllPhi2),
            v if v == Self::PeriphPllPhi3 as u8 => Some(Self::PeriphPllPhi3),
            v if v == Self::PeriphPllPhi4 as u8 => Some(Self::PeriphPllPhi4),
            v if v == Self::PeriphPllPhi5 as u8 => Some(Self::PeriphPllPhi5),
            v if v == Self::PeriphPllPhi6 as u8 => Some(Self::PeriphPllPhi6),
            v if v == Self::PeriphPllPhi7 as u8 => Some(Self::PeriphPllPhi7),
            v if v == Self::PeriphDfs1 as u8 => Some(Self::PeriphDfs1),
            v if v == Self::PeriphDfs2 as u8 => Some(Self::PeriphDfs2),
            v if v == Self::PeriphDfs3 as u8 => Some(Self::PeriphDfs3),
            v if v == Self::PeriphDfs4 as u8 => Some(Self::PeriphDfs4),
            v if v == Self::PeriphDfs5 as u8 => Some(Self::PeriphDfs5),
            v if v == Self::PeriphDfs6 as u8 => Some(Self::PeriphDfs6),
            v if v == Self::AccelPllPhi0 as u8 => Some(Self::AccelPllPhi0),
            v if v == Self::AccelPllPhi1 as u8 => Some(Self::AccelPllPhi1),
            v if v == Self::DdrPllPhi0 as u8 => Some(Self::DdrPllPhi0),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// MC_CGM Instance Identifier
// ---------------------------------------------------------------------------

/// Identifies which MC_CGM instance.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CgmInstance {
    Cgm0,
    Cgm1,
    Cgm2,
    Cgm5,
    Cgm6,
}

impl CgmInstance {
    /// Number of muxes available in this MC_CGM instance.
    fn num_muxes(self) -> usize {
        match self {
            CgmInstance::Cgm0 => 17, // mux 0..16
            CgmInstance::Cgm1 => 1,  // mux 0 only
            CgmInstance::Cgm2 => 10, // mux 0..9
            CgmInstance::Cgm5 => 1,  // mux 0 only
            CgmInstance::Cgm6 => 4,  // mux 0..3
        }
    }

    /// Short uppercase name for log messages.
    pub fn name(self) -> &'static str {
        match self {
            CgmInstance::Cgm0 => "CGM0",
            CgmInstance::Cgm1 => "CGM1",
            CgmInstance::Cgm2 => "CGM2",
            CgmInstance::Cgm5 => "CGM5",
            CgmInstance::Cgm6 => "CGM6",
        }
    }
}

// ---------------------------------------------------------------------------
// MC_CGM Driver
// ---------------------------------------------------------------------------

/// MC_CGM (Clock Generation Module) driver for a single instance.
///
/// Provides clock mux selection and divider configuration for routing PLLs
/// and oscillators to peripherals/cores.
pub struct McCgm {
    registers: StaticRef<McCgmRegisters>,
    instance: CgmInstance,
}

impl McCgm {
    /// Create a driver for the specified MC_CGM instance.
    pub const fn new(instance: CgmInstance) -> Self {
        let base = match instance {
            CgmInstance::Cgm0 => MC_CGM_0_BASE_ADDR,
            CgmInstance::Cgm1 => MC_CGM_1_BASE_ADDR,
            CgmInstance::Cgm2 => MC_CGM_2_BASE_ADDR,
            CgmInstance::Cgm5 => MC_CGM_5_BASE_ADDR,
            CgmInstance::Cgm6 => MC_CGM_6_BASE_ADDR,
        };
        Self {
            registers: unsafe { StaticRef::new(base as *const McCgmRegisters) },
            instance,
        }
    }

    /// Get the MC_CGM instance identifier.
    pub fn instance(&self) -> CgmInstance {
        self.instance
    }

    /// Select the clock source for a mux.
    ///
    /// # INIT-ONLY
    /// Spin-waits up to `HW_POLL_MAX` iterations (WCET ≈ 5 ms at 48 MHz FIRC).
    /// **Must only be called during board initialisation, before `kernel_loop()`.**
    /// Runtime clock switching is prohibited — see safety manual §CLOCK-INIT.
    ///
    /// Performs a glitchless clock switch (RM §24.1.1: "Glitchless clock
    /// switching").
    ///
    /// # Parameters
    /// - `mux`: mux index within this MC_CGM
    /// - `source`: desired clock source
    ///
    /// # Errors
    /// - [`ErrorCode::INVAL`]: mux index out of range
    /// - [`ErrorCode::BUSY`]: clock switch did not complete within `HW_POLL_MAX` iterations
    pub fn set_mux_source(&self, mux: usize, source: CgmClockSource) -> Result<(), ErrorCode> {
        let _name = self.instance.name();
        if mux >= self.instance.num_muxes() {
            return Err(ErrorCode::INVAL);
        }

        let regs = &*self.registers;
        let mux_regs = &regs.mux[mux];

        // If already on the desired source and not switching, leave the mux
        // alone — needed for the muxes that feed the M7 system bus
        // (e.g. MC_CGM_0 mux 0 → XBAR_2X_CLK). Toggling CLK_SW on a live
        // bus-feeder glitches the bus and silently wedges the M7.
        if mux_regs.css.read(MUX_CSS::SELSTAT) == source as u32
            && !mux_regs.css.is_set(MUX_CSS::SWIP)
        {
            return Ok(());
        }

        // Write the source selector and trigger the switch.
        mux_regs
            .csc
            .write(MUX_CSC::SELCTL.val(source as u32) + MUX_CSC::CLK_SW::SET);

        // Wait for the switch to complete.
        for _ in 0..HW_POLL_MAX {
            if mux_regs.css.read(MUX_CSS::SELSTAT) == source as u32
                && !mux_regs.css.is_set(MUX_CSS::SWIP)
            {
                return Ok(());
            }
        }
        Err(ErrorCode::BUSY)
    }

    /// Get the currently active clock source for a mux.
    ///
    /// Returns `None` if the mux index is out of range, or if the hardware
    /// reports a selector value that does not map to a known source.
    pub fn get_mux_source(&self, mux: usize) -> Option<CgmClockSource> {
        if mux >= self.instance.num_muxes() {
            return None;
        }
        let regs = &*self.registers;
        let raw = regs.mux[mux].css.read(MUX_CSS::SELSTAT) as u8;
        CgmClockSource::from_u8(raw)
    }

    /// Enable and configure a clock divider on a mux.
    ///
    /// # INIT-ONLY
    /// Spin-waits up to `HW_POLL_MAX` iterations (WCET ≈ 5 ms at 48 MHz FIRC).
    /// **Must only be called during board initialisation, before `kernel_loop()`.**
    /// Runtime divider updates are prohibited — see safety manual §CLOCK-INIT.
    ///
    /// # Parameters
    /// - `mux`: mux index
    /// - `div_index`: divider index (0 or 1)
    /// - `div_value`: divider value (actual division = div_value + 1)
    ///
    /// # Errors
    /// - [`ErrorCode::INVAL`]: mux or divider index out of range
    /// - [`ErrorCode::BUSY`]: divider update did not complete within `HW_POLL_MAX` iterations
    pub fn set_mux_divider(
        &self,
        mux: usize,
        div_index: usize,
        div_value: u16,
    ) -> Result<(), ErrorCode> {
        let _name = self.instance.name();
        if mux >= self.instance.num_muxes() || div_index >= MAX_DIV_PER_MUX {
            return Err(ErrorCode::INVAL);
        }

        let regs = &*self.registers;
        let mux_regs = &regs.mux[mux];

        let dc_reg = if div_index == 0 {
            &mux_regs.dc0
        } else {
            &mux_regs.dc1
        };

        dc_reg.write(MUX_DC::DE::SET + MUX_DC::DIV.val(div_value as u32));

        for _ in 0..HW_POLL_MAX {
            if !mux_regs.div_upd_stat.is_set(MUX_DIV_UPD_STAT::DIV_UPD_STAT) {
                return Ok(());
            }
        }
        Err(ErrorCode::BUSY)
    }

    /// Disable a clock divider on a mux.
    ///
    /// # Errors
    /// - [`ErrorCode::INVAL`]: mux or divider index out of range
    pub fn disable_mux_divider(&self, mux: usize, div_index: usize) -> Result<(), ErrorCode> {
        if mux >= self.instance.num_muxes() || div_index >= MAX_DIV_PER_MUX {
            return Err(ErrorCode::INVAL);
        }

        let regs = &*self.registers;
        let mux_regs = &regs.mux[mux];

        let dc_reg = if div_index == 0 {
            &mux_regs.dc0
        } else {
            &mux_regs.dc1
        };

        dc_reg.write(MUX_DC::DE::CLEAR + MUX_DC::DIV.val(0));
        Ok(())
    }

    /// Force a mux to the safe clock (FIRC).
    ///
    /// Used during reset domain management (RM §24.4).
    pub fn force_safe_clock(&self, mux: usize) -> Result<(), ErrorCode> {
        self.set_mux_source(mux, CgmClockSource::Firc)
    }
}
