#![allow(unused)]
// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Implementation of the memory protection unit for the Cortex-M0+, Cortex-M3,
//! Cortex-M4, and Cortex-M7

use core::cell::Cell;
use core::cmp;
use core::f32::MIN;
use core::fmt;
use core::num::NonZeroUsize;

use flux_support::register_bitfields;
use flux_support::*;
use crate::platform::mpu;
use crate::platform::mpu::AllocateAppMemoryError;
use crate::platform::mpu::AllocatedAppBreaks;
use crate::platform::mpu::Permissions;
use crate::utilities::cells::OptionalCell;
use crate::utilities::math;
use crate::utilities::StaticRef;
use crate::utilities::registers::{ReadWrite, ReadOnly};
use tock_registers::interfaces::{Readable, Writeable};

use super::MIN_REGION_SIZE;

flux_rs::defs! {
    fn xor(b: bitvec<32>, a: bitvec<32>) -> bitvec<32> { (a | b) - (a & b) }
    fn bv32(x:int) -> bitvec<32> { bv_int_to_bv32(x) }
    fn bit(reg: bitvec<32>, power_of_two: bitvec<32>) -> bool { reg & power_of_two != 0}
    fn extract(reg: bitvec<32>, mask:int, offset: int) -> bitvec<32> { (reg & bv32(mask)) >> bv32(offset) }

    // rbar
    fn rbar_global_region_enabled(reg: bitvec<32>) -> bool { bit(reg, 0x1) }
    fn rbar_valid_bit_set(reg: bitvec<32>) -> bool { bit(reg, 0x10) }
    fn rbar_region_number(reg: bitvec<32>) -> bitvec<32> { reg & 0xF }
    fn rbar_region_start(reg: bitvec<32>) -> bitvec<32> { reg & 0xFFFF_FFE0 }

    // rasr
    fn rasr_region_size(reg: bitvec<32>) -> bitvec<32> { 1 << (extract(reg, 0x0000003e, 1) + 1) }
    fn rasr_srd(reg: bitvec<32>) -> bitvec<32> { extract(reg, 0x0000_FF00, 8) }
    fn rasr_ap(reg: bitvec<32>) -> bitvec<32> { extract(reg, 0x0700_0000, 24) }
    fn rasr_xn(reg: bitvec<32>) -> bool { bit(reg, 0x10000000) }

    // ctrl
    fn enable(reg:bitvec<32>) -> bool { bit(reg, 0x00000001)}

    // fn mpu_configured_for(mpu: MPU, regions: RArray<CortexMRegion>, number_of_regions: int) -> bool {

    //     forall i in 0..8 {
    //         map_select(mpu.regions, i) == rbar(map_select(regions, i)) &&
    //         map_select(mpu.attrs, i) == rasr(map_select(regions, i))
    //     } 
    //     && number_of_regions == 16 => forall j in 8..16 {
    //         // basically these are all empty
    //         rbar_region_number(mpu.rbar) == bv32(j) &&
    //         !rbar_global_region_enabled(mpu.rasr) &&
    //         subregions_enabled_exactly(mpu.rasr, 0, 7)
    //     }
    // }

    fn enabled_srd_mask(first_subregion: bitvec<32>, last_subregion: bitvec<32>) -> bitvec<32> {
        ((bv32(1) << (last_subregion - first_subregion + 1)) - 1) << first_subregion 
    }

    fn disabled_srd_mask(first_subregion: bitvec<32>, last_subregion: bitvec<32>) -> bitvec<32> {
        xor(0xff, enabled_srd_mask(first_subregion, last_subregion))
    }

    fn perms_match_exactly(rasr: bitvec<32>, perms: mpu::Permissions) -> bool {
        let ap = rasr_ap(rasr);
        let xn = rasr_xn(rasr);
        if perms.r && perms.w && perms.x {
            // read write exec
            ap == 3 && xn
        } else if perms.r && perms.w && !perms.x {
            // read write
            ap == 3 && !xn
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

    fn subregions_enabled_exactly(rasr: bitvec<32>, first_subregion_no: bitvec<32>, last_subregion_no: bitvec<32>) -> bool {
        let emask = enabled_srd_mask(first_subregion_no, last_subregion_no);
        let dmask = disabled_srd_mask(first_subregion_no, last_subregion_no);
        let srd = rasr_srd(rasr);
        srd & emask == 0 && srd & dmask == dmask
    }

    fn first_subregion_from_logical(rstart: int, rsize: int, astart: int, asize: int) -> int {
        let subregion_size = rsize / 8;
        (astart - rstart) / subregion_size
    }

    fn last_subregion_from_logical(rstart: int, rsize: int, astart: int, asize: int) -> int {
        let subregion_size = rsize / 8;
        (astart + asize - rstart) / subregion_size - 1
    }

    fn can_access_exactly(rbar: FieldValueU32, rasr: FieldValueU32, rstart: int, rsize: int, astart: int, asize: int, perms: mpu::Permissions) -> bool {
        rbar_global_region_enabled(rbar.value) &&
        rbar_region_start(rbar.value) == bv32(rstart) &&
        rasr_region_size(rasr.value) == bv32(rsize) &&
        subregions_enabled_exactly(
            rasr.value, 
            bv32(first_subregion_from_logical(rstart, rsize, astart, asize)),
            bv32(last_subregion_from_logical(rstart, rsize, astart, asize))
        ) &&
        perms_match_exactly(rasr.value, perms)
    }
}

// VTOCK-TODO: supplementary proof?
#[flux_rs::sig(fn(n: u32{n < 32}) -> usize {r: r > 0 &&  r <= u32::MAX / 2 + 1})]
#[flux_rs::trusted]
fn power_of_two(n: u32) -> usize {
    1_usize << n
}

#[flux_rs::opaque]
#[flux_rs::refined_by(regions: Map<int, CortexMRegion>)]
struct RegionGhostState {}
impl RegionGhostState {
    #[flux_rs::trusted]
    const fn new() -> Self {
        Self {}
    }
}

#[flux_rs::opaque]
#[flux_rs::refined_by(regions: Map<int, bitvec<32>>, attrs: Map<int, bitvec<32>>)]
struct HwGhostState {}
impl HwGhostState {
    #[flux_rs::trusted]
    const fn new() -> Self {
        Self {}
    }
}

/// MPU Registers for the Cortex-M3, Cortex-M4 and Cortex-M7 families
/// Described in section 4.5 of
/// <http://infocenter.arm.com/help/topic/com.arm.doc.dui0553a/DUI0553A_cortex_m4_dgug.pdf>
#[repr(C)]
pub struct MpuRegisters {
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
    // #[flux_rs::sig(fn(self: &strg Self) ensures self: Self{mpu: mpu.ctrl & 0x00000001 == 0 })]
    pub(crate) unsafe fn clear_mpu(&mut self) {
        self.registers.ctrl.write(Control::ENABLE::CLEAR().into_inner());
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
    pub region_size: usize
}

// flux tracking the actual region size rather than
// the "logical region"
#[derive(Copy, Clone)]
#[flux_rs::opaque]
#[flux_rs::refined_by(region_no: int, astart: int, asize: int, rstart: int, rsize: int, perms: mpu::Permissions)]
struct GhostRegionState {}

impl GhostRegionState {
    // trusted intializer for ghost state stuff
    #[flux_rs::trusted]
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

    #[flux_rs::trusted]
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
    astart: int, // accessible start
    asize: int, // accessible size
    rstart: int,
    rsize: int,
    perms: mpu::Permissions
)]

pub(crate) struct CortexMRegion {
    #[field(Option<{l. CortexMLocation[l] | l.astart == astart && l.asize == asize && l.rstart == rstart && l.rsize == rsize }>[set])]
    location: Option<CortexMLocation>, // actually accessible start and size
    #[field({FieldValueU32<RegionBaseAddress::Register>[rbar] | rbar_region_number(rbar.value) == bv32(region_no) && rbar_valid_bit_set(rbar.value) })]
    base_address: FieldValueU32<RegionBaseAddress::Register>,
    #[field({FieldValueU32<RegionAttributes::Register>[rasr] | (set => can_access_exactly(rasr, rbar, rstart, rsize, astart, asize, perms)) && (!set => !rbar_global_region_enabled(rasr.value) && subregions_enabled_exactly(rasr.value, 0, 7))})]
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

#[flux_rs::trusted]
#[flux_rs::sig(fn (u8[@mask], usize[@i]) -> u8[bv_bv32_to_int(xor(bv32(mask), bv32(1) << bv32(i)))])]
fn xor_mask(mask: u8, i: usize) -> u8 {
    mask ^ (1 << i)
}

#[flux_rs::trusted]
fn next_aligned_power_of_two(po2_aligned_start: usize, min_size: usize) -> usize {
    if po2_aligned_start == 0 {
        return min_size.next_power_of_two();
    }
    
    // Find the largest power of 2 that divides start evenly
    let trailing_zeros = po2_aligned_start.trailing_zeros() as usize;
    let largest_pow2_divisor = 1usize << trailing_zeros;
    
    // Start with the minimum required size, rounded up to the next power of 2
    let min_power = min_size.next_power_of_two();
    
    // Find the smallest power of 2 that's >= min_power and a multiple of largest_pow2_divisor
    let multiplier = (min_power + largest_pow2_divisor - 1) / largest_pow2_divisor;
    largest_pow2_divisor * multiplier
}

impl CortexMRegion {

    pub(crate) fn create_bounded_region(
        region_number: usize,
        available_start: FluxPtrU8,
        available_size: usize,
        region_size: usize,
        permissions: mpu::Permissions
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
        // size = size.next_power_of_two();
        size = math::closest_power_of_two_usize(size);

        // region size must be aligned to start
        start += size - (start % size);

        // calculate subregions
        let subregion_size = size / 8;
        let num_subregions_enabled = region_size.div_ceil(subregion_size);
        let subregions_enabled_end = start + num_subregions_enabled * subregion_size;

        // make sure this fits within our available size
        if subregions_enabled_end > available_start.as_usize() + available_size {
            return None;
        }

        // create the region
        Some(CortexMRegion::new(
            start.as_fluxptr(),
            num_subregions_enabled * subregion_size,
            start.as_fluxptr(),
            size,
            region_number,
            Some((0, num_subregions_enabled - 1)),
            permissions
        ))
    }

    #[flux_rs::trusted]
    pub(crate) fn adjust_region_fixed_start(
        po2_aligned_start: FluxPtrU8,
        available_size: usize,
        region_size: usize, 
        region_number: usize, 
        permissions: mpu::Permissions
    ) -> Option<CortexMRegion> {
        let overflow_bound = (u32::MAX / 2 + 1) as usize;
        if region_size == 0 || region_size > overflow_bound || po2_aligned_start.as_usize() > overflow_bound {
            // cannot create such a region
            return None;
        }

        // get the smallest size >= region size which is a power of two and aligned to the start
        let min_region_size = flux_support::max_usize(256, region_size);
        let mut underlying_region_size = next_aligned_power_of_two(po2_aligned_start.as_usize(), min_region_size);

        if underlying_region_size > available_size {
            return None;
        }

        // calculate subreigons
        let subregion_size = underlying_region_size / 8;
        let num_subregions_enabled = region_size.div_ceil(subregion_size);
        let subregions_enabled_end = po2_aligned_start.as_usize() + num_subregions_enabled * subregion_size;


        // create the region
        Some(CortexMRegion::new(
            po2_aligned_start,
            num_subregions_enabled * subregion_size,
            po2_aligned_start,
            underlying_region_size,
            region_number,
            Some((0, num_subregions_enabled - 1)),
            permissions
        ))
    }

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
    )]
    pub(crate) fn create_exact_region(
        region_number: usize,
        mut start: FluxPtrU8,
        mut size: usize,
        permissions: mpu::Permissions
    ) -> Option<CortexMRegion> {
        // We can't allocate a size that isn't a power of 2 or a size that is < 32 since that will not fit the requirements for a subregion
        if !size.is_power_of_two() || size < 32 || size > (u32::MAX / 2 + 1) as usize {
            return None;
        }

        if start % size == 0 {
            // we can just create a region
            Some(CortexMRegion::new(
                start,
                size,
                start,
                size,
                region_number,
                None,
                permissions
            ))
        } else {
            // we need to use a region start that aligns to the region size
            // we can do this by aligning our region start to 256 
            // 256 is the minimum size we need for subregions, and any size greater than 256 
            // that is a power of two will divide the start evenly
            let underlying_region_start = start.wrapping_sub(start.as_usize() % 256).as_usize();

            // now let's find out total region size. This should be size * 2, size * 4, or size * 8
            // to be able to first the size requested into subregions. 
            // We find the size that's greater than 
            let (underlying_region_size, num_subregions) = if size * 2 >= 256 {
                // we can use 4 subregions to cover the size we want
                (size * 2, 4)
            } else if size * 4 >= 256 {
                // we can use 2 subregions to cover the size we want
                (size * 4, 2)
            } else {
                // we can use 1 subregion to cover the size we want
                (size * 8, 1)
            };

            if underlying_region_size > (u32::MAX / 2 + 1) as usize {
                return None
            }

            let subregion_size = underlying_region_size / 8;
            let underlying_region_start = start.as_usize().saturating_sub(start.as_usize() % subregion_size);
            assert!(underlying_region_start % underlying_region_size == 0);

            if underlying_region_start == 0 {
                // this is a pathological case so leaving it unhandled
                return None;
            } else {
                // we are good to go now. the start aligns to the total size and we know that the 
                // actual start we want will align at the subregion boundary
                let offset = start.as_usize() - underlying_region_start;
                let first_subregion = offset / subregion_size;
                let last_subregion = first_subregion + num_subregions - 1;
                Some(CortexMRegion::new(
                    start, 
                    size, 
                    underlying_region_start.as_fluxptr(),
                    underlying_region_size,
                    region_number,
                    Some((first_subregion, last_subregion)),
                    permissions
                ))
            }
        }
    }

    #[flux_rs::sig(
        fn (
            FluxPtrU8[@astart],
            usize[@asize],
            FluxPtrU8[@rstart], 
            usize[@rsize],
            usize[@no],
            Option<(usize,usize)>[@subregions], 
            mpu::Permissions[@perms]
        ) -> CortexMRegion {r: 
                r.astart == astart &&
                r.asize == asize &&
                r.region_no == no &&
                r.perms == perms &&
                r.set  
            }
        requires 
            // rsize % 8 == 0 && 
            rsize >= 32 &&
            (subregions => rsize >= 256) &&
            rsize <= u32::MAX / 2 + 1 
            // &&
            // rstart % rsize == 0
    )]
    #[flux_rs::trusted] // VTOCK todo: bitvector stuff
    fn new(
        logical_start: FluxPtrU8,
        logical_size: usize,
        region_start: FluxPtrU8,
        region_size: usize,
        region_num: usize,
        subregions: Option<(usize, usize)>,
        permissions: mpu::Permissions,
    ) -> CortexMRegion {
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

        // Base address register
        let base_address = RegionBaseAddress::ADDR().val((region_start.as_u32()) >> 5)
            + RegionBaseAddress::VALID::UseRBAR()
            + RegionBaseAddress::REGION().val(region_num as u32);

        // let size_value = math::log_base_two_u32_usize(region_size) - 1;
        let size_value = math::log_base_two(region_size as u32) - 1;

        // Attributes register
        let mut attributes = RegionAttributes::ENABLE::SET()
            + RegionAttributes::SIZE().val(size_value)
            + access
            + execute;

        // If using subregions, add a subregion mask. The mask is a 8-bit
        // bitfield where `0` indicates that the corresponding subregion is enabled.
        // To compute the mask, we start with all subregions disabled and enable
        // the ones in the inclusive range [min_subregion, max_subregion].
        if let Some((min_subregion, max_subregion)) = subregions {
            // let mask = (min_subregion..=max_subregion).fold(u8::MAX, |res, i| {
            //     // Enable subregions bit by bit (1 ^ 1 == 0)
            //     res ^ (1 << i)
            // });
            let mut mask= u8::MAX; 
            let mut i = min_subregion;
            while i <= max_subregion {
                mask = xor_mask(mask, i);
                i += 1;
            }
            attributes += RegionAttributes::SRD().val(mask as u32);
        }

        Self {
            location: Some(CortexMLocation {
                accessible_start: logical_start,
                accessible_size: logical_size,
                region_start,
                region_size
            }),
            base_address,
            attributes,
            ghost_region_state: GhostRegionState::set(
                logical_start,
                logical_size,
                region_start,
                region_size,
                region_num,
                permissions,
            ),
        }
    }

    #[flux_rs::sig(fn ({usize[@region_no] | region_no < 8}) -> Self {r: r.region_no == region_no && !r.set})]
    #[flux_rs::trusted] // VTOCK TODO: Bit vector
    pub(crate) fn empty(region_num: usize) -> CortexMRegion {
        CortexMRegion {
            location: None,
            base_address: RegionBaseAddress::VALID::UseRBAR()
                + RegionBaseAddress::REGION().val(region_num as u32),
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
            },
            _ => false
        }
    }

    pub(crate) fn accessible_start(&self) -> Option<FluxPtr> {
        Some(self.location?.accessible_start)
    }

    pub(crate) fn accessible_size(&self) -> Option<usize> {
        Some(self.location?.accessible_size)
    }

    pub(crate) fn region_size(&self) -> Option<usize> {
        Some(self.location?.region_size)
    }

}

impl<const NUM_REGIONS: usize> MPU<NUM_REGIONS> {

    
    // #[flux_rs::sig(fn(self: &strg Self) ensures self: Self{mpu: enable(mpu.ctrl)})]
    pub(crate) fn enable_app_mpu(&self) {
        // Enable the MPU, disable it during HardFault/NMI handlers, and allow
        // privileged code access to all unprotected memory.
        let bits = Control::ENABLE::SET() + Control::HFNMIENA::CLEAR() + Control::PRIVDEFENA::SET();
        self.registers.ctrl.write(
            bits.into_inner()
        );
    }

    // #[flux_rs::sig(fn(self: &strg Self) ensures self: Self{mpu: !enable(mpu.ctrl)})]
    pub(crate) fn disable_app_mpu(&self) {
        // The MPU is not enabled for privileged mode, so we don't have to do
        // anything
        self.registers.ctrl.write(Control::ENABLE::CLEAR().into_inner());
    }

    fn number_total_regions(&self) -> usize {
        self.registers.mpu_type.read(Type::DREGION().into_inner()) as usize
    }

    // #[flux_rs::sig(fn (self: &strg Self[@mpu], &RArray<CortexMRegion>[@regions]) ensures self: Self{c_mpu: mpu_configured_for(c_mpu, regions, NUM_REGIONS)})]
    #[flux_rs::trusted] // for now
    pub(crate) fn configure_mpu(&self, regions: &RArray<CortexMRegion>) {
        // If the hardware is already configured for this app and the app's MPU
        // configuration has not changed, then skip the hardware update.
        // if !self.hardware_is_configured_for.contains(&config.id()) || config.is_dirty() {
        // Set MPU regions
        for region in regions.iter() {
            self.registers.rbar.write(region.base_address().into_inner());
            self.registers.rasr.write(region.attributes().into_inner());
        }

        if NUM_REGIONS == 16 {
            for i in 8..16 {
                let region = CortexMRegion::empty(i);
                self.registers.rbar.write(region.base_address().into_inner());
                self.registers.rasr.write(region.attributes().into_inner());
            }
        }
    }
}

#[cfg(test)]
mod test_new {
    use super::CortexMRegion;
    use crate::platform::mpu::Permissions;
    use flux_support::FluxPtr;
    use super::*;

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

    fn subregions_from_logical(region_start: usize, region_size: usize, accessible_start: usize, accessible_size: usize) -> (usize, usize) {
        let subregion_size = region_size / 8;
        let first_subregion_no = (accessible_start - region_start) / subregion_size;
        let last_subregion_no = (accessible_start + accessible_size - region_start) / subregion_size - 1;
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

    fn test_region(region: CortexMRegion, region_start: usize, region_size: usize, accessible_start: usize, accessible_size: usize, region_number: usize, perms: Permissions) {
        // println!("start: {}, size: {}, number: {}, accessible_start: {}, accessible_size: {}, perms: {:?}", region_start, region_size, region_number, accessible_start, accessible_size, perms);
        region_number_set(region.base_address, region_number);
        global_region_enabled(region.attributes);
        region_start_set(region.base_address, region_start);
        region_size_set(region.attributes, region_size);
        let (fsr, lsr) = subregions_from_logical(region_start, region_size, accessible_start, accessible_size);
        srd_bits_set(region.attributes, fsr, lsr);
        perms_set(region.attributes, perms);
    }

    fn test_without_subregions(region_start: usize, region_size: usize, region_number: usize) {
        // all permissions
        for perm_i in 0..5 {
            let perms = usize_to_permissions(perm_i);
            let region = CortexMRegion::new(
                FluxPtr::from(region_start),
                region_size,
                FluxPtr::from(region_start),
                region_size,
                region_number,
                None,
                perms
            );
            test_region(region, region_start, region_size, region_start, region_size, region_number, perms);
        }
    }

    fn test_with_subregions(region_start: usize, region_size: usize, region_number: usize) {
        for subregion_start in 0..8 {
            for subregion_end in (subregion_start + 1)..8 {
                let subregions = Some((subregion_start, subregion_end - 1));
                let accessible_start = region_start + subregion_start * (region_size / 8);
                let accesible_size = (subregion_end - subregion_start) * (region_size / 8);
                // all permissions
                for perm_i in 0..5 {
                    let perms = usize_to_permissions(perm_i);
                    // regions
                    let region = CortexMRegion::new(
                        FluxPtr::from(accessible_start),
                        accesible_size,
                        FluxPtr::from(region_start),
                        region_size,
                        region_number,
                        subregions,
                        perms
                    );
                    test_region(region, region_start, region_size, accessible_start, accesible_size, region_number, perms);
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
