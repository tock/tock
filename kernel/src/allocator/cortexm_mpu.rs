#![allow(unused)]
// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Implementation of the memory protection unit for the Cortex-M0+, Cortex-M3,
//! Cortex-M4, and Cortex-M7


// TICK: 
// The trusted bits of code (importantly all the math theorems used in the proof of new)
// have been formally verified in Lean here: https://github.com/PLSysSec/vtock-lean

use core::cell::Cell;
use core::cmp;
use core::f32::MIN;
use core::fmt;
use core::num::NonZeroUsize;

use crate::platform::mpu;
use crate::platform::mpu::AllocateAppMemoryError;
use crate::platform::mpu::AllocatedAppBreaks;
use crate::platform::mpu::Permissions;
use crate::utilities::cells::OptionalCell;
use crate::utilities::math;
use crate::utilities::registers::{ReadOnly, ReadWrite};
use crate::utilities::StaticRef;
use flux_support::register_bitfields;
use flux_support::*;
use flux_support::capability::*;
use tock_registers::interfaces::{Readable, Writeable};

use super::AppBreaks;
use super::MIN_REGION_SIZE;

/* extern specs have to live here because the defs for these specs are defined here */

#[flux_rs::extern_spec]
impl usize {
    #[flux_rs::sig(fn (num: usize) -> usize{r: r >= num && pow2(r) && half_max(r)} requires half_max(num))]
    fn next_power_of_two(self) -> usize;

    #[flux_rs::sig(fn (num: usize) -> bool[pow2(num)])]
    fn is_power_of_two(self) -> bool;

    #[sig(fn(num: usize{num <= u32::MAX}) -> u32{r: r <= 32 && (num > 0 => r < 32) && aligned(num, to_pow2(r)) && (aligned(num, 2) => r > 0)})]
    fn trailing_zeros(self) -> u32;
}

/* a bunch of theorems and proof code */

#[flux_rs::reveal(aligned)]
#[flux_rs::sig(fn (usize[@x], usize[@y]) requires x > 0 && aligned(x, y) ensures x >= y)]
fn theorem_aligned_ge(_x: usize, _y: usize) {}

#[flux_rs::reveal(aligned)]
#[flux_rs::sig(fn (usize[@x], usize[@y]) requires x == 0 && y > 0 ensures aligned(x, y))]
fn theorem_aligned0(_x: usize, _y: usize) {}

#[flux_rs::reveal(to_pow2)]
#[flux_rs::sig(fn (x: usize) requires x > 0 && x < 32 ensures to_pow2(x) > 1)]
fn theorem_to_pow2_gt1(x: usize) {}

#[flux_rs::reveal(pow2, to_pow2)]
#[flux_rs::sig(fn (usize[@n]) requires n < 32 ensures pow2(to_pow2(n)))]
fn theorem_to_pow2_is_pow2(_n: usize) {}

#[flux_rs::trusted(reason = "math")]
#[flux_rs::sig(fn (usize[@x], usize[@y], usize[@z]) requires aligned(x, y) && z <= y && pow2(y) && pow2(z) ensures aligned(x, z))]
fn theorem_pow2_le_aligned(x: usize, y: usize, z: usize) {}

#[flux_rs::trusted(reason = "math")]
#[flux_rs::sig(fn (r:usize) requires pow2(r) && r >= 8 ensures octet(r))]
fn theorem_pow2_octet(_n: usize) {}

#[flux_rs::reveal(octet)]
#[flux_rs::sig(fn (r:usize) requires octet(r) ensures 8 * (r / 8) == r)]
fn theorem_div_octet(_n: usize) {}

#[flux_rs::trusted(reason = "math")]
#[flux_rs::sig(fn (x: usize, y: usize) requires y >= 32 && pow2(y) && aligned(x, y) ensures least_five_bits(bv32(x)) == 0)]
fn theorem_aligned_value_ge32_lowest_five_bits0(x: usize, y: usize) {}

#[flux_rs::reveal(octet, first_subregion_from_logical)]
#[flux_rs::sig(fn (rstart: FluxPtr, rsize: usize, astart: FluxPtr, asize: usize)
    requires rstart == astart && rsize == asize && rsize >= 32
    ensures first_subregion_from_logical(rstart, rsize, astart, asize) == 0
)]
fn theorem_first_subregion_0(rstart: FluxPtr, rsize: usize, astart: FluxPtr, asize: usize) {}

#[flux_rs::reveal(octet, last_subregion_from_logical)]
#[flux_rs::sig(fn (rstart: FluxPtr, rsize: usize, astart: FluxPtr, asize: usize)
    requires rstart == astart && rsize == asize && rsize >= 32 && octet(rsize)
    ensures last_subregion_from_logical(rstart, rsize, astart, asize) == 7
)]
fn theorem_last_subregion_7(rstart: FluxPtr, rsize: usize, astart: FluxPtr, asize: usize) {}

#[flux_rs::reveal(octet, subregions_disabled_bit_set)]
#[flux_rs::sig(fn (&FieldValueU32<RegionAttributes::Register>[@rasr])
    ensures
        enabled_srd_mask(0, 7) == 255 &&
        disabled_srd_mask(0, 7) == 0 &&
        subregions_disabled_bit_set(rasr.value, 0, 7)
)]
fn theorem_subregions_disabled_bit_set_0_7(
    attributes: &FieldValueU32<RegionAttributes::Register>,
) {}

#[flux_rs::sig(fn (&FieldValueU32<RegionAttributes::Register>[@rasr])
    requires
        subregions_enabled_bit_set(rasr.value, 0, 7)
    ensures
        enabled_srd_mask(0, 7) == 255 &&
        disabled_srd_mask(0, 7) == 0
)]
fn theorem_subregions_enabled_bit_set_0_7(
    attributes: &FieldValueU32<RegionAttributes::Register>,
) {}

/* our actual flux defs */

flux_rs::defs! {
    fn half_max(r: int) -> bool { r <= u32::MAX / 2 + 1}

    fn bv32(x:int) -> bitvec<32> { bv_int_to_bv32(x) }
    fn bit(reg: bitvec<32>, power_of_two: bitvec<32>) -> bool { reg & power_of_two != 0}
    fn extract(reg: bitvec<32>, mask:int, offset: int) -> bitvec<32> { (reg & bv32(mask)) >> bv32(offset) }

    fn least_five_bits(val: bitvec<32>) -> bitvec<32> { val & 0x1F }

    // rbar
    fn rbar_valid_bit_set(reg: bitvec<32>) -> bool { bit(reg, 0x10) }
    fn rbar_region_number(reg: bitvec<32>) -> bitvec<32> { reg & 0xF }
    // NOTE: don't shift by 5 because we need the last 5 bits as all 0
    fn rbar_region_start(reg: bitvec<32>) -> bitvec<32> { reg & 0xFFFF_FFE0 }

    // rasr
    fn rasr_global_region_enabled(reg: bitvec<32>) -> bool { bit(reg, 0x1) }
    fn exp2(n:bitvec<32>) -> bitvec<32> { (1 << n) }
    fn size_from_base2(base2_value:bitvec<32>) -> bitvec<32> { exp2 (base2_value + 1) }
    fn rasr_region_size(reg: bitvec<32>) -> bitvec<32> { size_from_base2(extract(reg, 0x0000003e, 1))}

    // fn rasr_region_size(reg: bitvec<32>) -> bitvec<32>      { 1 << (extract(reg, 0x0000003e, 1) + 1) }
    fn rasr_srd(reg: bitvec<32>) -> bitvec<32> { extract(reg, 0x0000_FF00, 8) }
    fn rasr_ap(reg: bitvec<32>) -> bitvec<32> { extract(reg, 0x0700_0000, 24) }
    fn rasr_xn(reg: bitvec<32>) -> bool { bit(reg, 0x10000000) }

    // ctrl
    fn enable(reg:bitvec<32>) -> bool { bit(reg, 0x00000001)}

    fn enabled_srd_mask(first_subregion: bitvec<32>, last_subregion: bitvec<32>) -> bitvec<32> {
        ((bv32(1) << ((last_subregion - first_subregion) + 1)) - 1) << first_subregion
    }

    fn disabled_srd_mask(first_subregion: bitvec<32>, last_subregion: bitvec<32>) -> bitvec<32> {
        bv_xor(0xff, enabled_srd_mask(first_subregion, last_subregion))
    }

    fn perms_match_exactly(rasr: bitvec<32>, perms: mpu::Permissions) -> bool {
        let ap = rasr_ap(rasr);
        let xn = rasr_xn(rasr);
        if perms.r && perms.w && perms.x {
            // read write exec
            ap == 3 && !xn
        } else if perms.r && perms.w && !perms.x {
            // read write
            ap == 3 && xn
        } else if perms.r && !perms.w && perms.x {
            // read exec
            (ap == 2 || ap == 6 || ap == 7) && !xn
        } else if perms.r && !perms.w && !perms.x {
            // read only
            (ap == 2 || ap == 6 || ap == 7) && xn
        } else if !perms.r && !perms.w && perms.x {
            (ap == 0 || ap == 1) && !xn
        } else {
            false
        }
    }

    #[hide]
    fn subregions_enabled_bit_set(rasr: bitvec<32>, first_subregion_no: bitvec<32>, last_subregion_no: bitvec<32>) -> bool {
        let emask = enabled_srd_mask(first_subregion_no, last_subregion_no);
        let srd = rasr_srd(rasr);
        ((srd & emask) == 0)
    }

    #[hide]
    fn subregions_disabled_bit_set(rasr: bitvec<32>, first_subregion_no: bitvec<32>, last_subregion_no: bitvec<32>) -> bool {
        let dmask = disabled_srd_mask(first_subregion_no, last_subregion_no);
        let srd = rasr_srd(rasr);
        ((srd & dmask) == dmask)
    }

    fn subregions_enabled_exactly(rasr: bitvec<32>, first_subregion_no: bitvec<32>, last_subregion_no: bitvec<32>) -> bool {
       subregions_enabled_bit_set(rasr, first_subregion_no, last_subregion_no) &&
       subregions_disabled_bit_set(rasr, first_subregion_no, last_subregion_no)
    }

    #[hide]
    fn to_pow2(n: int) -> int {
        let bv = bv32(n);
        bv_bv32_to_int(bv32(1) << bv)
    }

    #[hide]
    fn pow2(n:int) -> bool {
        let bv = bv32(n);
        n > 0 && (bv & (bv - 1)) == 0
    }

    #[hide]
    fn aligned(x: int, y: int) -> bool {
        x % y == 0
    }

    #[hide]
    fn octet(n: int) -> bool {
        n % 8 == 0
    }

    #[hide]
    fn first_subregion_from_logical(rstart: int, rsize: int, astart: int, asize: int) -> int {
        let subregion_size = rsize / 8;
        (astart - rstart) / subregion_size
    }

    #[hide]
    fn last_subregion_from_logical(rstart: int, rsize: int, astart: int, asize: int) -> int {
        let subregion_size = rsize / 8;
        (((astart + asize) - rstart) / subregion_size) - 1
    }

    fn rnum(region: CortexMRegion) -> int { region.region_no}
    fn rbar(region: CortexMRegion) -> bitvec<32>{ region.rbar.value }
    fn rasr(region: CortexMRegion) -> bitvec<32> { region.rasr.value }
}

/* bunch of code */

#[flux_rs::trusted(reason = "solver hanging")]
#[flux_rs::sig(fn (start: usize, size: usize) -> usize{r: r >= start && aligned(r, size)} requires size > 0 && start + size <= usize::MAX)]
fn align(start: usize, size: usize) -> usize {
    start + size - (start % size)
}

#[flux_rs::reveal(aligned)]
#[flux_rs::sig(fn (start: usize, size: usize) -> bool[aligned(start, size)] requires size > 0)]
fn is_aligned(start: usize, size: usize) -> bool {
    start % size == 0
}

// VTOCK-TODO: supplementary proof?
#[flux_rs::trusted(reason = "math support (bitwise arithmetic fact)")]
#[flux_rs::sig(fn(n: u32{n < 32}) -> usize{r: r == to_pow2(n) && r > 0 && r <= u32::MAX})]
fn power_of_two(n: u32) -> usize {
    1_usize << n
}

#[flux_rs::opaque]
#[flux_rs::refined_by(regions: Map<int, CortexMRegion>)]
struct RegionGhostState {}
impl RegionGhostState {
    #[flux_rs::trusted(reason = "ghost state")]
    const fn new() -> Self {
        Self {}
    }
}

#[flux_rs::opaque]
#[flux_rs::refined_by(regions: Map<int, bitvec<32>>, attrs: Map<int, bitvec<32>>)]
struct HwGhostState {}
impl HwGhostState {
    #[flux_rs::trusted(reason = "ghost state")]
    const fn new() -> Self {
        Self {}
    }
}

/// MPU Registers for the Cortex-M3, Cortex-M4 and Cortex-M7 families
/// Described in section 4.5 of
/// <http://infocenter.arm.com/help/topic/com.arm.doc.dui0553a/DUI0553A_cortex_m4_dgug.pdf>
#[repr(C)]
struct MpuRegisters {
    /// Indicates whether the MPU is present and, if so, how many regions it
    /// supports.
    pub mpu_type: ReadOnly<u32, Type::Register>,

    /// The control register:
    ///   * Enables the MPU (bit 0).
    ///   * Enables MPU in hard-fault, non-maskable interrupt (NMI).
    ///   * Enables the default memory map background region in privileged mode.
    pub ctrl: ReadWrite<u32, Control::Register>,

    /// Selects the region number (zero-indexed) referenced by the region base
    /// address and region attribute and size registers.
    pub rnr: ReadWrite<u32, RegionNumber::Register>,

    /// Defines the base address of the currently selected MPU region.
    pub rbar: ReadWrite<u32, RegionBaseAddress::Register>,

    /// Defines the region size and memory attributes of the selected MPU
    /// region. The bits are defined as in 4.5.5 of the Cortex-M4 user guide.
    pub rasr: ReadWrite<u32, RegionAttributes::Register>,
}

register_bitfields![u32,
    Type [
        /// The number of MPU instructions regions supported. Always reads 0.
        IREGION OFFSET(16) NUMBITS(8) [],
        /// The number of data regions supported. If this field reads-as-zero the
        /// processor does not implement an MPU
        DREGION OFFSET(8) NUMBITS(8) [],
        /// Indicates whether the processor support unified (0) or separate
        /// (1) instruction and data regions. Always reads 0 on the
        /// Cortex-M4.
        SEPARATE OFFSET(0) NUMBITS(1) []
    ],

    Control [
        /// Enables privileged software access to the default
        /// memory map
        PRIVDEFENA OFFSET(2) NUMBITS(1) [
            Enable = 0,
            Disable = 1
        ],
        /// Enables the operation of MPU during hard fault, NMI,
        /// and FAULTMASK handlers
        HFNMIENA OFFSET(1) NUMBITS(1) [
            Enable = 0,
            Disable = 1
        ],
        /// Enables the MPU
        ENABLE OFFSET(0) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ]
    ],

    RegionNumber [
        /// Region indicating the MPU region referenced by the MPU_RBAR and
        /// MPU_RASR registers. Range 0-7 corresponding to the MPU regions.FieldValue<
        REGION OFFSET(0) NUMBITS(8) []
    ],

    RegionBaseAddress [
        /// Base address of the currently selected MPU region.
        ADDR OFFSET(5) NUMBITS(27) [],
        /// MPU Region Number valid bit.
        VALID OFFSET(4) NUMBITS(1) [
            /// Use the base address specified in Region Number Register (RNR)
            UseRNR = 0,
            /// Use the value of the REGION field in this register (RBAR)
            UseRBAR = 1
        ],
        /// Specifies which MPU region to set if VALID is set to 1.
        REGION OFFSET(0) NUMBITS(4) []
    ],

    RegionAttributes [
        /// Enables instruction fetches/execute permission
        XN OFFSET(28) NUMBITS(1) [
            Enable = 0,
            Disable = 1
        ],
        /// Defines access permissions
        AP OFFSET(24) NUMBITS(3) [
            //                                 Privileged  Unprivileged
            //                                 Access      Access
            NoAccess = 0b000,               // --          --
            PrivilegedOnly = 0b001,         // RW          --
            UnprivilegedReadOnly = 0b010,   // RW          R-
            ReadWrite = 0b011,              // RW          RW
            Reserved = 0b100,               // undef       undef
            PrivilegedOnlyReadOnly = 0b101, // R-          --
            ReadOnly = 0b110,               // R-          R-
            ReadOnlyAlias = 0b111           // R-          R-
        ],
        /// Subregion disable bits
        SRD OFFSET(8) NUMBITS(8) [],
        /// Specifies the region size, being 2^(SIZE+1) (minimum 3)
        SIZE OFFSET(1) NUMBITS(5) [],
        /// Enables the region
        ENABLE OFFSET(0) NUMBITS(1) []
    ]
];

// const MPU_BASE_ADDRESS: StaticRef<MpuRegisters> =
//     unsafe { StaticRef::new(0xE000ED90 as *const MpuRegisters) };

/// State related to the real physical MPU.
///
/// There should only be one instantiation of this object as it represents
/// real hardware.
///
#[flux_rs::invariant(NUM_REGIONS == 8 || NUM_REGIONS == 16)]
pub struct MPU<const NUM_REGIONS: usize> {
    /// MMIO reference to MPU registers.
    registers: StaticRef<MpuRegisters>,
    /// Monotonically increasing counter for allocated regions, used
    /// to assign unique IDs to `CortexMConfig` instances.
    config_count: Cell<NonZeroUsize>,
    /// Optimization logic. This is used to indicate which application the MPU
    /// is currently configured for so that the MPU can skip updating when the
    /// kernel returns to the same app.
    hardware_is_configured_for: OptionalCell<NonZeroUsize>,
}

const MPU_BASE_ADDRESS: StaticRef<MpuRegisters> =
    unsafe { StaticRef::new(0xE000ED90 as *const MpuRegisters) };

impl<const NUM_REGIONS: usize> MPU<NUM_REGIONS> {
    pub const unsafe fn new() -> Self {
        assume(NUM_REGIONS == 8 || NUM_REGIONS == 16);
        Self {
            registers: MPU_BASE_ADDRESS,
            config_count: Cell::new(NonZeroUsize::MIN),
            hardware_is_configured_for: OptionalCell::empty(),
        }
    }

    // Function useful for boards where the bootloader sets up some
    // MPU configuration that conflicts with Tock's configuration:
    pub(crate) unsafe fn clear_mpu(&self) {
        self.registers
            .ctrl
            .write(Control::ENABLE::CLEAR().into_inner());
    }
}

impl fmt::Display for CortexMRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\r\n Cortex-M Region")?;
        if let Some(location) = self.location() {
            let access_bits = self.attributes().read(RegionAttributes::AP());
            let start = location.region_start.as_usize();
            write!(
                f,
                "\
                    \r\n  Region: [{:#010X}:{:#010X}], length: {} bytes; ({:#x})",
                start,
                start + location.region_size,
                location.region_size,
                // access_str,
                access_bits,
            )?;
            let subregion_bits = self.attributes().read(RegionAttributes::SRD());
            let subregion_size = location.region_size / 8;
            for j in 0..8 {
                write!(
                    f,
                    "\
                        \r\n    Sub-region {}: [{:#010X}:{:#010X}], {}",
                    j,
                    start + j * subregion_size,
                    start + (j + 1) * subregion_size,
                    if (subregion_bits >> j) & 1 == 0 {
                        "Enabled"
                    } else {
                        "Disabled"
                    },
                )?;
            }
        } else {
            write!(f, "\r\n  Region: Unused")?;
        }
        write!(f, "\r\n")
    }
}

#[derive(Copy, Clone)]
#[flux_rs::refined_by(astart: int, asize: int, rstart: int, rsize: int)]
struct CortexMLocation {
    #[field(FluxPtrU8[astart])]
    pub accessible_start: FluxPtrU8,
    #[field(usize[asize])]
    pub accessible_size: usize,
    #[field(FluxPtrU8[rstart])]
    pub region_start: FluxPtrU8,
    #[field(usize[rsize])]
    pub region_size: usize,
}

// flux tracking the actual region size rather than
// the "logical region"
#[derive(Copy, Clone)]
#[flux_rs::opaque]
#[flux_rs::refined_by(region_no: int, astart: int, asize: int, rstart: int, rsize: int, perms: mpu::Permissions)]
struct GhostRegionState {}

impl GhostRegionState {
    // trusted intializer for ghost state stuff
    #[flux_rs::trusted(reason = "ghost state")]
    #[flux_rs::sig(fn (
        FluxPtrU8[@astart],
        usize[@asize],
        FluxPtrU8[@rstart],
        usize[@rsize],
        usize[@region_num],
        mpu::Permissions[@perms]
    ) -> GhostRegionState[region_num, astart, asize, rstart, rsize, perms]
    )]
    fn set(
        logical_start: FluxPtrU8,
        logical_size: usize,
        region_start: FluxPtrU8,
        region_size: usize,
        region_num: usize,
        permissions: mpu::Permissions,
    ) -> Self {
        Self {}
    }

    #[flux_rs::trusted(reason = "ghost state")]
    #[flux_rs::sig(fn (
        usize[@region_num]
    ) -> GhostRegionState { r: r.region_no == region_num }
    )]
    fn unset(region_num: usize) -> Self {
        Self {}
    }
}

/// Struct storing configuration for a Cortex-M MPU region.
// if the region is set, the rbar bits encode the accessible start & region_num properly and the rasr bits encode the size and permissions properly
#[derive(Copy, Clone)]
#[flux_rs::refined_by(
    rbar: FieldValueU32,
    rasr: FieldValueU32,
    region_no: int,
    set: bool,
    astart: int, 
    asize: int, 
    rstart: int,
    rsize: int,
    perms: mpu::Permissions
)]
pub(crate) struct CortexMRegion {
    #[field(Option<{l. CortexMLocation[l] | l.astart == astart && l.asize == asize && l.rstart == rstart && l.rsize == rsize }>[set])]
    location: Option<CortexMLocation>, 
    #[field({FieldValueU32<RegionBaseAddress::Register>[rbar] | 
        rbar_region_number(rbar.value) == bv32(region_no) &&
        rbar_valid_bit_set(rbar.value)
    })]
    base_address: FieldValueU32<RegionBaseAddress::Register>,
    #[field({FieldValueU32<RegionAttributes::Register>[rasr] | 
        let first = first_subregion_from_logical(rstart, rsize, astart, asize);
        let last  = last_subregion_from_logical(rstart, rsize, astart, asize);
        let bv_first = bv32(first);
        let bv_last  = bv32(last);
        (set => 
            (
                rbar_region_start(rbar.value) == bv32(rstart) &&
                rasr_region_size(rasr.value) == bv32(rsize) &&
                subregions_enabled_bit_set(rasr.value, bv_first, bv_last) &&
                subregions_disabled_bit_set(rasr.value, bv_first, bv_last) &&
                perms_match_exactly(rasr.value, perms) 
            )
        ) &&
        (!set => 
            !rasr_global_region_enabled(rasr.value) &&
            subregions_enabled_exactly(rasr.value, 0, 7)
        )}
    )]
    attributes: FieldValueU32<RegionAttributes::Register>,
    #[field(GhostRegionState[region_no, astart, asize, rstart, rsize, perms])]
    ghost_region_state: GhostRegionState,
}

impl PartialEq<mpu::Region> for CortexMRegion {
    fn eq(&self, other: &mpu::Region) -> bool {
        self.location().map_or(
            false,
            |CortexMLocation {
                 accessible_start: addr,
                 accessible_size: size,
                 ..
             }| { addr == other.start_address() && size == other.size() },
        )
    }
}

#[flux_rs::trusted] // Bitwise arithmetic
#[flux_rs::sig(fn(num: u32) -> u32{r: (r < 32) && (num > 1 => r > 0) && (pow2(num) => (bv32(num) == exp2(bv32(r))))})]
fn log_base_two(num: u32) -> u32 {
    if num == 0 {
        0
    } else {
        31 - num.leading_zeros()
    }
}

#[flux_rs::trusted(reason = "math support (bitwise arithmetic fact)")]
// VTOCK Note: Realized this only works when enabled_mask is not 0 because
// 0xff ^ 0 == 1 but anything & 0 = 0. 
#[flux_rs::sig(fn ({usize[@fsr] | fsr <= lsr}, {usize[@lsr] | lsr < 8}) -> u8{r: 
    let mask = enabled_srd_mask(bv32(fsr), bv32(lsr));
    if mask == 0 {
        bv32(r) == mask
    } else {
        bv32(r) == bv_xor(0xff, mask)
    }
})]
fn subregion_mask(min_subregion: usize, max_subregion: usize) -> u8 {
    let enabled_mask = ((1 << (max_subregion - min_subregion + 1)) - 1) << min_subregion;
    if enabled_mask == 0 {
        enabled_mask
    } else {
        u8::MAX ^ enabled_mask
    }
}

#[flux_rs::trusted]
#[flux_rs::sig(fn (region_start: FluxPtrU8) -> u32{r: least_five_bits(bv32(region_start)) == 0 => bv32(r) << 5 == bv32(region_start)})]
fn region_start_rs32(region_start: FluxPtrU8) -> u32 {
    region_start.as_u32() >> 5
}

#[flux_rs::trusted(reason = "math support (valid usize to u32 cast)")]
#[flux_rs::sig(fn ({ usize[@n] | n <= u32::MAX }) -> u32[n])]
fn usize_to_u32(n: usize) -> u32 {
    n as u32
}

#[flux_rs::sig(
    fn (start: usize, min_size: usize) -> Option<usize{size: 
        size >= min_size && pow2(size) && aligned(start, size) && octet(size)
    }>
    requires 
        half_max(min_size) &&
        min_size >= 256
)]
// Should only be called with a start that aligns to a po2 
fn next_aligned_power_of_two(po2_aligned_start: usize, min_size: usize) -> Option<usize> {

    // if start is 0 everything aligns
    if po2_aligned_start == 0 {
        let size = min_size.next_power_of_two();
        theorem_aligned0(po2_aligned_start,size);
        theorem_pow2_octet(size);
        return Some(size);
    }

    if !is_aligned(po2_aligned_start, 2) {
        return None;
    }

    // Find the largest power of 2 that divides start 
    // VTOCK TODO: Should just be usize stuff
    assume(po2_aligned_start <= u32::MAX as usize);
    let mut trailing_zeros = po2_aligned_start.trailing_zeros() as usize;
    let largest_pow2_divisor = power_of_two(usize_to_u32(trailing_zeros));
    theorem_to_pow2_is_pow2(trailing_zeros);
    theorem_to_pow2_gt1(trailing_zeros);

    // all powers of two less than largest_pow2_divisors will align the start
    let min_power = min_size.next_power_of_two();
    if min_power <= largest_pow2_divisor && min_power >= 8 {
        theorem_pow2_octet(min_power);
        theorem_pow2_le_aligned(po2_aligned_start, largest_pow2_divisor, min_power);
        Some(min_power)
    } else {
        // in this case such a value doesn't exist
        None
    }
}

impl CortexMRegion {

    #[flux_rs::reveal(first_subregion_from_logical, last_subregion_from_logical)]
    #[flux_rs::sig(
        fn (
            region_number: usize,
            available_start: FluxPtrU8,
            available_size: usize,
            region_size: usize,
            perms: mpu::Permissions
        ) -> Option<{r. CortexMRegion[r] |
            r.set &&
            r.region_no == region_number &&
            r.perms == perms &&
            r.astart >= available_start &&
            r.astart == r.rstart &&
            r.astart + r.asize <= available_start + available_size &&
            r.asize >= region_size
        }>
        requires region_number < 16 
    )]
    pub(crate) fn create_bounded_region(
        region_number: usize,
        available_start: FluxPtrU8,
        available_size: usize,
        region_size: usize,
        permissions: mpu::Permissions,
    ) -> Option<CortexMRegion> {
        // creates a region with region_start and region_end = region_start + region_size within available start + available size

        let mut start = available_start.as_usize();
        let mut size = region_size;

        let overflow_bound = (u32::MAX / 2 + 1) as usize;
        if size == 0 || size > overflow_bound || start > overflow_bound {
            // cannot create such a region
            return None;
        }

        // size must be >= 256 and a power of two for subregions
        size = flux_support::max_usize(size, 256);
        size = size.next_power_of_two();

        // region size must be aligned to start
        start = align(start, size);

        theorem_pow2_octet(size);
        theorem_div_octet(size);

        // calculate subregions
        let subregion_size = size / 8;
        let num_subregions_enabled = region_size.div_ceil(subregion_size);

        let subregions_enabled_end = start + num_subregions_enabled * subregion_size;

        // make sure this fits within our available size
        if subregions_enabled_end > available_start.as_usize() + available_size {
            return None;
        }

        // create the region
        Some(CortexMRegion::new_with_srd(
            FluxPtr::from(start),
            num_subregions_enabled * subregion_size,
            FluxPtr::from(start),
            size,
            region_number,
            0,
            num_subregions_enabled - 1,
            permissions,
        ))
    }

    #[flux_rs::reveal(first_subregion_from_logical, last_subregion_from_logical)]
    #[flux_rs::sig(
        fn (
            po2_aligned_start: FluxPtrU8,
            available_size: usize,
            min_size: usize,
            region_number: usize,
            perms: mpu::Permissions
        ) -> Option<{r. CortexMRegion[r] |
            region_number < 16 &&
            r.set &&
            r.region_no == region_number &&
            r.perms == perms &&
            r.astart == po2_aligned_start &&
            r.rstart == po2_aligned_start &&
            r.astart + r.asize <= po2_aligned_start + available_size &&
            r.asize >= min_size
        }>
        requires region_number < 16
    )]
    pub(crate) fn update_region(
        region_start: FluxPtrU8,
        available_size: usize,
        region_size: usize,
        region_number: usize,
        permissions: mpu::Permissions,
    ) -> Option<CortexMRegion> {
        let overflow_bound = (u32::MAX / 2 + 1) as usize;
        if region_size == 0
            || region_size > overflow_bound
            || region_start.as_usize() > overflow_bound
        {
            // cannot create such a region
            return None;
        }

        // get the smallest size >= region size which is a power of two and aligned to the start
        let min_region_size = flux_support::max_usize(256, region_size);
        let mut underlying_region_size =
            next_aligned_power_of_two(region_start.as_usize(), min_region_size)?;

        if underlying_region_size > available_size
            || underlying_region_size > (u32::MAX / 2 + 1) as usize
        {
            return None;
        }

        theorem_div_octet(underlying_region_size);

        // calculate subreigons
        let subregion_size = underlying_region_size / 8;

        let num_subregions_enabled = region_size.div_ceil(subregion_size);

        let subregions_enabled_end =
            region_start.as_usize() + num_subregions_enabled * subregion_size;

        // create the region
        Some(CortexMRegion::new_with_srd(
            region_start,
            num_subregions_enabled * subregion_size,
            region_start,
            underlying_region_size,
            region_number,
            0, 
            num_subregions_enabled - 1,
            permissions,
        ))
    }

    #[flux_rs::reveal(first_subregion_from_logical, last_subregion_from_logical)]
    #[flux_rs::sig(
        fn (
            usize[@region_no],
            FluxPtrU8[@start],
            usize[@size],
            mpu::Permissions[@perms],
        ) -> Option<{r. CortexMRegion[r] | 
                r.set &&
                r.region_no == region_no &&
                r.perms == perms &&
                r.astart == start &&
                r.astart + r.asize == start + size
            }>
            requires region_no < 16
    )]
    pub(crate) fn create_exact_region(
        region_number: usize,
        start: FluxPtrU8,
        size: usize,
        permissions: mpu::Permissions,
    ) -> Option<CortexMRegion> {
        // We can't allocate a size that isn't a power of 2 or a size that is < 32 since that will not fit the requirements for a subregion
        if size > (u32::MAX / 2 + 1) as usize || !size.is_power_of_two() || size < 32 {
            return None;
        }

        if is_aligned(start.as_usize(), size) {
            // we can just create a region
            Some(CortexMRegion::new_no_srd(
                start,
                size,
                start,
                size,
                region_number,
                permissions,
            ))
        } else {
            let min_size = flux_support::max_usize(size, 256);
            let underlying_region_start = start.as_usize();
            // VTOCK: If the start passed is not even, we fail.
            // This is generally a sane thing to do because a start being odd means that 
            let underlying_region_size = next_aligned_power_of_two(start.as_usize(), min_size)?;

            // check overflows
            if underlying_region_size > (u32::MAX / 2 + 1) as usize {
                return None;
            }

            theorem_div_octet(underlying_region_size);

            // calculate subreigons
            let subregion_size = underlying_region_size / 8;

            // if the size isn't aligned to the subregion size we have a problem
            // since that means we cannot divide the requested size into an appropriate
            // number of subregions
            if !is_aligned(size, subregion_size) {
                return None;
            }
            theorem_aligned_ge(size, subregion_size);

            let num_subregions_enabled = size / subregion_size;
            assert(num_subregions_enabled <= 8);
            assert(num_subregions_enabled > 0);

            Some(CortexMRegion::new_with_srd(
                start,
                size,
                FluxPtr::from(underlying_region_start),
                underlying_region_size,
                region_number,
                0,
                num_subregions_enabled - 1,
                permissions,
            ))
        }
    }

    #[flux_rs::reveal(
        rbar_region_number,
        rbar_region_start, 
        rbar_valid_bit_set,
        least_five_bits
    )]
    #[flux_rs::sig(fn (region_start: FluxPtrU8, region_num: usize, region_size: usize) -> FieldValueU32<RegionBaseAddress::Register>{rbar:
        rbar_region_number(rbar.value) == bv32(region_num) &&
        rbar_region_start(rbar.value) == bv32(region_start) &&
        rbar_valid_bit_set(rbar.value) 
    }
    requires region_num < 16 && region_size >= 32 && pow2(region_size) && aligned(region_start, region_size)
    )]
    fn base_address_register(
        region_start: FluxPtrU8,
        region_num: usize,
        region_size: usize,
    ) -> FieldValueU32<RegionBaseAddress::Register> {
        theorem_aligned_value_ge32_lowest_five_bits0(region_start.as_usize(), region_size);
        let shifted_val = region_start_rs32(region_start);
        RegionBaseAddress::ADDR().val(shifted_val)
            + RegionBaseAddress::VALID::UseRBAR()
            + RegionBaseAddress::REGION().val(usize_to_u32(region_num))
    }

    #[flux_rs::reveal(
        subregions_enabled_bit_set,
        subregions_disabled_bit_set,
        octet,
        first_subregion_from_logical,
        last_subregion_from_logical
    )]
    #[flux_rs::sig(
        fn (region_size: usize, permissions: mpu::Permissions) -> FieldValueU32<RegionAttributes::Register>{rasr:
            rasr_global_region_enabled(rasr.value) &&
            rasr_region_size(rasr.value) == bv32(region_size) &&
            perms_match_exactly(rasr.value, permissions) && 
            subregions_enabled_exactly(rasr.value, 0, 7)
        }
        requires pow2(region_size) && octet(region_size) && 32 <= region_size && half_max(region_size) 
    )]
    fn attributes_register_no_srd(
        region_size: usize,
        permissions: mpu::Permissions,
    ) -> FieldValueU32<RegionAttributes::Register> {
        // Determine access and execute permissions
        let (access, execute) = match permissions {
            mpu::Permissions::ReadWriteExecute => (
                RegionAttributes::AP::ReadWrite(),
                RegionAttributes::XN::Enable(),
            ),
            mpu::Permissions::ReadWriteOnly => (
                RegionAttributes::AP::ReadWrite(),
                RegionAttributes::XN::Disable(),
            ),
            mpu::Permissions::ReadExecuteOnly => (
                RegionAttributes::AP::UnprivilegedReadOnly(),
                RegionAttributes::XN::Enable(),
            ),
            mpu::Permissions::ReadOnly => (
                RegionAttributes::AP::UnprivilegedReadOnly(),
                RegionAttributes::XN::Disable(),
            ),
            mpu::Permissions::ExecuteOnly => (
                RegionAttributes::AP::PrivilegedOnly(),
                RegionAttributes::XN::Enable(),
            ),
        };

        // let size_value = math::log_base_two_u32_usize(region_size) - 1;
        let region_size_u32 = usize_to_u32(region_size);
        let size_value = log_base_two(region_size_u32) - 1;

        // Attributes register
        RegionAttributes::ENABLE::SET()
            + RegionAttributes::SIZE().val(size_value)
            + access
            + execute
    }

    #[flux_rs::reveal(
        first_subregion_from_logical,
        last_subregion_from_logical,
        subregions_enabled_bit_set,
        subregions_disabled_bit_set
    )]
    #[flux_rs::sig(
        fn (
            FluxPtrU8[@astart],
            usize[@asize],
            FluxPtrU8[@rstart],
            usize[@rsize],
            usize[@rnum],
            usize[@fsr],
            usize[@lsr],
            mpu::Permissions[@perms]
        ) -> CortexMRegion {r:
                r.astart == astart &&
                r.asize == asize &&
                r.rstart == rstart &&
                r.rsize == rsize &&
                r.region_no == rnum &&
                r.perms == perms &&
                r.set
            }
        requires
            rnum < 16 &&
            rsize >= 32 &&
            rsize >= 256 &&
            pow2(rsize) &&
            aligned(rstart, rsize) &&
            fsr <= lsr &&
            lsr < 8 &&
            first_subregion_from_logical(rstart, rsize, astart, asize) == fsr &&
            last_subregion_from_logical(rstart, rsize, astart, asize) == lsr &&
            rsize <= u32::MAX / 2 + 1
    )]
    fn new_with_srd(
        logical_start: FluxPtrU8,
        logical_size: usize,
        region_start: FluxPtrU8,
        region_size: usize,
        region_num: usize,
        fsr: usize, 
        lsr: usize,
        permissions: mpu::Permissions,
    ) -> CortexMRegion {
        theorem_pow2_octet(region_size);
        // Base address register
        let base_address = Self::base_address_register(region_start, region_num, region_size);
        // Attributes register
        let mut attributes = Self::attributes_register_no_srd(region_size, permissions);

        let location = CortexMLocation {
            accessible_start: logical_start,
            accessible_size: logical_size,
            region_start,
            region_size,
        };

        let ghost_region_state = GhostRegionState::set(
            logical_start,
            logical_size,
            region_start,
            region_size,
            region_num,
            permissions,
        );

        // If using subregions, add a subregion mask. The mask is a 8-bit
        // bitfield where `0` indicates that the corresponding subregion is enabled.
        // To compute the mask, we start with all subregions disabled and enable
        // the ones in the inclusive range [min_subregion, max_subregion].
        let mask = subregion_mask(fsr, lsr);
        attributes += RegionAttributes::SRD().val(mask as u32);
        Self {
            location: Some(location),
            base_address,
            attributes,
            ghost_region_state,
        }
    }

    #[flux_rs::sig(
        fn (
            FluxPtrU8[@astart],
            usize[@asize],
            FluxPtrU8[@rstart],
            usize[@rsize],
            usize[@no],
            mpu::Permissions[@perms]
        ) -> CortexMRegion {r:
                r.astart == astart &&
                r.asize == asize &&
                r.rstart == rstart &&
                r.rsize == rsize &&
                r.region_no == no &&
                r.perms == perms &&
                r.set
            }
        requires
            no < 16 &&
            rsize == asize &&
            rstart == astart &&
            rsize >= 32 &&
            pow2(rsize) &&
            aligned(rstart, rsize) &&
            rsize <= u32::MAX / 2 + 1
    )]
    fn new_no_srd(
        logical_start: FluxPtrU8,
        logical_size: usize,
        region_start: FluxPtrU8,
        region_size: usize,
        region_num: usize,
        permissions: mpu::Permissions,
    ) -> CortexMRegion {
        theorem_pow2_octet(region_size);
        // Base address register
        let base_address = Self::base_address_register(region_start, region_num, region_size);
        // Attributes register
        let attributes = Self::attributes_register_no_srd(region_size, permissions);

        theorem_first_subregion_0(region_start, region_size, logical_start, logical_size);
        theorem_last_subregion_7(region_start, region_size, logical_start, logical_size);
        theorem_subregions_disabled_bit_set_0_7(&attributes);

        let location = CortexMLocation {
            accessible_start: logical_start,
            accessible_size: logical_size,
            region_start,
            region_size,
        };

        let ghost_region_state = GhostRegionState::set(
            logical_start,
            logical_size,
            region_start,
            region_size,
            region_num,
            permissions,
        );

        Self {
            location: Some(location),
            base_address,
            attributes,
            ghost_region_state,
        }
    }

    #[flux_rs::reveal(subregions_enabled_bit_set, subregions_disabled_bit_set)]
    #[flux_rs::sig(fn ({usize[@region_no] | region_no < 16}) -> Self {r: r.region_no == region_no && !r.set})]
    pub(crate) fn empty(region_num: usize) -> CortexMRegion {
        let clear = RegionAttributes::ENABLE::CLEAR();
        assert(clear.value() == 0);
        CortexMRegion {
            location: None,
            base_address: RegionBaseAddress::VALID::UseRBAR()
                + RegionBaseAddress::REGION().val(usize_to_u32(region_num)),
            attributes: RegionAttributes::ENABLE::CLEAR(),
            ghost_region_state: GhostRegionState::unset(region_num),
        }
    }

    #[flux_rs::sig(fn (&CortexMRegion[@addr, @attrs, @no, @set, @astart, @asize, @rstart, @rsize, @perms]) -> Option<{l. CortexMLocation[l] | l.astart == astart && l.asize == asize && l.rstart == rstart && l.rsize == rsize}>[set])]
    fn location(&self) -> Option<CortexMLocation> {
        self.location
    }

    #[flux_rs::sig(fn(&CortexMRegion[@addr, @attrs, @no, @set, @astart, @asize, @rstart, @rsize, @perms]) -> FieldValueU32<RegionBaseAddress::Register>[addr])]
    fn base_address(&self) -> FieldValueU32<RegionBaseAddress::Register> {
        self.base_address
    }

    #[flux_rs::sig(fn(&CortexMRegion[@addr, @attrs, @no, @set, @astart, @asize, @rstart, @rsize, @perms]) -> FieldValueU32<RegionAttributes::Register>[attrs])]
    fn attributes(&self) -> FieldValueU32<RegionAttributes::Register> {
        self.attributes
    }

    pub(crate) fn is_set(&self) -> bool {
        self.location.is_some()
    }

    #[flux_rs::sig(fn (&Self[@region1], &CortexMRegion[@region2]) -> bool[regions_overlap(region1, region2)])]
    pub(crate) fn region_overlaps(&self, other: &CortexMRegion) -> bool {
        match (self.location(), other.location()) {
            (Some(fst_region_loc), Some(snd_region_loc)) => {
                let fst_region_start = fst_region_loc.region_start.as_usize();
                let fst_region_end = fst_region_start + fst_region_loc.region_size;

                let snd_region_start = snd_region_loc.region_start.as_usize();
                let snd_region_end = snd_region_start + snd_region_loc.region_size;

                fst_region_start < snd_region_end && snd_region_start < fst_region_end
            }
            _ => false,
        }
    }

    #[flux_rs::sig(fn (&Self[@r]) -> Option<FluxPtr[r.astart]>[r.set])]
    pub(crate) fn accessible_start(&self) -> Option<FluxPtr> {
        match self.location() {
            Some(l) => Some(l.accessible_start),
            None => None
        }
    }

    #[flux_rs::sig(fn (&Self[@r]) -> Option<usize[r.asize]>[r.set])]
    pub(crate) fn accessible_size(&self) -> Option<usize> {
        match self.location() {
            Some(l) => Some(l.accessible_size),
            None => None
        }
    }

    #[flux_rs::sig(fn (&Self[@r]) -> Option<usize[r.rsize]>[r.set])]
    pub(crate) fn region_size(&self) -> Option<usize> {
        match self.location() {
            Some(l) => Some(l.region_size),
            None => None
        }
    }
}

impl<const NUM_REGIONS: usize> MPU<NUM_REGIONS> {

    #[flux_rs::sig(fn () -> FieldValueU32<Control::Register>{ctrl: enable(ctrl.value)})]
    fn enable_mpu_ctrl_bits() -> FieldValueU32<Control::Register> {
        Control::ENABLE::SET() + Control::HFNMIENA::CLEAR() + Control::PRIVDEFENA::SET()
    }

    #[flux_rs::sig(fn () -> FieldValueU32<Control::Register>{ctrl: !enable(ctrl.value)})]
    fn disable_mpu_ctrl_bits() -> FieldValueU32<Control::Register> {
        Control::ENABLE::CLEAR()
    }

    pub(crate) fn enable_app_mpu(&self) -> MpuEnabledCapability {
        // Enable the MPU, disable it during HardFault/NMI handlers, and allow
        // privileged code access to all unprotected memory.
        let bits = Self::enable_mpu_ctrl_bits();
        self.registers.ctrl.write(bits.into_inner());
        MpuEnabledCapability {}
    }

    pub(crate) fn disable_app_mpu(&self) {
        // The MPU is not enabled for privileged mode, so we don't have to do
        // anything
        let bits = Self::disable_mpu_ctrl_bits();
        self.registers
            .ctrl
            .write(bits.into_inner());
    }

    fn number_total_regions(&self) -> usize {
        self.registers.mpu_type.read(Type::DREGION().into_inner()) as usize
    }

    #[flux_rs::sig(fn (&Self[@mpu], { &RArray<CortexMRegion>[@regions] | app_regions_correct(regions, breaks) }, &AppBreaks[@breaks]))]
    #[flux_rs::trusted(reason = "needs specs for Iterator and Range")]
    pub(crate) fn configure_mpu(&self, regions: &RArray<CortexMRegion>, _ghost_breaks: &AppBreaks) {
        // If the hardware is already configured for this app and the app's MPU
        // configuration has not changed, then skip the hardware update.
        // if !self.hardware_is_configured_for.contains(&config.id()) || config.is_dirty() {
        // Set MPU regions
        for region in regions.iter() {
            self.registers
                .rbar
                .write(region.base_address().into_inner());
            self.registers.rasr.write(region.attributes().into_inner());
        }

        if NUM_REGIONS == 16 {
            for i in 8..16 {
                let region = CortexMRegion::empty(i);
                self.registers
                    .rbar
                    .write(region.base_address().into_inner());
                self.registers.rasr.write(region.attributes().into_inner());
            }
        }
    }
}

#[cfg(test)]
mod test_new {
    use super::CortexMRegion;
    use super::*;
    use crate::platform::mpu::Permissions;
    use flux_support::FluxPtr;

    fn usize_to_permissions(i: usize) -> Permissions {
        if i == 0 {
            Permissions::ReadWriteExecute
        } else if i == 1 {
            Permissions::ReadWriteOnly
        } else if i == 2 {
            Permissions::ReadExecuteOnly
        } else if i == 3 {
            Permissions::ReadOnly
        } else if i == 4 {
            Permissions::ExecuteOnly
        } else {
            panic!("Invalid Enum Variant")
        }
    }

    fn perms_set(rasr: FieldValueU32<RegionAttributes::Register>, perms: Permissions) {
        let ap = (rasr.value() & 0x07000000) >> 24;
        let xn = rasr.value() & 0x10000000 != 0;
        // All access should be unpriv and priv
        //
        // 001	Read/Write	No access	Privileged access only
        // 010	Read/Write	Read-only	Any unprivileged write generates a permission fault
        // 011	Read/Write	Read/Write	Full access
        // 100	unpredictable	unpredictable	Reserved
        // 101	Read-only	No access	Privileged read-only
        // 110	Read-only	Read-only	Privileged and unprivileged read-only
        // 111	Read-only	Read-only	Privileged and unprivileged read-only

        match perms {
            Permissions::ReadWriteExecute => {
                assert!(ap == 3);
                assert!(!xn);
            }
            Permissions::ReadWriteOnly => {
                assert!(ap == 3);
                assert!(xn);
            }
            Permissions::ReadExecuteOnly => {
                assert!(ap == 6 || ap == 7 || ap == 2);
                assert!(!xn);
            }
            Permissions::ReadOnly => {
                assert!(ap == 6 || ap == 7 || ap == 2);
                assert!(xn);
            }
            Permissions::ExecuteOnly => {
                // ap of 1 gives privileged read access the ok which I guess is fine
                // originally didn't have it but their implementation does set this,
                // presumably if the kernel needs to read something?
                assert!(ap == 0 || ap == 1);
                assert!(!xn);
            }
        }
    }

    fn subregions_from_logical(
        region_start: usize,
        region_size: usize,
        accessible_start: usize,
        accessible_size: usize,
    ) -> (usize, usize) {
        let subregion_size = region_size / 8;
        let first_subregion_no = (accessible_start - region_start) / subregion_size;
        let last_subregion_no =
            (accessible_start + accessible_size - region_start) / subregion_size - 1;
        (first_subregion_no, last_subregion_no)
    }

    fn enabled_srd_mask(first_subregion: usize, last_subregion: usize) -> usize {
        ((1 << (last_subregion - first_subregion + 1)) - 1) << first_subregion
    }

    fn disabled_srd_mask(first_subregion: usize, last_subregion: usize) -> usize {
        0xff ^ enabled_srd_mask(first_subregion, last_subregion)
    }

    fn srd_bits_set(rasr: FieldValueU32<RegionAttributes::Register>, fsr: usize, lsr: usize) {
        let enabled_mask = enabled_srd_mask(fsr, lsr) as u32;
        let disabled_mask = disabled_srd_mask(fsr, lsr) as u32;
        let srd_bits = (rasr.value() & 0x0000FF00) >> 8;
        assert!(srd_bits & enabled_mask == 0);
        assert!(srd_bits & disabled_mask == disabled_mask);
    }

    // masks out the first 3 bits of the rbar register
    fn region_number_set(rbar: FieldValueU32<RegionBaseAddress::Register>, region_number: usize) {
        assert!(rbar.value() & 0xF == region_number as u32)
    }

    fn global_region_enabled(rasr: FieldValueU32<RegionAttributes::Register>) {
        assert!(rasr.value() & 0x1 == 1)
    }

    fn region_start_set(rbar: FieldValueU32<RegionBaseAddress::Register>, region_start: usize) {
        assert!(rbar.value() & 0xFFFF_FFE0 == region_start as u32)
    }

    fn region_size_set(rasr: FieldValueU32<RegionAttributes::Register>, region_size: usize) {
        assert!((1 << ((rasr.value() & 0x0000003e) >> 1) + 1) == region_size as u32);
    }

    fn rbar_valid_set(rbar: FieldValueU32<RegionBaseAddress::Register>) {
        assert!(rbar.value() & 0x10 != 0);
    }

    fn test_region(
        region: CortexMRegion,
        region_start: usize,
        region_size: usize,
        accessible_start: usize,
        accessible_size: usize,
        region_number: usize,
        perms: Permissions,
    ) {
        // println!("start: {}, size: {}, number: {}, accessible_start: {}, accessible_size: {}, perms: {:?}", region_start, region_size, region_number, accessible_start, accessible_size, perms);
        region_number_set(region.base_address, region_number);
        global_region_enabled(region.attributes);
        region_start_set(region.base_address, region_start);
        region_size_set(region.attributes, region_size);
        let (fsr, lsr) =
            subregions_from_logical(region_start, region_size, accessible_start, accessible_size);
        srd_bits_set(region.attributes, fsr, lsr);
        perms_set(region.attributes, perms);
    }

    fn test_without_subregions(region_start: usize, region_size: usize, region_number: usize) {
        // all permissions
        for perm_i in 0..5 {
            let perms = usize_to_permissions(perm_i);
            let region = CortexMRegion::new_no_srd(
                FluxPtr::from(region_start),
                region_size,
                FluxPtr::from(region_start),
                region_size,
                region_number,
                perms,
            );
            test_region(
                region,
                region_start,
                region_size,
                region_start,
                region_size,
                region_number,
                perms,
            );
        }
    }

    fn test_with_subregions(region_start: usize, region_size: usize, region_number: usize) {
        for subregion_start in 0..8 {
            for subregion_end in (subregion_start + 1)..8 {
                let accessible_start = region_start + subregion_start * (region_size / 8);
                let accesible_size = (subregion_end - subregion_start) * (region_size / 8);
                // all permissions
                for perm_i in 0..5 {
                    let perms = usize_to_permissions(perm_i);
                    // regions
                    let region = CortexMRegion::new_with_srd(
                        FluxPtr::from(accessible_start),
                        accesible_size,
                        FluxPtr::from(region_start),
                        region_size,
                        region_number,
                        subregion_start, 
                        subregion_end - 1,
                        perms,
                    );
                    test_region(
                        region,
                        region_start,
                        region_size,
                        accessible_start,
                        accesible_size,
                        region_number,
                        perms,
                    );
                }
            }
        }
    }

    #[test]
    fn test_empty_exhaustive() {
        for region_num in 0..16 {
            let region = CortexMRegion::empty(region_num);
            let rbar = region.base_address();
            let rasr = region.attributes();
            region_number_set(rbar, region_num);
            rbar_valid_set(rbar);
            srd_bits_set(rasr, 0, 7);
        }
    }

    #[test]
    fn test_region_new_exhaustive() {
        // Region Size:
        // the region size is a power of two
        // the minimum region size possible is 32
        // if the region size is >= 256 then we can have subregions

        // Region Start:
        // the region start can be whatever as long as region_start + region_size <= u32::MAX
        // and it is aligned with the size
        // This should be a precondition

        // Accessible Start & Accessible Size aren't used.

        // Subregions must satisfy start <= end <= 8
        // TODO: Make sure this is the case when calls are made.

        // permissions: Can be any enum variants
        let mut region_size_po2 = 5;
        while region_size_po2 <= 32 {
            let region_size = 2_usize.pow(region_size_po2);
            for mut region_start in 0..((u32::MAX / 2 + 1) as usize) {
                if region_start % region_size != 0 {
                    region_start += region_size - (region_start % region_size);
                }

                if region_start as u32 > u32::MAX - region_size as u32 {
                    continue;
                };

                // 8 regions only
                for region_number in 0..16 {
                    if region_size >= 256 {
                        // subregions
                        test_with_subregions(region_start as usize, region_size, region_number);
                    }
                    // 16 regions
                    test_without_subregions(region_start as usize, region_size, region_number);
                }
            }
            region_size_po2 += 1;
        }
    }
}
