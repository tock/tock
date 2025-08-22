// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

//! Cortex-M Memory Protection Unit (MPU)
//!
//! Implementation of the memory protection unit for the Cortex-M33.

use core::cell::Cell;
use core::cmp;
use core::fmt;
use core::num::NonZeroUsize;

use kernel::platform::mpu;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, FieldValue, ReadOnly, ReadWrite,
};
use kernel::utilities::StaticRef;

/// Smallest allowable MPU region across all CortexM cores
/// Individual cores may have bigger min sizes, but never lower than 32
const CORTEXM_MIN_REGION_SIZE: usize = 32;

register_structs! {
    /// MPU Registers for the Armv8-M architecture
    pub MpuRegisters {
        /// MPU Type Register
        (0x0000 => mpu_type: ReadOnly<u32, MPU_TYPE::Register>),
        /// MPU Control Register
        (0x0004 => ctrl: ReadWrite<u32, MPU_CTRL::Register>),
        /// MPU Region Number Register
        (0x0008 => rnr: ReadWrite<u32, MPU_RNR::Register>),
        /// MPU Region Base Address Register
        (0x000C => rbar: ReadWrite<u32, MPU_RBAR::Register>),
        /// MPU Region Limit Address Register
        (0x0010 => rlar: ReadWrite<u32, MPU_RLAR::Register>),
        /// MPU Region Base Address Register Alias 1
        (0x0014 => rbar_a1: ReadWrite<u32, MPU_RBAR_A1::Register>),
        /// MPU Region Limit Address Register Alias 1
        (0x0018 => rlar_a1: ReadWrite<u32, MPU_RLAR_A1::Register>),
        /// MPU Region Base Address Register Alias 2
        (0x001C => rbar_a2: ReadWrite<u32, MPU_RBAR_A2::Register>),
        /// MPU Region Limit Address Register Alias 2
        (0x0020 => rlar_a2: ReadWrite<u32, MPU_RLAR_A2::Register>),
        /// MPU Region Base Address Register Alias 3
        (0x0024 => rbar_a3: ReadWrite<u32, MPU_RBAR_A3::Register>),
        /// MPU Region Limit Address Register Alias 3
        (0x0028 => rlar_a3: ReadWrite<u32, MPU_RLAR_A3::Register>),
        (0x002c => _reserved0),
        /// MPU Memory Attribute Indirection Register 0
        (0x0030 => mair0: ReadWrite<u32, MPU_MAIR0::Register>),
        /// MPU Memory Attribute Indirection Register 1
        (0x0034 => mair1: ReadWrite<u32, MPU_MAIR1::Register>),
        (0x0038 => @END),
    }
}

register_bitfields![u32,
    MPU_TYPE [
        /// Number of regions supported by the MPU
        DREGION OFFSET(8) NUMBITS(8) [],
        /// Indicates support for separate instructions and data address regions
        SEPARATE OFFSET(0) NUMBITS(1) []
    ],
    MPU_CTRL [
        /// Controls whether the default memory map is enabled for privileged software
        PRIVDEFENA OFFSET(2) NUMBITS(1) [],
        /// Controls whether handlers executing with priority less than 0 access memory with
        HFNMIENA OFFSET(1) NUMBITS(1) [],
        /// Enables the MPU
        ENABLE OFFSET(0) NUMBITS(1) []
    ],
    MPU_RNR [
        /// Indicates the memory region accessed by MPU_RBAR and MPU_RLAR
        REGION OFFSET(0) NUMBITS(8) []
    ],
    MPU_RBAR [
        /// Contains bits [31:5] of the lower inclusive limit of the selected MPU memory reg
        BASE OFFSET(5) NUMBITS(27) [],
        /// Defines the Shareability domain of this region for Normal memory
        SH OFFSET(3) NUMBITS(2) [],
        /// Defines the access permissions for this region
        AP OFFSET(1) NUMBITS(2) [
            ReadWritePrivilegedOnly = 0b00,
            ReadWrite = 0b01,
            ReadOnlyPrivilegedOnly = 0b10,
            ReadOnly = 0b11
        ],
        /// Defines whether code can be executed from this region
        XN OFFSET(0) NUMBITS(1) [
            Enable = 0,
            Disable = 1
        ]
    ],
    MPU_RLAR [
        /// Contains bits [31:5] of the upper inclusive limit of the selected MPU memory reg
        LIMIT OFFSET(5) NUMBITS(27) [],
        /// Privileged execute-never. Defines whether code can be executed from this privileged region.
        PXN OFFSET(4) NUMBITS(1) [
            Enable = 0,
            Disable = 1,
        ],
        /// Associates a set of attributes in the MPU_MAIR0 and MPU_MAIR1 fields
        ATTRINDX OFFSET(1) NUMBITS(3) [],
        /// Region enable
        ENABLE OFFSET(0) NUMBITS(1) []
    ],
    MPU_RBAR_A1 [
        /// Contains bits [31:5] of the lower inclusive limit of the selected MPU memory reg
        BASE OFFSET(5) NUMBITS(27) [],
        /// Defines the Shareability domain of this region for Normal memory
        SH OFFSET(3) NUMBITS(2) [],
        /// Defines the access permissions for this region
        AP OFFSET(1) NUMBITS(2) [],
        /// Defines whether code can be executed from this region
        XN OFFSET(0) NUMBITS(1) []
    ],
    MPU_RLAR_A1 [
        /// Contains bits [31:5] of the upper inclusive limit of the selected MPU memory reg
        LIMIT OFFSET(5) NUMBITS(27) [],
        /// Associates a set of attributes in the MPU_MAIR0 and MPU_MAIR1 fields
        ATTRINDX OFFSET(1) NUMBITS(3) [],
        /// Region enable
        EN OFFSET(0) NUMBITS(1) []
    ],
    MPU_RBAR_A2 [
        /// Contains bits [31:5] of the lower inclusive limit of the selected MPU memory reg
        BASE OFFSET(5) NUMBITS(27) [],
        /// Defines the Shareability domain of this region for Normal memory
        SH OFFSET(3) NUMBITS(2) [],
        /// Defines the access permissions for this region
        AP OFFSET(1) NUMBITS(2) [],
        /// Defines whether code can be executed from this region
        XN OFFSET(0) NUMBITS(1) []
    ],
    MPU_RLAR_A2 [
        /// Contains bits [31:5] of the upper inclusive limit of the selected MPU memory reg
        LIMIT OFFSET(5) NUMBITS(27) [],
        /// Associates a set of attributes in the MPU_MAIR0 and MPU_MAIR1 fields
        ATTRINDX OFFSET(1) NUMBITS(3) [],
        /// Region enable
        EN OFFSET(0) NUMBITS(1) []
    ],
    MPU_RBAR_A3 [
        /// Contains bits [31:5] of the lower inclusive limit of the selected MPU memory reg
        BASE OFFSET(5) NUMBITS(27) [],
        /// Defines the Shareability domain of this region for Normal memory
        SH OFFSET(3) NUMBITS(2) [],
        /// Defines the access permissions for this region
        AP OFFSET(1) NUMBITS(2) [],
        /// Defines whether code can be executed from this region
        XN OFFSET(0) NUMBITS(1) []
    ],
    MPU_RLAR_A3 [
        /// Contains bits [31:5] of the upper inclusive limit of the selected MPU memory reg
        LIMIT OFFSET(5) NUMBITS(27) [],
        /// Associates a set of attributes in the MPU_MAIR0 and MPU_MAIR1 fields
        ATTRINDX OFFSET(1) NUMBITS(3) [],
        /// Region enable
        EN OFFSET(0) NUMBITS(1) []
    ],
    MPU_MAIR0 [
        /// Memory attribute encoding for MPU regions with an AttrIndex of 3
        ATTR3 OFFSET(24) NUMBITS(8) [],
        /// Memory attribute encoding for MPU regions with an AttrIndex of 2
        ATTR2 OFFSET(16) NUMBITS(8) [],
        /// Memory attribute encoding for MPU regions with an AttrIndex of 1
        ATTR1 OFFSET(8) NUMBITS(8) [],
        /// Memory attribute encoding for MPU regions with an AttrIndex of 0
        ATTR0 OFFSET(0) NUMBITS(8) []
    ],
    MPU_MAIR1 [
        /// Memory attribute encoding for MPU regions with an AttrIndex of 7
        ATTR7 OFFSET(24) NUMBITS(8) [],
        /// Memory attribute encoding for MPU regions with an AttrIndex of 6
        ATTR6 OFFSET(16) NUMBITS(8) [],
        /// Memory attribute encoding for MPU regions with an AttrIndex of 5
        ATTR5 OFFSET(8) NUMBITS(8) [],
        /// Memory attribute encoding for MPU regions with an AttrIndex of 4
        ATTR4 OFFSET(0) NUMBITS(8) []
    ],
];

/// Function to align a pointer to 32 bytes.
fn align32(initial_ptr: *const u8) -> Result<*const u8, ()> {
    let memory_offset = initial_ptr.align_offset(32);
    if memory_offset == usize::MAX {
        return Err(());
    }

    let aligned_ptr = initial_ptr.wrapping_add(memory_offset);
    Ok(aligned_ptr)
}

/// State related to the real physical MPU.
///
/// There should only be one instantiation of this object as it represents
/// real hardware.
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

impl<const NUM_REGIONS: usize> MPU<NUM_REGIONS> {
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
        self.registers.ctrl.write(MPU_CTRL::ENABLE::CLEAR);
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
const APP_MEMORY_REGION_MAX_NUM: usize = 0;

impl<const NUM_REGIONS: usize> fmt::Display for CortexMConfig<NUM_REGIONS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\r\n Cortex-M MPU")?;
        for (i, region) in self.regions.iter().enumerate() {
            if let Some(location) = region.location {
                let access_bits = region.rbar_value.read(MPU_RBAR::AP);
                let access_str = match access_bits {
                    0b00 => "ReadWritePrivilegedOnly",
                    0b01 => "ReadWrite",
                    0b10 => "ReadOnlyPrivilegedOnly",
                    0b11 => "ReadOnly",
                    _ => "ERR",
                };
                let start = location.0 as usize;
                let end = location.1 as usize;
                write!(
                    f,
                    "\
                     \r\n  Region {}: [{:#010X}:{:#010X}], length: {} bytes; {} ({:#x})",
                    i,
                    start,
                    end,
                    end - start,
                    access_str,
                    access_bits,
                )?;
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
            if number == APP_MEMORY_REGION_MAX_NUM {
                continue;
            }
            if region.location.is_none() {
                return Some(number);
            }
        }
        None
    }
}

/// Struct storing configuration for a Cortex-M MPU region.
#[derive(Copy, Clone)]
pub struct CortexMRegion {
    location: Option<(*const u8, *const u8)>,
    rbar_value: FieldValue<u32, MPU_RBAR::Register>,
    rlar_value: FieldValue<u32, MPU_RLAR::Register>,
    region_num: usize,
}

impl PartialEq<mpu::Region> for CortexMRegion {
    fn eq(&self, other: &mpu::Region) -> bool {
        self.location.is_some_and(|(start, end)| {
            core::ptr::eq(start, other.start_address())
                && (end as usize - start as usize) == other.size()
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
        permissions: mpu::Permissions,
    ) -> Option<CortexMRegion> {
        // Logical size must be above minimum size for cortexM MPU regions and
        // and less than the size of the underlying physical region
        if logical_size < CORTEXM_MIN_REGION_SIZE || region_size < logical_size {
            return None;
        }

        // Determine access and execute permissions
        let (access, execute) = match permissions {
            mpu::Permissions::ReadWriteExecute => (MPU_RBAR::AP::ReadWrite, MPU_RBAR::XN::Enable),
            mpu::Permissions::ReadWriteOnly => (MPU_RBAR::AP::ReadWrite, MPU_RBAR::XN::Disable),
            mpu::Permissions::ReadExecuteOnly => (MPU_RBAR::AP::ReadOnly, MPU_RBAR::XN::Enable),
            mpu::Permissions::ReadOnly => (MPU_RBAR::AP::ReadOnly, MPU_RBAR::XN::Disable),
            mpu::Permissions::ExecuteOnly => {
                (MPU_RBAR::AP::ReadOnlyPrivilegedOnly, MPU_RBAR::XN::Enable)
            }
        };

        // Base Address register
        let rbar_value = MPU_RBAR::BASE.val((logical_start as u32) >> 5)
            + MPU_RBAR::SH.val(0)
            + access
            + execute;

        let logical_end = logical_start as usize + logical_size;

        // The end address must be aligned to 32 bytes.
        if logical_end % 32 != 0 {
            return None;
        }

        // Limit Address register
        let rlar_value = MPU_RLAR::ENABLE::SET
            + MPU_RLAR::LIMIT.val((logical_end as u32) >> 5)
            + MPU_RLAR::PXN::Disable
            + MPU_RLAR::ATTRINDX.val(0);

        Some(CortexMRegion {
            location: Some((region_start, region_start.wrapping_add(region_size))),
            rbar_value,
            rlar_value,
            region_num,
        })
    }

    fn empty(region_num: usize) -> CortexMRegion {
        CortexMRegion {
            location: None,
            rbar_value: MPU_RBAR::BASE.val(0),
            rlar_value: MPU_RLAR::ENABLE::CLEAR,
            region_num,
        }
    }

    fn overlaps(&self, other_start: *const u8, other_size: usize) -> bool {
        let other_start = other_start as usize;
        let other_end = other_start + other_size;

        let (region_start, region_end) = match self.location {
            Some((region_start, region_end)) => {
                let region_start = region_start as usize;
                let region_end = region_end as usize;
                (region_start, region_end)
            }
            None => return false,
        };

        region_start < other_end && other_start < region_end
    }
}

impl<const NUM_REGIONS: usize> mpu::MPU for MPU<NUM_REGIONS> {
    type MpuConfig = CortexMConfig<NUM_REGIONS>;

    fn enable_app_mpu(&self) {
        // Enable the MPU, disable it during HardFault/NMI handlers, and allow
        // privileged code access to all unprotected memory.
        self.registers
            .ctrl
            .write(MPU_CTRL::ENABLE::SET + MPU_CTRL::HFNMIENA::CLEAR + MPU_CTRL::PRIVDEFENA::SET);
    }

    fn disable_app_mpu(&self) {
        // The MPU is not enabled for privileged mode, so we don't have to do
        // anything
        self.registers.ctrl.write(MPU_CTRL::ENABLE::CLEAR);
    }

    fn number_total_regions(&self) -> usize {
        self.registers.mpu_type.read(MPU_TYPE::DREGION) as usize
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
        let mut region_calculation = || {
            // Check that no previously allocated regions overlap the unallocated memory.
            for region in config.regions.iter() {
                if region.overlaps(unallocated_memory_start, unallocated_memory_size) {
                    return Err(());
                }
            }

            let region_num = config.unused_region_number().ok_or(())?;

            let region_start = align32(unallocated_memory_start)?;
            let region_end = align32(region_start.wrapping_add(min_region_size))?;
            let region_size = unsafe { region_end.offset_from(region_start) };

            // Check for overflow
            if region_size < 0 {
                return Err(());
            }

            // Make sure the region fits in the unallocated memory.
            if region_size as usize > unallocated_memory_size {
                return Err(());
            }

            let region = CortexMRegion::new(
                region_start,
                region_size as usize,
                region_start,
                region_size as usize,
                region_num,
                permissions,
            )
            .ok_or(())?;

            config.regions[region_num] = region;
            config.is_dirty.set(true);

            Ok(mpu::Region::new(region_start, region_size as usize))
        };

        region_calculation().ok()
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

        if idx == APP_MEMORY_REGION_MAX_NUM {
            return Err(());
        }

        config.regions[idx] = CortexMRegion::empty(idx);
        config.is_dirty.set(true);

        Ok(())
    }

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
        let mut region_calculation = || {
            // Check that no previously allocated regions overlap the unallocated
            // memory.
            for region in config.regions.iter() {
                if region.overlaps(unallocated_memory_start, unallocated_memory_size) {
                    return Err(());
                }
            }

            // Make sure there is enough memory for app memory and kernel memory.
            let memory_size = cmp::max(
                min_memory_size,
                initial_app_memory_size + initial_kernel_memory_size,
            );

            // The region should start as close as possible to the start of the
            // unallocated memory.
            let region_start = align32(unallocated_memory_start)?;
            let region_end = align32(region_start.wrapping_add(memory_size))?;
            let region_size = unsafe { region_end.offset_from(region_start) };

            // Check for overflow
            if region_size < 0 {
                return Err(());
            }

            // Make sure the region fits in the unallocated memory.
            if region_size as usize > unallocated_memory_size {
                return Err(());
            }

            let logical_start = region_start;
            let logical_end = align32(logical_start.wrapping_add(initial_app_memory_size))?;
            let logical_size = unsafe { logical_end.offset_from(logical_start) };

            // Check for overflow
            if logical_size < 0 {
                return Err(());
            }

            let region = CortexMRegion::new(
                logical_start,
                logical_size as usize,
                region_start,
                region_size as usize,
                0,
                permissions,
            )
            .ok_or(())?;

            config.regions[0] = region;
            config.is_dirty.set(true);

            Ok((region_start, memory_size))
        };

        region_calculation().ok()
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
        let (region_start, region_end) = config.regions[0].location.ok_or(())?;
        // .map(|(region_start, region_end)| (region_start as usize, region_end as usize))?;

        // Check if the memory breaks are out of the allocated region.
        if (app_memory_break as usize) < (region_start as usize)
            || (app_memory_break as usize) >= (region_end as usize)
        {
            return Err(());
        }

        if (kernel_memory_break as usize) < (region_start as usize)
            || (kernel_memory_break as usize) >= (region_end as usize)
        {
            return Err(());
        }

        // Out of memory
        if (app_memory_break as usize) > (kernel_memory_break as usize) {
            return Err(());
        }

        let logical_start = region_start;
        let logical_end = align32(app_memory_break)?;
        let logical_size = unsafe { logical_end.offset_from(logical_start) };

        // Check for overflow
        if logical_size < 0 {
            return Err(());
        }

        // Check if the aligned memory doesn't go over the grants.
        if (logical_end as usize) > (kernel_memory_break as usize) {
            return Err(());
        }

        let region_size = unsafe { region_end.offset_from(region_start) };

        let region = CortexMRegion::new(
            logical_start,
            logical_size as usize,
            region_start,
            region_size as usize,
            0,
            permissions,
        )
        .ok_or(())?;

        config.regions[0] = region;
        config.is_dirty.set(true);

        Ok(())
    }

    fn configure_mpu(&self, config: &Self::MpuConfig) {
        // Set ATTR0 to Normal Memory, Outer and Inner Non-cacheable.
        self.registers
            .mair0
            .modify(MPU_MAIR0::ATTR0.val(0b0100_0100));
        // If the hardware is already configured for this app and the app's MPU
        // configuration has not changed, then skip the hardware update.
        if !self.hardware_is_configured_for.contains(&config.id) || config.is_dirty.get() {
            // Set MPU regions
            for region in config.regions.iter() {
                self.registers
                    .rnr
                    .modify(MPU_RNR::REGION.val(region.region_num as u32));
                self.registers.rbar.write(region.rbar_value);
                self.registers.rlar.write(region.rlar_value);
            }
            self.hardware_is_configured_for.set(config.id);
            config.is_dirty.set(false);
        }
    }
}
