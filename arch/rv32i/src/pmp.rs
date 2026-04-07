// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

extern crate flux_core;

use core::cell::Cell;
use core::fmt;
use core::num::NonZeroUsize;

use crate::csr;
use flux_support::capability::MpuEnabledCapability;
use flux_support::{register_bitfields_u8, FluxPtr, FluxPtrU8, FluxRange, Pair, RArray};
use flux_support::{FieldValueU32, LocalRegisterCopyU8, RegisterLongName};
use kernel::platform::mpu::{self, RegionDescriptor};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::FieldValue;

flux_rs::defs! {

    fn valid_size(x: int) -> bool { 0 <= x && x <= u32::MAX }

    fn is_empty(r: PMPUserRegion) -> bool {
        r.start >= r.end
    }

    fn contains(r: PMPUserRegion, i: int) -> bool {
        r.start <= i && i < r.end
    }

    fn saturating_sub(a: int, b: int) -> int {
        if a > b {
            a - b
        } else {
            0
        }
    }

    fn max(x: int, y: int) -> int {
        if x > y {
            x
        } else {
            y
        }
    }

    fn min(x: int, y: int) -> int {
        if x > y {
            y
        } else {
            x
        }
    }

    fn region_overlaps(range1: PMPUserRegion, start: int, end: int) -> bool {
        if !range1.is_set || is_empty(range1) || start >= end {
            false
        } else {
            max(range1.start, start) < min(range1.end, end)
        }
    }

    // PMP specific model
    fn bit(reg: bitvec<32>, power_of_two: bitvec<32>) -> bool { reg & power_of_two != 0}
    fn extract(reg: bitvec<32>, mask:int, offset: int) -> bitvec<32> { (reg & bv_int_to_bv32(mask)) >> bv_int_to_bv32(offset) }

    // See Figure 34. PMP configuration register format. in the RISCV ISA (Section 3.7)
    // For TORUserCFG

    fn permissions_match(perms: mpu::Permissions, reg: LocalRegisterCopyU8) -> bool {
        if (perms.x && perms.w && perms.r) {
            bit(reg.val, 1) && bit(reg.val, 2) && bit(reg.val, 4)
        } else if (perms.r && perms.w) {
            bit(reg.val, 1) && bit(reg.val, 2) && !bit(reg.val, 4)
        } else if (perms.r && perms.x) {
            bit(reg.val, 1) && !bit(reg.val, 2) && bit(reg.val, 4)
        } else if (perms.r) {
            bit(reg.val, 1) && !bit(reg.val, 2) && !bit(reg.val, 4)
        } else if (perms.x) {
            !bit(reg.val, 1) && !bit(reg.val, 2) && bit(reg.val, 4)
        } else {
            // nothing else supported
            false
        }
    }

    fn active_pmp_user_cfg_correct(cfg: TORUserPMPCFG, perms: mpu::Permissions) -> bool {
        // the permissions are correct encoded in the CFG reg.
        permissions_match(perms, cfg.reg) &&
        // L bit is clear - meaning the entry can be modified later
        !bit(cfg.reg.val, 1 << 7) &&
        // Addressing mode is Top of Range (TOR)
        extract(cfg.reg.val, 0b11000, 3) == 1
    }

    fn inactive_pmp_user_cfg_correct(cfg: TORUserPMPCFG) -> bool {
        // L bit is clear - meaning the entry can be modified later
        !bit(cfg.reg.val, 1 << 7) &&
        // Addressing mode is OFF - indicating a disabled region
        extract(cfg.reg.val, 0b11000, 3) == 0
    }

    fn cfg_reg_configured_correctly(cfg_reg: bitvec<32>, region: PMPUserRegion, idx: int) -> bool {
        // 4 regions can be packed into a cfg register -
        // the code packs the odd region in the first 4 bytes
        // and the even region in the third 4 bytes
        if (idx % 2 == 0) {
            // extract the first cfg region
            let odd_region_bits = extract(cfg_reg, 0x0000FF00, 8); // extracts bits 15 - 8 of the register because it's stored as BE
            odd_region_bits == region.tor_cfg.reg.val
        } else {
            // extract the third cfg region
            let even_region_bits = extract(cfg_reg, 0xFF000000, 24); // extracts bits 31 - 24 of the register because it's stored as BE
            even_region_bits == region.tor_cfg.reg.val
        }
    }

    fn addr_reg_configured_correctly(addr_registers: Map<int, bitvec<32>>, region: PMPUserRegion, idx: int) -> bool {
        if (idx % 2 == 0) {
            let even_addr_start_idx = idx * 2;
            let even_addr_end_idx = idx * 2 + 1;
            if region.tor_cfg.reg.val != 0 {
                let even_start_reg = map_select(addr_registers, even_addr_start_idx);
                let even_end_reg = map_select(addr_registers, even_addr_end_idx);

                // top of range - sanity check
                even_addr_start_idx + 1 == even_addr_end_idx &&
                even_start_reg == bv_int_to_bv32(region.start) >> 2 &&
                even_end_reg == bv_int_to_bv32(region.end) >> 2
            } else {
                true
            }
        } else {
            let odd_addr_start_idx = (idx - 1) * 2 + 2;
            let odd_addr_end_idx = (idx - 1) * 2 + 3;
            if region.tor_cfg.reg.val != 0 {
                let odd_start_reg = map_select(addr_registers, odd_addr_start_idx);
                let odd_end_reg = map_select(addr_registers, odd_addr_end_idx);

                // top of range - sanity check
                odd_addr_start_idx + 1 == odd_addr_end_idx &&
                odd_start_reg == bv_int_to_bv32(region.start) >> 2 &&
                odd_end_reg == bv_int_to_bv32(region.end) >> 2
            } else {
                true
            }
        }
    }

    fn region_configured_correctly(region: PMPUserRegion, old: HardwareState, new: HardwareState, idx: int) -> bool {
        let cfg_reg_idx = idx / 2;
        let cfg_reg = map_select(new.pmpcfg_registers, cfg_reg_idx);
        cfg_reg_configured_correctly(cfg_reg, region, idx) && addr_reg_configured_correctly(new.pmpaddr_registers, region, idx)
    }

    // uninterpreted since we don't have forall:
    // forall i. i >= 0 && i < bound, region_configured_correctly(hardware_state, i)
    fn all_regions_configured_correctly_up_to(bound: int, hardware_state: HardwareState) -> bool;
}

// Some axioms and verification state
//
// We want to prove that all regions up to a const generic are configured
// correctly but we don't have a forall.
//
// So instead, we do the classic inductive evidence trick

#[flux_rs::opaque]
#[flux_rs::refined_by(pmpcfg_registers: Map<int, bitvec<32>>, pmpaddr_registers: Map<int, bitvec<32>>)]
pub struct HardwareState {}

#[flux_rs::trusted(reason = "Flux Wrappers")]
impl HardwareState {
    pub fn new() -> Self {
        Self {}
    }

    #[flux_rs::sig(fn (&HardwareState[@hw]) -> HardwareState[hw])]
    pub fn snapshot(&self) -> Self {
        Self {}
    }
}

#[flux_rs::trusted(reason = "Proof Code")]
#[flux_rs::sig(fn (&HardwareState[@hw]) ensures all_regions_configured_correctly_up_to(0, hw))]
fn all_regions_configured_correctly_base(hardware_state: &HardwareState) {}

#[flux_rs::trusted(reason = "Proof Code")]
#[flux_rs::sig(fn (&PMPUserRegion<_>[@region], &HardwareState[@old], &HardwareState[@new], i: usize)
    requires all_regions_configured_correctly_up_to(i, old) && region_configured_correctly(region, old, new, i)
    ensures all_regions_configured_correctly_up_to(i + 1, new)
)]
fn all_regions_configured_correctly_step<const MPU_REGIONS: usize>(
    region: &PMPUserRegion<MPU_REGIONS>,
    old_hw: &HardwareState,
    new_hw: &HardwareState,
    i: usize,
) {
}

#[flux_rs::trusted(reason = "TCB")]
#[flux_rs::sig(
    fn (idx: usize, bits: usize, hw_state: &strg HardwareState[@hw])
        ensures hw_state: HardwareState[{
            pmpaddr_registers: map_store(hw.pmpaddr_registers, idx, bv_int_to_bv32(bits)),
            ..hw
        }]
)]
fn pmpaddr_set(idx: usize, bits: usize, hardware_state: &mut HardwareState) {
    csr::CSR.pmpaddr_set(idx, bits);
}

#[flux_rs::trusted(reason = "TCB")]
#[flux_rs::sig(
    fn (idx: usize, bits: usize, hw_state: &strg HardwareState[@hw])
        ensures hw_state: HardwareState[{
            pmpcfg_registers: map_store(hw.pmpcfg_registers, idx, bv_int_to_bv32(bits)),
            ..hw
        }]
)]
fn pmpconfig_set(idx: usize, bits: usize, hardware_state: &mut HardwareState) {
    csr::CSR.pmpconfig_set(idx, bits);
}

#[flux_rs::trusted(reason = "TCB")]
#[flux_rs::sig(
    fn (idx: usize, FieldValueU32<_>[@mask, @value], hw_state: &strg HardwareState[@hw])
        ensures hw_state: HardwareState[{
            pmpcfg_registers: map_store(
                hw.pmpcfg_registers,
                idx,
                (map_select(hw.pmpcfg_registers, idx) & bv_not(mask)) | value
            ),
            ..hw
        }]
)]
fn pmpconfig_modify(
    idx: usize,
    bits: FieldValueU32<csr::pmpconfig::pmpcfg::Register>,
    hardware_state: &mut HardwareState,
) {
    // SUPER annoying :(
    let bits_inner = bits.into_inner();
    let as_usize = FieldValue::<usize, csr::pmpconfig::pmpcfg::Register>::new(
        bits_inner.mask as usize,
        0,
        bits_inner.value as usize,
    );
    csr::CSR.pmpconfig_modify(idx, as_usize);
}

#[flux_rs::trusted(reason = "TCB")]
#[flux_rs::sig(fn (byte0: u8, byte1: u8, byte2: u8, byte3: u8) -> u32{b:
    extract(bv_int_to_bv32(b), 0xFF000000, 24) == bv_int_to_bv32(byte0) &&
    extract(bv_int_to_bv32(b), 0x00FF0000, 16) == bv_int_to_bv32(byte1) &&
    extract(bv_int_to_bv32(b), 0x0000FF00, 8) == bv_int_to_bv32(byte2) &&
    extract(bv_int_to_bv32(b), 0x000000FF, 0) == bv_int_to_bv32(byte3)
})]
fn u32_from_be_bytes(byte0: u8, byte1: u8, byte2: u8, byte3: u8) -> u32 {
    u32::from_be_bytes([byte0, byte1, byte2, byte3])
}

// We can't use an extern spec here because of the tuple! :(
#[flux_rs::trusted(reason = "Extern Spec")]
#[flux_rs::sig(fn (usize[@fst], u32[@snd]) -> usize[
    if (snd >= 32) {
        bv_bv32_to_int(bv_int_to_bv32(fst) >> bv_int_to_bv32(snd) & 31)
    } else {
        bv_bv32_to_int(bv_int_to_bv32(fst) >> bv_int_to_bv32(snd))
    }
])]
fn overflowing_shr(lhs: usize, rhs: u32) -> usize {
    return lhs.overflowing_shr(rhs).0 as usize;
}

register_bitfields_u8![u8,
    /// Generic `pmpcfg` octet.
    ///
    /// A PMP entry is configured through `pmpaddrX` and `pmpcfgX` CSRs, where a
    /// single `pmpcfgX` CSRs holds multiple octets, each affecting the access
    /// permission, addressing mode and "lock" attributes of a single `pmpaddrX`
    /// CSR. This bitfield definition represents a single, `u8`-backed `pmpcfg`
    /// octet affecting a single `pmpaddr` entry.
    pub pmpcfg_octet [
        r OFFSET(0) NUMBITS(1) [],
        w OFFSET(1) NUMBITS(1) [],
        x OFFSET(2) NUMBITS(1) [],
        a OFFSET(3) NUMBITS(2) [
            OFF = 0,
            TOR = 1,
            NA4 = 2,
            NAPOT = 3
        ],
        l OFFSET(7) NUMBITS(1) []
    ]
];

/// A `pmpcfg` octet for a user-mode (non-locked) TOR-addressed PMP region.
///
/// This is a wrapper around a [`pmpcfg_octet`] (`u8`) register type, which
/// guarantees that the wrapped `pmpcfg` octet is always set to be either
/// [`TORUserPMPCFG::OFF`] (set to `0x00`), or in a non-locked, TOR-addressed
/// configuration.
///
/// By accepting this type, PMP implements can rely on the above properties to
/// hold by construction and avoid runtime checks. For example, this type is
/// used in the [`TORUserPMP::configure_pmp`] method.
#[derive(Copy, Clone)]
#[flux_rs::refined_by(reg: LocalRegisterCopyU8)]
pub struct TORUserPMPCFG(
    #[field(LocalRegisterCopyU8<pmpcfg_octet::Register>[reg])]
    LocalRegisterCopyU8<pmpcfg_octet::Register>,
);

impl TORUserPMPCFG {
    #[flux_rs::sig(fn () -> TORUserPMPCFG{ cfg: inactive_pmp_user_cfg_correct(cfg) && cfg.reg.val == 0 })]
    pub const fn OFF() -> TORUserPMPCFG {
        TORUserPMPCFG(LocalRegisterCopyU8::new(0))
    }

    /// Extract the `u8` representation of the [`pmpcfg_octet`] register.
    #[flux_rs::sig(fn (&Self[@cfg]) -> u8[bv_bv32_to_int(cfg.reg.val)])]
    pub fn get(&self) -> u8 {
        self.0.get()
    }

    /// Extract a copy of the contained [`pmpcfg_octet`] register.
    pub fn get_reg(&self) -> LocalRegisterCopyU8<pmpcfg_octet::Register> {
        self.0
    }
}

impl PartialEq<TORUserPMPCFG> for TORUserPMPCFG {
    #[flux_rs::sig(fn (&Self[@this], &Self[@other]) -> bool[this.reg.val == other.reg.val])]
    fn eq(&self, other: &Self) -> bool {
        self.0.get() == other.0.get()
    }

    #[flux_rs::sig(fn (&Self[@this], &Self[@other]) -> bool[this.reg.val != other.reg.val])]
    fn ne(&self, other: &Self) -> bool {
        self.0.get() != other.0.get()
    }
}

impl Eq for TORUserPMPCFG {}

#[flux_rs::sig(fn (p: mpu::Permissions) -> TORUserPMPCFG{cfg: active_pmp_user_cfg_correct(cfg, p)})]
fn permissions_to_pmpcfg(p: mpu::Permissions) -> TORUserPMPCFG {
    let fv = match p {
        mpu::Permissions::ReadWriteExecute => {
            pmpcfg_octet::r::SET() + pmpcfg_octet::w::SET() + pmpcfg_octet::x::SET()
        }
        mpu::Permissions::ReadWriteOnly => {
            pmpcfg_octet::r::SET() + pmpcfg_octet::w::SET() + pmpcfg_octet::x::CLEAR()
        }
        mpu::Permissions::ReadExecuteOnly => {
            pmpcfg_octet::r::SET() + pmpcfg_octet::w::CLEAR() + pmpcfg_octet::x::SET()
        }
        mpu::Permissions::ReadOnly => {
            pmpcfg_octet::r::SET() + pmpcfg_octet::w::CLEAR() + pmpcfg_octet::x::CLEAR()
        }
        mpu::Permissions::ExecuteOnly => {
            pmpcfg_octet::r::CLEAR() + pmpcfg_octet::w::CLEAR() + pmpcfg_octet::x::SET()
        }
    };

    TORUserPMPCFG(LocalRegisterCopyU8::new(
        (fv + pmpcfg_octet::l::CLEAR() + pmpcfg_octet::a::TOR()).value(),
    ))
}

// impl From<mpu::Permissions> for TORUserPMPCFG {
//     fn from(p: mpu::Permissions) -> Self {
//         let fv = match p {
//             mpu::Permissions::ReadWriteExecute => {
//                 pmpcfg_octet::r::SET + pmpcfg_octet::w::SET + pmpcfg_octet::x::SET
//             }
//             mpu::Permissions::ReadWriteOnly => {
//                 pmpcfg_octet::r::SET + pmpcfg_octet::w::SET + pmpcfg_octet::x::CLEAR
//             }
//             mpu::Permissions::ReadExecuteOnly => {
//                 pmpcfg_octet::r::SET + pmpcfg_octet::w::CLEAR + pmpcfg_octet::x::SET
//             }
//             mpu::Permissions::ReadOnly => {
//                 pmpcfg_octet::r::SET + pmpcfg_octet::w::CLEAR + pmpcfg_octet::x::CLEAR
//             }
//             mpu::Permissions::ExecuteOnly => {
//                 pmpcfg_octet::r::CLEAR + pmpcfg_octet::w::CLEAR + pmpcfg_octet::x::SET
//             }
//         };

//         TORUserPMPCFG(LocalRegisterCopy::new(
//             (fv + pmpcfg_octet::l::CLEAR + pmpcfg_octet::a::TOR).value,
//         ))
//     }
// }

/// A RISC-V PMP memory region specification, configured in NAPOT mode.
///
/// This type checks that the supplied `start` and `size` values meet the RISC-V
/// NAPOT requirements, namely that
///
/// - the region is a power of two bytes in size
/// - the region's start address is aligned to the region size
/// - the region is at least 8 bytes long
///
/// By accepting this type, PMP implementations can rely on these requirements
/// to be verified. Furthermore, they can use the
/// [`NAPOTRegionSpec::napot_addr`] convenience method to retrieve an `pmpaddrX`
/// CSR value encoding this region's address and length.
#[derive(Copy, Clone, Debug)]
#[flux_rs::refined_by(start: int, size: int)]
#[flux_rs::invariant(size > 0)]
#[flux_rs::invariant(start + size <= usize::MAX)]
pub struct NAPOTRegionSpec {
    #[field(FluxPtrU8[start])]
    start: FluxPtrU8,
    #[field(usize[size])]
    size: usize,
}

impl NAPOTRegionSpec {
    /// Construct a new [`NAPOTRegionSpec`]
    ///
    /// This method accepts a `start` address and a region length. It returns
    /// `Some(region)` when all constraints specified in the
    /// [`NAPOTRegionSpec`]'s documentation are satisfied, otherwise `None`.
    #[flux_rs::sig(fn (start: FluxPtrU8, {usize[@size] | size > 0 && start + size <= usize::MAX}) -> Option<Self>)]
    pub fn new(start: FluxPtrU8, size: usize) -> Option<Self> {
        if !size.is_power_of_two() || start.as_usize() % size != 0 || size < 8 {
            None
        } else {
            Some(NAPOTRegionSpec { start, size })
        }
    }

    /// Retrieve the start address of this [`NAPOTRegionSpec`].
    pub fn start(&self) -> FluxPtrU8 {
        self.start
    }

    /// Retrieve the size of this [`NAPOTRegionSpec`].
    pub fn size(&self) -> usize {
        self.size
    }

    /// Retrieve the end address of this [`NAPOTRegionSpec`].
    pub fn end(&self) -> FluxPtrU8 {
        unsafe { self.start.add(self.size) }
    }

    /// Retrieve a `pmpaddrX`-CSR compatible representation of this
    /// [`NAPOTRegionSpec`]'s address and length. For this value to be valid in
    /// a `CSR` register, the `pmpcfgX` octet's `A` (address mode) value
    /// belonging to this `pmpaddrX`-CSR must be set to `NAPOT` (0b11).
    pub fn napot_addr(&self) -> usize {
        (self.start.as_usize() + (self.size - 1).overflowing_shr(1).0)
            .overflowing_shr(2)
            .0
    }
}

/// A RISC-V PMP memory region specification, configured in TOR mode.
///
/// This type checks that the supplied `start` and `end` addresses meet the
/// RISC-V TOR requirements, namely that
///
/// - the region's start address is aligned to a 4-byte boundary
/// - the region's end address is aligned to a 4-byte boundary
/// - the region is at least 4 bytes long
///
/// By accepting this type, PMP implementations can rely on these requirements
/// to be verified.
#[derive(Copy, Clone, Debug)]
pub struct TORRegionSpec {
    start: *const u8,
    end: *const u8,
}

impl TORRegionSpec {
    /// Construct a new [`TORRegionSpec`]
    ///
    /// This method accepts a `start` and `end` address. It returns
    /// `Some(region)` when all constraints specified in the [`TORRegionSpec`]'s
    /// documentation are satisfied, otherwise `None`.
    pub fn new(start: *const u8, end: *const u8) -> Option<Self> {
        if (start as usize) % 4 != 0
            || (end as usize) % 4 != 0
            || (end as usize)
                .checked_sub(start as usize)
                .map_or(true, |size| size < 4)
        {
            None
        } else {
            Some(TORRegionSpec { start, end })
        }
    }

    /// Retrieve the start address of this [`TORRegionSpec`].
    pub fn start(&self) -> *const u8 {
        self.start
    }

    /// Retrieve the end address of this [`TORRegionSpec`].
    pub fn end(&self) -> *const u8 {
        self.end
    }
}

/// Helper method to check if a [`PMPUserMPUConfig`] region overlaps with a
/// region specified by `other_start` and `other_size`.
///
/// Matching the RISC-V spec this checks `pmpaddr[i-i] <= y < pmpaddr[i]` for TOR
/// ranges.
#[flux_rs::sig(fn (&PMPUserRegion<_>[@r], start: usize, end: usize) -> bool[region_overlaps(r, start, end)])]
fn region_overlaps<const MPU_REGIONS: usize>(
    region: &PMPUserRegion<MPU_REGIONS>,
    start: usize,
    end: usize,
) -> bool {
    // PMP TOR regions are not inclusive on the high end, that is
    //     pmpaddr[i-i] <= y < pmpaddr[i].
    //
    // This happens to coincide with the definition of the Rust half-open Range
    // type, which provides a convenient `.contains()` method:

    // TODO: Use Range for real? Problem is the implementation is crazy
    let region_range = match (region.start, region.end) {
        (Some(start), Some(end)) => FluxRange {
            start: start.as_usize(),
            end: end.as_usize(),
        },
        _ => return false,
    };

    let other_range = FluxRange { start, end };

    // For a range A to overlap with a range B, either B's first or B's last
    // element must be contained in A, or A's first or A's last element must be
    // contained in B. As we deal with half-open ranges, ensure that neither
    // range is empty.
    //
    // This implementation is simple and stupid, and can be optimized. We leave
    // that as an exercise to the compiler.
    !region_range.is_empty()
        && !other_range.is_empty()
        && (region_range.contains(&other_range.start)
            || region_range.contains(&other_range.end.saturating_sub(1))
            || other_range.contains(&region_range.start)
            || other_range.contains(&region_range.end.saturating_sub(1)))
}

/// Print a table of the configured PMP regions, read from  the HW CSRs.
///
/// # Safety
///
/// This function is unsafe, as it relies on the PMP CSRs to be accessible, and
/// the hardware to feature `PHYSICAL_ENTRIES` PMP CSR entries. If these
/// conditions are not met, calling this function can result in undefinied
/// behavior (e.g., cause a system trap).
#[flux_rs::trusted(reason = "just used for debugging so who cares")]
pub unsafe fn format_pmp_entries<const PHYSICAL_ENTRIES: usize>(
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    for i in 0..PHYSICAL_ENTRIES {
        // Extract the entry's pmpcfgX register value. The pmpcfgX CSRs are
        // tightly packed and contain 4 octets beloging to individual
        // entries. Convert this into a u8-wide LocalRegisterCopy<u8,
        // pmpcfg_octet> as a generic register type, independent of the entry's
        // offset.
        let pmpcfg: LocalRegisterCopyU8<pmpcfg_octet::Register> = LocalRegisterCopyU8::new(
            csr::CSR
                .pmpconfig_get(i / 4)
                .overflowing_shr(((i % 4) * 8) as u32)
                .0 as u8,
        );

        // The address interpretation is different for every mode. Return both a
        // string indicating the PMP entry's mode, as well as the effective
        // start and end address (inclusive) affected by the region. For regions
        // that are OFF, we still want to expose the pmpaddrX register value --
        // thus return the raw unshifted value as the addr, and 0 as the
        // region's end.
        let (start_label, start, end, mode) = match pmpcfg.read_as_enum(pmpcfg_octet::a()) {
            Some(pmpcfg_octet::a::Value::OFF) => {
                let addr = csr::CSR.pmpaddr_get(i);
                ("pmpaddr", addr, 0, "OFF  ")
            }

            Some(pmpcfg_octet::a::Value::TOR) => {
                let start = if i > 0 {
                    csr::CSR.pmpaddr_get(i - 1)
                } else {
                    0
                };

                (
                    "  start",
                    start.overflowing_shl(2).0,
                    csr::CSR.pmpaddr_get(i).overflowing_shl(2).0.wrapping_sub(1),
                    "TOR  ",
                )
            }

            Some(pmpcfg_octet::a::Value::NA4) => {
                let addr = csr::CSR.pmpaddr_get(i).overflowing_shl(2).0;
                ("  start", addr, addr | 0b11, "NA4  ")
            }

            Some(pmpcfg_octet::a::Value::NAPOT) => {
                let pmpaddr = csr::CSR.pmpaddr_get(i);
                let encoded_size = pmpaddr.trailing_ones();
                let size_of_pmp_addr = core::mem::size_of_val(&pmpaddr);
                flux_support::assume(size_of_pmp_addr > 0); // TODO: sizeof
                if (encoded_size as usize) < (size_of_pmp_addr * 8 - 1) {
                    let start = pmpaddr - ((1 << encoded_size) - 1);
                    let end = start + (1 << (encoded_size + 1)) - 1;
                    (
                        "  start",
                        start.overflowing_shl(2).0,
                        end.overflowing_shl(2).0 | 0b11,
                        "NAPOT",
                    )
                } else {
                    ("  start", usize::MIN, usize::MAX, "NAPOT")
                }
            }

            None => {
                // We match on a 2-bit value with 4 variants, so this is
                // unreachable. However, don't insert a panic in case this
                // doesn't get optimized away:
                ("", 0, 0, "")
            }
        };

        // Ternary operator shortcut function, to avoid bulky formatting...
        fn t<T>(cond: bool, a: T, b: T) -> T {
            if cond {
                a
            } else {
                b
            }
        }

        write!(
            f,
            "  [{:02}]: {}={:#010X}, end={:#010X}, cfg={:#04X} ({}) ({}{}{}{})\r\n",
            i,
            start_label,
            start,
            end,
            pmpcfg.get(),
            mode,
            t(pmpcfg.is_set(pmpcfg_octet::l()), "l", "-"),
            t(pmpcfg.is_set(pmpcfg_octet::r()), "r", "-"),
            t(pmpcfg.is_set(pmpcfg_octet::w()), "w", "-"),
            t(pmpcfg.is_set(pmpcfg_octet::x()), "x", "-"),
        )?;
    }

    Ok(())
}

/// A RISC-V PMP implementation exposing a number of TOR memory protection
/// regions to the [`PMPUserMPU`].
///
/// The RISC-V PMP is complex and can be used to enforce memory protection in
/// various modes (Machine, Supervisor and User mode). Depending on the exact
/// extension set present (e.g., ePMP) and the machine's security configuration
/// bits, it may expose a vastly different set of constraints and application
/// semantics.
///
/// Because we can't possibly capture all of this in a single readable,
/// maintainable and efficient implementation, we implement a two-layer system:
///
/// - a [`TORUserPMP`] is a simple abstraction over some underlying PMP hardware
///   implementation, which exposes an interface to configure regions that are
///   active (enforced) in user-mode and can be configured for arbitrary
///   addresses on a 4-byte granularity.
///
/// - the [`PMPUserMPU`] takes this abstraction and implements the Tock kernel's
///   [`mpu::MPU`] trait. It worries about re-configuring memory protection when
///   switching processes, allocating memory regions of an appropriate size,
///   etc.
///
/// Implementors of a chip are free to define their own [`TORUserPMP`]
/// implementations, adhering to their specific PMP layout & constraints,
/// provided they implement this trait.
///
/// The `MAX_REGIONS` const generic is used to indicate the maximum number of
/// TOR PMP regions available to the [`PMPUserMPU`]. The PMP implementation may
/// provide less regions than indicated through `MAX_REGIONS`, for instance when
/// entries are enforced (locked) in machine mode. The number of available
/// regions may change at runtime. The current number of regions available to
/// the [`PMPUserMPU`] is indicated by the [`TORUserPMP::available_regions`]
/// method. However, when it is known that a number of regions are not available
/// for userspace protection, `MAX_REGIONS` can be used to reduce the memory
/// footprint allocated by stored PMP configurations, as well as the
/// re-configuration overhead.
pub trait TORUserPMP<const MAX_REGIONS: usize> {
    /// A placeholder to define const-assertions which are evaluated in
    /// [`PMPUserMPU::new`]. This can be used to, for instance, assert that the
    /// number of userspace regions does not exceed the number of hardware
    /// regions.
    const CONST_ASSERT_CHECK: ();

    /// The number of TOR regions currently available for userspace memory
    /// protection. Within `[0; MAX_REGIONS]`.
    ///
    /// The PMP implementation may provide less regions than indicated through
    /// `MAX_REGIONS`, for instance when entries are enforced (locked) in
    /// machine mode. The number of available regions may change at runtime. The
    /// implementation is free to map these regions to arbitrary PMP entries
    /// (and change this mapping at runtime), provided that they are enforced
    /// when the hart is in user-mode, and other memory regions are generally
    /// inaccessible when in user-mode.
    ///
    /// When allocating regions for kernel-mode protection, and thus reducing
    /// the number of regions available to userspace, re-configuring the PMP may
    /// fail. This is allowed behavior. However, the PMP must not remove any
    /// regions from the user-mode current configuration while it is active
    /// ([`TORUserPMP::enable_user_pmp`] has been called, and it has not been
    /// disabled through [`TORUserPMP::disable_user_pmp`]).
    fn available_regions(&self) -> usize;

    /// Configure the user-mode memory protection.
    ///
    /// This method configures the user-mode memory protection, to be enforced
    /// on a call to [`TORUserPMP::enable_user_pmp`].
    ///
    /// PMP implementations where configured regions are only enforced in
    /// user-mode may re-configure the PMP on this function invocation and
    /// implement [`TORUserPMP::enable_user_pmp`] as a no-op. If configured
    /// regions are enforced in machine-mode (for instance when using an ePMP
    /// with the machine-mode whitelist policy), the new configuration rules
    /// must not apply until [`TORUserPMP::enable_user_pmp`].
    ///
    /// The tuples as passed in the `regions` parameter are defined as follows:
    ///
    /// - first value ([`TORUserPMPCFG`]): the memory protection mode as
    ///   enforced on the region. A `TORUserPMPCFG` can be created from the
    ///   [`mpu::Permissions`] type. It is in a format compatible to the pmpcfgX
    ///   register, guaranteed to not have the lock (`L`) bit set, and
    ///   configured either as a TOR region (`A = 0b01`), or disabled (all bits
    ///   set to `0`).
    ///
    /// - second value (`*const u8`): the region's start addres. As a PMP TOR
    ///   region has a 4-byte address granularity, this address is rounded down
    ///   to the next 4-byte boundary.
    ///
    /// - third value (`*const u8`): the region's end addres. As a PMP TOR
    ///   region has a 4-byte address granularity, this address is rounded down
    ///   to the next 4-byte boundary.
    ///
    /// To disable a region, set its configuration to [`TORUserPMPCFG::OFF`]. In
    /// this case, the start and end addresses are ignored and can be set to
    /// arbitrary values.
    #[flux_rs::sig(fn (&Self, &[PMPUserRegion<_>; _], hw_state: &strg HardwareState) -> Result<(), ()>[#ok] ensures hw_state: HardwareState {hw:
        ok => all_regions_configured_correctly_up_to(MAX_REGIONS, hw)
    })]
    fn configure_pmp(
        &self,
        regions: &[PMPUserRegion<MAX_REGIONS>; MAX_REGIONS],
        hardware_state: &mut HardwareState,
    ) -> Result<(), ()>;

    /// Enable the user-mode memory protection.
    ///
    /// Enables the memory protection for user-mode, as configured through
    /// [`TORUserPMP::configure_pmp`]. Enabling the PMP for user-mode may make
    /// the user-mode accessible regions inaccessible to the kernel. For PMP
    /// implementations where configured regions are only enforced in user-mode,
    /// this method may be implemented as a no-op.
    ///
    /// If enabling the current configuration is not possible (e.g., because
    /// regions have been allocated to the kernel), this function must return
    /// `Err(())`. Otherwise, this function returns `Ok(())`.
    fn enable_user_pmp(&self) -> Result<(), ()>;

    /// Disable the user-mode memory protection.
    ///
    /// Disables the memory protection for user-mode. If enabling the user-mode
    /// memory protetion made user-mode accessible regions inaccessible to
    /// machine-mode, this method should make these regions accessible again.
    ///
    /// For PMP implementations where configured regions are only enforced in
    /// user-mode, this method may be implemented as a no-op. This method is not
    /// responsible for making regions inaccessible to user-mode. If previously
    /// configured regions must be made inaccessible,
    /// [`TORUserPMP::configure_pmp`] must be used to re-configure the PMP
    /// accordingly.
    fn disable_user_pmp(&self);
}

/// Struct storing userspace memory protection regions for the [`PMPUserMPU`].
pub struct PMPUserMPUConfig<const MAX_REGIONS: usize> {
    /// PMP config identifier, as generated by the issuing PMP implementation.
    id: NonZeroUsize,
    /// Indicates if the configuration has changed since the last time it was
    /// written to hardware.
    is_dirty: Cell<bool>,
    /// Array of MPU regions. Each region requires two physical PMP entries.
    regions: [(TORUserPMPCFG, *const u8, *const u8); MAX_REGIONS],
    /// Which region index (into the `regions` array above) is used
    /// for app memory (if it has been configured).
    app_memory_region: OptionalCell<usize>,
}

#[derive(Clone, Copy)]
#[flux_rs::opaque]
#[flux_rs::refined_by(start: int, end: int, perms: mpu::Permissions)]
pub struct RegionGhost {}

#[flux_rs::trusted]
impl RegionGhost {
    #[flux_rs::sig(fn (start: FluxPtr, end: FluxPtr, perms: mpu::Permissions) -> Self[start, end, perms])]
    pub fn new(_start: FluxPtr, _end: FluxPtr, _perms: mpu::Permissions) -> Self {
        Self {}
    }

    #[flux_rs::sig(fn () -> Self)]
    pub fn empty() -> Self {
        Self {}
    }
}

#[derive(Clone, Copy)]
#[flux_rs::refined_by(
    region_number: int,
    tor_cfg: TORUserPMPCFG,
    start: int,
    end: int,
    perms: mpu::Permissions,
    is_set: bool
)]
#[flux_rs::invariant(is_set => valid_size(end))]
#[flux_rs::invariant(is_set => end >= start)]
// NOTE: this max regions is really annoying and
// only here because Flux cannot normalize types
// from the MPU trait which does have this max regions
// that we want.
pub struct PMPUserRegion<const MAX_REGIONS: usize> {
    #[field(usize[region_number])]
    pub region_number: usize,
    #[field({TORUserPMPCFG[tor_cfg] |
        (is_set => active_pmp_user_cfg_correct(tor_cfg, perms)) &&
        (!is_set => inactive_pmp_user_cfg_correct(tor_cfg) && tor_cfg.reg.val == 0)
    })]
    pub tor: TORUserPMPCFG,
    #[field(Option<FluxPtrU8[start]>[is_set])]
    pub start: Option<FluxPtrU8>,
    #[field(Option<FluxPtrU8[end]>[is_set])]
    pub end: Option<FluxPtrU8>,
    #[field({ RegionGhost[start, end, perms] | is_set => start % 4 == 0 && end % 4 == 0 })]
    pub ghost: RegionGhost,
}

impl<const MPU_REGIONS: usize> PMPUserRegion<MPU_REGIONS> {
    #[flux_rs::sig(
        fn (region_number: usize, tor: TORUserPMPCFG, start: FluxPtrU8, end: FluxPtrU8, perms: mpu::Permissions) -> Self[region_number, tor, start, end, perms, true]
            requires end >= start && active_pmp_user_cfg_correct(tor, perms) && start % 4 == 0 && end % 4 == 0
    )]
    pub fn new(
        region_number: usize,
        tor: TORUserPMPCFG,
        start: FluxPtrU8,
        end: FluxPtrU8,
        perms: mpu::Permissions,
    ) -> Self {
        Self {
            region_number,
            tor,
            start: Some(start),
            end: Some(end),
            ghost: RegionGhost::new(start, end, perms),
        }
    }
}

#[flux_rs::assoc(fn start(r: Self) -> int { r.start })]
#[flux_rs::assoc(fn size(r: Self) -> int { r.end - r.start })]
#[flux_rs::assoc(fn is_set(r: Self) -> bool { r.is_set })]
#[flux_rs::assoc(fn rnum(r: Self) -> int { r.region_number })]
#[flux_rs::assoc(fn perms(r: Self) -> mpu::Permissions { r.perms })]
#[flux_rs::assoc(fn overlaps(r1: Self, start: int, end: int) -> bool { region_overlaps(r1, start, end) })]
impl<const MPU_REGIONS: usize> RegionDescriptor for PMPUserRegion<MPU_REGIONS> {
    #[flux_rs::sig(fn (&Self[@r]) -> Option<FluxPtrU8{ptr: Self::start(r) == ptr}>[Self::is_set(r)])]
    fn start(&self) -> Option<FluxPtrU8> {
        self.start
    }

    #[flux_rs::sig(fn (&Self[@r]) -> Option<usize{sz: Self::size(r) == sz && valid_size(sz) && valid_size(Self::start(r) + sz)}>[Self::is_set(r)])]
    fn size(&self) -> Option<usize> {
        match (self.start, self.end) {
            (Some(start), Some(end)) => Some(end.as_usize() - start.as_usize()),
            _ => None,
        }
    }

    #[flux_rs::sig(fn (&Self[@r]) -> bool[Self::is_set(r)])]
    fn is_set(&self) -> bool {
        self.start.is_some() && self.end.is_some()
    }

    #[flux_rs::sig(fn (rnum: usize) -> Self {r: !Self::is_set(r) && Self::rnum(r) == rnum})]
    fn default(region_number: usize) -> Self {
        Self {
            region_number,
            tor: TORUserPMPCFG::OFF(),
            start: None,
            end: None,
            ghost: RegionGhost::empty(),
        }
    }

    #[flux_rs::sig(fn (&Self[@r], start: usize, end: usize) -> bool[Self::overlaps(r, start, end)])]
    fn overlaps(&self, start: usize, end: usize) -> bool {
        region_overlaps(self, start, end)
    }

    #[flux_rs::sig(
        fn (
            region_number: usize,
            start: FluxPtrU8,
            size: usize,
            permissions: mpu::Permissions,
        ) -> Option<Self{r:
            Self::region_can_access_exactly(r, start, start + size, permissions)
        }>
        requires valid_size(start + size) && region_number < 8
    )]
    fn create_exact_region(
        region_num: usize,
        start: FluxPtrU8,
        size: usize,
        permissions: mpu::Permissions,
    ) -> Option<Self> {
        if (region_num >= MPU_REGIONS) {
            return None;
        }

        let start = start.as_usize();
        let size = size;

        // Region start always has to align to 4 bytes. Round up to a 4 byte
        // boundary if required:
        if start % 4 != 0 {
            return None;
        }

        // Region size always has to align to 4 bytes. Round up to a 4 byte
        // boundary if required:
        if size % 4 != 0 {
            return None;
        }

        // Regions must be at least 4 bytes in size.
        if size < 4 {
            return None;
        }

        let region = PMPUserRegion::new(
            region_num,
            permissions_to_pmpcfg(permissions),
            FluxPtrU8::from(start),
            FluxPtrU8::from(start + size),
            permissions,
        );

        Some(region)
    }

    #[flux_rs::sig(fn (
        max_region_number: usize,
        available_start: FluxPtrU8,
        available_size: usize,
        region_size: usize,
        permissions: mpu::Permissions,
    ) -> Option<Pair<Self, Self>{p:
            Self::start(p.fst) >= available_start &&
            ((!Self::is_set(p.snd)) =>
                Self::regions_can_access_exactly(
                    p.fst,
                    p.snd,
                    Self::start(p.fst),
                    Self::start(p.fst) + Self::size(p.fst),
                    permissions
                )
            ) &&
            (Self::is_set(p.snd) =>
                Self::regions_can_access_exactly(
                    p.fst,
                    p.snd,
                    Self::start(p.fst),
                    Self::start(p.fst) + Self::size(p.fst) + Self::size(p.snd),
                    permissions
                )
            ) &&
            !Self::is_set(p.snd)
        }> requires valid_size(available_start + available_size) && max_region_number > 0 && max_region_number < 8
    )]
    fn allocate_regions(
        region_number: usize,
        available_start: FluxPtrU8,
        available_size: usize,
        region_size: usize,
        permissions: mpu::Permissions,
    ) -> Option<Pair<Self, Self>> {
        // Meet the PMP TOR region constraints. For this, start with the
        // provided start address and size, transform them to meet the
        // constraints, and then check that we're still within the bounds of the
        // provided values:
        let mut start = available_start.as_usize();
        let mut size = region_size;

        // Region start always has to align to 4 bytes. Round up to a 4 byte
        // boundary if required:
        if start % 4 != 0 {
            start += 4 - (start % 4);
        }

        if region_size > u32::MAX as usize { return None;} // FLUX: else, overflows!

        // Region size always has to align to 4 bytes. Round up to a 4 byte
        // boundary if required:
        if size % 4 != 0 {
            size += 4 - (size % 4);
        }

        // Regions must be at least 4 bytes in size.
        if size < 4 {
            size = 4;
        }

        // Now, check to see whether the adjusted start and size still meet the
        // allocation constraints, namely ensure that
        //
        //     start + size <= unallocated_memory_start + unallocated_memory_size
        if start + size > available_start.as_usize() + available_size {
            // // We're overflowing the provided memory region, can't make
            // // allocation. Normally, we'd abort here.
            // //
            // // However, a previous implementation of this code was incorrect in
            // // that performed this check before adjusting the requested region
            // // size to meet PMP region layout constraints (4 byte alignment for
            // // start and end address). Existing applications whose end-address
            // // is aligned on a less than 4-byte bondary would thus be given
            // // access to additional memory which should be inaccessible.
            // // Unfortunately, we can't fix this without breaking existing
            // // applications. Thus, we perform the same insecure hack here, and
            // // give the apps at most an extra 3 bytes of memory, as long as the
            // // requested region as no write privileges.
            // //
            // // TODO: Remove this logic with as part of
            // // https://github.com/tock/tock/issues/3544
            // let writeable = match permissions {
            //     mpu::Permissions::ReadWriteExecute => true,
            //     mpu::Permissions::ReadWriteOnly => true,
            //     mpu::Permissions::ReadExecuteOnly => false,
            //     mpu::Permissions::ReadOnly => false,
            //     mpu::Permissions::ExecuteOnly => false,
            // };

            // if writeable || (start + size > available_start.as_usize() + available_size + 3) {
            //     return None;
            // }
            return None;
        }
        let region = PMPUserRegion::new(
            region_number,
            permissions_to_pmpcfg(permissions),
            FluxPtrU8::from(start),
            FluxPtrU8::from(start + size),
            permissions,
        );
        Some(Pair {
            fst: region,
            snd: RegionDescriptor::default(region_number + 1),
        })
    }

    #[flux_rs::sig(fn (
        region_start: FluxPtrU8,
        available_size: usize,
        region_size: usize,
        max_region_number: usize,
        permissions: mpu::Permissions,
    ) -> Option<Pair<Self, Self>{p:
        ((!Self::is_set(p.snd)) =>
            Self::regions_can_access_exactly(
                p.fst,
                p.snd,
                region_start,
                region_start + Self::size(p.fst),
                permissions
            ) &&
            Self::size(p.fst) >= region_size
        ) &&
        (Self::is_set(p.snd) =>
            Self::regions_can_access_exactly(
                p.fst,
                p.snd,
                region_start,
                region_start + Self::size(p.fst) + Self::size(p.snd),
                permissions
            ) &&
            Self::size(p.fst) + Self::size(p.snd) >= region_size
        )
    }> requires valid_size(region_start + available_size) && max_region_number > 0 && max_region_number < 8)]
    fn update_regions(
        region_start: FluxPtrU8,
        available_size: usize,
        region_size: usize,
        max_region_number: usize,
        permissions: mpu::Permissions,
    ) -> Option<Pair<Self, Self>> {
        // For this: We should get this from region_start's invariant.
        // It may be worth updating this function to take in the region
        // as a whole
        if region_start.as_usize() % 4 != 0 {
            return None;
        }

        if region_size == 0 {
            return None;
        }

        if region_size > available_size { // FLUX: should this be a precondition?
            return None;
        }
        let mut end = region_start.as_usize() + region_size;
        // Ensure that the requested app_memory_break complies with PMP
        // alignment constraints, namely that the region's end address is 4 byte
        // aligned:
        if end % 4 != 0 {
            end += 4 - (end % 4);
        }

        // Check if there is space for this region
        if end > region_start.as_usize() + available_size {
            return None;
        }

        // If we're not out of memory, return the region
        Some(Pair {
            fst: PMPUserRegion::new(
                max_region_number - 1,
                permissions_to_pmpcfg(permissions),
                region_start,
                FluxPtrU8::from(end),
                permissions,
            ),
            snd: RegionDescriptor::default(max_region_number),
        })
    }

    #[flux_rs::sig(fn (&Self[@r], start: FluxPtrU8, end: FluxPtrU8, perms: mpu::Permissions)
        requires Self::region_can_access_exactly(r, start, end, perms)
        ensures
            !Self::overlaps(r, 0, start) &&
            !Self::overlaps(r, end, u32::MAX)
    )]
    fn lemma_region_can_access_exactly_implies_no_overlap(
        &self,
        _start: FluxPtrU8,
        _end: FluxPtrU8,
        _perms: mpu::Permissions,
    ) {
    }

    #[flux_rs::sig(fn (&Self[@r1], &Self[@r2], start: FluxPtrU8, end: FluxPtrU8, perms: mpu::Permissions)
        requires Self::regions_can_access_exactly(r1, r2, start, end, perms)
        ensures
            !Self::overlaps(r1, 0, start) &&
            !Self::overlaps(r1, end, u32::MAX) &&
            !Self::overlaps(r2, 0, start) &&
            !Self::overlaps(r2, end, u32::MAX)
    )]
    fn lemma_regions_can_access_exactly_implies_no_overlap(
        _r1: &Self,
        r2: &Self,
        start: FluxPtrU8,
        end: FluxPtrU8,
        _perms: mpu::Permissions,
    ) {
        if !r2.is_set() {
            r2.lemma_region_not_set_implies_no_overlap(start, end);
        }
    }

    #[flux_rs::sig(fn (&Self[@r], access_end: FluxPtrU8, desired_end: FluxPtrU8)
        requires
            !Self::overlaps(r, access_end, u32::MAX) &&
            access_end <= desired_end
        ensures !Self::overlaps(r, desired_end, u32::MAX)
    )]
    fn lemma_no_overlap_le_addr_implies_no_overlap_addr(
        &self,
        _access_end: FluxPtrU8,
        _desired_end: FluxPtrU8,
    ) {
    }

    #[flux_rs::sig(fn (&Self[@r], start: FluxPtrU8, end: FluxPtrU8)
        requires !Self::is_set(r)
        ensures !Self::overlaps(r, start, end)
    )]
    fn lemma_region_not_set_implies_no_overlap(&self, start: FluxPtrU8, end: FluxPtrU8) {}

    #[flux_rs::sig(fn (&Self[@r],
            flash_start: FluxPtrU8,
            flash_end: FluxPtrU8,
            mem_start: FluxPtrU8,
            mem_end: FluxPtrU8
        )
        requires
            Self::region_can_access_exactly(r, flash_start, flash_end, mpu::Permissions { r: true, x: true, w: false })
            &&
            flash_end <= mem_start
        ensures
            !Self::overlaps(r, mem_start, mem_end)

    )]
    fn lemma_region_can_access_flash_implies_no_app_block_overlaps(
        &self,
        _flash_start: FluxPtrU8,
        _flash_end: FluxPtrU8,
        _mem_start: FluxPtrU8,
        _mem_end: FluxPtrU8,
    ) {
    }
}

impl<const MPU_REGIONS: usize> fmt::Display for PMPUserRegion<MPU_REGIONS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Ternary operator shortcut function, to avoid bulky formatting...
        fn t<T>(cond: bool, a: T, b: T) -> T {
            if cond {
                a
            } else {
                b
            }
        }
        let tor_user_pmpcfg = self.tor;
        let pmpcfg = tor_user_pmpcfg.get_reg();
        write!(
            f,
            "     #{:02}: start={:#010X}, end={:#010X}, cfg={:#04X} ({}) (-{}{}{})\r\n",
            self.region_number,
            self.start.unwrap_or(FluxPtrU8::null()).as_usize(),
            self.end.unwrap_or(FluxPtrU8::null()).as_usize(),
            pmpcfg.get(),
            t(pmpcfg.is_set(pmpcfg_octet::a()), "TOR", "OFF"),
            t(pmpcfg.is_set(pmpcfg_octet::r()), "r", "-"),
            t(pmpcfg.is_set(pmpcfg_octet::w()), "w", "-"),
            t(pmpcfg.is_set(pmpcfg_octet::x()), "x", "-"),
        )?;

        write!(f, " }}\r\n")?;
        Ok(())
    }
}

/// Adapter from a generic PMP implementation exposing TOR-type regions to the
/// Tock [`mpu::MPU`] trait. See [`TORUserPMP`].
pub struct PMPUserMPU<const MAX_REGIONS: usize, P: TORUserPMP<MAX_REGIONS> + 'static> {
    /// Monotonically increasing counter for allocated configurations, used to
    /// assign unique IDs to `PMPUserMPUConfig` instances.
    config_count: Cell<NonZeroUsize>,
    /// The configuration that the PMP was last configured for. Used (along with
    /// the `is_dirty` flag) to determine if PMP can skip writing the
    /// configuration to hardware.
    last_configured_for: OptionalCell<NonZeroUsize>,
    /// Underlying hardware PMP implementation, exposing a number (up to
    /// `P::MAX_REGIONS`) of memory protection regions with a 4-byte enforcement
    /// granularity.
    pub pmp: P,
}

impl<const MAX_REGIONS: usize, P: TORUserPMP<MAX_REGIONS> + 'static> PMPUserMPU<MAX_REGIONS, P> {
    pub fn new(pmp: P) -> Self {
        // Assigning this constant here ensures evaluation of the const
        // expression at compile time, and can thus be used to enforce
        // compile-time assertions based on the desired PMP configuration.
        #[allow(clippy::let_unit_value)]
        let _: () = P::CONST_ASSERT_CHECK;

        PMPUserMPU {
            config_count: Cell::new(NonZeroUsize::MIN),
            last_configured_for: OptionalCell::empty(),
            pmp,
        }
    }
}

impl<const MAX_REGIONS: usize, P: TORUserPMP<MAX_REGIONS> + 'static> kernel::platform::mpu::MPU
    for PMPUserMPU<MAX_REGIONS, P>
{
    // type MpuConfig = PMPUserMPUConfig<MAX_REGIONS>;
    type Region = PMPUserRegion<MAX_REGIONS>;

    #[flux_rs::trusted]
    fn enable_app_mpu(&self) -> MpuEnabledCapability {
        // TODO: This operation may fail when the PMP is not exclusively used
        // for userspace. Instead of panicing, we should handle this case more
        // gracefully and return an error in the `MPU` trait. Process
        // infrastructure can then attempt to re-schedule the process later on,
        // try to revoke some optional shared memory regions, or suspend the
        // process.
        self.pmp.enable_user_pmp().unwrap();
        MpuEnabledCapability {}
    }

    fn disable_app_mpu(&self) {
        self.pmp.disable_user_pmp()
    }

    fn number_total_regions(&self) -> usize {
        self.pmp.available_regions()
    }

    #[flux_rs::trusted(reason = "fixpoint encoding error")]
    fn configure_mpu(&self, config: &RArray<Self::Region>, id: usize, is_dirty: bool) {
        let mut ac_config: [Self::Region; MAX_REGIONS] =
            core::array::from_fn(|i| <Self::Region as mpu::RegionDescriptor>::default(i));
        for i in 0..MAX_REGIONS {
            if i < 8 {
                ac_config[i] = config.get(i);
            } else {
                ac_config[i] = <Self::Region as mpu::RegionDescriptor>::default(i);
            }
        }
        let mut hw = HardwareState::new();
        self.pmp.configure_pmp(&ac_config, &mut hw).unwrap();
    }
}

#[cfg(test)]
pub mod test {
    use super::{PMPUserRegion, TORUserPMP, TORUserPMPCFG};

    struct MockTORUserPMP;
    impl<const MPU_REGIONS: usize> TORUserPMP<MPU_REGIONS> for MockTORUserPMP {
        // Don't require any const-assertions in the MockTORUserPMP.
        const CONST_ASSERT_CHECK: () = ();

        fn available_regions(&self) -> usize {
            // For the MockTORUserPMP, we always assume to have the full number
            // of MPU_REGIONS available. More advanced tests may want to return
            // a different number here (to simulate kernel memory protection)
            // and make the configuration fail at runtime, for instance.
            MPU_REGIONS
        }

        fn configure_pmp(
            &self,
            _regions: &[PMPUserRegion],
            hardware_state: &mut HardwareState,
        ) -> Result<(), ()> {
            Ok(())
        }

        // #[flux_rs::sig(fn (x: usize) -> u32[x] requires x <= u32::MAX)]
        #[flux_rs::sig(fn (&Self) -> Result<(), ()>[true])]
        fn enable_user_pmp(&self) -> Result<(), ()> {
            Ok(())
        } // The kernel's MPU trait requires

        fn disable_user_pmp(&self) {}
    }

    // TODO: implement more test cases, such as:
    //
    // - Try to update the app memory break with an invalid pointer below its
    //   allocation's start address.

    #[test]
    fn test_mpu_region_no_overlap() {
        use crate::pmp::PMPUserMPU;
        use kernel::platform::mpu::{Permissions, MPU};

        let mpu: PMPUserMPU<8, MockTORUserPMP> = PMPUserMPU::new(MockTORUserPMP);
        let mut config = mpu
            .new_config()
            .expect("Failed to allocate the first MPU config");

        // Allocate a region which spans from 0x40000000 to 0x80000000 (this
        // meets PMP alignment constraints and will work on 32-bit and 64-bit
        // systems)
        let region_0 = mpu
            .allocate_region(
                0x40000000 as *const u8,
                0x40000000,
                0x40000000,
                Permissions::ReadWriteOnly,
                &mut config,
            )
            .expect(
                "Failed to allocate a well-aligned R/W MPU region with \
                 unallocated_memory_size == min_region_size",
            );
        assert!(region_0.start_address() == 0x40000000 as *const u8);
        assert!(region_0.size() == 0x40000000);

        // Try to allocate a region adjacent to `region_0`. This should work:
        let region_1 = mpu
            .allocate_region(
                0x80000000 as *const u8,
                0x10000000,
                0x10000000,
                Permissions::ReadExecuteOnly,
                &mut config,
            )
            .expect(
                "Failed to allocate a well-aligned R/W MPU region adjacent to \
                 another region",
            );
        assert!(region_1.start_address() == 0x80000000 as *const u8);
        assert!(region_1.size() == 0x10000000);

        // Remove the previously allocated `region_1`:
        mpu.remove_memory_region(region_1, &mut config)
            .expect("Failed to remove valid MPU region allocation");

        // Allocate another region which spans from 0xc0000000 to 0xd0000000
        // (this meets PMP alignment constraints and will work on 32-bit and
        // 64-bit systems), but this time allocate it using the
        // `allocate_app_memory_region` method. We want a region of `0x20000000`
        // bytes, but only the first `0x10000000` should be accessible to the
        // app.
        let (region_2_start, region_2_size) = mpu
            .allocate_app_memory_regions(
                0xc0000000 as *const u8,
                0x20000000,
                0x20000000,
                0x10000000,
                0x08000000,
                Permissions::ReadWriteOnly,
                &mut config,
            )
            .expect(
                "Failed to allocate a well-aligned R/W app memory MPU region \
                 with unallocated_memory_size == min_region_size",
            );
        assert!(region_2_start == 0xc0000000 as *const u8);
        assert!(region_2_size == 0x20000000);

        // --> General overlap tests involving both regions

        // Now, try to allocate another region that spans over both memory
        // regions. This should fail.
        assert!(mpu
            .allocate_region(
                0x40000000 as *const u8,
                0xc0000000,
                0xc0000000,
                Permissions::ReadOnly,
                &mut config,
            )
            .is_none());

        // Try to allocate a region that spans over parts of both memory
        // regions. This should fail.
        assert!(mpu
            .allocate_region(
                0x48000000 as *const u8,
                0x80000000,
                0x80000000,
                Permissions::ReadOnly,
                &mut config,
            )
            .is_none());

        // --> Overlap tests involving a single region (region_0)
        //
        // We define these in an array, such that we can run the tests with the
        // `region_0` defined (to confirm that the allocations are indeed
        // refused), and with `region_0` removed (to make sure they would work
        // in general).
        let overlap_region_0_tests = [
            (
                // Try to allocate a region that is contained within
                // `region_0`. This should fail.
                0x41000000 as *const u8,
                0x01000000,
                0x01000000,
                Permissions::ReadWriteOnly,
            ),
            (
                // Try to allocate a region that overlaps with `region_0` in the
                // front. This should fail.
                0x38000000 as *const u8,
                0x10000000,
                0x10000000,
                Permissions::ReadWriteExecute,
            ),
            (
                // Try to allocate a region that overlaps with `region_0` in the
                // back. This should fail.
                0x48000000 as *const u8,
                0x10000000,
                0x10000000,
                Permissions::ExecuteOnly,
            ),
            (
                // Try to allocate a region that spans over `region_0`. This
                // should fail.
                0x38000000 as *const u8,
                0x20000000,
                0x20000000,
                Permissions::ReadWriteOnly,
            ),
        ];

        // Make sure that the allocation requests fail with `region_0` defined:
        for (memory_start, memory_size, length, perms) in overlap_region_0_tests.iter() {
            assert!(mpu
                .allocate_region(*memory_start, *memory_size, *length, *perms, &mut config,)
                .is_none());
        }

        // Now, remove `region_0` and re-run the tests. Every test-case should
        // succeed now (in isolation, hence removing the successful allocations):
        mpu.remove_memory_region(region_0, &mut config)
            .expect("Failed to remove valid MPU region allocation");

        for region @ (memory_start, memory_size, length, perms) in overlap_region_0_tests.iter() {
            let allocation_res =
                mpu.allocate_region(*memory_start, *memory_size, *length, *perms, &mut config);

            match allocation_res {
                Some((region, _)) => {
                    mpu.remove_memory_region(region, &mut config)
                        .expect("Failed to remove valid MPU region allocation");
                }
                None => {
                    panic!(
                        "Failed to allocate region that does not overlap and \
                         should meet alignment constraints: {:?}",
                        region
                    );
                }
            }
        }

        // Make sure we can technically allocate a memory region that overlaps
        // with the kernel part of the `app_memory_region`.
        //
        // It is unclear whether this should be supported.
        let region_2 = mpu
            .allocate_region(
                0xd0000000 as *const u8,
                0x10000000,
                0x10000000,
                Permissions::ReadWriteOnly,
                &mut config,
            )
            .unwrap();
        assert!(region_2.start_address() == 0xd0000000 as *const u8);
        assert!(region_2.size() == 0x10000000);

        // Now, we can grow the app memory break into this region:
        mpu.update_app_memory_regions(
            0xd0000004 as *const u8,
            0xd8000000 as *const u8,
            Permissions::ReadWriteOnly,
            &mut config,
        )
        .expect("Failed to grow the app memory region into an existing other MPU region");

        // Now, we have two overlapping MPU regions. Remove `region_2`, and try
        // to reallocate it as `region_3`. This should fail now, demonstrating
        // that we managed to reach an invalid intermediate state:
        mpu.remove_memory_region(region_2, &mut config)
            .expect("Failed to remove valid MPU region allocation");
        assert!(mpu
            .allocate_region(
                0xd0000000 as *const u8,
                0x10000000,
                0x10000000,
                Permissions::ReadWriteOnly,
                &mut config,
            )
            .is_none());
    }
}

pub mod simple {
    use super::{pmpcfg_octet, HardwareState, PMPUserRegion, TORUserPMP, TORUserPMPCFG};
    use crate::{
        csr,
        pmp::{
            all_regions_configured_correctly_base, all_regions_configured_correctly_step,
            u32_from_be_bytes,
        },
    };
    use core::fmt;
    use flux_support::FluxPtr;
    use flux_support::{FieldValueU32, LocalRegisterCopyU8};

    /// A "simple" RISC-V PMP implementation.
    ///
    /// The SimplePMP does not support locked regions, kernel memory protection,
    /// or any ePMP features (using the mseccfg CSR). It is generic over the
    /// number of hardware PMP regions available. `AVAILABLE_ENTRIES` is
    /// expected to be set to the number of available entries.
    ///
    /// [`SimplePMP`] implements [`TORUserPMP`] to expose all of its regions as
    /// "top of range" (TOR) regions (each taking up two physical PMP entires)
    /// for use as a user-mode memory protection mechanism.
    ///
    /// Notably, [`SimplePMP`] implements `TORUserPMP<MPU_REGIONS>` over a
    /// generic `MPU_REGIONS` where `MPU_REGIONS <= (AVAILABLE_ENTRIES / 2)`. As
    /// PMP re-configuration can have a significiant runtime overhead, users are
    /// free to specify a small `MPU_REGIONS` const-generic parameter to reduce
    /// the runtime overhead induced through PMP configuration, at the cost of
    /// having less PMP regions available to use for userspace memory
    /// protection.
    #[flux_rs::refined_by(hw_state: HardwareState)]
    pub struct SimplePMP<const AVAILABLE_ENTRIES: usize> {
        #[field(HardwareState[hw_state])]
        hardware_state: HardwareState,
    }

    flux_rs::defs! {

        fn available_region_setup(i: int, old: HardwareState, new: HardwareState) -> bool {
            let cfg = map_select(new.pmpcfg_registers, i / 4);
            let region_offset = i % 4;

            if region_offset == 0 {
                extract(cfg, 0x00000018, 3) == 0 && !bit(cfg, 1 << 7)
            } else if region_offset == 1 {
                extract(cfg, 0x00001800, 11) == 0 && !bit(cfg, 1 << 15)
            } else if region_offset == 2 {
                extract(cfg, 0x00180000, 19) == 0 && !bit(cfg, 1 << 23)
            } else if region_offset == 3 {
                extract(cfg, 0x18000000, 27) == 0 && !bit(cfg, 1 << 31)
            } else {
                false
            }
        }

        // forall j, j >= 0 && j < i -> available_region_setup(i, hardware_state)
        fn all_available_regions_setup_up_to(i: int, hw: HardwareState) -> bool;
    }

    #[flux_rs::trusted(reason = "Proof Code")]
    #[flux_rs::sig(fn (&HardwareState[@hw]) ensures all_available_regions_setup_up_to(0, hw))]
    fn all_available_regions_setup_up_to_base(_: &HardwareState) {}

    #[flux_rs::trusted(reason = "Proof Code")]
    #[flux_rs::sig(fn (i: usize, &HardwareState[@old], &HardwareState[@new])
        requires all_available_regions_setup_up_to(i, old) && available_region_setup(i, old, new)
        ensures all_available_regions_setup_up_to(i + 1, new)
    )]
    fn all_available_regions_setup_up_to_step(i: usize, old: &HardwareState, new: &HardwareState) {}

    impl<const AVAILABLE_ENTRIES: usize> SimplePMP<AVAILABLE_ENTRIES> {
        pub unsafe fn new() -> Result<Self, ()> {
            // The SimplePMP does not support locked regions, kernel memory
            // protection, or any ePMP features (using the mseccfg CSR). Ensure
            // that we don't find any locked regions. If we don't have locked
            // regions and can still successfully execute code, this means that
            // we're not in the ePMP machine-mode lockdown mode, and can treat
            // our hardware as a regular PMP.
            //
            // Furthermore, we test whether we can use each entry (i.e. whether
            // it actually exists in HW) by flipping the RWX bits. If we can't
            // flip them, then `AVAILABLE_ENTRIES` is incorrect.  However, this
            // is not sufficient to check for locked regions, because of the
            // ePMP's rule-lock-bypass bit. If a rule is locked, it might be the
            // reason why we can execute code or read-write data in machine mode
            // right now. Thus, never try to touch a locked region, as we might
            // well revoke access to a kernel region!

            #[flux_rs::sig(fn (i: usize, hw_state: &strg HardwareState[@old])
                -> Result<{ i32 |  all_available_regions_setup_up_to(i + 1, new) }, ()>
                requires all_available_regions_setup_up_to(i, old)
                ensures hw_state: HardwareState[#new]
            )]
            #[flux_rs::trusted(reason = "VR:HANG")]
            fn configure_initial_pmp_idx(
                i: usize,
                hardware_state: &mut HardwareState,
            ) -> Result<i32, ()> {
                // NOTE: works over PMP entries - hence the mod 4 arithmetic when
                // checking a PMPCFG

                let old: HardwareState = hardware_state.snapshot();

                // Read the entry's CSR:
                #[flux_rs::trusted(reason = "TCB")]
                #[flux_rs::sig(fn (i: usize, &HardwareState[@hw]) -> usize[bv_bv32_to_int(map_select(hw.pmpcfg_registers, i))])]
                fn pmpconfig_get(i: usize, _: &HardwareState) -> usize {
                    csr::CSR.pmpconfig_get(i)
                }

                let pmpcfg_csr = pmpconfig_get(i / 4, &hardware_state);

                #[flux_rs::trusted(reason = "Flux integer conversion")]
                #[flux_rs::sig(fn (x: usize) -> u8[bv_bv32_to_int(extract(bv_int_to_bv32(x), 0xFF, 0))])]
                fn usize_to_u8_truncate(x: usize) -> u8 {
                    x as u8
                }

                #[flux_rs::trusted(reason = "Flux integer conversion")]
                // NOTE: trusted because usize == u32 here
                #[flux_rs::sig(fn (x: usize) -> u32[x] requires x <= u32::MAX)]
                fn usize_to_u32(x: usize) -> u32 {
                    x as u32
                }

                flux_rs::assert((i % 4) * 8 <= 24);

                // Extract the entry's pmpcfg octet:
                let pmpcfg: LocalRegisterCopyU8<pmpcfg_octet::Register> =
                    LocalRegisterCopyU8::new(usize_to_u8_truncate(super::overflowing_shr(
                        pmpcfg_csr,
                        usize_to_u32((i % 4) * 8),
                    )));

                // As outlined above, we never touch a locked region. Thus, bail
                // out if it's locked:
                if pmpcfg.is_set(pmpcfg_octet::l()) {
                    return Err(());
                }

                // Now that it's not locked, we can be sure that regardless of
                // any ePMP bits, this region is either ignored or entirely
                // denied for machine-mode access. Hence, we can change it in
                // arbitrary ways without breaking our own memory access. Try to
                // flip the R/W/X bits:
                use flux_rs::bitvec::BV32;
                // pmpcfg_csr ^ (7 << ((i % 4) * 8))
                // change xor to (a | b) & !(a & b)

                #[flux_rs::sig(fn (x: BV32, y: BV32) -> BV32[(x | y) & bv_not(x & y)])]
                fn xor(x: BV32, y: BV32) -> BV32 {
                    (x | y) & !(x & y)
                }

                let rwx_bits = xor(
                    BV32::from(pmpcfg_csr as u32),
                    BV32::from(7) << BV32::from(usize_to_u32((i % 4) * 8)),
                );
                let rwx_bits: u32 = rwx_bits.into();
                super::pmpconfig_set(i / 4, rwx_bits as usize, hardware_state);

                // Check if the CSR changed:
                if pmpcfg_csr == csr::CSR.pmpconfig_get(i / 4) {
                    // Didn't change! This means that this region is not backed
                    // by HW. Return an error as `AVAILABLE_ENTRIES` is
                    // incorrect:
                    return Err(());
                }

                // Finally, turn the region off:
                let off_bits = BV32::from(pmpcfg_csr as u32)
                    & !(BV32::from(0x18) << BV32::from(usize_to_u32((i % 4) * 8)));
                let off_bits: u32 = off_bits.into();

                super::pmpconfig_set(i / 4, off_bits as usize, hardware_state);

                all_available_regions_setup_up_to_step(i, &old, hardware_state);
                Ok(1669)
            }

            #[flux_rs::sig(fn (idx: usize, &HardwareState[@hw]) requires all_available_regions_setup_up_to(idx, hw))]
            fn assert_setup(idx: usize, _: &HardwareState) {}

            #[flux_rs::sig(fn (idx: usize, hw_state: &strg HardwareState[@og_hw], available_entries: usize)
                -> Result<{ i32 | all_available_regions_setup_up_to(available_entries, hw) }, ()>[#ok]
                requires
                    all_available_regions_setup_up_to(idx, og_hw)
                    && (idx >= available_entries => all_available_regions_setup_up_to(available_entries, og_hw))
                ensures hw_state: HardwareState[#hw]
            )]
            fn configure_initial_pmp_tail(
                i: usize,
                hardware_state: &mut HardwareState,
                available_entries: usize,
            ) -> Result<i32, ()> {
                if i >= available_entries {
                    flux_rs::assert(i >= available_entries);
                    assert_setup(available_entries, hardware_state);
                    return Ok(99);
                }
                let old = hardware_state.snapshot();
                configure_initial_pmp_idx(i, hardware_state)?;
                assert_setup(i + 1, &hardware_state);
                match configure_initial_pmp_tail(i + 1, hardware_state, available_entries) {
                    Ok(_) => return Ok(100),
                    Err(()) => return Err(()),
                }
            }

            // establish some verification specific details
            let mut hardware_state = HardwareState::new();
            all_available_regions_setup_up_to_base(&hardware_state);
            flux_support::const_assume!(AVAILABLE_ENTRIES > 0);

            configure_initial_pmp_tail(0, &mut hardware_state, AVAILABLE_ENTRIES)?;

            // Hardware PMP is verified to be in a compatible mode / state, and
            // has at least `AVAILABLE_ENTRIES` entries.
            Ok(SimplePMP { hardware_state })
        }
    }

    impl<const AVAILABLE_ENTRIES: usize, const MPU_REGIONS: usize> TORUserPMP<MPU_REGIONS>
        for SimplePMP<AVAILABLE_ENTRIES>
    {
        // Ensure that the MPU_REGIONS (starting at entry, and occupying two
        // entries per region) don't overflow the available entires.
        const CONST_ASSERT_CHECK: () = assert!(MPU_REGIONS <= (AVAILABLE_ENTRIES / 2));

        fn available_regions(&self) -> usize {
            // Always assume to have `MPU_REGIONS` usable TOR regions. We don't
            // support locked regions, or kernel protection.
            MPU_REGIONS
        }

        // This implementation is specific for 32-bit systems. We use
        // `u32::from_be_bytes` and then cast to usize, as it manages to compile
        // on 64-bit systems as well. However, this implementation will not work
        // on RV64I systems, due to the changed pmpcfgX CSR layout.
        #[flux_rs::sig(fn (&Self, &[PMPUserRegion<_>; _], hw_state: &strg HardwareState) -> Result<(), ()>[#ok] ensures hw_state: HardwareState {hw:
            ok => all_regions_configured_correctly_up_to(MPU_REGIONS, hw)
        })]
        fn configure_pmp(
            &self,
            regions: &[PMPUserRegion<MPU_REGIONS>; MPU_REGIONS],
            hardware_state: &mut HardwareState,
        ) -> Result<(), ()> {
            // configures region `i` and region `i + 1` correctly
            #[flux_rs::sig(fn (i: usize, &PMPUserRegion<_>[@er], &PMPUserRegion<_>[@or], hw_state: &strg HardwareState[@og_hw])
                // Note: these pre and post conditions (all_regions_configured) seem silly
                // but we need them because otherwise Flux forgets
                // all state after we return
                requires all_regions_configured_correctly_up_to(i, og_hw) && i <= (u32::MAX / 2) && i % 2 == 0
                ensures hw_state: HardwareState{new_hw: all_regions_configured_correctly_up_to(i + 2, new_hw) }
            )]
            fn configure_region_pair<const MPU_REGIONS: usize>(
                i: usize,
                even_region: &PMPUserRegion<MPU_REGIONS>,
                odd_region: &PMPUserRegion<MPU_REGIONS>,
                hardware_state: &mut HardwareState,
            ) {
                let old = hardware_state.snapshot();
                let even_region_start = match even_region.start {
                    Some(r) => r,
                    None => FluxPtr::null(),
                };
                let even_region_end = match even_region.end {
                    Some(r) => r,
                    None => FluxPtr::null(),
                };
                let odd_region_start = match odd_region.start {
                    Some(r) => r,
                    None => FluxPtr::null(),
                };
                let odd_region_end = match odd_region.end {
                    Some(r) => r,
                    None => FluxPtr::null(),
                };

                // We can configure two regions at once which, given that we
                // start at index 0 (an even offset), translates to a single
                // CSR write for the pmpcfgX register:
                super::pmpconfig_set(
                    i / 2,
                    u32_from_be_bytes(
                        odd_region.tor.get(),
                        TORUserPMPCFG::OFF().get(),
                        even_region.tor.get(),
                        TORUserPMPCFG::OFF().get(),
                    ) as usize,
                    hardware_state,
                );

                // Now, set the addresses of the respective regions, if they
                // are enabled, respectively:
                if even_region.tor != TORUserPMPCFG::OFF() {
                    super::pmpaddr_set(
                        i * 2 + 0,
                        super::overflowing_shr(even_region_start.as_usize(), 2),
                        hardware_state,
                    );

                    super::pmpaddr_set(
                        i * 2 + 1,
                        super::overflowing_shr(even_region_end.as_usize(), 2),
                        hardware_state,
                    );
                }

                if odd_region.tor != TORUserPMPCFG::OFF() {
                    super::pmpaddr_set(
                        i * 2 + 2,
                        super::overflowing_shr(odd_region_start.as_usize(), 2),
                        hardware_state,
                    );
                    super::pmpaddr_set(
                        i * 2 + 3,
                        super::overflowing_shr(odd_region_end.as_usize(), 2),
                        hardware_state,
                    );
                }
                all_regions_configured_correctly_step(even_region, &old, &hardware_state, i);
                all_regions_configured_correctly_step(
                    odd_region,
                    &hardware_state,
                    &hardware_state,
                    i + 1,
                );
            }

            // configures region `i` correctly
            #[flux_rs::sig(fn (i: usize, &PMPUserRegion<_>[@er], hw_state: &strg HardwareState[@og_hw])
                // Note: these pre and post conditions (all_regions_configured) seem silly
                // but we need them because otherwise Flux forgets
                // all state after we return
                requires all_regions_configured_correctly_up_to(i, og_hw) && i <= (u32::MAX / 2) && i % 2 == 0
                ensures hw_state: HardwareState{new_hw:
                    all_regions_configured_correctly_up_to(i + 1, new_hw)
                }
            )]
            fn configure_region<const MPU_REGIONS: usize>(
                i: usize,
                even_region: &PMPUserRegion<MPU_REGIONS>,
                hardware_state: &mut HardwareState,
            ) {
                let old = hardware_state.snapshot();
                let even_region_start = match even_region.start {
                    Some(r) => r,
                    None => FluxPtr::null(),
                };
                let even_region_end = match even_region.end {
                    Some(r) => r,
                    None => FluxPtr::null(),
                };

                // TODO: check overhead of code
                // Modify the first two pmpcfgX octets for this region:
                let bits = FieldValueU32::<csr::pmpconfig::pmpcfg::Register>::new(
                    0x0000FFFF,
                    0,
                    u32_from_be_bytes(0, 0, even_region.tor.get(), TORUserPMPCFG::OFF().get()),
                );

                super::pmpconfig_modify(i / 2, bits, hardware_state);

                // Set the addresses if the region is enabled:
                if even_region.tor != TORUserPMPCFG::OFF() {
                    super::pmpaddr_set(
                        i * 2 + 0,
                        super::overflowing_shr(even_region_start.as_usize(), 2),
                        hardware_state,
                    );
                    super::pmpaddr_set(
                        i * 2 + 1,
                        super::overflowing_shr(even_region_end.as_usize(), 2),
                        hardware_state,
                    );
                }
                all_regions_configured_correctly_step(even_region, &old, &hardware_state, i);
            }

            #[flux_rs::sig(
                fn (i: usize, core::slice::Iter<PMPUserRegion<_>>[@idx, @len], max_regions: usize, hw_state: &strg HardwareState[@og_hw])
                requires
                    all_regions_configured_correctly_up_to(i, og_hw)
                    && len == max_regions
                    && (idx < len => i == idx && i % 2 == 0)
                    && (idx >= len => all_regions_configured_correctly_up_to(max_regions, og_hw))
                ensures hw_state: HardwareState{new_hw: all_regions_configured_correctly_up_to(max_regions, new_hw)}
            )]
            fn configure_all_regions_tail<const MPU_REGIONS: usize>(
                i: usize,
                mut regions_iter: core::slice::Iter<'_, PMPUserRegion<MPU_REGIONS>>,
                max_regions: usize,
                hardware_state: &mut HardwareState,
            ) {
                // FLUX: the invariant here is i + regions_iter.len() == MPU_REGIONS, but ...
                flux_support::assume(i <= (u32::MAX / 2) as usize);
                if let Some(even_region) = regions_iter.next() {
                    let odd_region_opt = regions_iter.next();

                    match odd_region_opt {
                        None => {
                            configure_region(i, even_region, hardware_state);
                            configure_all_regions_tail(
                                i + 1,
                                regions_iter,
                                max_regions,
                                hardware_state,
                            );
                        }
                        Some(odd_region) => {
                            configure_region_pair(i, even_region, odd_region, hardware_state);
                            configure_all_regions_tail(
                                i + 2,
                                regions_iter,
                                max_regions,
                                hardware_state,
                            );
                        }
                    }
                }
            }

            // this should be an invariant but it's on a trait so things are weird
            if regions.len() == 0 {
                return Err(());
            }
            let regions_iter = regions.iter();
            // call lemma to establish the original precondition
            all_regions_configured_correctly_base(hardware_state);
            configure_all_regions_tail(0, regions_iter, MPU_REGIONS, hardware_state);

            Ok(())
        }

        fn enable_user_pmp(&self) -> Result<(), ()> {
            // No-op. The SimplePMP does not have any kernel-enforced regions.
            Ok(())
        }

        fn disable_user_pmp(&self) {
            // No-op. The SimplePMP does not have any kernel-enforced regions.
        }
    }

    impl<const AVAILABLE_ENTRIES: usize> fmt::Display for SimplePMP<AVAILABLE_ENTRIES> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, " PMP hardware configuration -- entries: \r\n")?;
            unsafe { super::format_pmp_entries::<AVAILABLE_ENTRIES>(f) }
        }
    }
}

// pub mod kernel_protection {
//     use super::{
//         all_regions_configured_correctly_base, all_regions_configured_correctly_step, pmpcfg_octet,
//         u32_from_be_bytes, HardwareState, NAPOTRegionSpec, PMPUserRegion, TORRegionSpec,
//         TORUserPMP, TORUserPMPCFG,
//     };
//     use crate::csr;
//     use core::fmt;
//     use flux_support::LocalRegisterCopyU8;
//     use flux_support::{FieldValueU32, FluxPtr};
//     use kernel::utilities::registers::FieldValue;

//     // ---------- Kernel memory-protection PMP memory region wrapper types -----
//     //
//     // These types exist primarily to avoid argument confusion in the
//     // [`KernelProtectionPMP`] constructor, which accepts the addresses of these
//     // memory regions as arguments. They further encode whether a region must
//     // adhere to the `NAPOT` or `TOR` addressing mode constraints:

//     /// The flash memory region address range.
//     ///
//     /// Configured in the PMP as a `NAPOT` region.
//     #[derive(Copy, Clone, Debug)]
//     pub struct FlashRegion(pub NAPOTRegionSpec);

//     /// The RAM region address range.
//     ///
//     /// Configured in the PMP as a `NAPOT` region.
//     #[derive(Copy, Clone, Debug)]
//     pub struct RAMRegion(pub NAPOTRegionSpec);

//     /// The MMIO region address range.
//     ///
//     /// Configured in the PMP as a `NAPOT` region.
//     #[derive(Copy, Clone, Debug)]
//     pub struct MMIORegion(pub NAPOTRegionSpec);

//     /// The PMP region specification for the kernel `.text` section.
//     ///
//     /// This is to be made accessible to machine-mode as read-execute.
//     /// Configured in the PMP as a `TOR` region.
//     #[derive(Copy, Clone, Debug)]
//     pub struct KernelTextRegion(pub TORRegionSpec);

//     /// A RISC-V PMP implementation which supports machine-mode (kernel) memory
//     /// protection, with a fixed number of "kernel regions" (such as `.text`,
//     /// flash, RAM and MMIO).
//     ///
//     /// This implementation will configure the PMP in the following way:
//     ///
//     ///   ```text
//     ///   |-------+-----------------------------------------+-------+---+-------|
//     ///   | ENTRY | REGION / ADDR                           | MODE  | L | PERMS |
//     ///   |-------+-----------------------------------------+-------+---+-------|
//     ///   |     0 | /                                     \ | OFF   |   |       |
//     ///   |     1 | \ Userspace TOR region #0             / | TOR   |   | ????? |
//     ///   |       |                                         |       |   |       |
//     ///   |     2 | /                                     \ | OFF   |   |       |
//     ///   |     3 | \ Userspace TOR region #1             / | TOR   |   | ????? |
//     ///   |       |                                         |       |   |       |
//     ///   | 4 ... | /                                     \ |       |   |       |
//     ///   | n - 8 | \ Userspace TOR region #x             / |       |   |       |
//     ///   |       |                                         |       |   |       |
//     ///   | n - 7 | "Deny-all" user-mode rule (all memory)  | NAPOT |   | ----- |
//     ///   |       |                                         |       |   |       |
//     ///   | n - 6 | --------------------------------------- | OFF   | X | ----- |
//     ///   | n - 5 | Kernel .text section                    | TOR   | X | R/X   |
//     ///   |       |                                         |       |   |       |
//     ///   | n - 4 | FLASH (spanning kernel & apps)          | NAPOT | X | R     |
//     ///   |       |                                         |       |   |       |
//     ///   | n - 3 | RAM (spanning kernel & apps)            | NAPOT | X | R/W   |
//     ///   |       |                                         |       |   |       |
//     ///   | n - 2 | MMIO                                    | NAPOT | X | R/W   |
//     ///   |       |                                         |       |   |       |
//     ///   | n - 1 | "Deny-all" machine-mode    (all memory) | NAPOT | X | ----- |
//     ///   |-------+-----------------------------------------+-------+---+-------|
//     ///   ```
//     ///
//     /// This implementation does not use any `mseccfg` protection bits (ePMP
//     /// functionality). To protect machine-mode (kernel) memory regions, regions
//     /// must be marked as locked. However, locked regions apply to both user-
//     /// and machine-mode. Thus, region `n - 7` serves as a "deny-all" user-mode
//     /// rule, which prohibits all accesses not explicitly allowed through rules
//     /// `< n - 7`. Kernel memory is made accessible underneath this "deny-all"
//     /// region, which does not apply to machine-mode.
//     ///
//     /// This PMP implementation supports the [`TORUserPMP`] interface with
//     /// `MPU_REGIONS <= ((AVAILABLE_ENTRIES - 7) / 2)`, to leave sufficient
//     /// space for the "deny-all" and kernel regions. This constraint is enforced
//     /// through the [`KernelProtectionPMP::CONST_ASSERT_CHECK`] associated
//     /// constant, which MUST be evaluated by the consumer of the [`TORUserPMP`]
//     /// trait (usually the [`PMPUserMPU`](super::PMPUserMPU) implementation).
//     #[flux_rs::invariant(AVAILABLE_ENTRIES >= 7)]
//     pub struct KernelProtectionPMP<const AVAILABLE_ENTRIES: usize>;
//     impl<const AVAILABLE_ENTRIES: usize> KernelProtectionPMP<AVAILABLE_ENTRIES> {
//         pub unsafe fn new(
//             flash: FlashRegion,
//             ram: RAMRegion,
//             mmio: MMIORegion,
//             kernel_text: KernelTextRegion,
//         ) -> Result<Self, ()> {
//             for i in 0..AVAILABLE_ENTRIES {
//                 // Read the entry's CSR:
//                 let pmpcfg_csr = csr::CSR.pmpconfig_get(i / 4);

//                 // Extract the entry's pmpcfg octet:
//                 let pmpcfg: LocalRegisterCopyU8<pmpcfg_octet::Register> = LocalRegisterCopyU8::new(
//                     pmpcfg_csr.overflowing_shr(((i % 4) * 8) as u32).0 as u8,
//                 );

//                 // As outlined above, we never touch a locked region. Thus, bail
//                 // out if it's locked:
//                 if pmpcfg.is_set(pmpcfg_octet::l()) {
//                     return Err(());
//                 }

//                 // Now that it's not locked, we can be sure that regardless of
//                 // any ePMP bits, this region is either ignored or entirely
//                 // denied for machine-mode access. Hence, we can change it in
//                 // arbitrary ways without breaking our own memory access. Try to
//                 // flip the R/W/X bits:
//                 csr::CSR.pmpconfig_set(i / 4, pmpcfg_csr ^ (7 << ((i % 4) * 8)));

//                 // Check if the CSR changed:
//                 if pmpcfg_csr == csr::CSR.pmpconfig_get(i / 4) {
//                     // Didn't change! This means that this region is not backed
//                     // by HW. Return an error as `AVAILABLE_ENTRIES` is
//                     // incorrect:
//                     return Err(());
//                 }

//                 // Finally, turn the region off:
//                 csr::CSR.pmpconfig_set(i / 4, pmpcfg_csr & !(0x18 << ((i % 4) * 8)));
//             }

//             // -----------------------------------------------------------------
//             // Hardware PMP is verified to be in a compatible mode & state, and
//             // has at least `AVAILABLE_ENTRIES` entries.
//             // -----------------------------------------------------------------

//             // Now we need to set up the various kernel memory protection
//             // regions, and the deny-all userspace region (n - 8), never
//             // modified.

//             // Helper to modify an arbitrary PMP entry. Because we don't know
//             // AVAILABLE_ENTRIES in advance, there's no good way to
//             // optimize this further.
//             fn write_pmpaddr_pmpcfg(i: usize, pmpcfg: u8, pmpaddr: usize) {
//                 csr::CSR.pmpaddr_set(i, pmpaddr);
//                 csr::CSR.pmpconfig_modify(
//                     i / 4,
//                     FieldValue::<usize, csr::pmpconfig::pmpcfg::Register>::new(
//                         0x000000FF_usize,
//                         (i % 4) * 8,
//                         u32::from_be_bytes([0, 0, 0, pmpcfg]) as usize,
//                     ),
//                 );
//             }

//             flux_support::const_assume!(AVAILABLE_ENTRIES >= 7);

//             // Set the kernel `.text`, flash, RAM and MMIO regions, in no
//             // particular order, with the exception of `.text` and flash:
//             // `.text` must precede flash, as otherwise we'd be revoking execute
//             // permissions temporarily. Given that we can currently execute
//             // code, this should not have any impact on our accessible memory,
//             // assuming that the provided regions are not otherwise aliased.

//             // MMIO at n - 2:
//             write_pmpaddr_pmpcfg(
//                 AVAILABLE_ENTRIES - 2,
//                 (pmpcfg_octet::a::NAPOT()
//                     + pmpcfg_octet::r::SET()
//                     + pmpcfg_octet::w::SET()
//                     + pmpcfg_octet::x::CLEAR()
//                     + pmpcfg_octet::l::SET())
//                 .value(),
//                 mmio.0.napot_addr(),
//             );

//             // RAM at n - 3:
//             write_pmpaddr_pmpcfg(
//                 AVAILABLE_ENTRIES - 3,
//                 (pmpcfg_octet::a::NAPOT()
//                     + pmpcfg_octet::r::SET()
//                     + pmpcfg_octet::w::SET()
//                     + pmpcfg_octet::x::CLEAR()
//                     + pmpcfg_octet::l::SET())
//                 .value(),
//                 ram.0.napot_addr(),
//             );

//             // `.text` at n - 6 and n - 5 (TOR region):
//             write_pmpaddr_pmpcfg(
//                 AVAILABLE_ENTRIES - 6,
//                 (pmpcfg_octet::a::OFF()
//                     + pmpcfg_octet::r::CLEAR()
//                     + pmpcfg_octet::w::CLEAR()
//                     + pmpcfg_octet::x::CLEAR()
//                     + pmpcfg_octet::l::SET())
//                 .value(),
//                 (kernel_text.0.start() as usize) >> 2,
//             );
//             write_pmpaddr_pmpcfg(
//                 AVAILABLE_ENTRIES - 5,
//                 (pmpcfg_octet::a::TOR()
//                     + pmpcfg_octet::r::SET()
//                     + pmpcfg_octet::w::CLEAR()
//                     + pmpcfg_octet::x::SET()
//                     + pmpcfg_octet::l::SET())
//                 .value(),
//                 (kernel_text.0.end() as usize) >> 2,
//             );

//             // flash at n - 4:
//             write_pmpaddr_pmpcfg(
//                 AVAILABLE_ENTRIES - 4,
//                 (pmpcfg_octet::a::NAPOT()
//                     + pmpcfg_octet::r::SET()
//                     + pmpcfg_octet::w::CLEAR()
//                     + pmpcfg_octet::x::CLEAR()
//                     + pmpcfg_octet::l::SET())
//                 .value(),
//                 flash.0.napot_addr(),
//             );

//             // Now that the kernel has explicit region definitions for any
//             // memory that it needs to have access to, we can deny other memory
//             // accesses in our very last rule (n - 1):
//             write_pmpaddr_pmpcfg(
//                 AVAILABLE_ENTRIES - 1,
//                 (pmpcfg_octet::a::NAPOT()
//                     + pmpcfg_octet::r::CLEAR()
//                     + pmpcfg_octet::w::CLEAR()
//                     + pmpcfg_octet::x::CLEAR()
//                     + pmpcfg_octet::l::SET())
//                 .value(),
//                 // the entire address space:
//                 0x7FFFFFFF,
//             );

//             // Finally, we configure the non-locked user-mode deny all
//             // rule. This must never be removed, or otherwise usermode will be
//             // able to access all locked regions (which are supposed to be
//             // exclusively accessible to kernel-mode):
//             write_pmpaddr_pmpcfg(
//                 AVAILABLE_ENTRIES - 7,
//                 (pmpcfg_octet::a::NAPOT()
//                     + pmpcfg_octet::r::CLEAR()
//                     + pmpcfg_octet::w::CLEAR()
//                     + pmpcfg_octet::x::CLEAR()
//                     + pmpcfg_octet::l::CLEAR())
//                 .value(),
//                 // the entire address space:
//                 0x7FFFFFFF,
//             );

//             // Setup complete
//             Ok(KernelProtectionPMP)
//         }
//     }

//     impl<const AVAILABLE_ENTRIES: usize, const MPU_REGIONS: usize> TORUserPMP<MPU_REGIONS>
//         for KernelProtectionPMP<AVAILABLE_ENTRIES>
//     {
//         /// Ensure that the MPU_REGIONS (starting at entry, and occupying two
//         /// entries per region) don't overflow the available entires, excluding
//         /// the 7 entires used for implementing the kernel memory protection.
//         const CONST_ASSERT_CHECK: () = assert!(MPU_REGIONS <= ((AVAILABLE_ENTRIES - 7) / 2));

//         fn available_regions(&self) -> usize {
//             // Always assume to have `MPU_REGIONS` usable TOR regions. We don't
//             // support locking additional regions at runtime.
//             MPU_REGIONS
//         }

//         // This implementation is specific for 32-bit systems. We use
//         // `u32::from_be_bytes` and then cast to usize, as it manages to compile
//         // on 64-bit systems as well. However, this implementation will not work
//         // on RV64I systems, due to the changed pmpcfgX CSR layout.
//         #[flux_rs::sig(fn (&Self, &[PMPUserRegion; _], hw_state: &strg HardwareState) -> Result<(), ()>[#ok] ensures hw_state: HardwareState {hw:
//             ok => all_regions_configured_correctly_up_to(MPU_REGIONS, hw)
//         })]
//         fn configure_pmp(
//             &self,
//             regions: &[PMPUserRegion; MPU_REGIONS],
//             hardware_state: &mut HardwareState,
//         ) -> Result<(), ()> {
//             // configures region `i` and region `i + 1` correctly
//             #[flux_rs::sig(fn (i: usize, &PMPUserRegion[@er], &PMPUserRegion[@or], hw_state: &strg HardwareState[@og_hw])
//                 // Note: these pre and post conditions (all_regions_configured) seem silly
//                 // but we need them because otherwise Flux forgets
//                 // all state after we return
//                 requires all_regions_configured_correctly_up_to(i, og_hw) && i % 2 == 0
//                 ensures hw_state: HardwareState{new_hw: all_regions_configured_correctly_up_to(i + 2, new_hw) }
//             )]
//             fn configure_region_pair(
//                 i: usize,
//                 even_region: &PMPUserRegion,
//                 odd_region: &PMPUserRegion,
//                 hardware_state: &mut HardwareState,
//             ) {
//                 let old = hardware_state.snapshot();
//                 let even_region_start = match even_region.start {
//                     Some(r) => r,
//                     None => FluxPtr::null(),
//                 };
//                 let even_region_end = match even_region.end {
//                     Some(r) => r,
//                     None => FluxPtr::null(),
//                 };
//                 let odd_region_start = match odd_region.start {
//                     Some(r) => r,
//                     None => FluxPtr::null(),
//                 };
//                 let odd_region_end = match odd_region.end {
//                     Some(r) => r,
//                     None => FluxPtr::null(),
//                 };

//                 // We can configure two regions at once which, given that we
//                 // start at index 0 (an even offset), translates to a single
//                 // CSR write for the pmpcfgX register:
//                 super::pmpconfig_set(
//                     i / 2,
//                     u32_from_be_bytes(
//                         odd_region.tor.get(),
//                         TORUserPMPCFG::OFF().get(),
//                         even_region.tor.get(),
//                         TORUserPMPCFG::OFF().get(),
//                     ) as usize,
//                     hardware_state,
//                 );

//                 // Now, set the addresses of the respective regions, if they
//                 // are enabled, respectively:
//                 if even_region.tor != TORUserPMPCFG::OFF() {
//                     super::pmpaddr_set(
//                         i * 2 + 0,
//                         super::overflowing_shr(even_region_start.as_usize(), 2),
//                         hardware_state,
//                     );

//                     super::pmpaddr_set(
//                         i * 2 + 1,
//                         super::overflowing_shr(even_region_end.as_usize(), 2),
//                         hardware_state,
//                     );
//                 }

//                 if odd_region.tor != TORUserPMPCFG::OFF() {
//                     super::pmpaddr_set(
//                         i * 2 + 2,
//                         super::overflowing_shr(odd_region_start.as_usize(), 2),
//                         hardware_state,
//                     );
//                     super::pmpaddr_set(
//                         i * 2 + 3,
//                         super::overflowing_shr(odd_region_end.as_usize(), 2),
//                         hardware_state,
//                     );
//                 }
//                 all_regions_configured_correctly_step(even_region, &old, &hardware_state, i);
//                 all_regions_configured_correctly_step(
//                     odd_region,
//                     &hardware_state,
//                     &hardware_state,
//                     i + 1,
//                 );
//             }

//             // configures region `i` correctly
//             #[flux_rs::sig(fn (i: usize, &PMPUserRegion[@er], hw_state: &strg HardwareState[@og_hw])
//                 // Note: these pre and post conditions (all_regions_configured) seem silly
//                 // but we need them because otherwise Flux forgets
//                 // all state after we return
//                 requires all_regions_configured_correctly_up_to(i, og_hw) && i % 2 == 0
//                 ensures hw_state: HardwareState{new_hw: all_regions_configured_correctly_up_to(i + 1, new_hw) }
//             )]
//             fn configure_region(
//                 i: usize,
//                 even_region: &PMPUserRegion,
//                 hardware_state: &mut HardwareState,
//             ) {
//                 let old = hardware_state.snapshot();
//                 let even_region_start = match even_region.start {
//                     Some(r) => r,
//                     None => FluxPtr::null(),
//                 };
//                 let even_region_end = match even_region.end {
//                     Some(r) => r,
//                     None => FluxPtr::null(),
//                 };

//                 // TODO: check overhead of code
//                 // Modify the first two pmpcfgX octets for this region:
//                 let bits = FieldValueU32::<csr::pmpconfig::pmpcfg::Register>::new(
//                     0x0000FFFF,
//                     0,
//                     u32_from_be_bytes(0, 0, even_region.tor.get(), TORUserPMPCFG::OFF().get()),
//                 );

//                 super::pmpconfig_modify(i / 2, bits, hardware_state);

//                 // Set the addresses if the region is enabled:
//                 if even_region.tor != TORUserPMPCFG::OFF() {
//                     super::pmpaddr_set(
//                         i * 2 + 0,
//                         super::overflowing_shr(even_region_start.as_usize(), 2),
//                         hardware_state,
//                     );
//                     super::pmpaddr_set(
//                         i * 2 + 1,
//                         super::overflowing_shr(even_region_end.as_usize(), 2),
//                         hardware_state,
//                     );
//                 }
//                 all_regions_configured_correctly_step(even_region, &old, &hardware_state, i);
//             }

//             #[flux_rs::sig(
//                 fn (i: usize, core::slice::Iter<PMPUserRegion>[@idx, @len], max_regions: usize, hw_state: &strg HardwareState[@og_hw])
//                 requires
//                     all_regions_configured_correctly_up_to(i, og_hw)
//                     && len == max_regions
//                     && (idx < len => i == idx && i % 2 == 0)
//                     && (idx >= len => all_regions_configured_correctly_up_to(max_regions, og_hw))
//                 ensures hw_state: HardwareState{new_hw: all_regions_configured_correctly_up_to(max_regions, new_hw)}
//             )]
//             fn configure_all_regions_tail(
//                 i: usize,
//                 mut regions_iter: core::slice::Iter<'_, PMPUserRegion>,
//                 max_regions: usize,
//                 hardware_state: &mut HardwareState,
//             ) {
//                 if let Some(even_region) = regions_iter.next() {
//                     let odd_region_opt = regions_iter.next();

//                     match odd_region_opt {
//                         None => {
//                             configure_region(i, even_region, hardware_state);
//                             configure_all_regions_tail(
//                                 i + 1,
//                                 regions_iter,
//                                 max_regions,
//                                 hardware_state,
//                             );
//                         }
//                         Some(odd_region) => {
//                             configure_region_pair(i, even_region, odd_region, hardware_state);
//                             configure_all_regions_tail(
//                                 i + 2,
//                                 regions_iter,
//                                 max_regions,
//                                 hardware_state,
//                             );
//                         }
//                     }
//                 }
//             }

//             // this should be an invariant but it's on a trait so things are weird
//             if regions.len() == 0 {
//                 return Err(());
//             }
//             let regions_iter = regions.iter();
//             // call lemma to establish the original precondition
//             all_regions_configured_correctly_base(hardware_state);
//             configure_all_regions_tail(0, regions_iter, MPU_REGIONS, hardware_state);

//             Ok(())
//         }

//         fn enable_user_pmp(&self) -> Result<(), ()> {
//             // No-op. User-mode regions are never enforced in machine-mode, and
//             // thus can be configured direct and may stay enabled in
//             // machine-mode.
//             Ok(())
//         }

//         fn disable_user_pmp(&self) {
//             // No-op. User-mode regions are never enforced in machine-mode, and
//             // thus can be configured direct and may stay enabled in
//             // machine-mode.
//         }
//     }

//     impl<const AVAILABLE_ENTRIES: usize> fmt::Display for KernelProtectionPMP<AVAILABLE_ENTRIES> {
//         fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//             write!(f, " PMP hardware configuration -- entries: \r\n")?;
//             unsafe { super::format_pmp_entries::<AVAILABLE_ENTRIES>(f) }
//         }
//     }
// }

// pub mod kernel_protection_mml_epmp {
//     use super::{
//         pmpcfg_octet, HardwareState, NAPOTRegionSpec, PMPUserRegion, TORRegionSpec, TORUserPMP,
//         TORUserPMPCFG,
//     };
//     use crate::csr;
//     use crate::pmp::permissions_to_pmpcfg;
//     use core::cell::Cell;
//     use core::fmt;
//     use flux_support::FluxPtr;
//     use flux_support::LocalRegisterCopyU8;
//     use kernel::platform::mpu;
//     use kernel::utilities::registers::interfaces::{Readable, Writeable};
//     use kernel::utilities::registers::FieldValue;

//     // ---------- Kernel memory-protection PMP memory region wrapper types -----
//     //
//     // These types exist primarily to avoid argument confusion in the
//     // [`KernelProtectionMMLEPMP`] constructor, which accepts the addresses of
//     // these memory regions as arguments. They further encode whether a region
//     // must adhere to the `NAPOT` or `TOR` addressing mode constraints:

//     /// The flash memory region address range.
//     ///
//     /// Configured in the PMP as a `NAPOT` region.
//     #[derive(Copy, Clone, Debug)]
//     pub struct FlashRegion(pub NAPOTRegionSpec);

//     /// The RAM region address range.
//     ///
//     /// Configured in the PMP as a `NAPOT` region.
//     #[derive(Copy, Clone, Debug)]
//     pub struct RAMRegion(pub NAPOTRegionSpec);

//     /// The MMIO region address range.
//     ///
//     /// Configured in the PMP as a `NAPOT` region.
//     #[derive(Copy, Clone, Debug)]
//     pub struct MMIORegion(pub NAPOTRegionSpec);

//     /// The PMP region specification for the kernel `.text` section.
//     ///
//     /// This is to be made accessible to machine-mode as read-execute.
//     /// Configured in the PMP as a `TOR` region.
//     #[derive(Copy, Clone, Debug)]
//     pub struct KernelTextRegion(pub TORRegionSpec);

//     /// A RISC-V ePMP implementation which supports machine-mode (kernel) memory
//     /// protection by using the machine-mode lockdown mode (MML), with a fixed
//     /// number of "kernel regions" (such as `.text`, flash, RAM and MMIO).
//     ///
//     /// This implementation will configure the ePMP in the following way:
//     ///
//     /// - `mseccfg` CSR:
//     ///   ```text
//     ///   |-------------+-----------------------------------------------+-------|
//     ///   | MSECCFG BIT | LABEL                                         | STATE |
//     ///   |-------------+-----------------------------------------------+-------|
//     ///   |           0 | Machine-Mode Lockdown (MML)                   |     1 |
//     ///   |           1 | Machine-Mode Whitelist Policy (MMWP)          |     1 |
//     ///   |           2 | Rule-Lock Bypass (RLB)                        |     0 |
//     ///   |-------------+-----------------------------------------------+-------|
//     ///   ```
//     ///
//     /// - `pmpaddrX` / `pmpcfgX` CSRs:
//     ///   ```text
//     ///   |-------+-----------------------------------------+-------+---+-------|
//     ///   | ENTRY | REGION / ADDR                           | MODE  | L | PERMS |
//     ///   |-------+-----------------------------------------+-------+---+-------|
//     ///   |     0 | --------------------------------------- | OFF   | X | ----- |
//     ///   |     1 | Kernel .text section                    | TOR   | X | R/X   |
//     ///   |       |                                         |       |   |       |
//     ///   |     2 | /                                     \ | OFF   |   |       |
//     ///   |     3 | \ Userspace TOR region #0             / | TOR   |   | ????? |
//     ///   |       |                                         |       |   |       |
//     ///   |     4 | /                                     \ | OFF   |   |       |
//     ///   |     5 | \ Userspace TOR region #1             / | TOR   |   | ????? |
//     ///   |       |                                         |       |   |       |
//     ///   | 6 ... | /                                     \ |       |   |       |
//     ///   | n - 4 | \ Userspace TOR region #x             / |       |   |       |
//     ///   |       |                                         |       |   |       |
//     ///   | n - 3 | FLASH (spanning kernel & apps)          | NAPOT | X | R     |
//     ///   |       |                                         |       |   |       |
//     ///   | n - 2 | RAM (spanning kernel & apps)            | NAPOT | X | R/W   |
//     ///   |       |                                         |       |   |       |
//     ///   | n - 1 | MMIO                                    | NAPOT | X | R/W   |
//     ///   |-------+-----------------------------------------+-------+---+-------|
//     ///   ```
//     ///
//     /// Crucially, this implementation relies on an unconfigured hardware PMP
//     /// implementing the ePMP (`mseccfg` CSR) extension, providing the Machine
//     /// Lockdown Mode (MML) security bit. This bit is required to ensure that
//     /// any machine-mode (kernel) protection regions (lock bit set) are only
//     /// accessible to kernel mode.
//     #[flux_rs::invariant(AVAILABLE_ENTRIES >= 3)]
//     pub struct KernelProtectionMMLEPMP<const AVAILABLE_ENTRIES: usize, const MPU_REGIONS: usize> {
//         user_pmp_enabled: Cell<bool>,
//         shadow_user_pmpcfgs: [Cell<TORUserPMPCFG>; MPU_REGIONS],
//     }

//     impl<const AVAILABLE_ENTRIES: usize, const MPU_REGIONS: usize>
//         KernelProtectionMMLEPMP<AVAILABLE_ENTRIES, MPU_REGIONS>
//     {
//         // Start user-mode TOR regions after the first kernel .text region:
//         const TOR_REGIONS_OFFSET: usize = 1;

//         pub unsafe fn new(
//             flash: FlashRegion,
//             ram: RAMRegion,
//             mmio: MMIORegion,
//             kernel_text: KernelTextRegion,
//         ) -> Result<Self, ()> {
//             for i in 0..AVAILABLE_ENTRIES {
//                 // Read the entry's CSR:
//                 let pmpcfg_csr = csr::CSR.pmpconfig_get(i / 4);

//                 // Extract the entry's pmpcfg octet:
//                 let pmpcfg: LocalRegisterCopyU8<pmpcfg_octet::Register> = LocalRegisterCopyU8::new(
//                     pmpcfg_csr.overflowing_shr(((i % 4) * 8) as u32).0 as u8,
//                 );

//                 // As outlined above, we never touch a locked region. Thus, bail
//                 // out if it's locked:
//                 if pmpcfg.is_set(pmpcfg_octet::l()) {
//                     return Err(());
//                 }

//                 // Now that it's not locked, we can be sure that regardless of
//                 // any ePMP bits, this region is either ignored or entirely
//                 // denied for machine-mode access. Hence, we can change it in
//                 // arbitrary ways without breaking our own memory access. Try to
//                 // flip the R/W/X bits:
//                 csr::CSR.pmpconfig_set(i / 4, pmpcfg_csr ^ (7 << ((i % 4) * 8)));

//                 // Check if the CSR changed:
//                 if pmpcfg_csr == csr::CSR.pmpconfig_get(i / 4) {
//                     // Didn't change! This means that this region is not backed
//                     // by HW. Return an error as `AVAILABLE_ENTRIES` is
//                     // incorrect:
//                     return Err(());
//                 }

//                 // Finally, turn the region off:
//                 csr::CSR.pmpconfig_set(i / 4, pmpcfg_csr & !(0x18 << ((i % 4) * 8)));
//             }

//             // -----------------------------------------------------------------
//             // Hardware PMP is verified to be in a compatible mode & state, and
//             // has at least `AVAILABLE_ENTRIES` entries. We have not yet checked
//             // whether the PMP is actually an _e_PMP. However, we don't want to
//             // produce a gadget to set RLB, and so the only safe way to test
//             // this is to set up the PMP regions and then try to enable the
//             // mseccfg bits.
//             // -----------------------------------------------------------------

//             // Helper to modify an arbitrary PMP entry. Because we don't know
//             // AVAILABLE_ENTRIES in advance, there's no good way to
//             // optimize this further.
//             fn write_pmpaddr_pmpcfg(i: usize, pmpcfg: u8, pmpaddr: usize) {
//                 // Important to set the address first. Locking the pmpcfg
//                 // register will also lock the adress register!
//                 csr::CSR.pmpaddr_set(i, pmpaddr);
//                 csr::CSR.pmpconfig_modify(
//                     i / 4,
//                     FieldValue::<usize, csr::pmpconfig::pmpcfg::Register>::new(
//                         0x000000FF_usize,
//                         (i % 4) * 8,
//                         u32::from_be_bytes([0, 0, 0, pmpcfg]) as usize,
//                     ),
//                 );
//             }

//             flux_support::const_assume!(AVAILABLE_ENTRIES >= 3);
//             // Set the kernel `.text`, flash, RAM and MMIO regions, in no
//             // particular order, with the exception of `.text` and flash:
//             // `.text` must precede flash, as otherwise we'd be revoking execute
//             // permissions temporarily. Given that we can currently execute
//             // code, this should not have any impact on our accessible memory,
//             // assuming that the provided regions are not otherwise aliased.

//             // `.text` at n - 5 and n - 4 (TOR region):
//             write_pmpaddr_pmpcfg(
//                 0,
//                 (pmpcfg_octet::a::OFF()
//                     + pmpcfg_octet::r::CLEAR()
//                     + pmpcfg_octet::w::CLEAR()
//                     + pmpcfg_octet::x::CLEAR()
//                     + pmpcfg_octet::l::SET())
//                 .value(),
//                 (kernel_text.0.start() as usize) >> 2,
//             );
//             write_pmpaddr_pmpcfg(
//                 1,
//                 (pmpcfg_octet::a::TOR()
//                     + pmpcfg_octet::r::SET()
//                     + pmpcfg_octet::w::CLEAR()
//                     + pmpcfg_octet::x::SET()
//                     + pmpcfg_octet::l::SET())
//                 .value(),
//                 (kernel_text.0.end() as usize) >> 2,
//             );

//             // MMIO at n - 1:
//             write_pmpaddr_pmpcfg(
//                 AVAILABLE_ENTRIES - 1,
//                 (pmpcfg_octet::a::NAPOT()
//                     + pmpcfg_octet::r::SET()
//                     + pmpcfg_octet::w::SET()
//                     + pmpcfg_octet::x::CLEAR()
//                     + pmpcfg_octet::l::SET())
//                 .value(),
//                 mmio.0.napot_addr(),
//             );

//             // RAM at n - 2:
//             write_pmpaddr_pmpcfg(
//                 AVAILABLE_ENTRIES - 2,
//                 (pmpcfg_octet::a::NAPOT()
//                     + pmpcfg_octet::r::SET()
//                     + pmpcfg_octet::w::SET()
//                     + pmpcfg_octet::x::CLEAR()
//                     + pmpcfg_octet::l::SET())
//                 .value(),
//                 ram.0.napot_addr(),
//             );

//             // flash at n - 3:
//             write_pmpaddr_pmpcfg(
//                 AVAILABLE_ENTRIES - 3,
//                 (pmpcfg_octet::a::NAPOT()
//                     + pmpcfg_octet::r::SET()
//                     + pmpcfg_octet::w::CLEAR()
//                     + pmpcfg_octet::x::CLEAR()
//                     + pmpcfg_octet::l::SET())
//                 .value(),
//                 flash.0.napot_addr(),
//             );

//             // Finally, attempt to enable the MSECCFG security bits, and verify
//             // that they have been set correctly. If they have not been set to
//             // the written value, this means that this hardware either does not
//             // support ePMP, or it was in some invalid state otherwise. We don't
//             // need to read back the above regions, as we previous verified that
//             // none of their entries were locked -- so writing to them must work
//             // even without RLB set.
//             //
//             // Set RLB(2) = 0, MMWP(1) = 1, MML(0) = 1
//             csr::CSR.mseccfg.set(0x00000003);

//             // Read back the MSECCFG CSR to ensure that the machine's security
//             // configuration was set properly. If this fails, we have set up the
//             // PMP in a way that would give userspace access to kernel
//             // space. The caller of this method must appropriately handle this
//             // error condition by ensuring that the platform will never execute
//             // userspace code!
//             if csr::CSR.mseccfg.get() != 0x00000003 {
//                 return Err(());
//             }

//             // Setup complete
//             const DEFAULT_USER_PMPCFG_OCTET: Cell<TORUserPMPCFG> = Cell::new(TORUserPMPCFG::OFF());
//             Ok(KernelProtectionMMLEPMP {
//                 user_pmp_enabled: Cell::new(false),
//                 shadow_user_pmpcfgs: [DEFAULT_USER_PMPCFG_OCTET; MPU_REGIONS],
//             })
//         }
//     }

//     impl<const AVAILABLE_ENTRIES: usize, const MPU_REGIONS: usize> TORUserPMP<MPU_REGIONS>
//         for KernelProtectionMMLEPMP<AVAILABLE_ENTRIES, MPU_REGIONS>
//     {
//         // Ensure that the MPU_REGIONS (starting at entry, and occupying two
//         // entries per region) don't overflow the available entires, excluding
//         // the 7 entries used for implementing the kernel memory protection:
//         const CONST_ASSERT_CHECK: () = assert!(MPU_REGIONS <= ((AVAILABLE_ENTRIES - 5) / 2));

//         fn available_regions(&self) -> usize {
//             // Always assume to have `MPU_REGIONS` usable TOR regions. We don't
//             // support locking additional regions at runtime.
//             MPU_REGIONS
//         }

//         // This implementation is specific for 32-bit systems. We use
//         // `u32::from_be_bytes` and then cast to usize, as it manages to compile
//         // on 64-bit systems as well. However, this implementation will not work
//         // on RV64I systems, due to the changed pmpcfgX CSR layout.
//         #[flux_rs::sig(fn (&Self, &[PMPUserRegion; _], hw_state: &strg HardwareState) -> Result<(), ()>[#ok] ensures hw_state: HardwareState {hw:
//             ok => all_regions_configured_correctly_up_to(MPU_REGIONS, hw)
//         })]
//         #[flux_rs::trusted]
//         fn configure_pmp(
//             &self,
//             regions: &[PMPUserRegion; MPU_REGIONS],
//             hardware_state: &mut HardwareState,
//         ) -> Result<(), ()> {
//             // Configure all of the regions' addresses and store their pmpcfg octets
//             // in our shadow storage. If the user PMP is already enabled, we further
//             // apply this configuration (set the pmpcfgX CSRs) by running
//             // `enable_user_pmp`:
//             for (i, (region, shadow_user_pmpcfg)) in regions
//                 .iter()
//                 .zip(self.shadow_user_pmpcfgs.iter())
//                 .enumerate()
//             {
//                 // The ePMP in MML mode does not support read-write-execute
//                 // regions. If such a region is to be configured, abort. As this
//                 // loop here only modifies the shadow state, we can simply abort and
//                 // return an error. We don't make any promises about the ePMP state
//                 // if the configuration files, but it is still being activated with
//                 // `enable_user_pmp`:
//                 if region.tor.get()
//                     == permissions_to_pmpcfg(mpu::Permissions::ReadWriteExecute).get()
//                 {
//                     return Err(());
//                 }

//                 // Set the CSR addresses for this region (if its not OFF, in which
//                 // case the hardware-configured addresses are irrelevant):
//                 if region.tor != TORUserPMPCFG::OFF() {
//                     csr::CSR.pmpaddr_set(
//                         (i + Self::TOR_REGIONS_OFFSET) * 2 + 0,
//                         (region.start.unwrap_or(FluxPtr::null()).as_usize())
//                             .overflowing_shr(2)
//                             .0,
//                     );
//                     csr::CSR.pmpaddr_set(
//                         (i + Self::TOR_REGIONS_OFFSET) * 2 + 1,
//                         (region.end.unwrap_or(FluxPtr::null()).as_usize())
//                             .overflowing_shr(2)
//                             .0,
//                     );
//                 }

//                 // Store the region's pmpcfg octet:
//                 shadow_user_pmpcfg.set(region.tor);
//             }

//             // If the PMP is currently active, apply the changes to the CSRs:
//             if self.user_pmp_enabled.get() {
//                 self.enable_user_pmp()?;
//             }

//             Ok(())
//         }

//         fn enable_user_pmp(&self) -> Result<(), ()> {
//             // We store the "enabled" PMPCFG octets of user regions in the
//             // `shadow_user_pmpcfg` field, such that we can re-enable the PMP
//             // without a call to `configure_pmp` (where the `TORUserPMPCFG`s are
//             // provided by the caller).

//             // Could use `iter_array_chunks` once that's stable.
//             let mut shadow_user_pmpcfgs_iter = self.shadow_user_pmpcfgs.iter();
//             let mut i = Self::TOR_REGIONS_OFFSET;

//             while let Some(first_region_pmpcfg) = shadow_user_pmpcfgs_iter.next() {
//                 // If we're at a "region" offset divisible by two (where "region" =
//                 // 2 PMP "entries"), then we can configure an entire `pmpcfgX` CSR
//                 // in one operation. As CSR writes are expensive, this is an
//                 // operation worth making:
//                 let second_region_opt = if i % 2 == 0 {
//                     shadow_user_pmpcfgs_iter.next()
//                 } else {
//                     None
//                 };

//                 if let Some(second_region_pmpcfg) = second_region_opt {
//                     // We're at an even index and have two regions to configure, so
//                     // do that with a single CSR write:
//                     csr::CSR.pmpconfig_set(
//                         i / 2,
//                         u32::from_be_bytes([
//                             second_region_pmpcfg.get().get(),
//                             TORUserPMPCFG::OFF().get(),
//                             first_region_pmpcfg.get().get(),
//                             TORUserPMPCFG::OFF().get(),
//                         ]) as usize,
//                     );

//                     i += 2;
//                 } else if i % 2 == 0 {
//                     // This is a single region at an even index. Thus, modify the
//                     // first two pmpcfgX octets for this region.
//                     csr::CSR.pmpconfig_modify(
//                         i / 2,
//                         FieldValue::<usize, csr::pmpconfig::pmpcfg::Register>::new(
//                             0x0000FFFF,
//                             0, // lower two octets
//                             u32::from_be_bytes([
//                                 0,
//                                 0,
//                                 first_region_pmpcfg.get().get(),
//                                 TORUserPMPCFG::OFF().get(),
//                             ]) as usize,
//                         ),
//                     );

//                     i += 1;
//                 } else {
//                     // This is a single region at an odd index. Thus, modify the
//                     // latter two pmpcfgX octets for this region.
//                     csr::CSR.pmpconfig_modify(
//                         i / 2,
//                         FieldValue::<usize, csr::pmpconfig::pmpcfg::Register>::new(
//                             0x0000FFFF,
//                             16, // higher two octets
//                             u32::from_be_bytes([
//                                 0,
//                                 0,
//                                 first_region_pmpcfg.get().get(),
//                                 TORUserPMPCFG::OFF().get(),
//                             ]) as usize,
//                         ),
//                     );

//                     i += 1;
//                 }
//             }

//             self.user_pmp_enabled.set(true);

//             Ok(())
//         }

//         fn disable_user_pmp(&self) {
//             // Simply set all of the user-region pmpcfg octets to OFF:

//             let mut user_region_pmpcfg_octet_pairs =
//                 (Self::TOR_REGIONS_OFFSET)..(Self::TOR_REGIONS_OFFSET + MPU_REGIONS);
//             while let Some(first_region_idx) = user_region_pmpcfg_octet_pairs.next() {
//                 let second_region_opt = if first_region_idx % 2 == 0 {
//                     user_region_pmpcfg_octet_pairs.next()
//                 } else {
//                     None
//                 };

//                 if let Some(_second_region_idx) = second_region_opt {
//                     // We're at an even index and have two regions to configure, so
//                     // do that with a single CSR write:
//                     csr::CSR.pmpconfig_set(
//                         first_region_idx / 2,
//                         u32::from_be_bytes([
//                             TORUserPMPCFG::OFF().get(),
//                             TORUserPMPCFG::OFF().get(),
//                             TORUserPMPCFG::OFF().get(),
//                             TORUserPMPCFG::OFF().get(),
//                         ]) as usize,
//                     );
//                 } else if first_region_idx % 2 == 0 {
//                     // This is a single region at an even index. Thus, modify the
//                     // first two pmpcfgX octets for this region.
//                     csr::CSR.pmpconfig_modify(
//                         first_region_idx / 2,
//                         FieldValue::<usize, csr::pmpconfig::pmpcfg::Register>::new(
//                             0x0000FFFF,
//                             0, // lower two octets
//                             u32::from_be_bytes([
//                                 0,
//                                 0,
//                                 TORUserPMPCFG::OFF().get(),
//                                 TORUserPMPCFG::OFF().get(),
//                             ]) as usize,
//                         ),
//                     );
//                 } else {
//                     // This is a single region at an odd index. Thus, modify the
//                     // latter two pmpcfgX octets for this region.
//                     csr::CSR.pmpconfig_modify(
//                         first_region_idx / 2,
//                         FieldValue::<usize, csr::pmpconfig::pmpcfg::Register>::new(
//                             0x0000FFFF,
//                             16, // higher two octets
//                             u32::from_be_bytes([
//                                 0,
//                                 0,
//                                 TORUserPMPCFG::OFF().get(),
//                                 TORUserPMPCFG::OFF().get(),
//                             ]) as usize,
//                         ),
//                     );
//                 }
//             }

//             self.user_pmp_enabled.set(false);
//         }
//     }

//     impl<const AVAILABLE_ENTRIES: usize, const MPU_REGIONS: usize> fmt::Display
//         for KernelProtectionMMLEPMP<AVAILABLE_ENTRIES, MPU_REGIONS>
//     {
//         fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//             write!(
//                 f,
//                 " ePMP configuration:\r\n  mseccfg: {:#08X}, user-mode PMP active: {:?}, entries:\r\n",
//                 csr::CSR.mseccfg.get(),
//                 self.user_pmp_enabled.get()
//             )?;
//             unsafe { super::format_pmp_entries::<AVAILABLE_ENTRIES>(f) }?;

//             write!(f, "  Shadow PMP entries for user-mode:\r\n")?;
//             for (i, shadowed_pmpcfg) in self.shadow_user_pmpcfgs.iter().enumerate() {
//                 let (start_pmpaddr_label, startaddr_pmpaddr, endaddr, mode) =
//                     if shadowed_pmpcfg.get() == TORUserPMPCFG::OFF() {
//                         (
//                             "pmpaddr",
//                             csr::CSR.pmpaddr_get((i + Self::TOR_REGIONS_OFFSET) * 2),
//                             0,
//                             "OFF",
//                         )
//                     } else {
//                         (
//                             "  start",
//                             csr::CSR
//                                 .pmpaddr_get((i + Self::TOR_REGIONS_OFFSET) * 2)
//                                 .overflowing_shl(2)
//                                 .0,
//                             csr::CSR
//                                 .pmpaddr_get((i + Self::TOR_REGIONS_OFFSET) * 2 + 1)
//                                 .overflowing_shl(2)
//                                 .0
//                                 | 0b11,
//                             "TOR",
//                         )
//                     };

//                 write!(
//                     f,
//                     "  [{:02}]: {}={:#010X}, end={:#010X}, cfg={:#04X} ({}  ) ({}{}{}{})\r\n",
//                     (i + Self::TOR_REGIONS_OFFSET) * 2 + 1,
//                     start_pmpaddr_label,
//                     startaddr_pmpaddr,
//                     endaddr,
//                     shadowed_pmpcfg.get().get(),
//                     mode,
//                     if shadowed_pmpcfg.get().get_reg().is_set(pmpcfg_octet::l()) {
//                         "l"
//                     } else {
//                         "-"
//                     },
//                     if shadowed_pmpcfg.get().get_reg().is_set(pmpcfg_octet::r()) {
//                         "r"
//                     } else {
//                         "-"
//                     },
//                     if shadowed_pmpcfg.get().get_reg().is_set(pmpcfg_octet::w()) {
//                         "w"
//                     } else {
//                         "-"
//                     },
//                     if shadowed_pmpcfg.get().get_reg().is_set(pmpcfg_octet::x()) {
//                         "x"
//                     } else {
//                         "-"
//                     },
//                 )?;
//             }

//             Ok(())
//         }
//     }
// }
