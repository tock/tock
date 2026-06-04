// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! XRDC_0 — the System-instance XRDC on S32G3.
//!
//! Covers all Cortex-M7 / A53 / eDMA / HSE / GMAC / FlexRay / debug / uSDHC
//! masters (RM §15.2.2 Table 28) and all peripherals in PAC groups 0–4
//! (RM §15.2.3, 543 PDAC slots) plus 14 MRC submodules (RM §15.2.4).
//!
//! Boards build a [`Config`] from [`Pdac`] / [`Mda`] / [`Mrgd`] entries,
//! declare it as a `const`, and pass it to [`Xrdc0::apply`]. All validation
//! happens at const-eval; the only runtime failure mode is `apply()` called
//! on an already-locked instance, which panics (5-second reflash workflow).
//!
//! # Example
//!
//! ```ignore
//! use nxp_s32g3::xrdc::{Access::*, Domain, MrgdEntry};
//! use nxp_s32g3::xrdc::xrdc_0::{Config, Mda, Master, Mrgd, Pdac, Peripheral::*, Xrdc0};
//!
//! const XRDC_0_CFG: Config = Config::new(
//!     /* masters     */ &[Mda::assign(Master::M7_0, Domain::M7_0)],
//!     /* peripherals */ &[Pdac::new(LinFlexD0).grant(Domain::M7_0, SupervisorRw)],
//!     /* regions     */ &[Mrgd::region(0x3420_0000, 0x342F_FFFF).grant(Domain::M7_0, FullRw)],
//! );
//!
//! // In start():
//! // let xrdc0 = static_init!(Xrdc0, Xrdc0::new());
//! // xrdc0.apply(&XRDC_0_CFG);
//! ```

use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::StaticRef;

use super::{
    invalidate_mda, invalidate_mrgd, invalidate_pdac_window, max_mrc_idx, nmrgd_for_mrc,
    pdac_register_for_slot, program_mda_bus, program_mda_core, program_mrgd, program_pdac,
    register_barrier, search_and_patch_mrgd, Access, BusInitiator, Domain, MdaRaw, MrcRange,
    MrgdPatchOutcome, MrgdRaw, MrgdTarget, PdacRaw, PrivAttr, SecureAttr, XrdcPatchError,
    XrdcRegisters, CR,
};

/// Base address of XRDC_0 (RM §15.7.3.1).
pub const XRDC_0_BASE: StaticRef<XrdcRegisters> = super::XRDC_0_BASE;

// =============================================================================
// Peripheral enumeration (PDAC slots)
// =============================================================================

/// XRDC_0-protected peripherals.
///
/// slots). Each variant's discriminant is its PDAC slot number per the S32G3
/// Reference Manual memory map;
#[repr(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Peripheral {
    // ---- PAC0 (slots 0..31) — clock / mode / pinmux infrastructure ----
    /// `MC_CGM_0` clock-generation module at `0x4003_0000`. PDAC slot 9.
    McCgm0 = 9,
    /// `MC_CGM_1` at `0x4003_4000`. PDAC slot 10.
    McCgm1 = 10,
    /// Core PLL (ARM PLL) at `0x4003_8000`. PDAC slot 11.
    CorePll = 11,
    /// Peripheral PLL at `0x4003_C000`. PDAC slot 12.
    PeriphPll = 12,
    /// Accelerator PLL at `0x4004_0000`. PDAC slot 13.
    AccelPll = 13,
    /// DRAM PLL at `0x4004_4000`. PDAC slot 14.
    DramPll = 14,
    /// FXOSC oscillator at `0x4005_0000`. PDAC slot 17.
    Xosc = 17,
    /// Core DFS at `0x4005_4000`. PDAC slot 18.
    CoreDfs = 18,
    /// Peripheral DFS at `0x4005_8000`. PDAC slot 19.
    PeriphDfs = 19,
    /// MC_RGM (Reset Generation Module) at `0x4007_8000`. PDAC slot 22.
    McRgm = 22,
    /// RDC (Reset Domain Controller) at `0x4008_0000`. PDAC slot 24.
    Rdc = 24,
    /// MC_ME (Mode Entry) at `0x4008_8000`. PDAC slot 25.
    McMe = 25,
    /// `SIUL2_0` pin-mux + IMCR at `0x4009_C000`. PDAC slot 28.
    Siul20 = 28,
    /// `MC_CGM_5` at `0x4006_8000`. PDAC slot 30.
    McCgm5 = 30,

    // ---- PAC1 (slots 128..161) — system timers / IPC / XRDC self / serial ----
    /// `SWT_0` software watchdog at `0x4010_0000`. PDAC slot 128.
    Swt0 = 128,
    /// `STM_1` system-timer at `0x4012_0000`. PDAC slot 132.
    Stm1 = 132,
    /// MSCM (Misc System Control Module — inter-core IRQ routing) at
    /// `0x4019_8000`. PDAC slot 140.
    Mscm = 140,
    /// `XRDC_0` self-reference at `0x401A_4000`. PDAC slot 142. **Required**:
    /// `apply()` programs lock bits on every PDAC entry _after_ `GVLD=1`, so
    /// the M7 must have continued write access to XRDC_0's own register
    /// block after the policy turns on.
    Xrdc0 = 142,
    /// `LINFlexD_0` UART register block at `0x401C_8000`. PDAC slot 145.
    LinFlexD0 = 145,
    /// `LINFlexD_1` UART register block at `0x401C_C000`. PDAC slot 146
    /// (same source as `LinFlexD0`).
    LinFlexD1 = 146,
}

// =============================================================================
// Master enumeration (MDAC initiators)
// =============================================================================

/// XRDC_0 bus-initiator (MDAC) slots. Discriminant = global MDAC submodule
/// index per RM §15.2.2 Table 28.
///
/// Initiators marked `is_bus = true` (eDMA, HSE, GMAC, FlexRay, uSDHC, …)
/// implement [`BusInitiator`] and thus expose the `.force_secure()` /
/// `.force_privileged()` builder methods on [`Mda`]. Core initiators
/// (M7s, A53s, Debug ETR, GIC, PCIe_0) do **not** implement it — calling
/// `.force_secure()` on them is a compile error at the `const`-construction
/// site of the [`Config`].
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Master {
    /// Cortex-A53 cluster 0 (RM XRDC_MDAC0). Core, 8 MDA words available.
    A53Cluster0 = 0,
    /// Cortex-A53 cluster 1 (RM XRDC_MDAC1).
    A53Cluster1 = 1,
    /// Cortex-M7_0 AXI (RM XRDC_MDAC8).
    M7_0Axi = 8,
    /// Cortex-M7_1 AXI (RM XRDC_MDAC9).
    M7_1Axi = 9,
    /// Cortex-M7_2 AXI (RM XRDC_MDAC10).
    M7_2Axi = 10,
    /// Cortex-M7_0 AHB (RM XRDC_MDAC16).
    M7_0Ahb = 16,
    /// Cortex-M7_1 AHB (RM XRDC_MDAC17).
    M7_1Ahb = 17,
    /// Cortex-M7_2 AHB (RM XRDC_MDAC18).
    M7_2Ahb = 18,
    /// Cortex-M7_3 AXI (RM XRDC_MDAC22).
    M7_3Axi = 22,
    /// Cortex-M7_3 AHB (RM XRDC_MDAC23).
    M7_3Ahb = 23,
    /// eDMA_0 (RM XRDC_MDAC6). Bus initiator (DFMT1).
    EDma0 = 6,
    /// eDMA_1 (RM XRDC_MDAC7). Bus initiator (DFMT1).
    EDma1 = 7,
    /// HSE_H (RM XRDC_MDAC11). Bus initiator (DFMT1).
    Hse = 11,
    /// GMAC_0 / Ethernet (RM XRDC_MDAC12). Bus initiator (DFMT1).
    Gmac = 12,
    /// FlexRay (RM XRDC_MDAC15). Bus initiator (DFMT1).
    FlexRay = 15,
    /// LLCE (RM XRDC_MDAC19). Bus initiator (DFMT1).
    Llce = 19,
    /// uSDHC (RM XRDC_MDAC20). Bus initiator (DFMT1).
    USdhc = 20,
    /// Debug ETR (RM XRDC_MDAC2). Bus initiator (DFMT1).
    DebugEtr = 2,
    /// GIC-500 (RM XRDC_MDAC3). Bus initiator (DFMT1).
    Gic500 = 3,
    /// PCIe_0 (RM XRDC_MDAC5). Bus initiator (DFMT1).
    Pcie0 = 5,
    /// Debug trace (RM XRDC_MDAC21). Bus initiator (DFMT1).
    DebugTrace = 21,
}

impl Master {
    /// `true` when this master uses MDA DFMT1 (single-word, optional SA/PA
    /// overrides, no PID matching). All non-core/non-A53/non-M7 initiators
    /// are DFMT1 per RM Table 28.
    pub const fn is_bus(self) -> bool {
        !matches!(
            self,
            Master::A53Cluster0
                | Master::A53Cluster1
                | Master::M7_0Axi
                | Master::M7_1Axi
                | Master::M7_2Axi
                | Master::M7_3Axi
                | Master::M7_0Ahb
                | Master::M7_1Ahb
                | Master::M7_2Ahb
                | Master::M7_3Ahb
        )
    }
}

// =============================================================================
// MRC address-coverage table for XRDC_0 (RM §15.2.4 Table 30)
// =============================================================================

/// Address-coverage windows of every XRDC_0 MRC.
///
/// The DRAM MRC (MRC0) is addressed in A53-view per the RM note
/// "always program M7_DRAM_ADDRESS + 20000000h in XRDC" — i.e. an M7-view
/// address `0x6yyy_yyyy` is given to [`Mrgd::region`] as `0x8yyy_yyyy`.
pub const MRC_RANGES: &[MrcRange] = &[
    // MRC0: DRAM (A53-view, 32-bit window).
    MrcRange {
        idx: 0,
        start: 0x8000_0000,
        end: 0xFFFF_FFFF,
        nmrgd: 16,
    },
    // MRC1 is documented as unused on S32G3; deliberately not listed.
    // MRC2: SRAM_0..3 (system SRAM bank A).
    MrcRange {
        idx: 2,
        start: 0x3400_0000,
        end: 0x344F_FFFF,
        nmrgd: 16,
    },
    // MRC3: SRAM_4..7.
    MrcRange {
        idx: 3,
        start: 0x3450_0000,
        end: 0x349F_FFFF,
        nmrgd: 16,
    },
    // MRC4: SRAM_8..11.
    MrcRange {
        idx: 4,
        start: 0x34A0_0000,
        end: 0x34EF_FFFF,
        nmrgd: 16,
    },
    // MRC5: SRAM_12..15.
    MrcRange {
        idx: 5,
        start: 0x34F0_0000,
        end: 0x353F_FFFF,
        nmrgd: 16,
    },
    // MRC6: Ncore registers + coherent PCIe_0/GMAC_0.
    MrcRange {
        idx: 6,
        start: 0x5040_0000,
        end: 0x504F_FFFF,
        nmrgd: 16,
    },
    // MRC7: S_FLASH (external QuadSPI) — two windows.
    MrcRange {
        idx: 7,
        start: 0x0000_0000,
        end: 0x1FFF_FFFF,
        nmrgd: 16,
    },
    MrcRange {
        idx: 7,
        start: 0x4100_0000,
        end: 0x417F_FFFF,
        nmrgd: 16,
    },
    // MRC8: S_LLCE.
    MrcRange {
        idx: 8,
        start: 0x4300_0000,
        end: 0x43FF_FFFF,
        nmrgd: 16,
    },
    // MRC9: S_GIC-500.
    MrcRange {
        idx: 9,
        start: 0x5080_0000,
        end: 0x509F_FFFF,
        nmrgd: 16,
    },
    // MRC10: PCIe_0 lower window (the upper window 0x58_0000_0000-0x5F_FFFF_FFFF
    // is 40-bit-only; outside the 32-bit `u32` address space exposed here).
    MrcRange {
        idx: 10,
        start: 0x5300_0000,
        end: 0x53FF_FFFF,
        nmrgd: 16,
    },
    // MRC11: M7 TCM 0-3 as visible from the system bus (NOT from the M7's own
    // ITCM/DTCM at 0x0/0x2000_0000 which bypass XRDC entirely).
    MrcRange {
        idx: 11,
        start: 0x2010_0000,
        end: 0x2010_FFFF,
        nmrgd: 12,
    },
    MrcRange {
        idx: 11,
        start: 0x2018_0000,
        end: 0x2018_FFFF,
        nmrgd: 12,
    },
    MrcRange {
        idx: 11,
        start: 0x2020_0000,
        end: 0x2020_FFFF,
        nmrgd: 12,
    },
    MrcRange {
        idx: 11,
        start: 0x2028_0000,
        end: 0x2028_FFFF,
        nmrgd: 12,
    },
    // MRC12: NoC_0 config.
    MrcRange {
        idx: 12,
        start: 0x5000_0000,
        end: 0x5007_FFFF,
        nmrgd: 4,
    },
    // MRC13: S_DBG_APB.
    MrcRange {
        idx: 13,
        start: 0x5100_0000,
        end: 0x51FF_FFFF,
        nmrgd: 16,
    },
];

// =============================================================================
// Per-instance newtype wrappers (compile-time routing to Xrdc0)
// =============================================================================

/// PDAC entry bound to XRDC_0's [`Peripheral`] population.
///
/// `repr(transparent)` over [`PdacRaw`] so const-folded entries lower to the
/// same register words as the chip-crate-internal representation.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Pdac(PdacRaw);

impl Pdac {
    /// New PDAC entry for `peripheral`, with no domains granted (all `DxACP =
    /// 000b`). Compose `.grant()` calls to add policy.
    pub const fn new(peripheral: Peripheral) -> Self {
        Self(PdacRaw::new(peripheral as u16))
    }

    /// Grant `access` to `domain` on this peripheral.
    pub const fn grant(self, domain: Domain, access: Access) -> Self {
        Self(self.0.grant(domain, access))
    }

    /// Read the global PDAC slot this entry targets. Used by [`Config::new`]
    /// for duplicate detection at const-eval.
    pub const fn slot(&self) -> u16 {
        self.0.slot
    }
}

/// MRGD entry bound to XRDC_0's MRC address-coverage table.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Mrgd(MrgdRaw);

impl Mrgd {
    /// New 32-byte-aligned memory region descriptor. Const-asserts at the
    /// declaration site that the range lies in exactly one of [`MRC_RANGES`].
    pub const fn region(start: u32, end: u32) -> Self {
        Self(MrgdRaw::region(start, end, MRC_RANGES))
    }

    /// Grant `access` to `domain` on this region.
    pub const fn grant(self, domain: Domain, access: Access) -> Self {
        Self(self.0.grant(domain, access))
    }

    /// MRC index this region maps to. Used by [`Config::new`] for the per-MRC
    /// descriptor-budget check at const-eval.
    pub const fn mrc(&self) -> u8 {
        self.0.mrc
    }
}

/// MDA entry bound to XRDC_0's [`Master`] population.
///
/// The DFMT bit is picked automatically from [`Master::is_bus`]; boards
/// never write it directly.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Mda(MdaRaw);

impl Mda {
    /// Assign `master` to `domain`. Bus initiators get DFMT1 + `SA = UseInput`
    /// + `PA = UseInput`; core initiators get DFMT0 + `PE = 00b`
    ///   (PID matching off) + `DIDS = 00b` (DID from this MDA word).
    pub const fn assign(master: Master, domain: Domain) -> Self {
        if master.is_bus() {
            Self(MdaRaw::bus(master as u8, domain))
        } else {
            Self(MdaRaw::core(master as u8, domain))
        }
    }

    /// Read the master this entry assigns. Used by [`Config::new`] for
    /// duplicate detection at const-eval.
    pub const fn master_idx(&self) -> u8 {
        self.0.master_idx
    }
}

// Bus-only override methods. We can't express "this method only exists when
// `Master::is_bus()` returns true at const-eval" through `where` clauses
// (Rust doesn't have value-dependent type bounds), so the bus-vs-core check
// is enforced by const-fn assertions inside `MdaRaw::with_sa` / `with_pa`,
// which panic at const-eval — i.e. compile time — for core initiators.
//
// These methods are exposed at the `Mda` newtype level so the chain
// `Mda::assign(Master::EDma0, Domain::M7_0).force_secure().force_privileged()`
// reads naturally.
impl Mda {
    /// Force the secure attribute on this bus initiator's outgoing
    /// transactions, regardless of what the master itself drives.
    ///
    /// Const-asserts (at const-eval) that `master.is_bus()` — applying this
    /// to a core initiator like [`Master::M7_0Axi`] is a **compile error**
    /// inside the board's `const XRDC_0_CFG = …` declaration.
    pub const fn force_secure(self) -> Self {
        Self(self.0.with_sa(SecureAttr::ForceSecure))
    }
    /// Force nonsecure attribute. See [`Mda::force_secure`].
    pub const fn force_nonsecure(self) -> Self {
        Self(self.0.with_sa(SecureAttr::ForceNonsecure))
    }
    /// Force the privileged attribute. See [`Mda::force_secure`].
    pub const fn force_privileged(self) -> Self {
        Self(self.0.with_pa(PrivAttr::ForcePrivileged))
    }
    /// Force the user (unprivileged) attribute. See [`Mda::force_secure`].
    pub const fn force_user(self) -> Self {
        Self(self.0.with_pa(PrivAttr::ForceUser))
    }
}

// Marker trait — opt-in for documentation only on this v1 (the actual
// override gating happens via the const-fn assertions above). Defining it
// here keeps the surface symmetric with the chip-crate `BusInitiator` trait
// in `mod.rs` and makes it easy to grow into a `where M: BusInitiator`
// bound if Rust ever gains value-dependent trait selection.
impl BusInitiator for Master {}

// =============================================================================
// Config — declarative const table, fully validated at const-eval
// =============================================================================

/// Complete XRDC_0 policy.
///
/// Construct via [`Config::new`] which runs every cross-entry consistency
/// check at const-eval: duplicate masters/peripherals are compile errors,
/// per-MRC descriptor-budget overflows are compile errors.
pub struct Config<'a> {
    pub(crate) masters: &'a [Mda],
    pub(crate) peripherals: &'a [Pdac],
    pub(crate) regions: &'a [Mrgd],
}

impl<'a> Config<'a> {
    /// Build the config and validate it at const-eval.
    ///
    /// Const-asserts:
    /// * No two [`Mda`] entries target the same `Master`.
    /// * No two [`Pdac`] entries target the same `Peripheral`.
    /// * For each [`MrcRange::idx`] that any [`Mrgd`] resolves to, the
    ///   number of regions ≤ `MrcRange::nmrgd`.
    ///
    /// Every violation panics inside a `const fn`, which the Rust compiler
    /// reports as an error at the `const XRDC_0_CFG = Config::new(…);` line
    /// in the board crate.
    pub const fn new(masters: &'a [Mda], peripherals: &'a [Pdac], regions: &'a [Mrgd]) -> Self {
        Self::assert_unique_masters(masters);
        Self::assert_unique_peripherals(peripherals);
        Self::assert_mrc_budgets(regions);
        Self {
            masters,
            peripherals,
            regions,
        }
    }

    const fn assert_unique_masters(masters: &[Mda]) {
        let mut i = 0;
        while i < masters.len() {
            let mut j = i + 1;
            while j < masters.len() {
                if masters[i].master_idx() == masters[j].master_idx() {
                    panic!("xrdc_0::Config::new: duplicate Master in `masters` slice");
                }
                j += 1;
            }
            i += 1;
        }
    }

    const fn assert_unique_peripherals(peripherals: &[Pdac]) {
        let mut i = 0;
        while i < peripherals.len() {
            let mut j = i + 1;
            while j < peripherals.len() {
                if peripherals[i].slot() == peripherals[j].slot() {
                    panic!("xrdc_0::Config::new: duplicate Peripheral in `peripherals` slice");
                }
                j += 1;
            }
            i += 1;
        }
    }

    const fn assert_mrc_budgets(regions: &[Mrgd]) {
        // The highest MRC index referenced anywhere in XRDC_0 — bounds the
        // per-MRC counter array used by this check.
        let max = max_mrc_idx(MRC_RANGES) as usize;
        let mut counts = [0u8; 14]; // RM Table 30: 14 MRCs on XRDC_0
        assert!(
            max < counts.len(),
            "xrdc_0: MRC_RANGES references an MRC index larger than the counter array (chip-crate bug)"
        );
        let mut i = 0;
        while i < regions.len() {
            let mrc = regions[i].mrc() as usize;
            counts[mrc] += 1;
            i += 1;
        }
        let mut m = 0usize;
        while m <= max {
            let budget = nmrgd_for_mrc(m as u8, MRC_RANGES);
            if counts[m] > budget {
                panic!(
                    "xrdc_0::Config::new: too many MRGDs for one MRC (exceeds NMRGD budget — see RM §15.2.4)"
                );
            }
            m += 1;
        }
    }
}

// =============================================================================
// Driver
// =============================================================================

/// XRDC_0 driver — programs the entire policy from a [`Config`] declared by
/// the board, then enables and locks.
pub struct Xrdc0 {
    registers: StaticRef<XrdcRegisters>,
}

impl Xrdc0 {
    /// Hardcoded XRDC_0 base — one instance per chip.
    pub const fn new() -> Self {
        Self {
            registers: XRDC_0_BASE,
        }
    }

    /// Program XRDC_0 from `cfg` and lock it for the duration of this power
    /// cycle. Behaviour:
    ///
    /// 1. If `CR[LK1]` is already set → `panic!("xrdc_0 already locked")`.
    ///    The only way to recover is a reset (which our 5-second
    ///    rebuild-and-reflash workflow handles).
    /// 2. `CR[GVLD] = 0` — disable XRDC policy evaluation while the policy is
    ///    rewritten.
    /// 3. Invalidate every MDA word, PDAC slot, and MRGD descriptor
    ///    (`VLD = Invalid`). This realises the **deny-by-default** semantic:
    ///    anything `cfg` doesn't mention is blocked once `GVLD` goes high.
    ///    `cfg.regions` using the W3/W1-revalidation dance documented in RM
    ///    §15.7.3. MRGD slot allocation within each MRC is sequential (first
    ///    region for MRC `c` lands in `MRGD[c*16+0]`, second in `MRGD[c*16+1]`, etc.).
    /// 5. `CR[GVLD] = 1` — XRDC evaluation atomically turns on with the new
    ///    policy.
    /// 6. Lock everything: per-entry `LK1`/`LK2` and finally `CR[LK1]`.
    ///
    /// `&self` is sufficient because all writes are atomic at the bus level
    /// and the driver does not maintain any internal state between steps.
    pub fn apply(&self, cfg: &Config<'_>) {
        let regs: &XrdcRegisters = &self.registers;

        // 1. Refuse to silently NOP on a locked instance.
        if regs.cr.is_set(CR::LK1) {
            panic!("xrdc_0::Xrdc0::apply: CR[LK1] is already set — reflash to reconfigure XRDC_0");
        }

        // 2. Disable evaluation while we rewrite the policy.
        regs.cr.modify(CR::GVLD::Disabled);
        register_barrier();

        // 3. Deny-by-default: zero every entry's VLD before programming.
        invalidate_mda(&regs.mda);
        invalidate_pdac_window(&regs.pdac_0_31);
        invalidate_pdac_window(&regs.pdac_128_161);
        invalidate_pdac_window(&regs.pdac_256_289);
        invalidate_pdac_window(&regs.pdac_384_408);
        invalidate_pdac_window(&regs.pdac_512_542);
        invalidate_mrgd(&regs.mrgd);
        register_barrier();

        // 4a. Program MDA entries.
        for entry in cfg.masters {
            let raw = entry.0;
            let slot = &regs.mda[raw.master_idx as usize];
            if raw.is_bus {
                program_mda_bus(slot, raw, /* lock = */ true);
            } else {
                program_mda_core(slot, raw, /* lock = */ true);
            }
        }

        // 4b. Program PDAC entries.
        for entry in cfg.peripherals {
            let raw = entry.0;
            let pdac = pdac_register_for_slot(regs, raw.slot);
            program_pdac(pdac, raw, /* lock = */ true);
        }

        // 4c. Program MRGD entries — sequential allocation within each MRC's
        // 16-slot window (RM §15.7.3.18 layout). Per-MRC counter sized for
        // the 14 MRCs on XRDC_0.
        let mut per_mrc = [0u8; 14];
        for entry in cfg.regions {
            let raw = entry.0;
            let mrc_idx = raw.mrc as usize;
            let slot_in_mrc = per_mrc[mrc_idx];
            per_mrc[mrc_idx] += 1;
            // Each MRC owns 16 contiguous MRGD entries (RM Table 30 — NMRGD
            // ≤ 16 even when only 4 or 12 descriptors are documented as
            // functional; the unused tail is reserved space). Config::new
            // has already const-asserted slot_in_mrc < NMRGD[mrc_idx].
            let mrgd = &regs.mrgd[mrc_idx * 16 + slot_in_mrc as usize];
            program_mrgd(mrgd, raw, /* lock = */ true);
        }

        // 5. Atomically enable XRDC evaluation with the new policy.
        register_barrier();
        regs.cr.modify(CR::GVLD::Enabled);
        register_barrier();

        // 6. Lock the control register so no further mutation is possible
        // until reset. (Per-entry LK1/LK2 were set above in their program_*
        // calls.)
        regs.cr.modify(CR::LK1::Locked);
    }
    /// Additive patch: program only the entries listed in `cfg` without
    /// Additive patch: program only the entries listed in `cfg` without
    /// invalidating any existing MDA/PDAC/MRGD slots. This preserves the
    /// existing base policy while granting additional domains access to
    /// Tock-managed peripherals and memory regions.
    ///
    /// Behaviour:
    /// 1. If `CR[LK1]` is already set → panic.
    /// 2. Does **not** touch `CR[GVLD]` — evaluation stays enabled globally.
    /// 3. Does **not** invalidate any entry.
    /// 4. For each PDAC: reads existing W0/W1, ORs in ACP bits, cycles VLD.
    /// 5. For each MDA: reads existing word, clears only the fields managed
    ///    by the driver (DID for core; DFMT/DID/DIDB/SA/PA for bus),
    ///    writes new values, sets VLD.
    /// 6. For each MRGD: searches the target MRC for an existing descriptor
    ///    with matching address range; if found, ORs in ACP bits. Otherwise
    ///    allocates in the first unused slot.
    /// 7. Does **not** set per-entry LK1/LK2.
    pub fn patch(&self, cfg: &Config<'_>) -> Result<(), XrdcPatchError> {
        let regs: &XrdcRegisters = &self.registers;

        if regs.cr.is_set(CR::LK1) {
            return Err(XrdcPatchError::LockedDescriptor);
        }

        for entry in cfg.masters {
            let raw = entry.0;
            super::patch_mda(&regs.mda[raw.master_idx as usize], raw)?;
        }

        for entry in cfg.peripherals {
            let raw = entry.0;
            super::patch_pdac(super::pdac_register_for_slot(regs, raw.slot), raw)?;
        }

        for entry in cfg.regions {
            let raw = entry.0;
            super::patch_mrgd(&regs.mrgd, raw, nmrgd_for_mrc(raw.mrc, MRC_RANGES) as usize)?;
        }
        Ok(())
    }

    /// Lock the XRDC control register (`CR[LK1]`). After this call no
    /// further `apply()` or `patch()` is possible until reset.
    ///
    /// Idempotent: multiple calls are harmless.
    pub fn lock(&self) {
        let regs: &XrdcRegisters = &self.registers;
        if !regs.cr.is_set(CR::LK1) {
            regs.cr.modify(CR::LK1::Locked);
        }
    }
    /// Runtime search-and-patch for a single MRGD descriptor.
    ///
    /// Searches XRDC_0's MRC window for an existing descriptor (according to
    /// `target`) and ORs in the ACP bits from `entry`.  If no match is found,
    /// allocates the first unused slot in the target MRC.
    ///
    /// This is an imperative escape hatch for cases where a prior boot stage
    /// (e.g. a boot ROM or secure monitor) programs MRGDs with run-time-determined
    /// bounds that Tock cannot know at compile time.  For fully static policy,
    /// prefer the declarative [`Config`] + [`Xrdc0::apply`] / [`Xrdc0::patch`] path.
    ///
    /// Never touches lock bits (`LK1`/`LK2`) on the descriptor.
    pub fn search_and_patch_mrgd(
        &self,
        target: MrgdTarget,
        entry: &Mrgd,
    ) -> Result<MrgdPatchOutcome, XrdcPatchError> {
        let raw = entry.0;
        let nmrgd = nmrgd_for_mrc(raw.mrc, MRC_RANGES) as usize;
        search_and_patch_mrgd(&self.registers.mrgd, raw, target, nmrgd)
    }
}

// =============================================================================
// Doctest assertions for compile-time guarantees
// =============================================================================

/// Const-eval rejection: applying `force_secure` to a core (M7_0) initiator is
/// a compile error.
///
/// ```compile_fail
/// use nxp_s32g3::xrdc::Domain;
/// use nxp_s32g3::xrdc::xrdc_0::{Mda, Master};
/// const _BAD: Mda = Mda::assign(Master::M7_0Axi, Domain::M7_0).force_secure();
/// ```
#[cfg(doc)]
fn _force_secure_on_core_is_compile_error() {}

/// Const-eval rejection: misaligned MRGD start address is a compile error.
///
/// ```compile_fail
/// use nxp_s32g3::xrdc::xrdc_0::Mrgd;
/// // 0x3420_0001 is not 32-byte aligned.
/// const _BAD: Mrgd = Mrgd::region(0x3420_0001, 0x342F_FFFF);
/// ```
#[cfg(doc)]
fn _misaligned_region_is_compile_error() {}

/// Const-eval rejection: address range that crosses MRC boundaries.
///
/// ```compile_fail
/// use nxp_s32g3::xrdc::xrdc_0::Mrgd;
/// // SRAM_0..3 ends at 0x344F_FFFF; the range below crosses into SRAM_4..7
/// // (a different MRC), which a single MRGD cannot cover.
/// const _BAD: Mrgd = Mrgd::region(0x3440_0000, 0x3450_FFFF);
/// ```
#[cfg(doc)]
fn _cross_mrc_region_is_compile_error() {}

/// Const-eval rejection: duplicate peripheral in the PDAC slice.
///
/// ```compile_fail
/// use nxp_s32g3::xrdc::{Access::*, Domain};
/// use nxp_s32g3::xrdc::xrdc_0::{Config, Pdac, Peripheral::*};
/// const _BAD: Config = Config::new(
///     &[],
///     &[
///         Pdac::new(LinFlexD0).grant(Domain::M7_0, FullRw),
///         Pdac::new(LinFlexD0).grant(Domain::A53,  FullRw),
///     ],
///     &[],
/// );
/// ```
#[cfg(doc)]
fn _duplicate_peripheral_is_compile_error() {}
