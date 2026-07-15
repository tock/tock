// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Extended Resource Domain Controller (XRDC) for NXP S32G3.
//!
//! This module provides:
//!
//! * The verbatim register block + bitfields translated from S32G3 RM §15.7.
//! * Shared semantic types ([`Access`], [`Domain`], [`MrgdRaw`], plus the
//!   per-instance generic skeletons of [`PdacRaw`] / [`MdaRaw`] / [`MrcRange`])
//!   consumed by the per-instance modules below.
//! * Per-instance modules [`xrdc_0`] (system XRDC, RM §15.2) and [`xrdc_1`]
//!   (accelerator XRDC, RM §15.3) that bind those types to the
//!   instance-specific master / peripheral enums and expose
//!   `Xrdc{N}::apply` drivers that program the entire policy from reset.
//!
//! Boards never speak directly to the register block or to the `*Raw` types
//! — they declare a `const Config` in terms of the per-instance newtype
//! entries (`xrdc_0::Pdac`/`Mda`/`Mrgd` or `xrdc_1::Pdac`/`Mda`/`Mrgd`) and
//! call the matching `apply()`. Cross-instance mixing is a compile error:
//! `xrdc_0::Master::M7_0Axi` and `xrdc_1::Master::Pcie1` are different
//! types.
pub mod xrdc_0;
pub mod xrdc_1;

#[cfg(not(all(target_arch = "arm", target_os = "none")))]
use core::sync::atomic::{compiler_fence, Ordering};
#[cfg(all(target_arch = "arm", target_os = "none"))]
use core::arch::asm;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

/// XRDC_0 controls Main SoC peripheral and memory access policy.
/// # Safety: The S32G3 RM §15.7.3.1 maps XRDC_0 at 0x401A_4000, and this
/// `StaticRef` is only used for volatile MMIO access through `XrdcRegisters`.
pub const XRDC_0_BASE: StaticRef<XrdcRegisters> =
    unsafe { StaticRef::new(0x401A_4000 as *const XrdcRegisters) };

/// XRDC_1 controls the second XRDC instance at the documented base address.
/// # Safety: The S32G3 RM §15.7.4.1 maps XRDC_1 at 0x4400_4000, and this
/// `StaticRef` is only used for volatile MMIO access through `XrdcRegisters`.
pub const XRDC_1_BASE: StaticRef<XrdcRegisters> =
    unsafe { StaticRef::new(0x4400_4000 as *const XrdcRegisters) };

const MDA_CFG_COUNT: usize = 24;
const MRC_CFG_COUNT: usize = 14;
const DOMAIN_COUNT: usize = 16;
const DOMAIN_ERROR_INSTANCE_COUNT: usize = 21;
const PID_COUNT: usize = 24;
const MDA_INSTANCE_COUNT: usize = 24;
const MRGD_COUNT: usize = 224;
const PDAC_PAC0_SLOT_COUNT: usize = 32;
const PDAC_PAC1_SLOT_COUNT: usize = 34;
const PDAC_PAC2_SLOT_COUNT: usize = 34;
const PDAC_PAC3_WINDOW_COUNT: usize = 25;
const PDAC_PAC4_WINDOW_COUNT: usize = 31;

register_structs! {
    /// Domain error capture register set for one MRC/PAC error instance.
    pub XrdcDomainErrorRegisters {
        /// Domain Error Word 0: captured violation address bits 31:0.
        (0x00 => pub w0: ReadOnly<u32, DERR_W0::Register>),
        /// Domain Error Word 1: captured violation state, port, attributes, and DID.
        (0x04 => pub w1: ReadOnly<u32, DERR_W1::Register>),
        /// Domain Error Word 2: captured violation address bits 39:32 for 40-bit errors.
        (0x08 => pub w2: ReadOnly<u32, DERR_W2::Register>),
        /// Domain Error Word 3: write `RECR=01b` to rearm this error capture instance.
        (0x0C => pub w3: ReadWrite<u32, DERR_W3::Register>),
        (0x10 => @END),
    }
}

register_structs! {

    /// Master domain assignment slot. DFMT0 uses up to eight words; DFMT1 uses word 0.
    pub XrdcMdaRegisters {
        /// Master Domain Assignment words for one bus initiator.
        (0x00 => pub word: [ReadWrite<u32, MDA::Register>; 8]),
        (0x20 => @END),
    }
}

register_structs! {

    /// Peripheral Domain Access Control register pair for one peripheral slot.
    pub XrdcPdacRegisters {
        /// PDAC Word 0: semaphore selector and D0-D7 ACP fields.
        (0x00 => pub w0: ReadWrite<u32, PDAC_W0::Register>),
        /// PDAC Word 1: valid/lock state and D8-D15 ACP fields.
        (0x04 => pub w1: ReadWrite<u32, PDAC_W1::Register>),
        (0x08 => @END),
    }
}

register_structs! {

    /// Memory Region Descriptor register set. Each descriptor occupies a 0x20-byte slot.
    pub XrdcMemoryRegionDescriptorRegisters {
        /// MRGD Word 0: memory region start address bits 35:5.
        (0x00 => pub w0: ReadWrite<u32, MRGD_W0::Register>),
        /// MRGD Word 1: memory region end address bits 35:5.
        (0x04 => pub w1: ReadWrite<u32, MRGD_W1::Register>),
        /// MRGD Word 2: semaphore selector and D0-D7 ACP fields.
        (0x08 => pub w2: ReadWrite<u32, MRGD_W2::Register>),
        /// MRGD Word 3: valid/lock state and D8-D15 ACP fields.
        (0x0C => pub w3: ReadWrite<u32, MRGD_W3::Register>),
        (0x10 => _reserved0),
        (0x20 => @END),
    }
}

register_structs! {

    /// XRDC register block. This is the XRDC_0 superset layout; XRDC_1 implements
    /// a smaller documented subset at the same leading offsets.
    pub XrdcRegisters {
        /// Control: XRDC status, revision fields, and global valid enable.
        (0x0000 => pub cr: ReadWrite<u32, CR::Register>),
        (0x0004 => _reserved0),
        /// Hardware Configuration 0: implemented domains, initiators, MRCs, PACs.
        (0x00F0 => pub hwcfg0: ReadOnly<u32, HWCFG0::Register>),
        /// Hardware Configuration 1: DID of the initiator reading this register.
        (0x00F4 => pub hwcfg1: ReadOnly<u32, HWCFG1::Register>),
        /// Hardware Configuration 2: bitmap of initiators with built-in PID registers.
        (0x00F8 => pub hwcfg2: ReadOnly<u32, HWCFG2::Register>),
        (0x00FC => _reserved1),
        /// Master Domain Assignment Configuration bytes MDACFG0..23.
        (0x0100 => pub mdacfg: [ReadOnly<u8, MDACFG::Register>; MDA_CFG_COUNT]),
        (0x0118 => _reserved2),
        /// Memory Region Configuration bytes MRCFG0..13.
        (0x0140 => pub mrcfg: [ReadOnly<u8, MRCFG::Register>; MRC_CFG_COUNT]),
        (0x014E => _reserved3),
        /// Domain Error Location registers DERRLOC0..15.
        (0x0200 => pub derrloc: [ReadOnly<u32, DERRLOC::Register>; DOMAIN_COUNT]),
        (0x0240 => _reserved4),
        /// Domain Error Word register sets for implemented MRC/PAC error instances.
        (0x0400 => pub derr: [XrdcDomainErrorRegisters; DOMAIN_ERROR_INSTANCE_COUNT]),
        (0x0550 => _reserved5),
        /// Process Identifier registers PID0..23; unimplemented PID slots are reserved holes.
        (0x0700 => pub pid: [ReadWrite<u32, PID::Register>; PID_COUNT]),
        (0x0760 => _reserved6),
        /// Master Domain Assignment slots MDA0..23.
        (0x0800 => pub mda: [XrdcMdaRegisters; MDA_INSTANCE_COUNT]),
        (0x0B00 => _reserved7),
        /// PDAC slots 0..31.
        (0x1000 => pub pdac_0_31: [XrdcPdacRegisters; PDAC_PAC0_SLOT_COUNT]),
        (0x1100 => _reserved8),
        /// PDAC slots 128..161.
        (0x1400 => pub pdac_128_161: [XrdcPdacRegisters; PDAC_PAC1_SLOT_COUNT]),
        (0x1510 => _reserved9),
        /// PDAC slots 256..289.
        (0x1800 => pub pdac_256_289: [XrdcPdacRegisters; PDAC_PAC2_SLOT_COUNT]),
        (0x1910 => _reserved10),
        /// PDAC address window covering slots 384..408; some slots in this range are holes.
        (0x1C00 => pub pdac_384_408: [XrdcPdacRegisters; PDAC_PAC3_WINDOW_COUNT]),
        (0x1CC8 => _reserved11),
        /// Memory Region Descriptors MRGD0..223.
        (0x2000 => pub mrgd: [XrdcMemoryRegionDescriptorRegisters; MRGD_COUNT]),
        (0x3C00 => _reserved12),
        /// PDAC address window covering slots 512..542; some slots in this range are holes.
        (0x4000 => pub pdac_512_542: [XrdcPdacRegisters; PDAC_PAC4_WINDOW_COUNT]),
        (0x40F8 => @END),
    }
}

register_bitfields![u8,
    /// Master Domain Assignment Configuration byte.
    MDACFG [
        /// Number of MDA registers associated with this initiator; zero means absent.
        NMDAR OFFSET(0) NUMBITS(4) []
    ],

    /// Memory Region Configuration byte.
    MRCFG [
        /// Number of memory region descriptors for this MRC: 0, 4, 8, 12, or 16.
        NMRGD OFFSET(0) NUMBITS(5) []
    ]
];

register_bitfields![u32,
    /// Control Register.
    CR [
        /// Lock bit prohibiting further writes to CR until reset.
        LK1 OFFSET(30) NUMBITS(1) [Unlocked = 0, Locked = 1],
        /// Virtualization-aware domain assignment support indicator.
        VAW OFFSET(8) NUMBITS(1) [NotVirtualizationAware = 0, VirtualizationAware = 1],
        /// Memory region descriptor format indicator; S32G3 uses SMPU-family format.
        MRF OFFSET(7) NUMBITS(1) [Reserved = 0, SmpuFamily = 1],
        /// Hardware revision level of the XRDC module.
        HRL OFFSET(1) NUMBITS(4) [],
        /// Global Valid enables XRDC policy evaluation when asserted.
        GVLD OFFSET(0) NUMBITS(1) [Disabled = 0, Enabled = 1]
    ],

    /// Hardware Configuration 0 Register.
    HWCFG0 [
        /// Module identifier.
        MID OFFSET(28) NUMBITS(4) [],
        /// Number of PACs minus one.
        NPAC OFFSET(24) NUMBITS(4) [],
        /// Number of MRCs minus one.
        NMRC OFFSET(16) NUMBITS(8) [],
        /// Number of bus initiators minus one.
        NMSTR OFFSET(8) NUMBITS(8) [],
        /// Number of domain IDs minus one.
        NDID OFFSET(0) NUMBITS(8) []
    ],

    /// Hardware Configuration 1 Register.
    HWCFG1 [
        /// Domain ID of the initiator reading HWCFG1.
        DID OFFSET(0) NUMBITS(4) []
    ],

    /// Hardware Configuration 2 Register.
    HWCFG2 [
        /// Per-initiator bitmap indicating built-in PID register presence.
        PIDP OFFSET(0) NUMBITS(32) []
    ],

    /// Domain Error Location Register.
    DERRLOC [
        /// PAC instance bits with access violations for this domain.
        PACINST OFFSET(16) NUMBITS(8) [],
        /// MRC instance bits with access violations for this domain.
        MRCINST OFFSET(0) NUMBITS(16) []
    ],

    /// Domain Error Word 0 Register.
    DERR_W0 [
        /// Captured target address bits 31:0 for the first violation after rearm.
        EADDR OFFSET(0) NUMBITS(32) []
    ],

    /// Domain Error Word 1 Register.
    DERR_W1 [
        /// Error state: none, single violation, or multiple violation overrun.
        EST OFFSET(30) NUMBITS(2) [],
        /// Encoded MRC port number; zero for PAC violations.
        EPORT OFFSET(24) NUMBITS(3) [],
        /// Indicates that DERR_W2 contains address bits 39:32.
        EA40FMT OFFSET(16) NUMBITS(1) [Bits32 = 0, Bits40 = 1],
        /// Captured access direction.
        ERW OFFSET(11) NUMBITS(1) [Read = 0, Write = 1],
        /// Captured secure/nonsecure, privileged/user, instruction/data attributes.
        EATR OFFSET(8) NUMBITS(3) [],
        /// Domain ID that caused the access violation.
        EDID OFFSET(0) NUMBITS(4) []
    ],

    /// Domain Error Word 2 Register.
    DERR_W2 [
        /// Captured target address bits 39:32 when EA40FMT is set.
        EADDR39_32 OFFSET(0) NUMBITS(8) []
    ],

    /// Domain Error Word 3 Register.
    DERR_W3 [
        /// Rearm error capture; write `01b` to clear captured error state.
        RECR OFFSET(30) NUMBITS(2) [NoEffect = 0, Rearm = 1]
    ],

    /// Process Identifier Register.
    PID [
        /// Lock field limiting writes to this PID register until reset.
        LK2 OFFSET(29) NUMBITS(2) [],
        /// Three-state model bit for initiators that do not support all secure states.
        TSM OFFSET(28) NUMBITS(1) [Disabled = 0, Enabled = 1],
        /// Enables special LK2 handling with locked master capture.
        ELK22H OFFSET(24) NUMBITS(1) [Disabled = 0, Enabled = 1],
        /// Initiator number that locked this register when ELK22H is enabled.
        LMNUM OFFSET(16) NUMBITS(6) [],
        /// Process identifier secure attribute for this initiator.
        PID OFFSET(0) NUMBITS(6) []
    ],

    /// Master Domain Assignment Register.
    MDA [
        /// Valid bit enabling this domain assignment when CR[GVLD] is asserted.
        VLD OFFSET(31) NUMBITS(1) [Invalid = 0, Valid = 1],
        /// Lock bit prohibiting further writes to this MDA word until reset.
        LK1 OFFSET(30) NUMBITS(1) [Unlocked = 0, Locked = 1],
        /// Domain assignment format: DFMT0 for core initiators, DFMT1 for bus initiators.
        DFMT OFFSET(29) NUMBITS(1) [Core = 0, Bus = 1],
        /// DFMT0 process identifier match value.
        PID OFFSET(16) NUMBITS(6) [],
        /// DFMT0 process identifier mask.
        PIDM OFFSET(8) NUMBITS(6) [],
        /// DFMT1 DID input bypass control.
        DIDB OFFSET(8) NUMBITS(1) [BypassInput = 0, UseInput = 1],
        /// DFMT0 process identifier enable/match mode.
        PE OFFSET(6) NUMBITS(2) [],
        /// DFMT1 secure attribute override.
        SA OFFSET(6) NUMBITS(2) [],
        /// DFMT0 DID source select.
        DIDS OFFSET(4) NUMBITS(2) [],
        /// DFMT1 privileged attribute override.
        PA OFFSET(4) NUMBITS(2) [],
        /// Domain ID value used by this assignment.
        DID OFFSET(0) NUMBITS(4) []
    ],

    /// Peripheral Domain Access Control Word 0 Register.
    PDAC_W0 [
        /// Include the selected semaphore in DdACP evaluation.
        SE OFFSET(30) NUMBITS(1) [Disabled = 0, Enabled = 1],
        /// Hardware semaphore number used when SE is enabled.
        SNUM OFFSET(24) NUMBITS(4) [],
        /// Domain 7 access control policy.
        D7ACP OFFSET(21) NUMBITS(3) [],
        /// Domain 6 access control policy.
        D6ACP OFFSET(18) NUMBITS(3) [],
        /// Domain 5 access control policy.
        D5ACP OFFSET(15) NUMBITS(3) [],
        /// Domain 4 access control policy.
        D4ACP OFFSET(12) NUMBITS(3) [],
        /// Domain 3 access control policy.
        D3ACP OFFSET(9) NUMBITS(3) [],
        /// Domain 2 access control policy.
        D2ACP OFFSET(6) NUMBITS(3) [],
        /// Domain 1 access control policy.
        D1ACP OFFSET(3) NUMBITS(3) [],
        /// Domain 0 access control policy.
        D0ACP OFFSET(0) NUMBITS(3) []
    ],

    /// Peripheral Domain Access Control Word 1 Register.
    PDAC_W1 [
        /// Valid bit enabling this PDAC pair when CR[GVLD] is asserted.
        VLD OFFSET(31) NUMBITS(1) [Invalid = 0, Valid = 1],
        /// Lock field limiting writes to this PDAC pair until reset.
        LK2 OFFSET(29) NUMBITS(2) [],
        /// Domain 15 access control policy.
        D15ACP OFFSET(21) NUMBITS(3) [],
        /// Domain 14 access control policy.
        D14ACP OFFSET(18) NUMBITS(3) [],
        /// Domain 13 access control policy.
        D13ACP OFFSET(15) NUMBITS(3) [],
        /// Domain 12 access control policy.
        D12ACP OFFSET(12) NUMBITS(3) [],
        /// Domain 11 access control policy.
        D11ACP OFFSET(9) NUMBITS(3) [],
        /// Domain 10 access control policy.
        D10ACP OFFSET(6) NUMBITS(3) [],
        /// Domain 9 access control policy.
        D9ACP OFFSET(3) NUMBITS(3) [],
        /// Domain 8 access control policy.
        D8ACP OFFSET(0) NUMBITS(3) []
    ],

    /// Memory Region Descriptor Word 0 Register.
    MRGD_W0 [
        /// Start address bits 35:5 of a 32-byte-aligned memory region.
        SRTADDR OFFSET(1) NUMBITS(31) []
    ],

    /// Memory Region Descriptor Word 1 Register.
    MRGD_W1 [
        /// End address bits 35:5 of a 32-byte-aligned memory region.
        ENDADDR OFFSET(1) NUMBITS(31) []
    ],

    /// Memory Region Descriptor Word 2 Register.
    MRGD_W2 [
        /// Include the selected semaphore in DdACP evaluation.
        SE OFFSET(30) NUMBITS(1) [Disabled = 0, Enabled = 1],
        /// Hardware semaphore number used when SE is enabled.
        SNUM OFFSET(24) NUMBITS(4) [],
        /// Domain 7 access control policy.
        D7ACP OFFSET(21) NUMBITS(3) [],
        /// Domain 6 access control policy.
        D6ACP OFFSET(18) NUMBITS(3) [],
        /// Domain 5 access control policy.
        D5ACP OFFSET(15) NUMBITS(3) [],
        /// Domain 4 access control policy.
        D4ACP OFFSET(12) NUMBITS(3) [],
        /// Domain 3 access control policy.
        D3ACP OFFSET(9) NUMBITS(3) [],
        /// Domain 2 access control policy.
        D2ACP OFFSET(6) NUMBITS(3) [],
        /// Domain 1 access control policy.
        D1ACP OFFSET(3) NUMBITS(3) [],
        /// Domain 0 access control policy.
        D0ACP OFFSET(0) NUMBITS(3) []
    ],

    /// Memory Region Descriptor Word 3 Register.
    MRGD_W3 [
        /// Valid bit enabling this MRGD when CR[GVLD] is asserted.
        VLD OFFSET(31) NUMBITS(1) [Invalid = 0, Valid = 1],
        /// Lock field limiting writes to this MRGD until reset.
        LK2 OFFSET(29) NUMBITS(2) [],
        /// Domain 15 access control policy.
        D15ACP OFFSET(21) NUMBITS(3) [],
        /// Domain 14 access control policy.
        D14ACP OFFSET(18) NUMBITS(3) [],
        /// Domain 13 access control policy.
        D13ACP OFFSET(15) NUMBITS(3) [],
        /// Domain 12 access control policy.
        D12ACP OFFSET(12) NUMBITS(3) [],
        /// Domain 11 access control policy.
        D11ACP OFFSET(9) NUMBITS(3) [],
        /// Domain 10 access control policy.
        D10ACP OFFSET(6) NUMBITS(3) [],
        /// Domain 9 access control policy.
        D9ACP OFFSET(3) NUMBITS(3) [],
        /// Domain 8 access control policy.
        D8ACP OFFSET(0) NUMBITS(3) []
    ]
];
/// Mask for the ACP level field in PDAC_W1 / MRGD_W3 (D8..D15, bits 0–23).
/// RM §15.7.3.17: 3 bits per domain × 8 domains.
const ACP_HI_MASK: u32 = 0x00FF_FFFF;
/// Mask for the ACP level field in MRGD_W2 (D0..D7, bits 0–23).
/// RM §15.7.3.18: 3 bits per domain × 8 domains.
const ACP_LO_MASK: u32 = 0x00FF_FFFF;
/// Mask for the LK2 field (bits 29–30) in PDAC_W1 / MRGD_W3.
const LK2_MASK: u32 = 0b11u32 << 29;

// =============================================================================
// New configurable XRDC API
// =============================================================================
//
// The types below are the chip-crate-shared building blocks for the per-XRDC-
// instance modules (currently [`xrdc_0`]). Boards never instantiate `*Raw`
// types directly; they go through the per-instance newtype wrappers that bind
// the right peripheral / master enum and address-coverage table.

/// Number of XRDC domain IDs implemented on S32G3.
///
/// RM §15.2.1: "The global XRDC configuration defines 16 domains." S32G3
/// physically wires up 8 — see [`Domain`] for the named variants.
pub const XRDC_DOMAIN_REG_COUNT: usize = 16;

/// 1:1 alias of RM §15.8.6.8 Table 49 (Domain ACP specification).
///
/// The hardware exposes exactly these 8 access-control levels per (domain ×
/// resource); no other combinations of (secure × privileged × R/W) are
/// representable.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Access {
    /// `000b` — no access in any (secure × privileged) state.
    None = 0b000,
    /// `001b` — read-only in both secure modes; no nonsecure access.
    SecureReadOnly = 0b001,
    /// `010b` — R/W in secure-privileged only. Closest mapping to a
    /// "supervisor-only" peripheral on a Tock-on-M7-secure kernel.
    SupervisorRw = 0b010,
    /// `011b` — R/W in both secure modes (priv & user); no nonsecure access.
    SecureRw = 0b011,
    /// `100b` — secure R/W + nonsecure-privileged read; nonsecure-user blocked.
    SecureRwNsPrivRead = 0b100,
    /// `101b` — secure R/W + nonsecure read-only (both priv & user).
    SecureRwNsRead = 0b101,
    /// `110b` — R/W everywhere except nonsecure-user.
    NoNsUser = 0b110,
    /// `111b` — R/W in every (secure × privileged) combination, no restriction.
    FullRw = 0b111,
}

/// Domain identifiers wired on S32G3.
///
/// The discriminants are the DID values written verbatim into MDA `DID`
/// fields and into the `D{n}ACP` bit position selectors in PDAC / MRGD
/// register words.
///
/// Sources:
/// - D0..D7: RM §15.2.2 Table 28 (master → domain wiring on XRDC_0).
/// - D12..D15: RM §15.3.5 Table 37 — the four PFE host interfaces are
/// - D12..D15: RM §15.3.5 Table 37 — the four PFE host interfaces are
///   hard-wired to DIDs 0xC..0xF on XRDC_1 via the per-transaction DID input
///   on `XRDC_MDAC1..4` (PFE_HIF) when `DIDB = UseInput`.
///
/// D8..D11 are unassigned by the S32G3 master map and are deliberately not
/// representable. If a future board needs them, add the variant with an RM
/// cite and extend the const-assert block below.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Domain {
    /// JTAG / Debug ETR (RM Table 28).
    Debugger = 0,
    /// Cortex-M7_0 (AXI + AHB initiators).
    M7_0 = 1,
    /// Cortex-M7_1 (AXI + AHB initiators).
    M7_1 = 2,
    /// Cortex-M7_2 (AXI + AHB initiators).
    M7_2 = 3,
    /// Cortex-M7_3 (AXI + AHB initiators).
    M7_3 = 4,
    /// eDMA_0 + eDMA_1.
    EDma = 5,
    /// HSE_H security engine.
    Hse = 6,
    /// Cortex-A53 cluster 0 + cluster 1.
    A53 = 7,
    /// PFE host interface 0 (XRDC_1 only, RM §15.3.5 Table 37).
    PfeHif0 = 12,
    /// PFE host interface 1 (XRDC_1 only, RM §15.3.5 Table 37).
    PfeHif1 = 13,
    /// PFE host interface 2 (XRDC_1 only, RM §15.3.5 Table 37).
    PfeHif2 = 14,
    /// PFE host interface 3 (XRDC_1 only, RM §15.3.5 Table 37).
    PfeHif3 = 15,
}

/// Mask for the low-bank ACP word (D0..D7) — the low 24 bits of PDAC_W0 /
/// MRGD_W2.
///
/// Returns the OR-able `u32` mask matching the typed
/// `PDAC_W0::D{n}ACP.shift` / `MRGD_W2::D{n}ACP.shift` constants.
/// Const-fn. For domains in the high bank (`Domain` discriminant ≥ 8) this
/// returns 0 — the high bits land in [`acp_bits_hi`] and the chip-crate
/// driver always OR-s both into the right register word at apply time.
pub(crate) const fn acp_bits_lo(domain: Domain, access: Access) -> u32 {
    let n = domain as u32;
    if n < 8 {
        (access as u32) << (n * 3)
    } else {
        0
    }
}

/// Mask for the high-bank ACP word (D8..D15) — the low 24 bits of PDAC_W1 /
/// MRGD_W3.
///
/// Returns the OR-able `u32` mask matching the typed
/// `PDAC_W1::D{n}ACP.shift` / `MRGD_W3::D{n}ACP.shift` constants. Const-fn.
/// Mirrors [`acp_bits_lo`] for the upper bank; returns 0 for low-bank
/// domains.
pub(crate) const fn acp_bits_hi(domain: Domain, access: Access) -> u32 {
    let n = domain as u32;
    if n >= 8 {
        (access as u32) << ((n - 8) * 3)
    } else {
        0
    }
}

/// Marker trait gating bus-initiator-only secure and privileged override
/// methods (DFMT1).
///
/// The chip crate does not give core initiators (DFMT0) these methods because
/// the corresponding `SA` / `PA` fields don't exist in that format. Attempts
/// to call `.force_secure()` on a core master from a `const` context fail at
/// const-eval with a clear message.
pub trait BusInitiator: Copy + 'static {}

/// One MRC's address coverage on an XRDC instance.
///
/// post-translation addresses per RM §15.2.4.
/// `nmrgd` is the descriptor budget for this MRC (mirrored by RM `MRCFG[NMRGD]`).
pub struct MrcRange {
    /// MRC submodule index within the XRDC instance.
    pub idx: u8,
    /// Inclusive start address (A53-view).
    pub start: u32,
    /// Inclusive end address (A53-view).
    pub end: u32,
    /// Number of MRGD descriptors this MRC owns.
    pub nmrgd: u8,
}

/// Look up the MRC index covering `addr` in `ranges`. Const-fn; returns
/// `None` if no MRC owns this address (which `MrgdEntry::region` turns into
/// a const-eval panic with a meaningful message).
pub(crate) const fn mrc_for_addr(addr: u32, ranges: &[MrcRange]) -> Option<u8> {
    let mut i = 0;
    while i < ranges.len() {
        let r = &ranges[i];
        if addr >= r.start && addr <= r.end {
            return Some(r.idx);
        }
        i += 1;
    }
    None
}

/// Largest `NMRGD` budget any MRC in `ranges` exposes for `mrc`. Identical
/// MRC index may appear multiple times (e.g. MRC7 covers two QSPI windows);
/// the budget is the same per RM so any occurrence is authoritative.
pub(crate) const fn nmrgd_for_mrc(mrc: u8, ranges: &[MrcRange]) -> u8 {
    let mut i = 0;
    while i < ranges.len() {
        if ranges[i].idx == mrc {
            return ranges[i].nmrgd;
        }
        i += 1;
    }
    0
}

/// Largest MRC index referenced in `ranges`. Used by const-fn validators to
/// size per-MRC counters without depending on the chip's `NMRC + 1`.
pub(crate) const fn max_mrc_idx(ranges: &[MrcRange]) -> u8 {
    let mut i = 0;
    let mut m = 0;
    while i < ranges.len() {
        if ranges[i].idx > m {
            m = ranges[i].idx;
        }
        i += 1;
    }
    m
}

/// Chip-crate-internal representation of a [`xrdc_0::Mrgd`] entry.
///
/// Stores precomputed register words (`SRTADDR`, `ENDADDR`, `D0-7 ACP`,
/// `D8-15 ACP`) plus the MRC index resolved at const-eval. Per-instance
/// newtype wrappers expose construction.
#[derive(Copy, Clone)]
pub struct MrgdRaw {
    /// MRGD_W0 SRTADDR field, pre-shifted (bits 1..32 of the encoded address).
    pub(crate) srtaddr_field: u32,
    /// MRGD_W1 ENDADDR field, pre-shifted.
    pub(crate) endaddr_field: u32,
    /// D0..D7 ACP bits (low 24 bits of MRGD_W2).
    pub(crate) acp_lo: u32,
    /// D8..D15 ACP bits (low 24 bits of MRGD_W3).
    pub(crate) acp_hi: u32,
    /// Resolved MRC submodule index (within the XRDC instance).
    pub(crate) mrc: u8,
}

impl MrgdRaw {
    /// Build an entry. Const-asserts:
    /// * `start <= end`
    /// * `start` is 32-byte aligned (low 5 bits 0)
    /// * `end` ends on a 32-byte boundary (low 5 bits 1)
    /// * the address range falls in exactly one MRC's coverage window in `ranges`
    pub const fn region(start: u32, end: u32, ranges: &[MrcRange]) -> Self {
        assert!(start <= end, "MrgdEntry::region: start address exceeds end");
        assert!(
            start & 0x1F == 0,
            "MrgdEntry::region: start must be 32-byte aligned (RM §15.7.3.18 SRTADDR is bits 35:5)"
        );
        assert!(
            end & 0x1F == 0x1F,
            "MrgdEntry::region: end must terminate on a 32-byte boundary (low 5 bits = 0x1F)"
        );
        let mrc_start = match mrc_for_addr(start, ranges) {
            Some(m) => m,
            None => {
                panic!("MrgdEntry::region: start address has no MRC coverage on this XRDC instance")
            }
        };
        let mrc_end = match mrc_for_addr(end, ranges) {
            Some(m) => m,
            None => {
                panic!("MrgdEntry::region: end address has no MRC coverage on this XRDC instance")
            }
        };
        assert!(
            mrc_start == mrc_end,
            "MrgdEntry::region: address range crosses MRC boundaries (a single MRGD must lie in one MRC)"
        );
        // SRTADDR/ENDADDR fields are bits 35:5 placed at bit-offset 1 in their
        // words. Mask to 32 bits and shift down by 4 (5 RAZ bits − 1 field
        // offset = 4) to land in the right slot. Verified by the unit test in
        // this module.
        let srtaddr_field = (start & !0x1F) >> 4;
        let endaddr_field = (end & !0x1F) >> 4;
        Self {
            srtaddr_field,
            endaddr_field,
            acp_lo: 0,
            acp_hi: 0,
            mrc: mrc_start,
        }
    }

    /// Grant `access` to `domain` on this region. Composes the per-domain
    /// `DxACP` bits into the stored ACP words at const-eval.
    pub const fn grant(mut self, domain: Domain, access: Access) -> Self {
        self.acp_lo |= acp_bits_lo(domain, access);
        self.acp_hi |= acp_bits_hi(domain, access);
        self
    }
}
/// How to locate an MRGD descriptor to patch at runtime.
///
/// Used by the XRDC_0 and XRDC_1 `search_and_patch_mrgd` methods when a prior
/// boot stage may have already programmed the descriptor with
/// run-time-determined bounds.
#[derive(Debug)]
pub enum MrgdTarget {
    /// Match a descriptor whose `SRTADDR`/`ENDADDR` exactly equal the entry.
    ExactRange,
    /// Match the first descriptor whose range contains `probe_addr`.
    ContainsAddress(u32),
}
/// Outcome of an XRDC additive patch.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MrgdPatchOutcome {
    /// An existing descriptor was found and its ACP bits were replaced.
    PatchedExisting { slot: usize },
    /// No static descriptor matched; a free slot was allocated.
    AllocatedNew { slot: usize },
}

/// A policy patch could not safely be applied.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum XrdcPatchError {
    LockedDescriptor,
    NoFreeDescriptor,
    MissingTarget,
    AmbiguousTarget,
}

/// Chip-crate-internal representation of a [`xrdc_0::Pdac`] entry.
///
/// `slot` is the global PDAC slot number for this XRDC instance (each instance
/// has its own slot layout; the per-instance `Peripheral` enum encodes the
/// integer slot in its discriminant).
#[derive(Copy, Clone)]
pub struct PdacRaw {
    pub(crate) slot: u16,
    /// PDAC_W0: SE=0, SNUM=0, D0..D7 ACP packed.
    pub(crate) w0: u32,
    /// D8..D15 ACP bits, packed into the low 24 bits of PDAC_W1. The VLD /
    /// LK2 fields are added by the driver at apply time.
    pub(crate) w1_acp: u32,
}

impl PdacRaw {
    pub const fn new(slot: u16) -> Self {
        Self {
            slot,
            w0: 0,
            w1_acp: 0,
        }
    }

    pub const fn grant(mut self, domain: Domain, access: Access) -> Self {
        self.w0 |= acp_bits_lo(domain, access);
        self.w1_acp |= acp_bits_hi(domain, access);
        self
    }
}

/// Chip-crate-internal representation of a [`xrdc_0::Mda`] entry.
///
/// Encodes the precomputed `MDA_Wx` register word for one initiator. The
/// per-instance newtype wrapper picks the DFMT bit based on the master's
/// initiator class (core vs bus).
#[derive(Copy, Clone)]
pub struct MdaRaw {
    /// Initiator (MDA submodule) index — global within the XRDC instance.
    pub(crate) master_idx: u8,
    /// Pre-computed register word: VLD is added by the driver at apply time.
    pub(crate) word: u32,
    /// `true` if this initiator uses DFMT1 (bus master). Used by the apply
    /// helper to clear `DIDB` for bus initiators so the input DID is ignored.
    pub(crate) is_bus: bool,
}

/// MDA `DFMT` bit position (RM §15.7.3.14/15).
pub(crate) const MDA_DFMT_SHIFT: u32 = 29;
/// MDA `DIDB` (DFMT1 only) — `0 = BypassInput` (DID from this word),
/// `1 = UseInput` (DID taken from the bus master's per-transaction input).
/// PFE host-interface tagging requires `UseInput` per RM §15.3.5.
pub(crate) const MDA_DIDB_SHIFT: u32 = 8;
/// MDA `SA` (secure-attribute override) field — 2 bits at offset 6 (DFMT1).
pub(crate) const MDA_SA_SHIFT: u32 = 6;
/// MDA `PA` (privileged-attribute override) field — 2 bits at offset 4 (DFMT1).
pub(crate) const MDA_PA_SHIFT: u32 = 4;
/// MDA `DID` field — 4 bits at offset 0.
pub(crate) const MDA_DID_SHIFT: u32 = 0;

/// `SA` encoding (DFMT1): values follow the RM enumeration.
#[repr(u32)]
#[derive(Copy, Clone)]
pub enum SecureAttr {
    /// `00b` — bus-master input secure attribute used unchanged.
    UseInput = 0b00,
    /// `10b` — force secure regardless of bus-master input.
    ForceSecure = 0b10,
    /// `11b` — force nonsecure regardless of bus-master input.
    ForceNonsecure = 0b11,
}

/// `PA` encoding (DFMT1): values follow the RM enumeration.
#[repr(u32)]
#[derive(Copy, Clone)]
pub enum PrivAttr {
    /// `00b` — bus-master input privileged attribute used unchanged.
    UseInput = 0b00,
    /// `10b` — force privileged regardless of bus-master input.
    ForcePrivileged = 0b10,
    /// `11b` — force user regardless of bus-master input.
    ForceUser = 0b11,
}

impl MdaRaw {
    /// Build a DFMT1 (bus initiator) entry with `SA=UseInput`, `PA=UseInput`,
    /// `DID=domain`. `DIDB` stays 0 (DID from this MDA word, not from the
    /// bus-master input). The driver sets `VLD` at apply time.
    pub const fn bus(master_idx: u8, domain: Domain) -> Self {
        let word = (1 << MDA_DFMT_SHIFT) | ((domain as u32) << MDA_DID_SHIFT);
        Self {
            master_idx,
            word,
            is_bus: true,
        }
    }

    /// Build a DFMT0 (core initiator) entry. `DIDS = 00b` (use DID from this
    /// word), `PE = 00b` (PID matching disabled), `DID = domain`. The driver
    /// sets `VLD` at apply time. PID matching is not exposed in this v1
    /// surface; each core initiator gets one MDA word.
    pub const fn core(master_idx: u8, domain: Domain) -> Self {
        let word = (domain as u32) << MDA_DID_SHIFT;
        Self {
            master_idx,
            word,
            is_bus: false,
        }
    }

    /// Patch the `SA` field of a DFMT1 entry. Const-asserts `is_bus` so
    /// applying this to a core initiator is a const-eval panic.
    pub const fn with_sa(mut self, sa: SecureAttr) -> Self {
        assert!(
            self.is_bus,
            "MdaEntry::force_secure/force_nonsecure: SA override is DFMT1-only (bus initiators)"
        );
        // Clear the 2-bit SA field then OR in the new value.
        self.word &= !(0b11u32 << MDA_SA_SHIFT);
        self.word |= (sa as u32) << MDA_SA_SHIFT;
        self
    }

    /// Patch the `PA` field of a DFMT1 entry. Const-asserts `is_bus`.
    pub const fn with_pa(mut self, pa: PrivAttr) -> Self {
        assert!(
            self.is_bus,
            "MdaEntry::force_privileged/force_user: PA override is DFMT1-only (bus initiators)"
        );
        self.word &= !(0b11u32 << MDA_PA_SHIFT);
        self.word |= (pa as u32) << MDA_PA_SHIFT;
        self
    }

    /// Switch the DFMT1 `DIDB` bit to `UseInput` so the bus master's
    /// per-transaction input DID propagates through this MDA word, ignoring
    /// the static `DID` field set by [`bus`](Self::bus). Required for the
    /// PFE host-interface tagging mode (RM §15.3.5: PFE updates the DID
    /// input on the fly between HIF 0..3 → DID 0xC..0xF).
    ///
    /// Const-asserts `is_bus` so applying this to a core initiator is a
    /// const-eval panic.
    pub const fn with_didb_use_input(mut self) -> Self {
        assert!(
            self.is_bus,
            "MdaEntry::with_didb_use_input: DIDB is DFMT1-only (bus initiators)"
        );
        self.word |= 1u32 << MDA_DIDB_SHIFT;
        self
    }
}

// =============================================================================
// apply-time helpers shared between XRDC instances
// =============================================================================

/// Synchronize XRDC policy publication with the hardware and compiler.
///
/// NXP's AUTOSAR RTD `Xrdc_Memory_Config_Descriptor`,
/// `Xrdc_Peripheral_Access_Config`, `Xrdc_Domain_Init`, and
/// `Xrdc_Ip_Init_Privileged` in `Xrdc_Ip.c` bracket descriptor, MDA, and
/// `CR[GVLD]` publication with `DSB; ISB`. The host implementation preserves
/// source ordering for register-backed unit tests only; it provides no hardware
/// synchronization guarantee.
pub(crate) fn xrdc_sync() {
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    unsafe {
        // Deliberately omit `nomem`: MMIO accesses must not move across this
        // driver-specific synchronization point.
        asm!("dsb sy", "isb", options(nostack, preserves_flags));
    }
    #[cfg(not(all(target_arch = "arm", target_os = "none")))]
    compiler_fence(Ordering::SeqCst);
}

/// Resolve a global PDAC slot to its register index within the XRDC register
/// block's PDAC window arrays. Returns `Err(())` for slots that fall in the
/// reserved gaps between PDAC windows — boards never legitimately produce
/// such slots (the per-instance `Peripheral` enum only enumerates valid
/// peripherals), so apply panics rather than silently NOPing on `Err`.
pub(crate) fn pdac_register_for_slot(regs: &XrdcRegisters, slot: u16) -> &XrdcPdacRegisters {
    let slot = slot as usize;
    if slot < PDAC_PAC0_SLOT_COUNT {
        &regs.pdac_0_31[slot]
    } else if (128..128 + PDAC_PAC1_SLOT_COUNT).contains(&slot) {
        &regs.pdac_128_161[slot - 128]
    } else if (256..256 + PDAC_PAC2_SLOT_COUNT).contains(&slot) {
        &regs.pdac_256_289[slot - 256]
    } else if (384..384 + PDAC_PAC3_WINDOW_COUNT).contains(&slot) {
        &regs.pdac_384_408[slot - 384]
    } else if (512..512 + PDAC_PAC4_WINDOW_COUNT).contains(&slot) {
        &regs.pdac_512_542[slot - 512]
    } else {
        panic!(
            "xrdc::pdac_register_for_slot: PDAC slot is in a reserved gap (chip-crate Peripheral enum is wrong)"
        );
    }
}

/// Walk the PDAC arrays touched by the per-instance config and invalidate
/// every slot that is not in the `entries` list. Together with the
/// programmed entries this realises the deny-by-default semantic.
pub(crate) fn invalidate_pdac_window(window: &[XrdcPdacRegisters]) {
    for pdac in window {
        pdac.w1.modify(PDAC_W1::VLD::Invalid);
    }
}

/// Invalidate every MDA word in `slots` — deny-by-default before programming.
///
/// Caller passes the slice that belongs to this XRDC instance: `&regs.mda`
/// for XRDC_0 (24 slots), `&regs.mda[..8]` for XRDC_1 (8 slots per RM
/// §15.3.2 Table 34). Writing `VLD::Invalid` to MDA slots beyond the
/// instance's documented count would scribble reserved register space.
pub(crate) fn invalidate_mda(slots: &[XrdcMdaRegisters]) {
    for slot in slots {
        for word in slot.word.iter() {
            word.modify(MDA::VLD::Invalid);
        }
    }
}

/// Invalidate every MRGD descriptor in `mrgds` — same deny-by-default
/// rationale. Caller scopes the slice to the instance's MRC count × 16.
pub(crate) fn invalidate_mrgd(mrgds: &[XrdcMemoryRegionDescriptorRegisters]) {
    for mrgd in mrgds {
        mrgd.w3.modify(MRGD_W3::VLD::Invalid);
    }
}

/// Program one PDAC entry. Performs the W2-style "clear VLD → write policy →
/// re-assert VLD" dance documented in RM §15.7.3.17 LK2/VLD interaction.
pub(crate) fn program_pdac(pdac: &XrdcPdacRegisters, raw: PdacRaw, lock: bool) {
    xrdc_sync();
    pdac.w1.modify(PDAC_W1::VLD::Invalid);
    xrdc_sync();
    // SE/SNUM stay zero (no hardware semaphore in v1).
    pdac.w0.set(raw.w0);
    xrdc_sync();
    let mut w1 = raw.w1_acp | (1u32 << 31); // VLD = Valid
    if lock {
        // LK2 = 11b — locked until reset.
        w1 |= 0b11u32 << 29;
    }
    pdac.w1.set(w1);
    xrdc_sync();
}

/// Program one MRGD descriptor. Mirrors the PDAC dance for W3.
pub(crate) fn program_mrgd(mrgd: &XrdcMemoryRegionDescriptorRegisters, raw: MrgdRaw, lock: bool) {
    xrdc_sync();
    mrgd.w3.modify(MRGD_W3::VLD::Invalid);
    xrdc_sync();
    // `MrgdRaw::region` already pre-shifts `srtaddr_field` / `endaddr_field`
    // into their final MRGD_W{0,1} register-word position (RM §15.7.3.18:
    // SRTADDR/ENDADDR sit at bit 1, carrying address bits 35:5). Writing
    // them raw is exactly what apply needs — DO NOT add a `<< 1` here or you
    // double-shift and brick the address. Unit-tested in `tests::program_mrgd_writes_pre_shifted_register_words`.
    mrgd.w0.set(raw.srtaddr_field);
    mrgd.w1.set(raw.endaddr_field);
    mrgd.w2.set(raw.acp_lo);
    xrdc_sync();
    let mut w3 = raw.acp_hi | (1u32 << 31); // VLD = Valid
    if lock {
        w3 |= 0b11u32 << 29;
    }
    mrgd.w3.set(w3);
    xrdc_sync();
}

/// Program one MDA word for a core initiator (DFMT0). Core MDACs have up to
/// 8 words; in v1 we only ever use word 0 and zero-validate the rest in the
/// pre-apply invalidation pass.
pub(crate) fn program_mda_core(slot: &XrdcMdaRegisters, raw: MdaRaw, lock: bool) {
    xrdc_sync();
    slot.word[0].modify(MDA::VLD::Invalid);
    xrdc_sync();
    let mut w = raw.word | (1u32 << 31); // VLD = Valid
    if lock {
        w |= 1u32 << 30; // LK1 = Locked
    }
    slot.word[0].set(w);
    xrdc_sync();
}
/// Program one MDA word for a bus initiator (DFMT1).
pub(crate) fn program_mda_bus(slot: &XrdcMdaRegisters, raw: MdaRaw, lock: bool) {
    program_mda_core(slot, raw, lock);
}

/// Additively patch one PDAC entry, preserving hardware-owned fields.
pub(crate) fn patch_pdac(pdac: &XrdcPdacRegisters, raw: PdacRaw) -> Result<(), XrdcPatchError> {
    let w1 = pdac.w1.get();
    if w1 & LK2_MASK != 0 {
        return Err(XrdcPatchError::LockedDescriptor);
    }
    xrdc_sync();
    pdac.w1.set(w1 & !(1u32 << 31));
    xrdc_sync();
    let w0 = pdac.w0.get();
    pdac.w0.set((w0 & !ACP_LO_MASK) | (raw.w0 & ACP_LO_MASK));
    xrdc_sync();
    pdac.w1
        .set((w1 & !ACP_HI_MASK) | (raw.w1_acp & ACP_HI_MASK) | (1u32 << 31));
    xrdc_sync();
    Ok(())
}

/// Additively patch one MDA word, preserving hardware-owned fields.
pub(crate) fn patch_mda(slot: &XrdcMdaRegisters, raw: MdaRaw) -> Result<(), XrdcPatchError> {
    let current = slot.word[0].get();
    if current & (1u32 << 30) != 0 {
        return Err(XrdcPatchError::LockedDescriptor);
    }
    let mask = if raw.is_bus {
        (1u32 << MDA_DFMT_SHIFT)
            | (1u32 << MDA_DIDB_SHIFT)
            | (0b11u32 << MDA_SA_SHIFT)
            | (0b11u32 << MDA_PA_SHIFT)
            | (0xFu32 << MDA_DID_SHIFT)
    } else {
        0xFu32 << MDA_DID_SHIFT
    };
    let new = (current & !mask) | raw.word;
    xrdc_sync();
    slot.word[0].set(new & !(1u32 << 31));
    xrdc_sync();
    slot.word[0].set(new | (1u32 << 31));
    xrdc_sync();
    Ok(())
}
/// Additively patch one static MRGD descriptor.
pub(crate) fn patch_mrgd(
    mrgds: &[XrdcMemoryRegionDescriptorRegisters],
    raw: MrgdRaw,
    nmrgd: usize,
) -> Result<(), XrdcPatchError> {
    let mrc_base = raw.mrc as usize * 16;
    let mrc_end = mrc_base + nmrgd;
    let mut first_unused = None;
    let mut matched = None;

    for i in mrc_base..mrc_end {
        let mrgd = &mrgds[i];
        let w3 = mrgd.w3.get();
        if w3 & (1u32 << 31) != 0 {
            if mrgd.w0.get() == raw.srtaddr_field && mrgd.w1.get() == raw.endaddr_field {
                if matched.replace(i).is_some() {
                    return Err(XrdcPatchError::AmbiguousTarget);
                }
            }
        } else if first_unused.is_none() {
            first_unused = Some(i);
        }
    }

    if let Some(i) = matched {
        let mrgd = &mrgds[i];
        let w3 = mrgd.w3.get();
        if w3 & LK2_MASK != 0 {
            return Err(XrdcPatchError::LockedDescriptor);
        }
        xrdc_sync();
        mrgd.w3.set(w3 & !(1u32 << 31));
        xrdc_sync();
        let w2 = mrgd.w2.get();
        mrgd.w2
            .set((w2 & !ACP_LO_MASK) | (raw.acp_lo & ACP_LO_MASK));
        xrdc_sync();
        mrgd.w3
            .set((w3 & !(1u32 << 31) & !ACP_HI_MASK) | (raw.acp_hi & ACP_HI_MASK) | (1u32 << 31));
        xrdc_sync();
        Ok(())
    } else if let Some(i) = first_unused {
        let mrgd = &mrgds[i];
        xrdc_sync();
        mrgd.w0.set(raw.srtaddr_field);
        mrgd.w1.set(raw.endaddr_field);
        mrgd.w2.set(raw.acp_lo);
        xrdc_sync();
        mrgd.w3.set(raw.acp_hi | (1u32 << 31));
        xrdc_sync();
        Ok(())
    } else {
        Err(XrdcPatchError::NoFreeDescriptor)
    }
}
/// Search and patch one MRGD descriptor within its hardware NMRGD budget.
///
/// Exact static ranges may allocate a free descriptor. A runtime containment
/// search must identify policy installed by an earlier boot stage, so a miss is
/// reported rather than allocating a second overlapping descriptor.
pub(crate) fn search_and_patch_mrgd(
    mrgds: &[XrdcMemoryRegionDescriptorRegisters],
    raw: MrgdRaw,
    target: MrgdTarget,
    nmrgd: usize,
) -> Result<MrgdPatchOutcome, XrdcPatchError> {
    let mrc_base = raw.mrc as usize * 16;
    let mrc_end = mrc_base + nmrgd;
    let mut first_unused = None;
    let mut match_idx = None;

    for i in mrc_base..mrc_end {
        let mrgd = &mrgds[i];
        let w3 = mrgd.w3.get();
        if w3 & (1u32 << 31) != 0 {
            let matched = match target {
                MrgdTarget::ExactRange => {
                    mrgd.w0.get() == raw.srtaddr_field && mrgd.w1.get() == raw.endaddr_field
                }
                MrgdTarget::ContainsAddress(probe_addr) => {
                    let srtaddr = (mrgd.w0.get() & !1u32) << 4;
                    let endaddr = ((mrgd.w1.get() & !1u32) << 4) | 0x1F;
                    probe_addr >= srtaddr && probe_addr <= endaddr
                }
            };
            if matched {
                if match_idx.replace(i).is_some() {
                    return Err(XrdcPatchError::AmbiguousTarget);
                }
            }
        } else if first_unused.is_none() {
            first_unused = Some(i);
        }
    }

    if let Some(i) = match_idx {
        let mrgd = &mrgds[i];
        let w3 = mrgd.w3.get();
        if w3 & LK2_MASK != 0 {
            return Err(XrdcPatchError::LockedDescriptor);
        }
        xrdc_sync();
        mrgd.w3.set(w3 & !(1u32 << 31));
        xrdc_sync();
        let w2 = mrgd.w2.get();
        mrgd.w2
            .set((w2 & !ACP_LO_MASK) | (raw.acp_lo & ACP_LO_MASK));
        xrdc_sync();
        mrgd.w3
            .set((w3 & !(1u32 << 31) & !ACP_HI_MASK) | (raw.acp_hi & ACP_HI_MASK) | (1u32 << 31));
        xrdc_sync();
        Ok(MrgdPatchOutcome::PatchedExisting { slot: i })
    } else if matches!(target, MrgdTarget::ContainsAddress(_)) {
        Err(XrdcPatchError::MissingTarget)
    } else if let Some(i) = first_unused {
        let mrgd = &mrgds[i];
        xrdc_sync();
        mrgd.w0.set(raw.srtaddr_field);
        mrgd.w1.set(raw.endaddr_field);
        mrgd.w2.set(raw.acp_lo);
        xrdc_sync();
        mrgd.w3.set(raw.acp_hi | (1u32 << 31));
        xrdc_sync();
        Ok(MrgdPatchOutcome::AllocatedNew { slot: i })
    } else {
        Err(XrdcPatchError::NoFreeDescriptor)
    }
}

/// Allocate a static MRGD only after proving no valid descriptor overlaps it.
///
/// This is for a cold boot that has no predecessor-owned descriptor. Unlike
/// [`search_and_patch_mrgd`] containment matching, it never guesses around an
/// unknown overlapping policy.
pub(crate) fn allocate_unmapped_exact_mrgd(
    mrgds: &[XrdcMemoryRegionDescriptorRegisters],
    raw: MrgdRaw,
    nmrgd: usize,
) -> Result<MrgdPatchOutcome, XrdcPatchError> {
    let mrc_base = raw.mrc as usize * 16;
    let mrc_end = mrc_base + nmrgd;
    let target_start = raw.srtaddr_field & !1;
    let target_end = raw.endaddr_field & !1;
    let mut first_unused = None;

    for i in mrc_base..mrc_end {
        let mrgd = &mrgds[i];
        let w3 = mrgd.w3.get();
        if w3 & (1u32 << 31) == 0 {
            if first_unused.is_none() {
                first_unused = Some(i);
            }
            continue;
        }

        let start = mrgd.w0.get() & !1;
        let end = mrgd.w1.get() & !1;
        if start <= target_end && target_start <= end {
            return Err(XrdcPatchError::AmbiguousTarget);
        }
    }

    let Some(i) = first_unused else {
        return Err(XrdcPatchError::NoFreeDescriptor);
    };
    let mrgd = &mrgds[i];
    xrdc_sync();
    mrgd.w0.set(raw.srtaddr_field);
    mrgd.w1.set(raw.endaddr_field);
    mrgd.w2.set(raw.acp_lo);
    xrdc_sync();
    mrgd.w3.set(raw.acp_hi | (1u32 << 31));
    xrdc_sync();
    Ok(MrgdPatchOutcome::AllocatedNew { slot: i })
}

// =============================================================================
// Compile-time invariants — verify the arithmetic packer agrees with the
// typed `tock-registers` bitfield definitions.
// =============================================================================
//
// `acp_bits_lo` assumes 3 bits per domain starting at bit 0 of PDAC_W0 / MRGD_W2.
// If the bitfield definitions ever drift, these const_asserts fail at build
// time with a meaningful message. They are the bridge that justifies the
// arithmetic packer over `Field::val + Field::val + …` (which is not const fn
// in tock-registers 0.10 — see fields.rs:275).

const _: () = {
    // Anchors: D0ACP is at bit 0; D7ACP is at bit 21. Together these prove the
    // uniform "3 bits × domain index" layout the low-bank packer relies on.
    assert!(
        PDAC_W0::D0ACP.shift == 0,
        "PDAC_W0::D0ACP must start at bit 0"
    );
    assert!(
        PDAC_W0::D7ACP.shift == 21,
        "PDAC_W0::D7ACP must start at bit 21 (3 bits × 7)"
    );
    assert!(
        MRGD_W2::D0ACP.shift == 0,
        "MRGD_W2::D0ACP must start at bit 0"
    );
    assert!(
        MRGD_W2::D7ACP.shift == 21,
        "MRGD_W2::D7ACP must start at bit 21"
    );
    // High-bank anchors: D8ACP at bit 0, D15ACP at bit 21 of PDAC_W1 /
    // MRGD_W3 — same 3-bit-per-domain layout, restarted at bit 0. The
    // `acp_bits_hi` packer relies on these offsets when packing PFE host
    // interfaces D12..D15.
    assert!(
        PDAC_W1::D8ACP.shift == 0,
        "PDAC_W1::D8ACP must start at bit 0"
    );
    assert!(
        PDAC_W1::D15ACP.shift == 21,
        "PDAC_W1::D15ACP must start at bit 21 (3 bits × (15-8))"
    );
    assert!(
        MRGD_W3::D8ACP.shift == 0,
        "MRGD_W3::D8ACP must start at bit 0"
    );
    assert!(
        MRGD_W3::D15ACP.shift == 21,
        "MRGD_W3::D15ACP must start at bit 21"
    );
    // Domain enum discriminants must match the hardware DID values.
    assert!(Domain::Debugger as u32 == 0);
    assert!(Domain::M7_0 as u32 == 1);
    assert!(Domain::A53 as u32 == 7);
    assert!(Domain::PfeHif0 as u32 == 12);
    assert!(Domain::PfeHif3 as u32 == 15);
    // Access enum discriminants must match RM Table 49 ACP codes.
    assert!(Access::None as u32 == 0b000);
    assert!(Access::SupervisorRw as u32 == 0b010);
    assert!(Access::FullRw as u32 == 0b111);
    // Spot-check the low-bank packer end-to-end: A53 (D7) granted FullRw
    // lands at bits 21..24 with value 0b111 → 0xE00000.
    assert!(acp_bits_lo(Domain::A53, Access::FullRw) == 0x00E0_0000);
    // Low bank ignores high-bank domains.
    assert!(acp_bits_lo(Domain::PfeHif0, Access::FullRw) == 0);
    // Spot-check the high-bank packer end-to-end: PfeHif0 (D12) granted
    // FullRw lands at bits 12..15 of W1/W3 (offset (12-8)*3 = 12) with value
    // 0b111 → 0x7000.
    assert!(acp_bits_hi(Domain::PfeHif0, Access::FullRw) == 0x0000_7000);
    // High bank ignores low-bank domains.
    assert!(acp_bits_hi(Domain::M7_0, Access::FullRw) == 0);
};

// =============================================================================
// Host unit tests — verify precomputed register words against RM-derived
// reference values. Runs with `cargo test -p nxp_s32g3 --target
// x86_64-unknown-linux-gnu --lib` (no MMIO needed; the tests target the
// arithmetic that turns a board's declarative const into the exact `u32`
// the apply sequence will store).
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- PDAC packing ----------

    /// Granting one (domain, access) lands at the correct bit position in
    /// PDAC_W0 with the documented ACP code.
    #[test]
    fn pdac_single_grant_lands_at_typed_bitfield_position() {
        let raw = PdacRaw::new(145).grant(Domain::M7_0, Access::SupervisorRw);
        // M7_0 = DID 1; SupervisorRw = 0b010. Field D1ACP at shift 3.
        let expected_w0 = (Access::SupervisorRw as u32) << PDAC_W0::D1ACP.shift;
        assert_eq!(raw.w0, expected_w0);
        assert_eq!(
            raw.w1_acp, 0,
            "D1 is in the low bank; W1 ACP bits stay zero"
        );
        assert_eq!(raw.slot, 145);
    }

    /// Multiple `.grant()` calls OR together without clobbering each other.
    #[test]
    fn pdac_multi_grant_ors_into_register_word() {
        let raw = PdacRaw::new(140)
            .grant(Domain::M7_0, Access::SupervisorRw)
            .grant(Domain::A53, Access::FullRw)
            .grant(Domain::Hse, Access::SecureRw);
        let expected = ((Access::SupervisorRw as u32) << PDAC_W0::D1ACP.shift)
            | ((Access::FullRw as u32) << PDAC_W0::D7ACP.shift)
            | ((Access::SecureRw as u32) << PDAC_W0::D6ACP.shift);
        assert_eq!(raw.w0, expected);
        assert_eq!(raw.w1_acp, 0);
    }

    /// Granting the same domain a second time monotonically widens (OR
    /// semantics) — duplicate calls do not overwrite. This is the documented
    /// behaviour of `.grant()` and the test pins it so it can't silently flip.
    #[test]
    fn pdac_repeat_grant_or_widens() {
        // 0b010 | 0b101 = 0b111 — second grant adds NS read bits on top of
        // SupervisorRw without dropping the secure-priv-RW bit.
        let raw = PdacRaw::new(0)
            .grant(Domain::M7_0, Access::SupervisorRw)
            .grant(Domain::M7_0, Access::SecureRwNsRead);
        let expected = ((Access::SupervisorRw as u32 | Access::SecureRwNsRead as u32)
            << PDAC_W0::D1ACP.shift) as u32;
        assert_eq!(raw.w0, expected);
    }

    // ---------- MDA core (DFMT0) ----------

    /// Core initiator MDA word: DFMT=0, DID=domain, no override bits set.
    #[test]
    fn mda_core_word_format() {
        let raw = MdaRaw::core(/* master_idx = */ 8, Domain::M7_0);
        assert!(!raw.is_bus);
        assert_eq!(raw.master_idx, 8);
        // Bits 31..30 (VLD/LK1) and bit 29 (DFMT) all zero in the precomputed
        // word — the apply helper layers VLD and LK1 on top.
        assert_eq!(
            raw.word & (1 << 29),
            0,
            "DFMT must be 0 for core initiators"
        );
        // DID lives in bits 3..0; M7_0 = 1.
        assert_eq!(raw.word & 0xF, 1);
    }

    // ---------- MDA bus (DFMT1) ----------

    /// Bus initiator MDA word: DFMT=1, DID=domain, SA=00b, PA=00b.
    #[test]
    fn mda_bus_word_format() {
        let raw = MdaRaw::bus(/* master_idx = */ 11, Domain::Hse);
        assert!(raw.is_bus);
        assert_eq!(raw.master_idx, 11);
        assert_ne!(raw.word & (1 << 29), 0, "DFMT must be 1 for bus initiators");
        assert_eq!(raw.word & 0xF, Domain::Hse as u32, "DID = Hse (6)");
        assert_eq!(
            (raw.word >> MDA_SA_SHIFT) & 0b11,
            0,
            "SA defaults to UseInput"
        );
        assert_eq!(
            (raw.word >> MDA_PA_SHIFT) & 0b11,
            0,
            "PA defaults to UseInput"
        );
    }

    /// `.with_sa(ForceSecure)` writes `10b` into bits 7..6 without disturbing
    /// other fields.
    #[test]
    fn mda_bus_force_secure_patches_sa_field() {
        let raw = MdaRaw::bus(11, Domain::Hse).with_sa(SecureAttr::ForceSecure);
        assert_eq!((raw.word >> MDA_SA_SHIFT) & 0b11, 0b10);
        // DID untouched.
        assert_eq!(raw.word & 0xF, Domain::Hse as u32);
        // PA untouched.
        assert_eq!((raw.word >> MDA_PA_SHIFT) & 0b11, 0);
    }

    /// `.with_pa(ForcePrivileged)` writes `10b` into bits 5..4 without
    /// disturbing other fields.
    #[test]
    fn mda_bus_force_privileged_patches_pa_field() {
        let raw = MdaRaw::bus(11, Domain::Hse).with_pa(PrivAttr::ForcePrivileged);
        assert_eq!((raw.word >> MDA_PA_SHIFT) & 0b11, 0b10);
        // SA untouched.
        assert_eq!((raw.word >> MDA_SA_SHIFT) & 0b11, 0);
    }

    // ---------- MRGD address encoding ----------

    /// `region()` must produce `srtaddr_field` / `endaddr_field` that, after
    /// the `program_mrgd` shift, equal the RM-derived MRGD_W0 / MRGD_W1
    /// register words. RM §15.7.3.18: SRTADDR / ENDADDR occupy bits 31:1 and
    /// carry bits 35:5 of the (40-bit) address.
    ///
    /// For a 32-bit address `a`, the expected register word is
    /// `((a & !0x1F) >> 5) << MRGD_W0::SRTADDR.shift` (which is
    /// `(a & !0x1F) >> 4` since `shift == 1`).
    #[test]
    fn mrgd_region_field_encodes_rm_bits_35_5_at_offset_1() {
        const RANGES: &[MrcRange] = &[MrcRange {
            idx: 2,
            start: 0x3400_0000,
            end: 0x344F_FFFF,
            nmrgd: 16,
        }];
        let raw = MrgdRaw::region(0x3420_0000, 0x342F_FFFF, RANGES);

        let expected_w0 = ((0x3420_0000_u32 & !0x1F) >> 5) << MRGD_W0::SRTADDR.shift;
        let expected_w1 = ((0x342F_FFFF_u32 & !0x1F) >> 5) << MRGD_W1::ENDADDR.shift;

        assert_eq!(
            raw.srtaddr_field, expected_w0,
            "srtaddr_field MUST be the final MRGD_W0 register word \
             (RM §15.7.3.18: SRTADDR = bits 35:5 of start address, placed at \
             register bit 1) — program_mrgd writes raw.srtaddr_field directly",
        );
        assert_eq!(
            raw.endaddr_field, expected_w1,
            "endaddr_field MUST be the final MRGD_W1 register word",
        );
        assert_eq!(raw.mrc, 2, "0x3420_0000 lives in MRC2 (SRAM_0..3)");
    }

    /// Bottom-of-range and top-of-32-bit corners exercise the alignment math
    /// (start needs low 5 bits == 0, end needs low 5 bits == 0x1F).
    #[test]
    fn mrgd_region_extreme_addresses() {
        const RANGES: &[MrcRange] = &[MrcRange {
            idx: 0,
            start: 0x0000_0000,
            end: 0xFFFF_FFFF,
            nmrgd: 16,
        }];
        // Smallest possible region: a single 32-byte line at zero.
        let raw_lo = MrgdRaw::region(0x0000_0000, 0x0000_001F, RANGES);
        assert_eq!(raw_lo.srtaddr_field, 0);
        // End-field = bits 31:5 of 0x1F = 0, shifted up by MRGD_W1 offset 1.
        assert_eq!(raw_lo.endaddr_field, 0);

        // A 32-byte line at the very top of 32-bit space. With low 5 bits of
        // start = 0 and low 5 bits of end = 0x1F, the field math collapses
        // start and end to the same MRGD_W{0,1} register value (one line).
        let raw_hi = MrgdRaw::region(0xFFFF_FFE0, 0xFFFF_FFFF, RANGES);
        let expected_field = (0xFFFF_FFE0_u32 >> 5) << MRGD_W0::SRTADDR.shift;
        assert_eq!(raw_hi.srtaddr_field, expected_field);
        assert_eq!(raw_hi.endaddr_field, expected_field);
        // For 32-bit-only addresses bits 35:32 are zero, so the top SRTADDR
        // register bits (above bit 28) are guaranteed zero. This pins the
        // upper-bound for sanity.
        assert_eq!(raw_hi.srtaddr_field & 0xF000_0000, 0);
    }

    // ---------- End-to-end: program_* writes the precomputed reg word ----------
    //
    // These tests are what makes the host-test surface valuable: they
    // construct a fake register block in zero-initialised heap memory, run
    // the actual `program_pdac` / `program_mrgd` / `program_mda_*` helpers
    // against it, and read the resulting register words back to compare
    // against the RM-derived expected values. They catch any mismatch between
    // the pre-computed `*Raw` fields and what `apply` actually streams to
    // MMIO — exactly the class of bug that bricks a board at GVLD=1.
    //
    // Safety: `tock-registers`' `ReadWrite<u32, _>` is `#[repr(transparent)]`
    // over `UnsafeCell<u32>`, which itself is `repr(transparent)` over `u32`.
    // `register_structs!` arranges the cells contiguously at the declared
    // byte offsets, with `_reserved` filler in any gaps. So a zero-initialised
    // backing `[u32; N]` of the right length aliases as a valid register
    // block for read/write purposes on the host (where there is no real MMIO
    // hardware to honour the volatile semantics).

    /// `program_mrgd` writes the pre-shifted register words verbatim. This
    /// guards against the `<< 1` double-shift regression that would otherwise
    /// silently double every MRGD address.
    #[test]
    fn program_mrgd_writes_pre_shifted_register_words() {
        const RANGES: &[MrcRange] = &[MrcRange {
            idx: 2,
            start: 0x3400_0000,
            end: 0x344F_FFFF,
            nmrgd: 16,
        }];
        let raw =
            MrgdRaw::region(0x3420_0000, 0x342F_FFFF, RANGES).grant(Domain::M7_0, Access::FullRw);

        // 4 u32s = MRGD_W{0,1,2,3}; XrdcMemoryRegionDescriptorRegisters is 0x10 bytes.
        let mut backing = [0u32; 4];
        // SAFETY: ReadWrite<u32, _> is repr(transparent) over UnsafeCell<u32>,
        // which is repr(transparent) over u32. The register block declares 4
        // contiguous u32-sized cells at byte offsets 0, 4, 8, 12 — exactly
        // matching `backing`'s layout.
        let regs: &XrdcMemoryRegionDescriptorRegisters =
            unsafe { &*(backing.as_mut_ptr() as *const XrdcMemoryRegionDescriptorRegisters) };

        program_mrgd(regs, raw, /* lock = */ true);

        // Expected register words derived directly from RM §15.7.3.18:
        let expected_w0 = ((0x3420_0000_u32 & !0x1F) >> 5) << MRGD_W0::SRTADDR.shift;
        let expected_w1 = ((0x342F_FFFF_u32 & !0x1F) >> 5) << MRGD_W1::ENDADDR.shift;
        let expected_w2 = (Access::FullRw as u32) << MRGD_W2::D1ACP.shift;
        // W3: ACP-hi bits (none here) | VLD=1 at bit 31 | LK2=11b at bits 30:29.
        let expected_w3 = 0u32 | (1 << 31) | (0b11 << 29);

        assert_eq!(
            backing[0], expected_w0,
            "MRGD_W0 must equal start[35:5]<<1; mismatch means program_mrgd \
             is double-shifting the address (regression in the SRTADDR write)",
        );
        assert_eq!(backing[1], expected_w1, "MRGD_W1 ENDADDR encoding");
        assert_eq!(backing[2], expected_w2, "MRGD_W2 ACP-lo packing");
        assert_eq!(backing[3], expected_w3, "MRGD_W3 VLD + LK2");
    }

    /// `program_pdac` writes the pre-computed `w0` verbatim and OR-stamps
    /// `VLD=1` (and `LK2=11b` when locked) into `w1`.
    #[test]
    fn program_pdac_writes_w0_verbatim_and_validates_w1() {
        let raw = PdacRaw::new(145)
            .grant(Domain::M7_0, Access::SupervisorRw)
            .grant(Domain::A53, Access::FullRw);

        // 2 u32s = PDAC_W{0,1}; XrdcPdacRegisters is 0x08 bytes.
        let mut backing = [0u32; 2];
        let regs: &XrdcPdacRegisters =
            unsafe { &*(backing.as_mut_ptr() as *const XrdcPdacRegisters) };

        program_pdac(regs, raw, /* lock = */ true);

        let expected_w0 = ((Access::SupervisorRw as u32) << PDAC_W0::D1ACP.shift)
            | ((Access::FullRw as u32) << PDAC_W0::D7ACP.shift);
        let expected_w1 = (1 << 31) | (0b11 << 29); // VLD=Valid | LK2=Locked

        assert_eq!(backing[0], expected_w0, "PDAC_W0 ACP-lo packing");
        assert_eq!(backing[1], expected_w1, "PDAC_W1 VLD + LK2");
    }

    /// `program_mda_core` lays down DFMT=0, DID=domain, VLD=1, and LK1=1.
    #[test]
    fn program_mda_core_writes_dfmt0_word_with_vld_and_lk1() {
        let raw = MdaRaw::core(/* master_idx = */ 8, Domain::M7_0);

        // XrdcMdaRegisters wraps `word: [ReadWrite<u32, MDA::Register>; 8]`
        // for the 8 PID-match slots; we only use slot 0.
        let mut backing = [0u32; 8];
        let regs: &XrdcMdaRegisters =
            unsafe { &*(backing.as_mut_ptr() as *const XrdcMdaRegisters) };

        program_mda_core(regs, raw, /* lock = */ true);

        let expected = (1u32 << 31) | (1u32 << 30) | (Domain::M7_0 as u32);
        assert_eq!(
            backing[0], expected,
            "MDA word[0]: VLD=1 | LK1=1 | DID=M7_0"
        );
        // No spillover into the other 7 PID-match slots.
        for i in 1..8 {
            assert_eq!(backing[i], 0, "MDA word[{i}] must be untouched");
        }
    }

    /// `program_mda_bus` lays down DFMT=1, DID=domain, VLD=1, LK1=1.
    /// `.with_sa(ForceSecure).with_pa(ForcePrivileged)` overrides survive.
    #[test]
    fn program_mda_bus_writes_dfmt1_word_with_overrides() {
        let raw = MdaRaw::bus(/* master_idx = */ 11, Domain::Hse)
            .with_sa(SecureAttr::ForceSecure)
            .with_pa(PrivAttr::ForcePrivileged);

        let mut backing = [0u32; 8];
        let regs: &XrdcMdaRegisters =
            unsafe { &*(backing.as_mut_ptr() as *const XrdcMdaRegisters) };

        program_mda_bus(regs, raw, /* lock = */ true);

        // VLD | LK1 | DFMT=Bus | SA=ForceSecure | PA=ForcePrivileged | DID=Hse
        let expected = (1u32 << 31)
            | (1u32 << 30)
            | (1u32 << MDA_DFMT_SHIFT)
            | (0b10u32 << MDA_SA_SHIFT)
            | (0b10u32 << MDA_PA_SHIFT)
            | (Domain::Hse as u32);
        assert_eq!(backing[0], expected);
    }

    /// `.grant()` on a region behaves like PDAC grant: stores ACP bits into
    /// `acp_lo` at the right offset.
    #[test]
    fn mrgd_grant_packs_into_acp_lo() {
        const RANGES: &[MrcRange] = &[MrcRange {
            idx: 2,
            start: 0x3400_0000,
            end: 0x344F_FFFF,
            nmrgd: 16,
        }];
        let raw =
            MrgdRaw::region(0x3420_0000, 0x342F_FFFF, RANGES).grant(Domain::M7_0, Access::FullRw);
        let expected_acp_lo = (Access::FullRw as u32) << MRGD_W2::D1ACP.shift;
        assert_eq!(raw.acp_lo, expected_acp_lo);
        assert_eq!(raw.acp_hi, 0);
    }

    // ---------- Coverage lookup helpers ----------

    /// `mrc_for_addr` resolves both endpoints to the same MRC when they're
    /// in-range, and returns `None` outside any window.
    #[test]
    fn mrc_for_addr_resolution() {
        const RANGES: &[MrcRange] = &[
            MrcRange {
                idx: 2,
                start: 0x3400_0000,
                end: 0x344F_FFFF,
                nmrgd: 16,
            },
            MrcRange {
                idx: 7,
                start: 0x0000_0000,
                end: 0x03FF_FFFF,
                nmrgd: 4,
            },
        ];
        assert_eq!(mrc_for_addr(0x3420_0000, RANGES), Some(2));
        assert_eq!(mrc_for_addr(0x344F_FFFF, RANGES), Some(2));
        assert_eq!(mrc_for_addr(0x0000_0000, RANGES), Some(7));
        assert_eq!(mrc_for_addr(0x8000_0000, RANGES), None);
    }

    /// `nmrgd_for_mrc` returns the budget regardless of declaration order.
    #[test]
    fn nmrgd_for_mrc_returns_budget() {
        const RANGES: &[MrcRange] = &[
            MrcRange {
                idx: 7,
                start: 0x0000_0000,
                end: 0x03FF_FFFF,
                nmrgd: 4,
            },
            MrcRange {
                idx: 2,
                start: 0x3400_0000,
                end: 0x344F_FFFF,
                nmrgd: 16,
            },
        ];
        assert_eq!(nmrgd_for_mrc(2, RANGES), 16);
        assert_eq!(nmrgd_for_mrc(7, RANGES), 4);
    }
    // ======================================================================
    // search_and_patch_mrgd parity tests vs the old raw-MMIO board code.
    // ======================================================================
    /// Standby SRAM scenario: a prior boot stage has pre-programmed a descriptor
    /// that contains the probe address.  The old raw code overwrote W2 with
    /// `0x00FFFFFF` (all D0-D7 FullRw) and preserved everything else in W3.
    /// The new code ORs only the explicitly requested domains.
    ///
    /// This test pins the new register words so we can compare them with
    /// the old behaviour and catch any drift.
    #[test]
    fn search_and_patch_mrgd_contains_hse_existing_descriptor() {
        // Each MRGD slot is 32 bytes (4 u32 words + 4 u32 reserved/padding).
        const WORDS_PER_SLOT: usize = 8;
        let mut backing = [0u32; WORDS_PER_SLOT * 16];
        let mrgds: &[XrdcMemoryRegionDescriptorRegisters] =
            unsafe { core::slice::from_raw_parts(backing.as_mut_ptr() as *const _, 16) };
        // Prior-boot-stage descriptor at slot 1: Standby SRAM 0x24000000-0x24007FFF, NSE=1
        backing[1 * WORDS_PER_SLOT + 0] = 0x0240_0001; // W0: SRTADDR + NSE=1
        backing[1 * WORDS_PER_SLOT + 1] = 0x0240_07FE; // W1: ENDADDR
        backing[1 * WORDS_PER_SLOT + 2] = 0x00FF_0000; // W2: D4-D7 already granted
        backing[1 * WORDS_PER_SLOT + 3] = 0x8000_0000; // W3: VLD=1
        let raw = MrgdRaw::region(
            0x2400_0000,
            0x2400_7FFF,
            &[MrcRange {
                idx: 0,
                start: 0x2400_0000,
                end: 0x33FF_FFFF,
                nmrgd: 4,
            }],
        )
        .grant(Domain::M7_0, Access::FullRw)
        .grant(Domain::A53, Access::FullRw);
        let outcome =
            search_and_patch_mrgd(mrgds, raw, MrgdTarget::ContainsAddress(0x2400_6008), 4).unwrap();
        assert_eq!(
            outcome,
            MrgdPatchOutcome::PatchedExisting { slot: 1 },
            "must patch the prior-boot-stage descriptor at slot 1"
        );
        // W0 and W1 must be untouched (NSE bit preserved).
        assert_eq!(backing[1 * WORDS_PER_SLOT + 0], 0x0240_0001, "W0 unchanged");
        assert_eq!(backing[1 * WORDS_PER_SLOT + 1], 0x0240_07FE, "W1 unchanged");
        // W2: ACP level is replaced, not OR'd. The patch specifies D1 (M7_0)
        // and D7 (A53) FullRw; all other domains are reset to None.
        let expected_w2 = ((Access::FullRw as u32) << MRGD_W2::D1ACP.shift)
            | ((Access::FullRw as u32) << MRGD_W2::D7ACP.shift);
        assert_eq!(
            backing[1 * WORDS_PER_SLOT + 2],
            expected_w2,
            "W2: ACP replaced with patch levels, prior D4-D7 grants cleared"
        );
        assert_eq!(
            backing[1 * WORDS_PER_SLOT + 3],
            0x8000_0000,
            "W3 unchanged (no acp_hi requested)"
        );
    }
    #[test]
    fn search_and_patch_mrgd_alloc_matches_old_raw_code_when_all_domains_granted() {
        const WORDS_PER_SLOT: usize = 8;
        let mut backing = [0u32; WORDS_PER_SLOT * 16];
        let mrgds: &[XrdcMemoryRegionDescriptorRegisters] =
            unsafe { core::slice::from_raw_parts(backing.as_mut_ptr() as *const _, 16) };
        let raw = MrgdRaw::region(
            0x2400_0000,
            0x2400_7FFF,
            &[MrcRange {
                idx: 0,
                start: 0x2400_0000,
                end: 0x33FF_FFFF,
                nmrgd: 4,
            }],
        )
        .grant(Domain::Debugger, Access::FullRw)
        .grant(Domain::M7_0, Access::FullRw)
        .grant(Domain::M7_1, Access::FullRw)
        .grant(Domain::M7_2, Access::FullRw)
        .grant(Domain::M7_3, Access::FullRw)
        .grant(Domain::EDma, Access::FullRw)
        .grant(Domain::Hse, Access::FullRw)
        .grant(Domain::A53, Access::FullRw);
        let outcome = search_and_patch_mrgd(mrgds, raw, MrgdTarget::ExactRange, 4).unwrap();
        assert_eq!(
            outcome,
            MrgdPatchOutcome::AllocatedNew { slot: 0 },
            "must allocate in first unused slot"
        );
        assert_eq!(backing[0 * WORDS_PER_SLOT + 0], 0x0240_0000, "W0 SRTADDR");
        assert_eq!(backing[0 * WORDS_PER_SLOT + 1], 0x0240_07FE, "W1 ENDADDR");
        assert_eq!(
            backing[0 * WORDS_PER_SLOT + 2],
            0x00FF_FFFF,
            "W2 all D0-D7 FullRw"
        );
        assert_eq!(backing[0 * WORDS_PER_SLOT + 3], 0x8000_0000, "W3 VLD=1");
    }
    #[test]
    fn search_and_patch_mrgd_alloc_with_board_policy_is_more_restrictive_than_old() {
        const WORDS_PER_SLOT: usize = 8;
        let mut backing = [0u32; WORDS_PER_SLOT * 16];
        let mrgds: &[XrdcMemoryRegionDescriptorRegisters] =
            unsafe { core::slice::from_raw_parts(backing.as_mut_ptr() as *const _, 16) };
        let raw = MrgdRaw::region(
            0x2400_0000,
            0x2400_7FFF,
            &[MrcRange {
                idx: 0,
                start: 0x2400_0000,
                end: 0x33FF_FFFF,
                nmrgd: 4,
            }],
        )
        .grant(Domain::M7_0, Access::FullRw)
        .grant(Domain::A53, Access::FullRw);
        let _outcome = search_and_patch_mrgd(mrgds, raw, MrgdTarget::ExactRange, 4).unwrap();
        let expected_w2 = (Access::FullRw as u32) << MRGD_W2::D1ACP.shift
            | (Access::FullRw as u32) << MRGD_W2::D7ACP.shift;
        assert_eq!(backing[0 * WORDS_PER_SLOT + 2], expected_w2);
        assert_ne!(
            backing[0 * WORDS_PER_SLOT + 2],
            0x00FF_FFFF,
            "new code is more restrictive than old raw code"
        );
        assert_eq!(
            backing[0 * WORDS_PER_SLOT + 3],
            0x8000_0000,
            "W3 VLD=1, no lock"
        );
    }
    #[test]
    fn exact_range_does_not_match_when_hse_set_nse() {
        const WORDS_PER_SLOT: usize = 8;
        let mut backing = [0u32; WORDS_PER_SLOT * 16];
        let mrgds: &[XrdcMemoryRegionDescriptorRegisters] =
            unsafe { core::slice::from_raw_parts(backing.as_mut_ptr() as *const _, 16) };
        // Prior-boot-stage descriptor with NSE=1.
        backing[0 * WORDS_PER_SLOT + 1] = 0x0240_07FE;
        backing[0 * WORDS_PER_SLOT + 2] = 0;
        backing[0 * WORDS_PER_SLOT + 3] = 0x8000_0000;
        let raw = MrgdRaw::region(
            0x2400_0000,
            0x2400_7FFF,
            &[MrcRange {
                idx: 0,
                start: 0x2400_0000,
                end: 0x33FF_FFFF,
                nmrgd: 4,
            }],
        )
        .grant(Domain::M7_0, Access::FullRw);
        // ExactRange: W0 differs by NSE bit → no match → allocate new.
        let outcome = search_and_patch_mrgd(mrgds, raw, MrgdTarget::ExactRange, 4).unwrap();
        assert_eq!(
            outcome,
            MrgdPatchOutcome::AllocatedNew { slot: 1 },
            "ExactRange must NOT match when NSE differs"
        );
        // Verify the newly allocated slot has the raw srtaddr (no NSE bit).
        assert_eq!(backing[1 * WORDS_PER_SLOT + 0], raw.srtaddr_field);
    }
    #[test]
    fn contains_address_matches_even_when_hse_set_nse() {
        const WORDS_PER_SLOT: usize = 8;
        let mut backing = [0u32; WORDS_PER_SLOT * 16];
        let mrgds: &[XrdcMemoryRegionDescriptorRegisters] =
            unsafe { core::slice::from_raw_parts(backing.as_mut_ptr() as *const _, 16) };
        backing[0 * WORDS_PER_SLOT + 0] = 0x0240_0001; // W0: SRTADDR + NSE=1
        backing[0 * WORDS_PER_SLOT + 1] = 0x0240_07FE;
        backing[0 * WORDS_PER_SLOT + 2] = 0;
        backing[0 * WORDS_PER_SLOT + 3] = 0x8000_0000;
        // The VLD bit must be set or the existing-descriptor path will not match.
        assert_eq!(backing[0 * WORDS_PER_SLOT + 3], 0x8000_0000);
        let raw = MrgdRaw::region(
            0x2400_0000,
            0x2400_7FFF,
            &[MrcRange {
                idx: 0,
                start: 0x2400_0000,
                end: 0x33FF_FFFF,
                nmrgd: 4,
            }],
        )
        .grant(Domain::M7_0, Access::FullRw);
        let outcome =
            search_and_patch_mrgd(mrgds, raw, MrgdTarget::ContainsAddress(0x2400_6008), 4).unwrap();
        assert_eq!(
            outcome,
            MrgdPatchOutcome::PatchedExisting { slot: 0 },
            "ContainsAddress must match even with NSE set"
        );
    }

    /// `patch_pdac` replaces the ACP level rather than OR-ing bits.
    /// Pre-C1: SupervisorRw (0b010) | SecureRwNsRead (0b101) = FullRw (0b111).
    /// Post-fix: the result must be exactly the new level (0b101).
    #[test]
    fn patch_pdac_replaces_acp_level_not_ors() {
        let mut backing = [0u32; 2];
        let regs: &XrdcPdacRegisters =
            unsafe { &*(backing.as_mut_ptr() as *const XrdcPdacRegisters) };

        // Initialise with SupervisorRw (0b010) for M7_0 (D1).
        let initial = PdacRaw::new(1).grant(Domain::M7_0, Access::SupervisorRw);
        program_pdac(regs, initial, /* lock = */ false);

        // Patch with SecureRwNsRead (0b101) for the same domain.
        let patch = PdacRaw::new(1).grant(Domain::M7_0, Access::SecureRwNsRead);
        patch_pdac(regs, patch).unwrap();

        let expected_w0 = (Access::SecureRwNsRead as u32) << PDAC_W0::D1ACP.shift;
        assert_eq!(backing[0], expected_w0, "ACP must be replaced, not OR'd");

        // Explicitly assert the old OR result (0b111 = FullRw) did NOT happen.
        let or_result =
            (Access::SupervisorRw as u32 | Access::SecureRwNsRead as u32) << PDAC_W0::D1ACP.shift;
        assert_ne!(
            backing[0], or_result,
            "OR-ing ACP levels would produce FullRw (0b111); that is a privilege escalation"
        );
    }

    /// `patch_mrgd` replaces the ACP level on a matching descriptor.
    #[test]
    fn patch_mrgd_replaces_acp_level_not_ors() {
        const WORDS_PER_SLOT: usize = 4;
        let mut backing = [0u32; WORDS_PER_SLOT * 16];
        let mrgds: &[XrdcMemoryRegionDescriptorRegisters] =
            unsafe { core::slice::from_raw_parts(backing.as_mut_ptr() as *const _, 16) };

        // Pre-program a descriptor with SupervisorRw (0b010).
        let initial = MrgdRaw::region(
            0x3420_0000,
            0x342F_FFFF,
            &[MrcRange {
                idx: 0,
                start: 0x3000_0000,
                end: 0x3FFF_FFFF,
                nmrgd: 16,
            }],
        )
        .grant(Domain::M7_0, Access::SupervisorRw);
        program_mrgd(&mrgds[0], initial, /* lock = */ false);

        // Patch with SecureRwNsRead (0b101).
        let patch = MrgdRaw::region(
            0x3420_0000,
            0x342F_FFFF,
            &[MrcRange {
                idx: 0,
                start: 0x3000_0000,
                end: 0x3FFF_FFFF,
                nmrgd: 16,
            }],
        )
        .grant(Domain::M7_0, Access::SecureRwNsRead);
        patch_mrgd(mrgds, patch, 16).unwrap();

        let expected_w2 = (Access::SecureRwNsRead as u32) << MRGD_W2::D1ACP.shift;
        assert_eq!(
            backing[0 * WORDS_PER_SLOT + 2],
            expected_w2,
            "ACP must be replaced"
        );

        let or_result =
            (Access::SupervisorRw as u32 | Access::SecureRwNsRead as u32) << MRGD_W2::D1ACP.shift;
        assert_ne!(
            backing[0 * WORDS_PER_SLOT + 2],
            or_result,
            "OR-ing ACP levels would produce FullRw (0b111)"
        );
    }

    /// `patch_mrgd` silently skips locked descriptors.
    #[test]
    fn patch_mrgd_skips_locked_descriptor() {
        const WORDS_PER_SLOT: usize = 4;
        let mut backing = [0u32; WORDS_PER_SLOT * 16];
        let mrgds: &[XrdcMemoryRegionDescriptorRegisters] =
            unsafe { core::slice::from_raw_parts(backing.as_mut_ptr() as *const _, 16) };

        let initial = MrgdRaw::region(
            0x3420_0000,
            0x342F_FFFF,
            &[MrcRange {
                idx: 0,
                start: 0x3000_0000,
                end: 0x3FFF_FFFF,
                nmrgd: 16,
            }],
        )
        .grant(Domain::M7_0, Access::SupervisorRw);
        program_mrgd(&mrgds[0], initial, /* lock = */ true); // LK2 != 0

        let patch = MrgdRaw::region(
            0x3420_0000,
            0x342F_FFFF,
            &[MrcRange {
                idx: 0,
                start: 0x3000_0000,
                end: 0x3FFF_FFFF,
                nmrgd: 16,
            }],
        )
        .grant(Domain::M7_0, Access::FullRw);
        assert_eq!(
            patch_mrgd(mrgds, patch, 16),
            Err(XrdcPatchError::LockedDescriptor)
        );

        let expected_w2 = (Access::SupervisorRw as u32) << MRGD_W2::D1ACP.shift;
        assert_eq!(
            backing[0 * WORDS_PER_SLOT + 2],
            expected_w2,
            "locked descriptor must not be modified"
        );
    }
    #[test]
    fn search_and_patch_mrgd_reports_missing_contains_target_without_allocating() {
        const WORDS_PER_SLOT: usize = 8;
        const RANGES: &[MrcRange] = &[MrcRange {
            idx: 0,
            start: 0x3000_0000,
            end: 0x3FFF_FFFF,
            nmrgd: 4,
        }];
        let mut backing = [0u32; WORDS_PER_SLOT * 16];
        let mrgds: &[XrdcMemoryRegionDescriptorRegisters] =
            unsafe { core::slice::from_raw_parts(backing.as_mut_ptr() as *const _, 16) };
        let existing = MrgdRaw::region(0x3100_0000, 0x3100_001F, RANGES)
            .grant(Domain::M7_0, Access::SupervisorRw);
        program_mrgd(&mrgds[0], existing, false);
        let raw =
            MrgdRaw::region(0x3200_0000, 0x3200_001F, RANGES).grant(Domain::M7_0, Access::FullRw);

        assert_eq!(
            search_and_patch_mrgd(mrgds, raw, MrgdTarget::ContainsAddress(0x3200_0000), 4),
            Err(XrdcPatchError::MissingTarget)
        );
        assert_eq!(
            backing[0..4],
            [
                existing.srtaddr_field,
                existing.endaddr_field,
                existing.acp_lo,
                1 << 31
            ]
        );
        assert_eq!(
            backing[8..32],
            [0; 24],
            "ContainsAddress misses must not allocate"
        );
    }

    #[test]
    fn patch_mrgd_reports_ambiguous_target_without_writing_either_match() {
        const WORDS_PER_SLOT: usize = 8;
        const RANGES: &[MrcRange] = &[MrcRange {
            idx: 0,
            start: 0x3000_0000,
            end: 0x3FFF_FFFF,
            nmrgd: 4,
        }];
        let mut backing = [0u32; WORDS_PER_SLOT * 16];
        let mrgds: &[XrdcMemoryRegionDescriptorRegisters] =
            unsafe { core::slice::from_raw_parts(backing.as_mut_ptr() as *const _, 16) };
        let raw =
            MrgdRaw::region(0x3420_0000, 0x3420_001F, RANGES).grant(Domain::M7_0, Access::FullRw);
        program_mrgd(&mrgds[0], raw, false);
        program_mrgd(&mrgds[1], raw, false);
        let before = backing;

        assert_eq!(
            patch_mrgd(mrgds, raw, 4),
            Err(XrdcPatchError::AmbiguousTarget)
        );
        assert_eq!(
            backing, before,
            "ambiguous static targets must not be modified"
        );
    }

    #[test]
    fn patch_mrgd_reports_no_free_descriptor_within_nmrgd_budget() {
        const WORDS_PER_SLOT: usize = 8;
        const RANGES: &[MrcRange] = &[MrcRange {
            idx: 0,
            start: 0x3000_0000,
            end: 0x3FFF_FFFF,
            nmrgd: 4,
        }];
        let mut backing = [0u32; WORDS_PER_SLOT * 16];
        let mrgds: &[XrdcMemoryRegionDescriptorRegisters] =
            unsafe { core::slice::from_raw_parts(backing.as_mut_ptr() as *const _, 16) };
        for slot in 0..4 {
            let offset = slot as u32 * 0x20;
            let existing = MrgdRaw::region(0x3100_0000 + offset, 0x3100_001F + offset, RANGES);
            program_mrgd(&mrgds[slot], existing, false);
        }
        let before = backing;
        let raw = MrgdRaw::region(0x3200_0000, 0x3200_001F, RANGES);

        assert_eq!(
            patch_mrgd(mrgds, raw, 4),
            Err(XrdcPatchError::NoFreeDescriptor)
        );
        assert_eq!(
            backing, before,
            "only slots within nmrgd may be searched or changed"
        );
    }

    #[test]
    fn search_and_patch_mrgd_exact_range_allocates_first_free_slot() {
        const WORDS_PER_SLOT: usize = 8;
        const RANGES: &[MrcRange] = &[MrcRange {
            idx: 0,
            start: 0x3000_0000,
            end: 0x3FFF_FFFF,
            nmrgd: 4,
        }];
        let mut backing = [0u32; WORDS_PER_SLOT * 16];
        let mrgds: &[XrdcMemoryRegionDescriptorRegisters] =
            unsafe { core::slice::from_raw_parts(backing.as_mut_ptr() as *const _, 16) };
        program_mrgd(
            &mrgds[0],
            MrgdRaw::region(0x3100_0000, 0x3100_001F, RANGES),
            false,
        );
        let raw =
            MrgdRaw::region(0x3200_0000, 0x3200_001F, RANGES).grant(Domain::A53, Access::FullRw);

        assert_eq!(
            search_and_patch_mrgd(mrgds, raw, MrgdTarget::ExactRange, 4),
            Ok(MrgdPatchOutcome::AllocatedNew { slot: 1 })
        );
        assert_eq!(
            backing[8..12],
            [
                raw.srtaddr_field,
                raw.endaddr_field,
                raw.acp_lo,
                raw.acp_hi | (1 << 31)
            ]
        );
    }
    #[test]
    fn allocate_unmapped_exact_mrgd_allocates_empty_first_slot_with_exact_words() {
        const WORDS_PER_SLOT: usize = 8;
        const RANGES: &[MrcRange] = &[MrcRange {
            idx: 0,
            start: 0x3000_0000,
            end: 0x3FFF_FFFF,
            nmrgd: 4,
        }];
        let mut backing = [0u32; WORDS_PER_SLOT * 16];
        let mrgds: &[XrdcMemoryRegionDescriptorRegisters] =
            unsafe { core::slice::from_raw_parts(backing.as_mut_ptr() as *const _, 16) };
        let raw = MrgdRaw::region(0x3200_0000, 0x3200_001F, RANGES)
            .grant(Domain::M7_0, Access::FullRw)
            .grant(Domain::A53, Access::SecureRwNsRead);

        assert_eq!(
            allocate_unmapped_exact_mrgd(mrgds, raw, 4),
            Ok(MrgdPatchOutcome::AllocatedNew { slot: 0 })
        );
        assert_eq!(
            backing[0..4],
            [
                raw.srtaddr_field,
                raw.endaddr_field,
                raw.acp_lo,
                raw.acp_hi | (1 << 31)
            ],
            "empty allocation must write the requested MRGD words verbatim"
        );
        assert_eq!(backing[4..], [0; WORDS_PER_SLOT * 16 - 4]);
    }

    #[test]
    fn allocate_unmapped_exact_mrgd_rejects_partial_overlap_without_mutation() {
        const WORDS_PER_SLOT: usize = 8;
        const RANGES: &[MrcRange] = &[MrcRange {
            idx: 0,
            start: 0x3000_0000,
            end: 0x3FFF_FFFF,
            nmrgd: 4,
        }];
        let mut backing = [0u32; WORDS_PER_SLOT * 16];
        let mrgds: &[XrdcMemoryRegionDescriptorRegisters] =
            unsafe { core::slice::from_raw_parts(backing.as_mut_ptr() as *const _, 16) };
        let existing = MrgdRaw::region(0x3200_0000, 0x3200_003F, RANGES)
            .grant(Domain::Hse, Access::SupervisorRw);
        program_mrgd(&mrgds[0], existing, false);
        let before = backing;
        let raw =
            MrgdRaw::region(0x3200_0020, 0x3200_005F, RANGES).grant(Domain::M7_0, Access::FullRw);

        assert_eq!(
            allocate_unmapped_exact_mrgd(mrgds, raw, 4),
            Err(XrdcPatchError::AmbiguousTarget)
        );
        assert_eq!(
            backing, before,
            "an overlapping descriptor must not be changed or bypassed"
        );
    }

    #[test]
    fn allocate_unmapped_exact_mrgd_reports_no_free_descriptor_without_mutation() {
        const WORDS_PER_SLOT: usize = 8;
        const RANGES: &[MrcRange] = &[MrcRange {
            idx: 0,
            start: 0x3000_0000,
            end: 0x3FFF_FFFF,
            nmrgd: 4,
        }];
        let mut backing = [0u32; WORDS_PER_SLOT * 16];
        let mrgds: &[XrdcMemoryRegionDescriptorRegisters] =
            unsafe { core::slice::from_raw_parts(backing.as_mut_ptr() as *const _, 16) };
        for slot in 0..4 {
            let offset = slot as u32 * 0x20;
            program_mrgd(
                &mrgds[slot],
                MrgdRaw::region(0x3100_0000 + offset, 0x3100_001F + offset, RANGES),
                false,
            );
        }
        let before = backing;
        let raw = MrgdRaw::region(0x3200_0000, 0x3200_001F, RANGES);

        assert_eq!(
            allocate_unmapped_exact_mrgd(mrgds, raw, 4),
            Err(XrdcPatchError::NoFreeDescriptor)
        );
        assert_eq!(backing, before, "a full NMRGD budget must remain unchanged");
    }
}
