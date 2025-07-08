// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Cortex-M Memory Protection Unit (MPU)
//!
//! Implementation of the memory protection unit for the Cortex-M0+, Cortex-M3,
//! Cortex-M4, and Cortex-M7.

use core::cell::Cell;
use core::cmp;
use core::fmt;
use core::num::NonZeroUsize;

use kernel::platform::mpu;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::math;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, FieldValue, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

/// Smallest allowable MPU region across all CortexM cores
/// Individual cores may have bigger min sizes, but never lower than 32
const CORTEXM_MIN_REGION_SIZE: usize = 32;

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
        /// MPU_RASR registers. Range 0-7 corresponding to the MPU regions.
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

/// State related to the real physical MPU.
///
/// There should only be one instantiation of this object as it represents
/// real hardware.
pub struct MPU<const NUM_REGIONS: usize, const MIN_REGION_SIZE: usize> {
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

impl<const NUM_REGIONS: usize, const MIN_REGION_SIZE: usize> MPU<NUM_REGIONS, MIN_REGION_SIZE> {
    pub const unsafe fn new(registers: StaticRef<MpuRegisters>) -> Self {
        Self {
            registers,
            config_count: Cell::new(NonZeroUsize::MIN),
            hardware_is_configured_for: OptionalCell::empty(),
        }
    }

    // Function useful for boards where the bootloader sets up some
    // MPU configuration that conflicts with Tock's configuration:
    pub unsafe fn clear_mpu(&self) {
        self.registers.ctrl.write(Control::ENABLE::CLEAR);
    }
}

/// Per-process struct storing MPU configuration for cortex-m MPUs.
///
/// The cortex-m MPU has eight regions, all of which must be configured (though
/// unused regions may be configured as disabled). This struct caches the result
/// of region configuration calculation.
pub struct CortexMConfig<const NUM_REGIONS: usize> {
    /// Unique ID for this configuration, assigned from a
    /// monotonically increasing counter in the MPU struct.
    id: NonZeroUsize,
    /// The computed region configuration for this process.
    regions: [CortexMRegion; NUM_REGIONS],
    /// Has the configuration changed since the last time the this process
    /// configuration was written to hardware?
    is_dirty: Cell<bool>,
}

/// Records the index of the last region used for application RAM memory.
/// Regions 0-APP_MEMORY_REGION_MAX_NUM are used for application RAM. Regions
/// with indices above APP_MEMORY_REGION_MAX_NUM can be used for other MPU
/// needs.
const APP_MEMORY_REGION_MAX_NUM: usize = 1;

impl<const NUM_REGIONS: usize> fmt::Display for CortexMConfig<NUM_REGIONS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\r\n Cortex-M MPU")?;
        for (i, region) in self.regions.iter().enumerate() {
            if let Some(location) = region.location() {
                let access_bits = region.attributes().read(RegionAttributes::AP);
                let access_str = match access_bits {
                    0b000 => "NoAccess",
                    0b001 => "PrivilegedOnly",
                    0b010 => "UnprivilegedReadOnly",
                    0b011 => "ReadWrite",
                    0b100 => "Reserved",
                    0b101 => "PrivilegedOnlyReadOnly",
                    0b110 => "ReadOnly",
                    0b111 => "ReadOnlyAlias",
                    _ => "ERR",
                };
                let start = location.0 as usize;
                write!(
                    f,
                    "\
                     \r\n  Region {}: [{:#010X}:{:#010X}], length: {} bytes; {} ({:#x})",
                    i,
                    start,
                    start + location.1,
                    location.1,
                    access_str,
                    access_bits,
                )?;
                let subregion_bits = region.attributes().read(RegionAttributes::SRD);
                let subregion_size = location.1 / 8;
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

impl<const NUM_REGIONS: usize> CortexMConfig<NUM_REGIONS> {
    fn unused_region_number(&self) -> Option<usize> {
        for (number, region) in self.regions.iter().enumerate() {
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

/// Struct storing configuration for a Cortex-M MPU region.
#[derive(Copy, Clone)]
pub struct CortexMRegion {
    location: Option<(*const u8, usize)>,
    base_address: FieldValue<u32, RegionBaseAddress::Register>,
    attributes: FieldValue<u32, RegionAttributes::Register>,
}

impl PartialEq<mpu::Region> for CortexMRegion {
    fn eq(&self, other: &mpu::Region) -> bool {
        self.location.is_some_and(|(addr, size)| {
            core::ptr::eq(addr, other.start_address()) && size == other.size()
        })
    }
}

impl CortexMRegion {
    fn new(
        logical_start: *const u8,
        logical_size: usize,
        region_start: *const u8,
        region_size: usize,
        region_num: usize,
        subregions: Option<(usize, usize)>,
        permissions: mpu::Permissions,
    ) -> Option<CortexMRegion> {
        // Logical size must be above minimum size for cortexM MPU regions and
        // and less than the size of the underlying physical region
        if logical_size < CORTEXM_MIN_REGION_SIZE || region_size < logical_size {
            return None;
        }

        // Determine access and execute permissions
        let (access, execute) = match permissions {
            mpu::Permissions::ReadWriteExecute => (
                RegionAttributes::AP::ReadWrite,
                RegionAttributes::XN::Enable,
            ),
            mpu::Permissions::ReadWriteOnly => (
                RegionAttributes::AP::ReadWrite,
                RegionAttributes::XN::Disable,
            ),
            mpu::Permissions::ReadExecuteOnly => (
                RegionAttributes::AP::UnprivilegedReadOnly,
                RegionAttributes::XN::Enable,
            ),
            mpu::Permissions::ReadOnly => (
                RegionAttributes::AP::UnprivilegedReadOnly,
                RegionAttributes::XN::Disable,
            ),
            mpu::Permissions::ExecuteOnly => (
                RegionAttributes::AP::PrivilegedOnly,
                RegionAttributes::XN::Enable,
            ),
        };

        // Base address register
        let base_address = RegionBaseAddress::ADDR.val((region_start as u32) >> 5)
            + RegionBaseAddress::VALID::UseRBAR
            + RegionBaseAddress::REGION.val(region_num as u32);

        let size_value = math::log_base_two(region_size as u32) - 1;

        // Attributes register
        let mut attributes = RegionAttributes::ENABLE::SET
            + RegionAttributes::SIZE.val(size_value)
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
            attributes += RegionAttributes::SRD.val(mask as u32);
        }

        Some(CortexMRegion {
            location: Some((logical_start, logical_size)),
            base_address,
            attributes,
        })
    }

    fn empty(region_num: usize) -> CortexMRegion {
        CortexMRegion {
            location: None,
            base_address: RegionBaseAddress::VALID::UseRBAR
                + RegionBaseAddress::REGION.val(region_num as u32),
            attributes: RegionAttributes::ENABLE::CLEAR,
        }
    }

    fn location(&self) -> Option<(*const u8, usize)> {
        self.location
    }

    fn base_address(&self) -> FieldValue<u32, RegionBaseAddress::Register> {
        self.base_address
    }

    fn attributes(&self) -> FieldValue<u32, RegionAttributes::Register> {
        self.attributes
    }

    fn overlaps(&self, other_start: *const u8, other_size: usize) -> bool {
        let other_start = other_start as usize;
        let other_end = other_start + other_size;

        let (region_start, region_end) = match self.location {
            Some((region_start, region_size)) => {
                let region_start = region_start as usize;
                let region_end = region_start + region_size;
                (region_start, region_end)
            }
            None => return false,
        };

        region_start < other_end && other_start < region_end
    }
}

impl<const NUM_REGIONS: usize, const MIN_REGION_SIZE: usize> mpu::MPU
    for MPU<NUM_REGIONS, MIN_REGION_SIZE>
{
    type MpuConfig = CortexMConfig<NUM_REGIONS>;

    fn enable_app_mpu(&self) {
        // Enable the MPU, disable it during HardFault/NMI handlers, and allow
        // privileged code access to all unprotected memory.
        self.registers
            .ctrl
            .write(Control::ENABLE::SET + Control::HFNMIENA::CLEAR + Control::PRIVDEFENA::SET);
    }

    fn disable_app_mpu(&self) {
        // The MPU is not enabled for privileged mode, so we don't have to do
        // anything
        self.registers.ctrl.write(Control::ENABLE::CLEAR);
    }

    fn number_total_regions(&self) -> usize {
        self.registers.mpu_type.read(Type::DREGION) as usize
    }

    fn new_config(&self) -> Option<Self::MpuConfig> {
        let id = self.config_count.get();
        self.config_count.set(id.checked_add(1)?);

        // Allocate the regions with index `0` first, then use `reset_config` to
        // write the properly-indexed `CortexMRegion`s:
        let mut ret = CortexMConfig {
            id,
            regions: [CortexMRegion::empty(0); NUM_REGIONS],
            is_dirty: Cell::new(true),
        };

        self.reset_config(&mut ret);

        Some(ret)
    }

    fn reset_config(&self, config: &mut Self::MpuConfig) {
        for i in 0..NUM_REGIONS {
            config.regions[i] = CortexMRegion::empty(i);
        }

        config.is_dirty.set(true);
    }

    fn allocate_region(
        &self,
        unallocated_memory_start: *const u8,
        unallocated_memory_size: usize,
        min_region_size: usize,
        permissions: mpu::Permissions,
        config: &mut Self::MpuConfig,
    ) -> Option<mpu::Region> {
        // Check that no previously allocated regions overlap the unallocated memory.
        for region in config.regions.iter() {
            if region.overlaps(unallocated_memory_start, unallocated_memory_size) {
                return None;
            }
        }

        let region_num = config.unused_region_number()?;

        // Logical region
        let mut start = unallocated_memory_start as usize;
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
                    1_usize << tz
                } else {
                    // This case means `start` is 0.
                    let mut ceil = math::closest_power_of_two(size as u32) as usize;
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
                size = math::closest_power_of_two(size as u32) as usize;
                start += size - (start % size);

                region_start = start;
                region_size = size;
            }
        }

        // Check that our logical region fits in memory.
        if start + size > (unallocated_memory_start as usize) + unallocated_memory_size {
            return None;
        }

        let region = CortexMRegion::new(
            start as *const u8,
            size,
            region_start as *const u8,
            region_size,
            region_num,
            subregions,
            permissions,
        )?;

        config.regions[region_num] = region;
        config.is_dirty.set(true);

        Some(mpu::Region::new(start as *const u8, size))
    }

    fn remove_memory_region(
        &self,
        region: mpu::Region,
        config: &mut Self::MpuConfig,
    ) -> Result<(), ()> {
        let (idx, _r) = config
            .regions
            .iter()
            .enumerate()
            .find(|(_idx, r)| **r == region)
            .ok_or(())?;

        if idx <= APP_MEMORY_REGION_MAX_NUM {
            return Err(());
        }

        config.regions[idx] = CortexMRegion::empty(idx);
        config.is_dirty.set(true);

        Ok(())
    }

    // When allocating memory for apps, we use two regions, each a power of two
    // in size. By using two regions we halve their size, and also halve their
    // alignment restrictions.
    fn allocate_app_memory_region(
        &self,
        unallocated_memory_start: *const u8,
        unallocated_memory_size: usize,
        min_memory_size: usize,
        initial_app_memory_size: usize,
        initial_kernel_memory_size: usize,
        permissions: mpu::Permissions,
        config: &mut Self::MpuConfig,
    ) -> Option<(*const u8, usize)> {
        // Check that no previously allocated regions overlap the unallocated
        // memory.
        for region in config.regions.iter() {
            if region.overlaps(unallocated_memory_start, unallocated_memory_size) {
                return None;
            }
        }

        // Make sure there is enough memory for app memory and kernel memory.
        let memory_size = cmp::max(
            min_memory_size,
            initial_app_memory_size + initial_kernel_memory_size,
        );

        // Size must be a power of two, so:
        // https://www.youtube.com/watch?v=ovo6zwv6DX4.
        let mut memory_size_po2 = math::closest_power_of_two(memory_size as u32) as usize;
        let exponent = math::log_base_two(memory_size_po2 as u32);

        // Check for compliance with the constraints of the MPU.
        if exponent < 9 {
            // Region sizes must be 256 bytes or larger to support subregions.
            // Since we are using two regions, and each must be at least 256
            // bytes, we need the entire memory region to be at least 512 bytes.
            memory_size_po2 = 512;
        } else if exponent > 32 {
            // Region sizes must be 4GB or smaller.
            return None;
        }

        // Region size is the actual size the MPU region will be set to, and is
        // half of the total power of two size we are allocating to the app.
        let mut region_size = memory_size_po2 / 2;

        // The region should start as close as possible to the start of the
        // unallocated memory.
        let mut region_start = unallocated_memory_start as usize;

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
        let mut num_enabled_subregions = initial_app_memory_size * 8 / region_size + 1;

        let subregion_size = region_size / 8;

        // Calculates the end address of the enabled subregions and the initial
        // kernel memory break.
        let subregions_enabled_end = region_start + num_enabled_subregions * subregion_size;
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

            num_enabled_subregions = initial_app_memory_size * 8 / region_size + 1;
        }

        // Make sure the region fits in the unallocated memory.
        if region_start + memory_size_po2
            > (unallocated_memory_start as usize) + unallocated_memory_size
        {
            return None;
        }

        // Get the number of subregions enabled in each of the two MPU regions.
        let num_enabled_subregions0 = cmp::min(num_enabled_subregions, 8);
        let num_enabled_subregions1 = num_enabled_subregions.saturating_sub(8);

        let region0 = CortexMRegion::new(
            region_start as *const u8,
            region_size,
            region_start as *const u8,
            region_size,
            0,
            Some((0, num_enabled_subregions0 - 1)),
            permissions,
        )?;

        // We cannot have a completely unused MPU region
        let region1 = if num_enabled_subregions1 == 0 {
            CortexMRegion::empty(1)
        } else {
            CortexMRegion::new(
                (region_start + region_size) as *const u8,
                region_size,
                (region_start + region_size) as *const u8,
                region_size,
                1,
                Some((0, num_enabled_subregions1 - 1)),
                permissions,
            )?
        };

        config.regions[0] = region0;
        config.regions[1] = region1;
        config.is_dirty.set(true);

        Some((region_start as *const u8, memory_size_po2))
    }

    fn update_app_memory_region(
        &self,
        app_memory_break: *const u8,
        kernel_memory_break: *const u8,
        permissions: mpu::Permissions,
        config: &mut Self::MpuConfig,
    ) -> Result<(), ()> {
        // Get first region, or error if the process tried to update app memory
        // MPU region before it was created.
        let (region_start_ptr, region_size) = config.regions[0].location().ok_or(())?;
        let region_start = region_start_ptr as usize;

        let app_memory_break = app_memory_break as usize;
        let kernel_memory_break = kernel_memory_break as usize;

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

        let subregions_enabled_end = region_start + subregion_size * num_enabled_subregions;

        // If we can no longer cover app memory with an MPU region without
        // overlapping kernel memory, we fail.
        if subregions_enabled_end > kernel_memory_break {
            return Err(());
        }

        // Get the number of subregions enabled in each of the two MPU regions.
        let num_enabled_subregions0 = cmp::min(num_enabled_subregions, 8);
        let num_enabled_subregions1 = num_enabled_subregions.saturating_sub(8);

        let region0 = CortexMRegion::new(
            region_start as *const u8,
            region_size,
            region_start as *const u8,
            region_size,
            0,
            Some((0, num_enabled_subregions0 - 1)),
            permissions,
        )
        .ok_or(())?;

        let region1 = if num_enabled_subregions1 == 0 {
            CortexMRegion::empty(1)
        } else {
            CortexMRegion::new(
                (region_start + region_size) as *const u8,
                region_size,
                (region_start + region_size) as *const u8,
                region_size,
                1,
                Some((0, num_enabled_subregions1 - 1)),
                permissions,
            )
            .ok_or(())?
        };

        config.regions[0] = region0;
        config.regions[1] = region1;
        config.is_dirty.set(true);

        Ok(())
    }

    fn configure_mpu(&self, config: &Self::MpuConfig) {
        // If the hardware is already configured for this app and the app's MPU
        // configuration has not changed, then skip the hardware update.
        if !self.hardware_is_configured_for.contains(&config.id) || config.is_dirty.get() {
            // Set MPU regions
            for region in config.regions.iter() {
                self.registers.rbar.write(region.base_address());
                self.registers.rasr.write(region.attributes());
            }
            self.hardware_is_configured_for.set(config.id);
            config.is_dirty.set(false);
        }
    }
}
