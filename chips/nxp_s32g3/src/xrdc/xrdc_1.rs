// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! XRDC_1 — the Accelerator-instance XRDC on S32G3.
//!
//! Covers the off-chassis NoC masters (RM §15.3.2 Table 34: PCIe_1, PFE
//! host interfaces 0..3 + PFE_DDR + PFE_UTIL, USB), the 6 MRC submodules
//! that protect their target windows (RM §15.3.3 Table 35), and `PAC0`
//! (RM §15.3.4 Table 36 — peripheral group 4, 14 documented slots
//! at `0x4400_0000..0x440F_FFFF`).
//!
//! Boards build a [`Config`] from [`Pdac`] / [`Mda`] / [`Mrgd`] entries,
//! declare it as a `const`, and pass it to [`Xrdc1::apply`]. All validation
//! happens at const-eval; the only runtime failure mode is `apply()` called
//! on an already-locked instance, which panics (5-second reflash workflow).
//!
//! # Cross-instance separation
//!
//! XRDC_1's [`Master`] and [`Peripheral`] populations are disjoint from
//! XRDC_0's — `xrdc_0::Master::M7_0Axi` is a different type from
//! `xrdc_1::Master::Pcie1` and the compiler refuses to put one into the
//! other's [`Config`]. Both instances share the chip-crate-wide [`Domain`]
//! enum so PFE host interfaces (D12..D15) flow through naturally.
//!
//! # PFE host-interface tagging
//!
//! RM §15.3.5 Table 37 fixes the PFE per-HIF DIDs at `0xC..0xF`. To
//! propagate them through `XRDC_MDAC1..4`, board code MUST chain
//! [`Mda::with_didb_use_input`] on each PFE_HIF master so MDA `DIDB`
//! switches to `UseInput` and the PFE's per-transaction DID overrides
//! the static `DID` field on that MDA word.
//!
//! # Example
//!
//! ```ignore
//! use nxp_s32g3::xrdc::{Access::*, Domain};
//! use nxp_s32g3::xrdc::xrdc_1::{Config, Mda, Master, Mrgd, Pdac, Peripheral::*, Xrdc1};
//!
//! // Allow PCIe_1 access to its own config window; PFE host interfaces
//! // each get their own DID for the PFE address window.
//! const XRDC_1_CFG: Config = Config::new(
//!     /* masters     */ &[
//!         Mda::assign(Master::Pcie1, Domain::M7_0),
//!         Mda::assign(Master::PfeHifBdFetch,  Domain::PfeHif0).with_didb_use_input(),
//!         Mda::assign(Master::PfeHifBdUpdate, Domain::PfeHif0).with_didb_use_input(),
//!         Mda::assign(Master::PfeHifDataWrite, Domain::PfeHif0).with_didb_use_input(),
//!         Mda::assign(Master::PfeHifDataRead,  Domain::PfeHif0).with_didb_use_input(),
//!     ],
//!     /* peripherals */ &[
//!         Pdac::new(Xrdc1).grant(Domain::M7_0, FullRw),  // self-reference for apply() to lock
//!         Pdac::new(Pcie1).grant(Domain::M7_0, SupervisorRw),
//!     ],
//!     /* regions     */ &[
//!         Mrgd::region(0x4500_0000, 0x45FF_FFFF).grant(Domain::M7_0, FullRw), // SerDes_1/PCIe_1
//!     ],
//! );
//!
//! // In start():
//! // let xrdc1 = static_init!(Xrdc1, Xrdc1::new());
//! // xrdc1.apply(&XRDC_1_CFG);
//! ```

use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::StaticRef;

use super::{
    allocate_unmapped_exact_mrgd, invalidate_mda, invalidate_mrgd, invalidate_pdac_window,
    max_mrc_idx, nmrgd_for_mrc, pdac_register_for_slot, program_mda_bus, program_mda_core,
    program_mrgd, program_pdac, register_barrier, search_and_patch_mrgd, Access, BusInitiator,
    Domain, MdaRaw, MrcRange, MrgdPatchOutcome, MrgdRaw, MrgdTarget, PdacRaw, PrivAttr, SecureAttr,
    XrdcPatchError, XrdcRegisters, CR,
};

/// Base address of XRDC_1 (RM §15.7.4.1).
pub const XRDC_1_BASE: StaticRef<XrdcRegisters> = super::XRDC_1_BASE;

/// Number of MDA submodules exposed by XRDC_1 (RM §15.3.2 Table 34). Bounds
/// the slice [`Xrdc1::apply`] passes to [`invalidate_mda`] so it never
/// writes past XRDC_1's documented register window into reserved space.
const MDA_INSTANCE_COUNT: usize = 8;
/// Number of MRC submodules on XRDC_1 (RM §15.3.3 Table 35). Bounds the
/// `[u8; _]` counter array in [`Config::new`] and the MRGD slice that
/// `apply` invalidates.
const MRC_COUNT: usize = 6;
/// Each MRC owns 16 contiguous MRGD entries in the shared register layout
/// (RM §15.7.3.18 / §15.7.4.18). Total MRGD descriptors on XRDC_1 =
/// `MRC_COUNT * MRGD_PER_MRC`.
const MRGD_PER_MRC: usize = 16;

// =============================================================================
// Peripheral enumeration (PDAC slots)
// =============================================================================

/// XRDC_1-protected peripherals (Peripheral Group 4 / `XRDC_1 PAC0`).
///
/// 14 slots). Each variant's discriminant is its PDAC slot number per the
/// S32G3 Reference Manual memory map
#[repr(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Peripheral {
    /// `SIUL2_1` off-chassis pin-mux + IMCR at `0x4401_0000`. PDAC slot 0.
    Siul21 = 0,
    /// `XRDC_1` self-reference at `0x4400_4000`. PDAC slot 1. **Required**:
    /// `apply()` programs lock bits on every PDAC entry _after_ `GVLD=1`,
    /// so the M7 must have continued write access to XRDC_1's own register
    /// block after the policy turns on.
    Xrdc1 = 1,
    /// `STM_TS` (off-chassis system-timer / timestamp) at `0x4400_C000`.
    /// PDAC slot 2.
    StmTs = 2,
    /// `MC_CGM_2` clock-generation module (PFE / GMAC clock domain) at
    /// `0x4401_8000`. PDAC slot 3.
    McCgm2 = 3,
    /// Standby SRAM controller config window at `0x4402_8000`. PDAC slot 4.
    StdbySramCfg = 4,
    /// USB controller at `0x4406_4000`. PDAC slot 5.
    Usb = 5,
    /// PCIe_1 controller at `0x4410_0000`. PDAC slot 6.
    Pcie1 = 6,
    /// ERM-LLCE (Error Reporting Module — LLCE) at `0x4403_0000`. PDAC slot 8.
    ErmLlce = 8,
    /// ERM-PFE (Error Reporting Module — PFE master windows 0..15) at
    /// `0x4403_4000..0x4404_3FFF`. PDAC slot 9.
    ErmPfe = 9,
    /// EIM_MISC (Error Injection Module — miscellaneous) at `0x4404_C000`.
    /// PDAC slot 10.
    EimMisc = 10,
    /// EIM_LLCE (Error Injection Module — LLCE) at `0x4405_0000`. PDAC slot 11.
    EimLlce = 11,
    /// EIM-PFE (Error Injection Module — PFE 0..6 windows) at
    /// `0x4405_4000..0x4405_AFFF`. PDAC slot 12.
    EimPfe = 12,
    /// ERM-Standby SRAM at `0x4404_4000`. PDAC slot 13.
    ErmStandbySram = 13,
}

// =============================================================================
// Master enumeration (MDAC initiators)
// =============================================================================

/// XRDC_1 bus-initiator (MDAC) slots. Discriminant = MDAC submodule index
/// per RM §15.3.2 Table 34.
///
/// Every XRDC_1 master is a non-processor bus initiator (DFMT1), so all
/// variants implement [`BusInitiator`] and the `.force_secure()` /
/// `.force_privileged()` / `.with_didb_use_input()` builder methods are
/// always available on [`Mda`].
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Master {
    /// PCIe_1 (RM XRDC_MDAC0). Bus initiator (DFMT1).
    Pcie1 = 0,
    /// PFE_HIF buffer-descriptor fetch (RM XRDC_MDAC1). Bus initiator,
    /// PFE-tagged DID per RM §15.3.5 — board MUST chain `with_didb_use_input`.
    PfeHifBdFetch = 1,
    /// PFE_HIF buffer-descriptor update (RM XRDC_MDAC2). See `PfeHifBdFetch`.
    PfeHifBdUpdate = 2,
    /// PFE_HIF data write (RM XRDC_MDAC3). See `PfeHifBdFetch`.
    PfeHifDataWrite = 3,
    /// PFE_HIF data read (RM XRDC_MDAC4). See `PfeHifBdFetch`.
    PfeHifDataRead = 4,
    /// PFE_DDR (alias PFE_PKT_MASTER) (RM XRDC_MDAC5). Bus initiator.
    PfeDdr = 5,
    /// PFE_UTIL (RM XRDC_MDAC6). Bus initiator.
    PfeUtil = 6,
    /// USB controller (RM XRDC_MDAC7). Bus initiator.
    Usb = 7,
}

impl Master {
    /// All XRDC_1 masters are bus initiators (DFMT1) — RM §15.3.2 Table 34
    /// lists every entry as Nonprocessor.
    pub const fn is_bus(self) -> bool {
        true
    }
}

// =============================================================================
// MRC address-coverage table for XRDC_1 (RM §15.3.3 Table 35)
// =============================================================================

/// Address-coverage windows of every XRDC_1 MRC.
///
/// MRC2 is intentionally omitted: RM Table 35 lists it as "coherent accesses
/// from PCIe_1 and PFE" with target ranges defined dynamically by the NoC
/// coherency configuration, not a fixed address window. A future board that
/// needs to grant a coherent MRGD must add the resolved address range here
/// with an explicit citation to its NoC setup.
pub const MRC_RANGES: &[MrcRange] = &[
    // MRC0: STDBY_SRAM target window.
    MrcRange {
        idx: 0,
        start: 0x2400_0000,
        end: 0x33FF_FFFF,
        nmrgd: 4,
    },
    // MRC1: LLCE (accesses from XRDC_1 masters — distinct from XRDC_0's MRC8
    // grant which covers M7/A53/eDMA access to the same physical window).
    MrcRange {
        idx: 1,
        start: 0x4300_0000,
        end: 0x43FF_FFFF,
        nmrgd: 4,
    },
    // MRC3: SerDes_1 register window (PCIe_1 controller config).
    MrcRange {
        idx: 3,
        start: 0x4500_0000,
        end: 0x45FF_FFFF,
        nmrgd: 4,
    },
    // MRC4: PFE register window.
    MrcRange {
        idx: 4,
        start: 0x4600_0000,
        end: 0x46FF_FFFF,
        nmrgd: 8,
    },
    // MRC5: NoC_1 config window.
    MrcRange {
        idx: 5,
        start: 0x4700_0000,
        end: 0x4707_FFFF,
        nmrgd: 4,
    },
];

// =============================================================================
// Per-instance newtype wrappers (compile-time routing to Xrdc1)
// =============================================================================

/// PDAC entry bound to XRDC_1's [`Peripheral`] population.
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

/// MRGD entry bound to XRDC_1's MRC address-coverage table.
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

/// MDA entry bound to XRDC_1's [`Master`] population.
///
/// All XRDC_1 masters are DFMT1 (bus initiators), so the override builder
/// methods (`force_secure`, `force_privileged`, `with_didb_use_input`) are
/// always available without per-variant gating.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Mda(MdaRaw);

impl Mda {
    /// Assign `master` to `domain`. Defaults: `SA = UseInput`, `PA =
    /// UseInput`, `DIDB = BypassInput` (DID taken from this MDA word). Chain
    /// [`with_didb_use_input`](Self::with_didb_use_input) on PFE_HIF masters
    /// to let the PFE's per-transaction DID propagate.
    pub const fn assign(master: Master, domain: Domain) -> Self {
        Self(MdaRaw::bus(master as u8, domain))
    }

    /// Read the master this entry assigns. Used by [`Config::new`] for
    /// duplicate detection at const-eval.
    pub const fn master_idx(&self) -> u8 {
        self.0.master_idx
    }

    /// Force the secure attribute on this master's outgoing transactions,
    /// regardless of what the master itself drives.
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
    /// Flip `DIDB` to `UseInput` so the bus master's per-transaction input
    /// DID propagates. Required for PFE_HIF masters per RM §15.3.5 (PFE
    /// updates DID between HIF 0..3 → DID 0xC..0xF).
    pub const fn with_didb_use_input(self) -> Self {
        Self(self.0.with_didb_use_input())
    }
}

// All XRDC_1 masters are bus initiators — see [`Master::is_bus`].
impl BusInitiator for Master {}

// =============================================================================
// Config — declarative const table, fully validated at const-eval
// =============================================================================

/// Complete XRDC_1 policy.
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
    /// reports as an error at the `const XRDC_1_CFG = Config::new(…);` line
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
                    panic!("xrdc_1::Config::new: duplicate Master in `masters` slice");
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
                    panic!("xrdc_1::Config::new: duplicate Peripheral in `peripherals` slice");
                }
                j += 1;
            }
            i += 1;
        }
    }

    const fn assert_mrc_budgets(regions: &[Mrgd]) {
        // The highest MRC index referenced anywhere in XRDC_1 — bounds the
        // per-MRC counter array used by this check.
        let max = max_mrc_idx(MRC_RANGES) as usize;
        let mut counts = [0u8; MRC_COUNT]; // RM Table 35: 6 MRCs on XRDC_1
        assert!(
            max < counts.len(),
            "xrdc_1: MRC_RANGES references an MRC index larger than the counter array (chip-crate bug)"
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
                    "xrdc_1::Config::new: too many MRGDs for one MRC (exceeds NMRGD budget — see RM §15.3.3 Table 35)"
                );
            }
            m += 1;
        }
    }
}

// =============================================================================
// Driver
// =============================================================================

/// XRDC_1 driver — programs the entire policy from a [`Config`] declared by
/// the board, then enables and locks.
pub struct Xrdc1 {
    registers: StaticRef<XrdcRegisters>,
}

impl Xrdc1 {
    /// Hardcoded XRDC_1 base — one instance per chip.
    pub const fn new() -> Self {
        Self {
            registers: XRDC_1_BASE,
        }
    }

    /// Program XRDC_1 from `cfg` and lock it for the duration of this power
    /// cycle. Behaviour mirrors [`super::xrdc_0::Xrdc0::apply`] but bounds
    /// every register-block iteration to XRDC_1's documented MDAC/MRC/PAC
    /// counts (MDA_INSTANCE_COUNT, MRC_COUNT × MRGD_PER_MRC,
    /// `XrdcRegisters::pdac_0_31` only — RM §15.3.4 Table 36: XRDC_1 has
    /// just one PAC group) so writes never spill into reserved register
    /// space.
    pub fn apply(&self, cfg: &Config<'_>) {
        let regs: &XrdcRegisters = &self.registers;

        // 1. Refuse to silently NOP on a locked instance.
        if regs.cr.is_set(CR::LK1) {
            panic!("xrdc_1::Xrdc1::apply: CR[LK1] is already set — reflash to reconfigure XRDC_1");
        }

        // 2. Disable evaluation while we rewrite the policy.
        regs.cr.modify(CR::GVLD::Disabled);
        register_barrier();

        // 3. Deny-by-default: zero every entry's VLD before programming, but
        //    only within XRDC_1's documented register window. Touching the
        //    XRDC_0-only PAC1..PAC4 windows or MDA/MRGD slots beyond the
        //    instance's count would write reserved register space.
        invalidate_mda(&regs.mda[..MDA_INSTANCE_COUNT]);
        invalidate_pdac_window(&regs.pdac_0_31);
        invalidate_mrgd(&regs.mrgd[..MRC_COUNT * MRGD_PER_MRC]);
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
        // 16-slot window (RM §15.7.4.18 layout). Per-MRC counter sized for
        // the 6 MRCs on XRDC_1.
        let mut per_mrc = [0u8; MRC_COUNT];
        for entry in cfg.regions {
            let raw = entry.0;
            let mrc_idx = raw.mrc as usize;
            let slot_in_mrc = per_mrc[mrc_idx];
            per_mrc[mrc_idx] += 1;
            // Each MRC owns 16 contiguous MRGD entries (RM Table 35 — NMRGD
            // ≤ 16 even when only 4 or 8 descriptors are documented as
            // functional; the unused tail is reserved space). Config::new
            // has already const-asserted slot_in_mrc < NMRGD[mrc_idx].
            let mrgd = &regs.mrgd[mrc_idx * MRGD_PER_MRC + slot_in_mrc as usize];
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
    /// Additive patch for XRDC_1. Mirrors [`super::xrdc_0::Xrdc0::patch`]
    /// but bounds register access to XRDC_1's documented MDAC/MRC/PAC counts.
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
            super::patch_mrgd(
                &regs.mrgd[..MRC_COUNT * MRGD_PER_MRC],
                raw,
                nmrgd_for_mrc(raw.mrc, MRC_RANGES) as usize,
            )?;
        }
        Ok(())
    }

    /// Lock the XRDC_1 control register (`CR[LK1]`). Idempotent.
    pub fn lock(&self) {
        let regs: &XrdcRegisters = &self.registers;
        if !regs.cr.is_set(CR::LK1) {
            regs.cr.modify(CR::LK1::Locked);
        }
    }
    /// Runtime search-and-patch for a single MRGD descriptor.
    ///
    /// Searches XRDC_1's MRC window for an existing descriptor (according to
    /// `target`) and ORs in the ACP bits from `entry`.  If no match is found,
    /// allocates the first unused slot in the target MRC.
    ///
    /// This is an imperative escape hatch for cases where a prior boot stage
    /// (e.g. a boot ROM or secure monitor) programs MRGDs with run-time-determined
    /// bounds that Tock cannot know at compile time.  For fully static policy,
    /// prefer the declarative [`Config`] + [`Xrdc1::apply`] / [`Xrdc1::patch`] path.
    ///
    /// Never touches lock bits (`LK1`/`LK2`) on the descriptor.
    pub fn search_and_patch_mrgd(
        &self,
        target: MrgdTarget,
        entry: &Mrgd,
    ) -> Result<MrgdPatchOutcome, XrdcPatchError> {
        let raw = entry.0;
        let mrgd_slice = &self.registers.mrgd[..MRC_COUNT * MRGD_PER_MRC];
        let nmrgd = nmrgd_for_mrc(raw.mrc, MRC_RANGES) as usize;
        search_and_patch_mrgd(mrgd_slice, raw, target, nmrgd)
    }

    /// Allocate a static descriptor only after proving that no valid MRC
    /// descriptor overlaps it. Intended solely for cold boot recovery when a
    /// predecessor policy does not own the requested range.
    pub fn allocate_unmapped_exact_mrgd(
        &self,
        entry: &Mrgd,
    ) -> Result<MrgdPatchOutcome, XrdcPatchError> {
        let raw = entry.0;
        let mrgd_slice = &self.registers.mrgd[..MRC_COUNT * MRGD_PER_MRC];
        let nmrgd = nmrgd_for_mrc(raw.mrc, MRC_RANGES) as usize;
        allocate_unmapped_exact_mrgd(mrgd_slice, raw, nmrgd)
    }
}

// =============================================================================
// Doctest assertions for compile-time guarantees
// =============================================================================

/// Const-eval rejection: misaligned MRGD start address is a compile error.
///
/// ```compile_fail
/// use nxp_s32g3::xrdc::xrdc_1::Mrgd;
/// // 0x4600_0001 is not 32-byte aligned.
/// const _BAD: Mrgd = Mrgd::region(0x4600_0001, 0x46FF_FFFF);
/// ```
#[cfg(doc)]
fn _misaligned_region_is_compile_error() {}

/// Const-eval rejection: address range that crosses MRC boundaries.
///
/// ```compile_fail
/// use nxp_s32g3::xrdc::xrdc_1::Mrgd;
/// // SerDes_1 (MRC3) ends at 0x45FF_FFFF; PFE (MRC4) starts at 0x4600_0000.
/// // A single MRGD cannot straddle two MRCs.
/// const _BAD: Mrgd = Mrgd::region(0x4500_0000, 0x4600_001F);
/// ```
#[cfg(doc)]
fn _cross_mrc_region_is_compile_error() {}

/// Const-eval rejection: duplicate peripheral in the PDAC slice.
///
/// ```compile_fail
/// use nxp_s32g3::xrdc::{Access::*, Domain};
/// use nxp_s32g3::xrdc::xrdc_1::{Config, Pdac, Peripheral::*};
/// const _BAD: Config = Config::new(
///     &[],
///     &[
///         Pdac::new(Pcie1).grant(Domain::M7_0, FullRw),
///         Pdac::new(Pcie1).grant(Domain::A53,  FullRw),
///     ],
///     &[],
/// );
/// ```
#[cfg(doc)]
fn _duplicate_peripheral_is_compile_error() {}

/// Const-eval rejection: cross-instance type confusion. `xrdc_0::Mda` cannot
/// flow into an `xrdc_1::Config`.
///
/// ```compile_fail
/// use nxp_s32g3::xrdc::Domain;
/// use nxp_s32g3::xrdc::xrdc_0;
/// use nxp_s32g3::xrdc::xrdc_1::Config;
/// const _BAD: Config = Config::new(
///     &[xrdc_0::Mda::assign(xrdc_0::Master::M7_0Axi, Domain::M7_0)],
///     &[],
///     &[],
/// );
/// ```
#[cfg(doc)]
fn _cross_instance_master_is_compile_error() {}

// =============================================================================
// Host unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xrdc::{MDA_DFMT_SHIFT, MDA_DIDB_SHIFT, MDA_DID_SHIFT};
    // Bring PDAC_W1 / MRGD_W3 typed fields into scope for shift verification.
    use crate::xrdc::{MRGD_W3, PDAC_W0, PDAC_W1};

    /// All 8 XRDC_1 masters report `is_bus()=true` per RM §15.3.2 Table 34.
    #[test]
    fn every_master_is_a_bus_initiator() {
        for m in [
            Master::Pcie1,
            Master::PfeHifBdFetch,
            Master::PfeHifBdUpdate,
            Master::PfeHifDataWrite,
            Master::PfeHifDataRead,
            Master::PfeDdr,
            Master::PfeUtil,
            Master::Usb,
        ] {
            assert!(m.is_bus(), "{:?} must be a bus initiator", m as u8);
        }
    }

    /// `Mda::assign` produces a DFMT1 word with the correct DID for the
    /// canonical PFE_DDR master (DID 1 → M7_0 in this contrived example).
    #[test]
    fn mda_assign_produces_dfmt1_word() {
        let mda = Mda::assign(Master::PfeDdr, Domain::M7_0);
        let raw = mda.0;
        assert!(raw.is_bus);
        assert_eq!(raw.master_idx, Master::PfeDdr as u8);
        assert_ne!(
            raw.word & (1 << MDA_DFMT_SHIFT),
            0,
            "DFMT must be 1 for XRDC_1 masters"
        );
        assert_eq!(raw.word & 0xF, Domain::M7_0 as u32, "DID = M7_0");
        assert_eq!(
            raw.word & (1 << MDA_DIDB_SHIFT),
            0,
            "DIDB defaults to BypassInput"
        );
    }

    /// PFE_HIF masters need `with_didb_use_input` so PFE-tagged DIDs
    /// propagate; the bit lights up at MDA bit 8 (`DIDB`).
    #[test]
    fn pfe_hif_master_with_didb_use_input_sets_bit_8() {
        let mda = Mda::assign(Master::PfeHifBdFetch, Domain::PfeHif0).with_didb_use_input();
        let raw = mda.0;
        assert_ne!(
            raw.word & (1 << MDA_DIDB_SHIFT),
            0,
            "with_didb_use_input must set MDA[DIDB]=1 (bit 8) per RM §15.3.5"
        );
        // DID still carries the static fallback (PfeHif0 = 12).
        assert_eq!((raw.word >> MDA_DID_SHIFT) & 0xF, Domain::PfeHif0 as u32);
    }

    /// Granting a PFE host interface lands in the **high bank** of PDAC_W1,
    /// not in PDAC_W0 — the previous packer would silently drop these bits.
    #[test]
    fn pdac_grant_to_pfe_hif0_lands_in_high_bank_w1() {
        let raw = Pdac::new(Peripheral::Pcie1)
            .grant(Domain::PfeHif0, Access::FullRw)
            .0;
        assert_eq!(raw.w0, 0, "PfeHif0 is D12 — no bits in low-bank W0");
        let expected_w1_acp = (Access::FullRw as u32) << PDAC_W1::D12ACP.shift;
        assert_eq!(
            raw.w1_acp, expected_w1_acp,
            "FullRw for D12 must land at PDAC_W1::D12ACP (bit 12)"
        );
    }

    /// A grant that mixes low- and high-bank domains populates both `w0`
    /// and `w1_acp` correctly without cross-contamination.
    #[test]
    fn pdac_mixed_low_high_bank_grants() {
        let raw = Pdac::new(Peripheral::Pcie1)
            .grant(Domain::M7_0, Access::SupervisorRw) // D1, low bank
            .grant(Domain::PfeHif3, Access::FullRw) // D15, high bank
            .0;
        let expected_w0 = (Access::SupervisorRw as u32) << PDAC_W0::D1ACP.shift;
        let expected_w1 = (Access::FullRw as u32) << PDAC_W1::D15ACP.shift;
        assert_eq!(raw.w0, expected_w0);
        assert_eq!(raw.w1_acp, expected_w1);
    }

    /// `Mrgd::region` resolves to the correct MRC for each documented
    /// XRDC_1 address window and rejects addresses outside any window.
    #[test]
    fn mrgd_region_resolves_xrdc_1_mrcs() {
        // PFE address window → MRC4.
        let r = MrgdRaw::region(0x4600_0000, 0x46FF_FFFF, MRC_RANGES);
        assert_eq!(r.mrc, 4);
        // SerDes_1 → MRC3.
        let r = MrgdRaw::region(0x4500_0000, 0x45FF_FFFF, MRC_RANGES);
        assert_eq!(r.mrc, 3);
        // STDBY_SRAM → MRC0.
        let r = MrgdRaw::region(0x2400_0000, 0x2400_001F, MRC_RANGES);
        assert_eq!(r.mrc, 0);
        // LLCE → MRC1.
        let r = MrgdRaw::region(0x4300_0000, 0x4300_001F, MRC_RANGES);
        assert_eq!(r.mrc, 1);
        // NoC_1 → MRC5.
        let r = MrgdRaw::region(0x4700_0000, 0x4707_FFFF, MRC_RANGES);
        assert_eq!(r.mrc, 5);
    }

    /// MRGD grant to a PFE host interface lands in MRGD_W3 (high bank), so
    /// the apply path's `acp_hi` write is what carries the bit through.
    #[test]
    fn mrgd_grant_to_pfe_hif_lands_in_acp_hi() {
        let raw = Mrgd::region(0x4600_0000, 0x46FF_FFFF)
            .grant(Domain::PfeHif2, Access::FullRw)
            .0;
        let expected_acp_hi = (Access::FullRw as u32) << MRGD_W3::D14ACP.shift;
        assert_eq!(
            raw.acp_lo, 0,
            "PfeHif2 is D14 — no bits in MRGD_W2 low bank"
        );
        assert_eq!(raw.acp_hi, expected_acp_hi);
    }

    /// Config::new accepts a non-empty, validated XRDC_1 policy.
    #[test]
    fn config_new_accepts_validated_policy() {
        const _OK: Config = Config::new(
            &[
                Mda::assign(Master::Pcie1, Domain::M7_0),
                Mda::assign(Master::PfeHifBdFetch, Domain::PfeHif0).with_didb_use_input(),
            ],
            &[
                Pdac::new(Peripheral::Xrdc1).grant(Domain::M7_0, Access::FullRw),
                Pdac::new(Peripheral::Pcie1).grant(Domain::M7_0, Access::SupervisorRw),
            ],
            &[Mrgd::region(0x4500_0000, 0x45FF_FFFF).grant(Domain::M7_0, Access::FullRw)],
        );
    }

    /// Sanity: the MRC counter array in `Config::new` is big enough for the
    /// largest MRC index any board can produce via `MRC_RANGES`.
    #[test]
    fn mrc_count_covers_max_index() {
        assert!((max_mrc_idx(MRC_RANGES) as usize) < MRC_COUNT);
    }
}
