#![allow(unused)]
// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Implementation of the memory protection unit for the Cortex-M0+, Cortex-M3,
//! Cortex-M4, and Cortex-M7

use core::cell::Cell;
use core::cmp;
use core::fmt;
use core::num::NonZeroUsize;

use flux_support::register_bitfields;
use flux_support::*;
use kernel::platform::mpu;
use kernel::platform::mpu::AllocateAppMemoryError;
use kernel::platform::mpu::AllocatedAppBreaks;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::math;

// VTOCK-TODO: NUM_REGIONS currently fixed to 8. Need to also handle 16
flux_rs::defs! {
    fn bv32(x:int) -> bitvec<32> { bv_int_to_bv32(x) }
    fn bit(reg: bitvec<32>, power_of_two: bitvec<32>) -> bool { reg & power_of_two != 0}
    fn extract(reg: bitvec<32>, mask:int, offset: int) -> bitvec<32> { reg & (bv32(mask) << bv32(offset)) }

    // TODO: auto-generate field definitions somehow
    // TODO: make more type safe with aliases
    // TODO: well-formedness predicates
    // CTRL
    fn enable(reg:bitvec<32>) -> bool { bit(reg, 0x00000001)}
    fn hfnmiena(reg:bitvec<32>) -> bool { bit(reg, 0x00000002)}
    fn privdefena(reg:bitvec<32>) -> bool { bit(reg, 0x00000004)}
    // RNR
    fn num(reg:bitvec<32>) -> bitvec<32> { extract(reg, 0x000000ff, 0) }
    // Rbar
    fn valid(reg:bitvec<32>) -> bool { bit(reg, 0x00000010)}
    fn region(reg:bitvec<32>) -> bitvec<32> { extract(reg, 0x0000000f, 0)}
    fn addr(reg:bitvec<32>) -> bitvec<32> {  extract(reg, 0xffffffe0, 5)}
    // Rasr
    fn xn(reg:bitvec<32>) -> bool { bit(reg, 0x08000000)}
    fn region_enable(reg:bitvec<32>) -> bool { bit(reg, 0x00000001)}
    fn ap(reg:bitvec<32>) -> bitvec<32> { extract(reg, 0x07000000, 24) }
    fn srd(reg:bitvec<32>) -> bitvec<32> { extract(reg, 0x0000ff00, 8) }
    fn size(reg:bitvec<32>) -> bitvec<32> { bv32(1) << (extract(reg, 0x0000003e, 1) + 1) }

    fn value(fv: FieldValueU32) -> bitvec<32> { fv.value}
    fn rbar(region: CortexMRegion) -> bitvec<32> { value(region.rbar) }
    fn rasr(region: CortexMRegion) -> bitvec<32> { value(region.rasr) }


    fn map_set<T>(m: Map<int, T>, k: int, v: T) -> Map<int, T> { map_store(m, k, v) }
    fn map_get<T>(m: Map<int, T>, k:int) -> T { map_select(m, k) }
    fn map_def(v: bitvec<32>) -> Map<int, bitvec<32>> { map_default(v) }

    fn mpu_configured_for(mpu: MPU, config: CortexMConfig) -> bool {
        forall i in 0..8 {
            map_get(mpu.regions, i) == rbar(map_get(config.regions, i)) &&
            map_get(mpu.attrs, i) == rasr(map_get(config.regions, i))
        }
    }

    // https://developer.arm.com/documentation/dui0552/a/cortex-m3-peripherals/optional-memory-protection-unit/mpu-access-permission-attributes?lang=en
    fn user_can_read(rasr: bitvec<32>) -> bool {
        ap(rasr) == 2 ||
        ap(rasr) == 3 ||
        ap(rasr) == 6 ||
        ap(rasr) == 7
    }

    // https://developer.arm.com/documentation/dui0552/a/cortex-m3-peripherals/optional-memory-protection-unit/mpu-access-permission-attributes?lang=en
    fn user_can_write(rasr: bitvec<32>) -> bool {
        ap(rasr) == 3
    }

    /*

    // Need to verify non-overlapping? or implement last?
    // desugar into 28 line predicate?
    // TODO: verify whole thing, not little toy one
    // Idea: safely overapproximate -- every region that can service an address must satisfy the rules
    // -- is this actually sound?

    forall region in self.regions. last(|r|
        r.can_service(addr, size))) ==>
        user_access_succeeds(region.rbar, region.rasr, perms) // Done
        addr.aligned_to(arch.alignment) &&
        addr.aligned_to(size))
    */

    // VR: STARTING FROM SCRATCH

    fn region_no(r: CortexMRegion) -> int {
        r.region_no
    }

    fn astart(r: CortexMRegion) -> int {
        r.astart
    }

    fn enabled_srd_mask(first_subregion_no: bitvec<32>, last_subregion_no: bitvec<32>) -> bitvec<32> {
        (bv32(1) << (last_subregion_no - first_subregion_no + 1)) - 1
    }

    fn disabled_srd_mask(last_subregion_no: bitvec<32>) -> bitvec<32> {
        ((bv32(1) << (8 - last_subregion_no)) - 1) << last_subregion_no
    }

    fn perms_match_exactly(rasr: bitvec<32>, perms: mpu::Permissions) -> bool {
        perms.r == user_can_read(rasr) &&
        perms.w == user_can_write(rasr) &&
        perms.x == !xn(rasr)
    }

    fn subregions_enabled_exactly(rasr: bitvec<32>, first_subregion_no: bitvec<32>, last_subregion_no: bitvec<32>) -> bool {
        // Min size = 256
        size(rasr) >= 256 &&
        // Check bits first_subregion..=end_subregion are 0
        srd(rasr) & enabled_srd_mask(first_subregion_no, last_subregion_no) == 0 &&
        // Check bits last_subregion_end..=7 are 1 if there are any bits left
        last_subregion_no < 7 => srd(rasr) & disabled_srd_mask(last_subregion_no) == disabled_srd_mask(last_subregion_no)
    }

    fn first_subregion(rbar: FieldValueU32, rasr: FieldValueU32, astart: int) -> bitvec<32> {
        (bv32(astart) - addr(value(rbar))) / (size(value(rasr)) / 8)
    }

    fn last_subregion(rbar: FieldValueU32, rasr: FieldValueU32, astart: int, asize: int) -> bitvec<32> {
        first_subregion(rbar, rasr, astart) + (bv32(asize) / (size(value(rasr)) / 8))
    }

    fn can_access_exactly(rbar: FieldValueU32, rasr: FieldValueU32, astart: int, asize: int, perms: mpu::Permissions) -> bool {
        // accessible regions is bounded by the physical region
        addr(value(rbar)) <= bv32(astart) &&
        addr(value(rbar)) + size(value(rasr)) >= bv32(astart + asize) &&
        // the global region is enabled
        region_enable(value(rasr)) &&
        // the permissions match
        perms_match_exactly(value(rasr), perms) &&
        // and the subregions are set correctly
        subregions_enabled_exactly(value(rasr), first_subregion(rbar, rasr, astart), last_subregion(rbar, rasr, astart, asize))
    }

    fn region_can_access(region: CortexMRegion, start: int, end: int, perms: mpu::Permissions) -> bool {
        // region set
        region.set &&
        // region's accesible block contains the start..end (exclusive) checked
        start >= region.astart &&
        end <= region.astart + region.asize &&
        // and perms are correct
        region.perms == perms
    }

    fn region_cant_access_at_all(region: CortexMRegion, start: int, end: int) -> bool {
        // WHY is this different than !region_can_access:
        //  1. We don't want to talk about permissions at all here - it shouldn't be allocated at all
        //  2. region_can_access talks about everything from start..(start + size) being
        //  included in one region. However, here we want to say that there is no subslice of
        //  start..(start + size) that is accessible via the current region we are looking at
        !region.set ||
        // NO slice of start..(start + size) is included in the region
        // i.e. the start..end is entirely before the region start
        end < region.astart ||
        // or the start is entirely after region_start + region_size
        start > region.astart + region.asize
    }

    fn config_can_access_flash(config: CortexMConfig, fstart: int, fend: int) -> bool {
        // checks the flash is accessible with read and execute perms
        region_can_access(map_get(config, 2), fstart, fend, mpu::Permissions {r: true, w: false, x: true})
    }

    fn can_access_heap_split(region0: CortexMRegion, region1: CortexMRegion, hstart: int, hend: int) -> bool {
        region0.astart + region0.asize == region1.astart &&
        hstart >= region0.astart &&
        hend <= region1.astart + region1.asize &&
        region0.perms == mpu::Permissions {r: true, w: true, x: false} &&
        region1.perms == mpu::Permissions {r: true, w: true, x: false}
    }

    fn config_can_access_heap(config: CortexMConfig, hstart: int, hend: int) -> bool {
        // checks the heap is accessible with read and write perms
        // either you can access it through 0
        region_can_access(map_get(config, 0), hstart, hend, mpu::Permissions {r: true, w: true, x: false}) ||
        // or its accessible through the combination of 0 and 1
        can_access_heap_split(map_get(config, 0), map_get(config, 1), hstart, hend)
    }

    fn config_cant_access_at_all(config: CortexMConfig, start: int, end: int) -> bool {
        forall i in 0..8 {
            region_cant_access_at_all(map_get(config, i), start, end)
        }
    }

    fn ipc_cant_access_process_mem(config: CortexMConfig, fstart: int, fend: int, hstart: int, hend: int) -> bool {
        forall i in 3..8 {
            region_cant_access_at_all(map_get(config, i), fstart, fend) &&
            region_cant_access_at_all(map_get(config, i), hstart, hend)
        }
    }
}

// VTOCK_TODO: better solution for hardware register spooky-action-at-a-distance
/* VTOCK TODOS
    3. Implement can_service
*/

// VTOCK-TODO: supplementary proof?
#[flux_rs::sig(fn(n: u32{n < 32}) -> usize {r: r > 0 &&  r <= u32::MAX / 2 + 1})]
#[flux_rs::trusted]
fn power_of_two(n: u32) -> usize {
    1_usize << n
}

#[flux_rs::opaque]
#[flux_rs::refined_by(regions: Map<int, CortexMRegion>)]
pub struct RegionGhostState {}
impl RegionGhostState {
    #[flux_rs::trusted]
    const fn new() -> Self {
        Self {}
    }
}

#[flux_rs::opaque]
#[flux_rs::refined_by(regions: Map<int, bitvec<32>>, attrs: Map<int, bitvec<32>>)]
pub struct HwGhostState {}
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
#[flux_rs::refined_by(ctrl: bitvec<32>, rnr: bitvec<32>, rbar: bitvec<32>, rasr: bitvec<32>, regions: Map<int, bitvec<32>>, attrs: Map<int, bitvec<32>>)]
pub struct MpuRegisters {
    /// Indicates whether the MPU is present and, if so, how many regions it
    /// supports.
    // VTOCK-TODO: this should be read-only
    pub mpu_type: ReadWriteU32<Type::Register>,

    /// The control register:
    ///   * Enables the MPU (bit 0).
    ///   * Enables MPU in hard-fault, non-maskable interrupt (NMI).
    ///   * Enables the default memory map background region in privileged mode.
    #[field(ReadWriteU32<Control::Register>[ctrl])]
    pub ctrl: ReadWriteU32<Control::Register>,

    /// Selects the region number (zero-indexed) referenced by the region base
    /// address and region attribute and size registers.
    #[field(ReadWriteU32<RegionNumber::Register>[rnr])]
    pub rnr: ReadWriteU32<RegionNumber::Register>,

    /// Defines the base address of the currently selected MPU region.
    #[field(ReadWriteU32<RegionBaseAddress::Register>[rbar])]
    pub rbar: ReadWriteU32<RegionBaseAddress::Register>,

    /// Defines the region size and memory attributes of the selected MPU
    /// region. The bits are defined as in 4.5.5 of the Cortex-M4 user guide.
    #[field(ReadWriteU32<RegionAttributes::Register>[rasr])]
    pub rasr: ReadWriteU32<RegionAttributes::Register>,

    #[field(HwGhostState[regions, attrs])]
    hw_state: HwGhostState,
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
#[flux_rs::invariant(MIN_REGION_SIZE > 0 && MIN_REGION_SIZE <= u32::MAX / 2 + 1)]
#[flux_rs::refined_by(ctrl: bitvec<32>, rnr: bitvec<32>, rbar: bitvec<32>, rasr: bitvec<32>, regions: Map<int, bitvec<32>>, attrs: Map<int, bitvec<32>>)]
pub struct MPU<const MIN_REGION_SIZE: usize> {
    /// MMIO reference to MPU registers.
    #[field(MpuRegisters[ctrl, rnr, rbar, rasr, regions, attrs])]
    registers: MpuRegisters,
    /// Monotonically increasing counter for allocated regions, used
    /// to assign unique IDs to `CortexMConfig` instances.
    #[field({Cell<NonZeroUsize> | MIN_REGION_SIZE > 0 && MIN_REGION_SIZE < 2147483648})]
    config_count: Cell<NonZeroUsize>,
    /// Optimization logic. This is used to indicate which application the MPU
    /// is currently configured for so that the MPU can skip updating when the
    /// kernel returns to the same app.
    hardware_is_configured_for: OptionalCell<NonZeroUsize>,
}

impl<const MIN_REGION_SIZE: usize> MPU<MIN_REGION_SIZE> {
    pub const unsafe fn new() -> Self {
        assume(MIN_REGION_SIZE > 0);
        assume(MIN_REGION_SIZE < 2147483648);

        let mpu_addr = 0xE000ED90;
        let mpu_type = ReadWriteU32::new(mpu_addr);
        let ctrl = ReadWriteU32::new(mpu_addr + 4);
        let rnr = ReadWriteU32::new(mpu_addr + 8);
        let rbar = ReadWriteU32::new(mpu_addr + 12);
        let rasr = ReadWriteU32::new(mpu_addr + 16);
        let regs = MpuRegisters {
            mpu_type,
            ctrl,
            rnr,
            rbar,
            rasr,
            hw_state: HwGhostState::new(),
        };

        Self {
            registers: regs,
            config_count: Cell::new(NonZeroUsize::MIN),
            hardware_is_configured_for: OptionalCell::empty(),
        }
    }

    // Function useful for boards where the bootloader sets up some
    // MPU configuration that conflicts with Tock's configuration:
    #[flux_rs::sig(fn(self: &strg Self) ensures self: Self{mpu: mpu.ctrl & 0x00000001 == 0 })]
    pub unsafe fn clear_mpu(&mut self) {
        self.registers.ctrl.write(Control::ENABLE::CLEAR());
    }

    // VTOCK CODE
    #[flux_rs::trusted]
    #[flux_rs::sig(
        fn(self: &strg Self[@mpu], &CortexMRegion[@addr, @attrs, @no, @set, @astart, @asize, @rstart, @rsize, @perms]) ensures
            self: Self[mpu.ctrl, mpu.rnr, addr.value, attrs.value,
                map_store(mpu.regions, no, addr.value),
                map_store(mpu.attrs, no, attrs.value)]
    )]
    fn commit_region(&mut self, region: &CortexMRegion) {
        self.registers.rbar.write(region.base_address());
        self.registers.rasr.write(region.attributes());
    }
}

/// Per-process struct storing MPU configuration for cortex-m MPUs.
///
/// The cortex-m MPU has eight regions, all of which must be configured (though
/// unused regions may be configured as disabled). This struct caches the result
/// of region configuration calculation.

const NUM_REGIONS: usize = 8;

// #[flux_rs::refined_by(regions: Map<int, bitvec<32>>, attrs: Map<int, bitvec<32>>)]
#[flux_rs::refined_by(regions: Map<int, CortexMRegion>)]
pub struct CortexMConfig {
    /// Unique ID for this configuration, assigned from a
    /// monotonically increasing counter in the MPU struct.
    id: NonZeroUsize,
    /// The computed region configuration for this process.
    regions: [CortexMRegion; 8],
    /// Has the configuration changed since the last time the this process
    /// configuration was written to hardware?
    is_dirty: Cell<bool>,

    #[field(RegionGhostState[regions])]
    region_state: RegionGhostState,
}

/// Records the index of the last region used for application RAM and flash memory.
/// Regions 0-APP_MEMORY_REGION_MAX_NUM are used for application RAM and flash. Regions
/// with indices above APP_MEMORY_REGION_MAX_NUM can be used for other MPU
/// needs.
///
/// Note the process heap will be region 0 and possibly region 1. Process flash will be region 2
const APP_MEMORY_REGION_MAX_NUM: usize = 2;
const HEAP_REGION1: usize = 0;
const HEAP_REGION2: usize = 1;
const FLASH_REGION: usize = 2;

impl fmt::Display for CortexMConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\r\n Cortex-M MPU")?;
        for (i, region) in self.regions_iter().enumerate() {
            if let Some(location) = region.location() {
                let access_bits = region.attributes().read(RegionAttributes::AP());
                let start = location.accessible_start.as_usize();
                write!(
                    f,
                    "\
                     \r\n  Region {}: [{:#010X}:{:#010X}], length: {} bytes; ({:#x})",
                    i,
                    start,
                    start + location.accesible_size,
                    location.accesible_size,
                    // access_str,
                    access_bits,
                )?;
                let subregion_bits = region.attributes().read(RegionAttributes::SRD());
                let subregion_size = location.accesible_size / 8; // VTock BUG : This is wrong - cannot use logical size to compute the subregion size
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
                write!(f, "\r\n  Region {}: Unused", i)?;
            }
        }
        write!(f, "\r\n")
    }
}

impl CortexMConfig {
    fn id(&self) -> NonZeroUsize {
        self.id
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty.get()
    }

    fn set_dirty(&self, b: bool) {
        self.is_dirty.set(b)
    }

    #[flux_rs::trusted]
    #[flux_rs::sig(fn(&CortexMConfig[@self], {usize[@idx] | idx < 8}) -> &CortexMRegion{r: r == map_get(self.regions, idx) && idx == region_no(map_get(self.regions, idx))})]
    fn get_region(&self, idx: usize) -> &CortexMRegion {
        &self.regions[idx]
    }

    // map_set
    #[flux_rs::sig(fn(self: &strg Self[@regions], idx: usize, region: CortexMRegion[@r]) ensures self: Self[map_set(regions, idx, r)])]
    #[flux_rs::trusted] // needs a spec for index
    fn region_set(&mut self, idx: usize, region: CortexMRegion) {
        self.regions[idx] = region
    }

    #[flux_rs::trusted]
    fn regions_iter(&self) -> core::slice::Iter<'_, CortexMRegion> {
        self.regions.iter()
    }

    #[flux_rs::sig(fn(&CortexMConfig[@c]) -> Option<usize{idx: idx > 2 && idx < 8 }>)]
    #[flux_rs::trusted] // need spec for enumerate for this to work
    fn unused_region_number(&self) -> Option<usize> {
        for (number, region) in self.regions_iter().enumerate() {
            if number <= APP_MEMORY_REGION_MAX_NUM {
                continue;
            }
            if let None = region.location() {
                return Some(number);
            }
        }
        None
    }
}

#[derive(Copy, Clone)]
#[flux_rs::refined_by(astart: int, asize: int, rstart: int, rsize: int)]
struct CortexMLocation {
    #[field(FluxPtrU8[astart])]
    pub accessible_start: FluxPtrU8,
    #[field(usize[asize])]
    pub accesible_size: usize,
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
pub struct CortexMRegion {
    #[field(Option<{l. CortexMLocation[l] | l.astart == astart && l.asize == asize && l.rstart == rstart && l.rsize == rsize }>[set])]
    location: Option<CortexMLocation>, // actually accessible start and size
    #[field({FieldValueU32<RegionBaseAddress::Register>[rbar] | set => region(value(rbar)) == bv32(region_no)})]
    base_address: FieldValueU32<RegionBaseAddress::Register>,
    #[field({FieldValueU32<RegionAttributes::Register>[rasr] | (set => can_access_exactly(rasr, rbar, astart, asize, perms)) && (!set => !region_enable(value(rasr)))})]
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
                 accesible_size: size,
                 ..
             }| { addr == other.start_address() && size == other.size() },
        )
    }
}

impl CortexMRegion {
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
        requires (subregions => rsize >= 256) && rsize < u32::MAX
    )]
    #[flux_rs::trusted] // VTOCK TODO: this one is a beast
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

        let size_value = math::log_base_two_u32_usize(region_size) - 1;

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
            let mask = (min_subregion..=max_subregion).fold(u8::MAX, |res, i| {
                // Enable subregions bit by bit (1 ^ 1 == 0)
                res ^ (1 << i)
            });
            attributes += RegionAttributes::SRD().val(mask as u32);
        }

        Self {
            location: Some(CortexMLocation {
                accessible_start: logical_start,
                accesible_size: logical_size,
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
    fn empty(region_num: usize) -> CortexMRegion {
        CortexMRegion {
            location: None,
            base_address: RegionBaseAddress::VALID::UseRBAR()
                + RegionBaseAddress::REGION().val(region_num as u32),
            attributes: RegionAttributes::ENABLE::CLEAR(),
            ghost_region_state: GhostRegionState::unset(region_num),
        }
    }

    #[flux_rs::sig(fn (&CortexMRegion[@addr, @attrs, @no, @set, @astart, @asize, @rstart, @rsize, @perms]) -> Option<{l. CortexMLocation[l] | l.astart == astart && l.asize == asize && l.rstart == rstart && l.rsize == rsize}>)]
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

    fn overlaps(&self, other_start: FluxPtrU8, other_size: usize) -> bool {
        let other_start = other_start.as_usize();
        let other_end = other_start + other_size;

        let (region_start, region_end) = match self.location() {
            Some(CortexMLocation {
                accessible_start: region_start,
                accesible_size: region_size,
                ..
            }) => {
                let region_start = region_start.as_usize();
                let region_end = region_start + region_size;
                (region_start, region_end)
            }
            None => return false,
        };

        region_start < other_end && other_start < region_end
    }
}

impl<const MIN_REGION_SIZE: usize> MPU<MIN_REGION_SIZE> {
    #[flux_rs::sig(
        fn (
            _,
            usize[@region_no],
            FluxPtrU8[@start],
            usize[@size],
            usize[@minsz],
            mpu::Permissions[@perms],
            _
        ) -> Option<{r. CortexMRegion[r] | 
                r.set &&
                r.region_no == region_no &&
                r.perms == perms &&
                r.astart + r.asize <= start + size &&
                r.asize >= minsz &&
                (size == minsz => start == r.astart && size == r.asize)
            }>
        requires minsz > 0 && minsz <= u32::MAX / 2 + 1 && size <= u32::MAX / 2 + 1 && start <= u32::MAX / 2 + 1
    )]
    fn create_region(
        &self,
        region_num: usize,
        unallocated_memory_start: FluxPtrU8,
        unallocated_memory_size: usize,
        min_region_size: usize,
        permissions: mpu::Permissions,
        config: &CortexMConfig,
    ) -> Option<CortexMRegion> {
        // Check that no previously allocated regions overlap the unallocated memory.
        for region in config.regions_iter() {
            if region.overlaps(unallocated_memory_start, unallocated_memory_size) {
                return None;
            }
        }

        // Logical region
        let mut start = unallocated_memory_start.as_usize();
        let mut size = min_region_size;

        // Region start always has to align to minimum region size bytes
        if start % MIN_REGION_SIZE != 0 {
            start += MIN_REGION_SIZE - (start % MIN_REGION_SIZE);
        }

        // Regions must be at least minimum region size bytes
        if size < MIN_REGION_SIZE {
            size = MIN_REGION_SIZE;
        }

        // Physical MPU region (might be larger than logical region if some subregions are disabled)
        let mut region_start = start;
        let mut region_size = size;
        let mut subregions = None;
        // We can only create an MPU region if the size is a power of two and it divides
        // the start address. If this is not the case, the first thing we try to do to
        // cover the memory region is to use a larger MPU region and expose certain subregions.
        if size.count_ones() > 1 || start % size != 0 {
            // Which (power-of-two) subregion size would align with the start
            // address?
            //
            // We find this by taking smallest binary substring of the start
            // address with exactly one bit:
            //
            //      1 << (start.trailing_zeros())
            let subregion_size = {
                let tz = start.trailing_zeros();
                if tz < 32 {
                    // Find the largest power of two that divides `start`
                    // 1_usize << tz
                    power_of_two(tz)
                } else {
                    // This case means `start` is 0.

                    // VTOCK Bug?
                    // This is interesting. We are able to prove the case this way
                    // assert(size <= (u32::MAX / 2 + 1) as usize);
                    //
                    // but casting the usize to u32 does not work:
                    // assert(size as u32 <= u32::MAX / 2 + 1);
                    // if size as u32 > u32::MAX / 2 + 1 {
                    //     return None
                    // }
                    let mut ceil = math::closest_power_of_two_usize(size);
                    if ceil < 256 {
                        ceil = 256
                    }
                    ceil / 8
                }
            };

            // Once we have a subregion size, we get a region size by
            // multiplying it by the number of subregions per region.
            let underlying_region_size = subregion_size * 8;

            // Finally, we calculate the region base by finding the nearest
            // address below `start` that aligns with the region size.
            let underlying_region_start = start - (start % underlying_region_size);

            // If `size` doesn't align to the subregion size, extend it.
            if size % subregion_size != 0 {
                size += subregion_size - (size % subregion_size);
            }

            let end = start + size;
            let underlying_region_end = underlying_region_start + underlying_region_size;

            // To use subregions, the region must be at least 256 bytes. Also, we need
            // the amount of left over space in the region after `start` to be at least as
            // large as the memory region we want to cover.
            if subregion_size >= 32 && underlying_region_end >= end {
                // The index of the first subregion to activate is the number of
                // regions between `region_start` (MPU) and `start` (memory).
                let min_subregion = (start - underlying_region_start) / subregion_size;

                // The index of the last subregion to activate is the number of
                // regions that fit in `len`, plus the `min_subregion`, minus one
                // (because subregions are zero-indexed).
                let max_subregion = min_subregion + size / subregion_size - 1;

                region_start = underlying_region_start;
                region_size = underlying_region_size;
                subregions = Some((min_subregion, max_subregion));
            } else {
                // In this case, we can't use subregions to solve the alignment
                // problem. Instead, we round up `size` to a power of two and
                // shift `start` up in memory to make it align with `size`.

                // VTOCK Bug - this can overflow and there is no check like the one below
                if size > (u32::MAX / 2 + 1) as usize {
                    return None;
                }
                size = math::closest_power_of_two_usize(size);
                start += size - (start % size);

                region_start = start;
                region_size = size;
            }
        }

        // Check that our logical region fits in memory.
        if start + size > (unallocated_memory_start.as_usize()) + unallocated_memory_size {
            return None;
        }

        if region_size > u32::MAX as usize {
            return None;
        }

        Some(CortexMRegion::new(
            FluxPtr::from(start),
            size,
            FluxPtr::from(region_start),
            region_size,
            region_num,
            subregions,
            permissions,
        ))
    }
}

#[flux_rs::assoc(fn enabled(self: Self) -> bool {enable(self.ctrl)} )]
#[flux_rs::assoc(fn configured_for(self: Self, config: CortexMConfig) -> bool {mpu_configured_for(self, config)} )]
#[flux_rs::assoc(fn config_can_access_flash(c: CortexMConfig, fstart: int, fend: int) -> bool { config_can_access_flash(c, fstart, fend) })]
#[flux_rs::assoc(fn config_can_access_heap(c: CortexMConfig, hstart: int, hend: int) -> bool { config_can_access_heap(c, hstart, hend) })]
#[flux_rs::assoc(fn config_cant_access_at_all(c: CortexMConfig, start: int, end: int) -> bool { config_cant_access_at_all(c, start, end) } )]
#[flux_rs::assoc(fn ipc_cant_access_process_mem(c: CortexMConfig, fstart: int, fend: int, hstart: int, hend: int) -> bool { ipc_cant_access_process_mem(c, fstart, fend, hstart, hend) } )]
impl<const MIN_REGION_SIZE: usize> mpu::MPU for MPU<MIN_REGION_SIZE> {
    type MpuConfig = CortexMConfig;

    // #[flux_rs::sig(fn(self: &strg Self) ensures self: Self)]
    #[flux_rs::sig(fn(self: &strg Self) ensures self: Self{mpu: enable(mpu.ctrl)})]
    fn enable_app_mpu(&mut self) {
        // Enable the MPU, disable it during HardFault/NMI handlers, and allow
        // privileged code access to all unprotected memory.
        self.registers.ctrl.write(
            Control::ENABLE::SET() + Control::HFNMIENA::CLEAR() + Control::PRIVDEFENA::SET(),
        );
    }

    #[flux_rs::sig(fn(self: &strg Self) ensures self: Self{mpu: !enable(mpu.ctrl)})]
    fn disable_app_mpu(&mut self) {
        // The MPU is not enabled for privileged mode, so we don't have to do
        // anything
        self.registers.ctrl.write(Control::ENABLE::CLEAR());
    }

    fn number_total_regions(&self) -> usize {
        self.registers.mpu_type.read(Type::DREGION()) as usize
    }

    #[flux_rs::sig(fn (_) -> Option<{c. CortexMConfig[c] | config_cant_access_at_all(c, 0, u32::MAX)}>)]
    fn new_config(&self) -> Option<CortexMConfig> {
        let id = self.config_count.get();
        self.config_count.set(id.checked_add(1)?);

        // Allocate the regions with index `0` first, then use `reset_config` to
        // write the properly-indexed `CortexMRegion`s:
        let mut ret = CortexMConfig {
            id,
            regions: [CortexMRegion::empty(0); 8],
            is_dirty: Cell::new(true),
            region_state: RegionGhostState::new(),
        };

        self.reset_config(&mut ret);

        Some(ret)
    }

    #[flux_rs::sig(fn (_, config: &strg CortexMConfig) ensures config: CortexMConfig{c: config_cant_access_at_all(c, 0, u32::MAX)})]
    fn reset_config(&self, config: &mut CortexMConfig) {
        config.region_set(0, CortexMRegion::empty(0));
        config.region_set(1, CortexMRegion::empty(1));
        config.region_set(2, CortexMRegion::empty(2));
        config.region_set(3, CortexMRegion::empty(3));
        config.region_set(4, CortexMRegion::empty(4));
        config.region_set(5, CortexMRegion::empty(5));
        config.region_set(6, CortexMRegion::empty(6));
        config.region_set(7, CortexMRegion::empty(7));

        config.set_dirty(true);
    }

    #[flux_rs::sig(fn(
        _,
        FluxPtrU8[@memstart],
        usize[@memsz],
        usize[@minsz],
        mpu::Permissions[@perms],
        config: &strg CortexMConfig[@old_c],
    ) -> Option<mpu::Region>
        requires minsz > 0 && minsz <= u32::MAX / 2 + 1 && memsz <= u32::MAX / 2 + 1 && memstart <= u32::MAX / 2 + 1
        ensures config: CortexMConfig
    )]
    fn allocate_region(
        &self,
        unallocated_memory_start: FluxPtrU8,
        unallocated_memory_size: usize,
        min_region_size: usize,
        permissions: mpu::Permissions,
        config: &mut CortexMConfig,
    ) -> Option<mpu::Region> {
        let region_num = config.unused_region_number()?;
        let region = self.create_region(
            region_num,
            unallocated_memory_start,
            unallocated_memory_size,
            min_region_size,
            permissions,
            config,
        )?;
        config.region_set(region_num, region);
        config.set_dirty(true);

        Some(mpu::Region::new(
            region.location()?.accessible_start,
            region.location()?.accesible_size,
        ))
    }

    #[flux_rs::sig(fn(
        _,
        region: mpu::Region[@memstart, @memsz],
        config: &strg CortexMConfig[@c],
    ) -> Result<(), ()>
    ensures config: CortexMConfig)]
    fn remove_memory_region(
        &self,
        region: mpu::Region,
        config: &mut CortexMConfig,
    ) -> Result<(), ()> {
        let (idx, _r) = config
            .regions_iter()
            .enumerate()
            .find(|(_idx, r)| **r == region)
            .ok_or(())?;

        if idx <= APP_MEMORY_REGION_MAX_NUM {
            return Err(());
        }
        assume(idx < 8); // need spec for find

        config.region_set(idx, CortexMRegion::empty(idx));
        config.set_dirty(true);

        Ok(())
    }

    // When allocating memory for apps, we use two regions, each a power of two
    // in size. By using two regions we halve their size, and also halve their
    // alignment restrictions.
    #[flux_rs::sig(
        fn (
            &Self,
            FluxPtrU8[@mem_start],
            usize,
            usize[@min_mem_sz],
            usize[@appmsz],
            usize[@kernelmsz],
            FluxPtrU8[@fstart],
            usize[@fsz],
            config: &strg CortexMConfig[@old_c],
        ) -> Result<{b. mpu::AllocatedAppBreaksAndSize[b] |
            b.app_break <= b.memory_start + b.memory_size - kernelmsz &&
            b.app_break >= b.memory_start + appmsz &&
            config_can_access_flash(new_c, fstart, fstart + fsz) &&
            config_can_access_heap(new_c, b.memory_start, b.app_break) &&
            config_cant_access_at_all(new_c, 0, fstart - 1) &&
            config_cant_access_at_all(new_c, fstart + fsz + 1, b.memory_start - 1) &&
            config_cant_access_at_all(new_c, b.app_break + 1, u32::MAX) 
        }, mpu::AllocateAppMemoryError>
        requires 
            fstart + fsz < mem_start &&
            min_mem_sz > 0 &&
            min_mem_sz <= u32::MAX / 2 + 1 &&
            appmsz > 0 &&
            kernelmsz > 0 &&
            appmsz + kernelmsz <= u32::MAX / 2 + 1 &&
            fstart > 0 &&
            fstart <= u32::MAX / 2 + 1 && 
            fsz > 0 &&
            fsz <= u32::MAX / 2 + 1 &&
            appmsz + kernelmsz < u32::MAX && 
            config_cant_access_at_all(old_c, 0, u32::MAX)
        ensures config: CortexMConfig[#new_c]
    )]
    #[flux_rs::trusted_impl] // fixpoint encoding
    fn allocate_app_memory_regions(
        &self,
        unallocated_memory_start: FluxPtrU8,
        unallocated_memory_size: usize,
        min_memory_size: usize,
        initial_app_memory_size: usize,
        initial_kernel_memory_size: usize,
        flash_start: FluxPtrU8Mut,
        flash_size: usize,
        config: &mut CortexMConfig,
    ) -> Result<mpu::AllocatedAppBreaksAndSize, mpu::AllocateAppMemoryError> {
        // first allocate flash
        let region = self
            .create_region(
                FLASH_REGION,
                flash_start,
                flash_size,
                flash_size,
                mpu::Permissions::ReadExecuteOnly,
                config,
            )
            .ok_or(AllocateAppMemoryError::FlashError)?;
        config.region_set(FLASH_REGION, region);
        // VTOCK TODO: Is this necessary?
        config.set_dirty(true);

        // Check that no previously allocated regions overlap the unallocated
        // memory.
        for region in config.regions_iter() {
            if region.overlaps(unallocated_memory_start, unallocated_memory_size) {
                return Err(AllocateAppMemoryError::HeapError);
            }
        }

        // Make sure there is enough memory for app memory and kernel memory.
        let memory_size = max_usize(
            min_memory_size,
            initial_app_memory_size + initial_kernel_memory_size,
        );

        // Size must be a power of two, so:
        // https://www.youtube.com/watch?v=ovo6zwv6DX4.
        let mut memory_size_po2 = math::closest_power_of_two_usize(memory_size);
        let exponent = math::log_base_two_u32_usize(memory_size_po2);

        // // Check for compliance with the constraints of the MPU.
        if exponent < 9 {
            // Region sizes must be 256 bytes or larger to support subregions.
            // Since we are using two regions, and each must be at least 256
            // bytes, we need the entire memory region to be at least 512 bytes.
            memory_size_po2 = 512;
        } else if exponent > 32 {
            // Region sizes must be 4GB or smaller.
            return Err(AllocateAppMemoryError::HeapError);
        }

        // Region size is the actual size the MPU region will be set to, and is
        // half of the total power of two size we are allocating to the app.
        let mut region_size = memory_size_po2 / 2;

        // The region should start as close as possible to the start of the
        // unallocated memory.
        let mut region_start = unallocated_memory_start.as_usize();

        // If the start and length don't align, move region up until it does.
        if region_start % region_size != 0 {
            region_start += region_size - (region_start % region_size);
        }

        // We allocate two MPU regions exactly over the process memory block,
        // and we disable subregions at the end of this region to disallow
        // access to the memory past the app break. As the app break later
        // increases, we will be able to linearly grow the logical region
        // covering app-owned memory by enabling more and more subregions. The
        // Cortex-M MPU supports 8 subregions per region, so the size of this
        // logical region is always a multiple of a sixteenth of the MPU region
        // length.

        // Determine the number of subregions to enable.
        // Want `round_up(app_memory_size / subregion_size)`.
        let mut subregion_size = region_size / 8;

        let mut num_enabled_subregions = initial_app_memory_size.div_ceil(subregion_size);

        // Calculates the end address of the enabled subregions and the initial
        // kernel memory break.
        let subregions_enabled_end = region_start + num_enabled_subregions * subregion_size;
        //let kernel_memory_break = region_start + memory_size_po2 - initial_kernel_memory_size;

        // VTOCK TODO: checked_sub spec? Where is that coming from?
        // let kernel_memory_break =
        //     (region_start + memory_size_po2).checked_sub(initial_kernel_memory_size).ok_or(mpu::AllocateAppMemoryError::HeapError)?;
        if region_start + memory_size_po2 < initial_kernel_memory_size {
            return Err(mpu::AllocateAppMemoryError::HeapError);
        }
        let kernel_memory_break = region_start + memory_size_po2 - initial_kernel_memory_size;

        // If the last subregion covering app-owned memory overlaps the start of
        // kernel-owned memory, we make the entire process memory block twice as
        // big so there is plenty of space between app-owned and kernel-owned
        // memory.
        if subregions_enabled_end > kernel_memory_break {
            memory_size_po2 *= 2;
            region_size *= 2;

            if region_start % region_size != 0 {
                region_start += region_size - (region_start % region_size);
            }

            subregion_size = region_size / 8;
            num_enabled_subregions = initial_app_memory_size.div_ceil(subregion_size);
        }

        // Make sure the region fits in the unallocated memory.
        if region_start + memory_size_po2
            > (unallocated_memory_start.as_usize()) + unallocated_memory_size
        {
            return Err(AllocateAppMemoryError::HeapError);
        }

        // Get the number of subregions enabled in each of the two MPU regions.
        let num_enabled_subregions0 = min_usize(num_enabled_subregions, 8);
        let num_enabled_subregions1 = num_enabled_subregions.saturating_sub(8);

        let region0 = CortexMRegion::new(
            FluxPtr::from(region_start),
            num_enabled_subregions0 * subregion_size,
            FluxPtr::from(region_start),
            region_size,
            HEAP_REGION1,
            Some((0, num_enabled_subregions0 - 1)),
            mpu::Permissions::ReadWriteOnly,
        );

        // We cannot have a completely unused MPU region
        let region1 = if num_enabled_subregions1 == 0 {
            CortexMRegion::empty(1)
        } else {
            CortexMRegion::new(
                    FluxPtr::from(region_start + region_size),
                num_enabled_subregions1 * subregion_size,
                FluxPtr::from(region_start + region_size),
                region_size,
                HEAP_REGION2,
                Some((0, num_enabled_subregions1 - 1)),
                mpu::Permissions::ReadWriteOnly,
            )
        };

        config.region_set(HEAP_REGION1, region0);
        config.region_set(HEAP_REGION2, region1);
        config.set_dirty(true);

        // VTOCK TODO: Without this assume proving the gap between flash end and mem start is not possible
        // because flux believes that the region start computed could be >= u32::MAX. We should ideally be 
        // able to constrain the addresses passed to this function to something within reason...
        assume(region_start + memory_size_po2 < u32::MAX as usize);
        let app_break = region_start + subregion_size * num_enabled_subregions;
        Ok(mpu::AllocatedAppBreaksAndSize::new(FluxPtr::from(region_start), FluxPtr::from(app_break), memory_size_po2))
    }

    #[flux_rs::sig(
        fn (
            &Self,
            FluxPtrU8[@mem_start],
            FluxPtrU8[@old_app_break],
            FluxPtrU8Mut[@app_break],
            FluxPtrU8Mut[@kernel_break],
            FluxPtrU8Mut[@fstart],
            usize[@fsz],
            config: &strg CortexMConfig[@old_c],
        ) -> Result<{b. mpu::AllocatedAppBreaks[b] | 
            b.app_break <= kernel_break &&
            b.app_break >= app_break &&
            b.memory_start == mem_start &&
            config_can_access_flash(new_c, fstart, fstart + fsz) &&
            config_can_access_heap(new_c, b.memory_start, b.app_break) &&
            config_cant_access_at_all(new_c, 0, fstart - 1) &&
            config_cant_access_at_all(new_c, fstart + fsz + 1, b.memory_start - 1) &&
            config_cant_access_at_all(new_c, b.app_break + 1, u32::MAX) &&
            ipc_cant_access_process_mem(new_c, fstart, fstart + fsz, b.memory_start, u32::MAX)
        }, ()>[#res]
        requires 
            fstart + fsz < mem_start &&
            app_break - mem_start <= u32::MAX / 2 + 1 &&
            app_break > mem_start &&
            config_can_access_flash(old_c, fstart, fstart + fsz) &&
            config_can_access_heap(old_c, mem_start, old_app_break) &&
            config_cant_access_at_all(old_c, 0, fstart - 1) &&
            config_cant_access_at_all(old_c, fstart + fsz + 1, mem_start - 1) &&
            config_cant_access_at_all(old_c, old_app_break + 1, u32::MAX) &&
            // VTOCK TODO: 
            // I understand that there is a possibility that some IPC region might 
            // have access to mem_start - old_app_break. Therefore, if we shrink the app break,
            // the config may still have access to the old memory. It would make sense to me if we 
            // had to say region_cant_access_at_all(..., mem_start, old_app_break). But somehow, even 
            // u32::MAX - 1 doesn't work as the end address here... 
            ipc_cant_access_process_mem(old_c, fstart, fstart + fsz, mem_start, u32::MAX)
        ensures config: CortexMConfig[#new_c], !res => old_c == new_c
    )]
    #[flux_rs::trusted_impl] // fixpoint encoding
    fn update_app_memory_regions(
        &self,
        mem_start: FluxPtrU8,
        _old_app_memory_break: FluxPtrU8,
        app_memory_break: FluxPtrU8,
        kernel_memory_break: FluxPtrU8,
        flash_start: FluxPtrU8Mut,
        flash_size: usize,
        config: &mut CortexMConfig,
    ) -> Result<mpu::AllocatedAppBreaks, ()> {
        // Get second region for flash and make sure it's allocated
        let CortexMLocation {
            accessible_start: flash_region_start,
            accesible_size: flash_region_size,
            region_start: _, 
            region_size: _
        } = config.get_region(FLASH_REGION).location().ok_or(())?;
        // if the flash region doesn't match exactly, something has gone terribly wrong
        if flash_region_start != flash_start || flash_region_size != flash_region_size {
            return Err(());
        }

        // Get first region, or error if the process tried to update app memory
        // MPU region before it was created.
        let CortexMLocation {
            accessible_start: _,
            accesible_size: _,
            region_start: region_start_ptr,
            region_size,
        } = config.get_region(HEAP_REGION1).location().ok_or(())?; 
        let region_start = region_start_ptr.as_usize();

        // if the region start and memory start don't match, something has gone terribly wrong
        if mem_start.as_usize() != region_start {
            return Err(());
        }

        // VTOCK todo: can we prove this?
        // from allocate_app_memory_region
        assume(region_size >= 256 && region_size < u32::MAX as usize); 
        assume(region_size % 8 == 0);
        assume(region_start + region_size * 2 >= kernel_memory_break.as_usize());

        let app_memory_break = app_memory_break.as_usize();
        let kernel_memory_break = kernel_memory_break.as_usize();

        // Out of memory
        if app_memory_break > kernel_memory_break {
            return Err(());
        }

        // Number of bytes the process wants access to.
        let app_memory_size = app_memory_break - region_start;

        // There are eight subregions for every region in the Cortex-M3/4 MPU.
        let subregion_size = region_size / 8;

        // Determine the number of subregions to enable.
        // Want `round_up(app_memory_size / subregion_size)`.
        let num_enabled_subregions = app_memory_size.div_ceil(subregion_size);
        assert(num_enabled_subregions <= 16);

        let subregions_enabled_end = region_start + subregion_size * num_enabled_subregions;

        // If we can no longer cover app memory with an MPU region without
        // overlapping kernel memory, we fail.
        if subregions_enabled_end > kernel_memory_break {
            return Err(());
        }

        // // Get the number of subregions enabled in each of the two MPU regions.
        let num_enabled_subregions0 = min_usize(num_enabled_subregions, 8);
        let num_enabled_subregions1 = num_enabled_subregions.saturating_sub(8);

        let region0 = CortexMRegion::new(
            FluxPtr::from(region_start),
            num_enabled_subregions0 * subregion_size,
            FluxPtr::from(region_start),
            region_size,
            HEAP_REGION1,
            Some((0, num_enabled_subregions0 - 1)),
            mpu::Permissions::ReadWriteOnly,
        );

        let region1 = if num_enabled_subregions1 == 0 {
            CortexMRegion::empty(HEAP_REGION2)
        } else {
            CortexMRegion::new(
                FluxPtr::from(region_start + region_size),
                num_enabled_subregions1 * subregion_size,
                FluxPtr::from(region_start + region_size),
                region_size,
                HEAP_REGION2,
                Some((0, num_enabled_subregions1 - 1)),
                mpu::Permissions::ReadWriteOnly,
            )
        };

        config.region_set(HEAP_REGION1, region0);
        config.region_set(HEAP_REGION2, region1);
        config.set_dirty(true);

        Ok(mpu::AllocatedAppBreaks::new(
            FluxPtr::from(region_start),
            FluxPtr::from(subregions_enabled_end),
        ))
    }

    // TODO: reimplement dirty tracking
    // TODO: add for loop back in
    #[flux_rs::sig(fn(self: &strg Self, &CortexMConfig[@config]) ensures self: Self{mpu: mpu_configured_for(mpu, config)})]
    #[flux_rs::trusted_impl] // fixpoint encoding
    fn configure_mpu(&mut self, config: &CortexMConfig) {
        // If the hardware is already configured for this app and the app's MPU
        // configuration has not changed, then skip the hardware update.
        // if !self.hardware_is_configured_for.contains(&config.id()) || config.is_dirty() {
        // Set MPU regions
        self.commit_region(config.get_region(0));
        self.commit_region(config.get_region(1));
        self.commit_region(config.get_region(2));
        self.commit_region(config.get_region(3));
        self.commit_region(config.get_region(4));
        self.commit_region(config.get_region(5));
        self.commit_region(config.get_region(6));
        self.commit_region(config.get_region(7));
        // for region in config.regions_iter() {
        //     self.commit_region(region);
        // }
        // self.hardware_is_configured_for.set(config.id());
        // config.set_dirty(false);
        // }
    }
}

// TODO: simplify configured_for
// -- requires proving that beyond 8, everything is the same
// -- alternately, just requires a special macro
// TODO: better solution than trusted `get_region`?
// TODO: once there is support for double projections in specs, remove `value()` function
// -- alternately, dont refine CortexMRegion by FieldValue but just by bitvec?
