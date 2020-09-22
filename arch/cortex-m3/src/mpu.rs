//! Implementation of the memory protection unit for the Cortex-M3 and
//! Cortex-M4.

use core::cell::Cell;
use core::cmp;
use core::fmt;
use kernel;
use kernel::common::cells::OptionalCell;
use kernel::common::math;
use kernel::common::registers::{register_bitfields, FieldValue, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::mpu;
use kernel::AppId;

/// MPU Registers for the Cortex-M3 and Cortex-M4 families
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

const MPU_BASE_ADDRESS: StaticRef<MpuRegisters> =
    unsafe { StaticRef::new(0xE000ED90 as *const MpuRegisters) };

/// State related to the real physical MPU.
///
/// There should only be one instantiation of this object as it represents
/// real hardware.
pub struct MPU {
    /// MMIO reference to MPU registers.
    registers: StaticRef<MpuRegisters>,
    /// Optimization logic. This is used to indicate which application the MPU
    /// is currently configured for so that the MPU can skip updating when the
    /// kernel returns to the same app.
    hardware_is_configured_for: OptionalCell<AppId>,
}

impl MPU {
    pub const unsafe fn new() -> MPU {
        MPU {
            registers: MPU_BASE_ADDRESS,
            hardware_is_configured_for: OptionalCell::empty(),
        }
    }
}

/// Per-process struct storing MPU configuration for cortex-m MPUs.
///
/// The cortex-m MPU has eight regions, all of which must be configured (though
/// unused regions may be configured as disabled). This struct caches the result
/// of region configuration calculation
pub struct CortexMConfig {
    /// The computed region configuration for this process.
    regions: [CortexMRegion; 8],
    /// Has the configuration changed since the last time the this process
    /// configuration was written to hardware?
    is_dirty: Cell<bool>,
}

const APP_MEMORY_REGION_NUM: usize = 0;

impl Default for CortexMConfig {
    fn default() -> CortexMConfig {
        CortexMConfig {
            regions: [
                CortexMRegion::empty(0),
                CortexMRegion::empty(1),
                CortexMRegion::empty(2),
                CortexMRegion::empty(3),
                CortexMRegion::empty(4),
                CortexMRegion::empty(5),
                CortexMRegion::empty(6),
                CortexMRegion::empty(7),
            ],
            is_dirty: Cell::new(true),
        }
    }
}

impl fmt::Display for CortexMConfig {
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

impl CortexMConfig {
    fn unused_region_number(&self) -> Option<usize> {
        for (number, region) in self.regions.iter().enumerate() {
            if number == APP_MEMORY_REGION_NUM {
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

impl CortexMRegion {
    fn new(
        logical_start: *const u8,
        logical_size: usize,
        region_start: *const u8,
        region_size: usize,
        region_num: usize,
        subregions: Option<(usize, usize)>,
        permissions: mpu::Permissions,
    ) -> CortexMRegion {
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
            let mask = (min_subregion..=max_subregion).fold(u8::max_value(), |res, i| {
                // Enable subregions bit by bit (1 ^ 1 == 0)
                res ^ (1 << i)
            });
            attributes += RegionAttributes::SRD.val(mask as u32);
        }

        CortexMRegion {
            location: Some((logical_start, logical_size)),
            base_address: base_address,
            attributes: attributes,
        }
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

        if region_start < other_end && other_start < region_end {
            true
        } else {
            false
        }
    }
}

impl kernel::mpu::MPU for MPU {
    type MpuConfig = CortexMConfig;

    fn clear_mpu(&self) {
        self.registers.ctrl.write(Control::ENABLE::CLEAR);
    }

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

        // Region start always has to align to 32 bytes
        if start % 32 != 0 {
            start += 32 - (start % 32);
        }

        // Regions must be at least 32 bytes
        if size < 32 {
            size = 32;
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
                    (1 as usize) << tz
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
        );

        config.regions[region_num] = region;
        config.is_dirty.set(true);

        Some(mpu::Region::new(start as *const u8, size))
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
        // Check that no previously allocated regions overlap the unallocated memory.
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

        // Size must be a power of two, so: https://www.youtube.com/watch?v=ovo6zwv6DX4
        let mut region_size = math::closest_power_of_two(memory_size as u32) as usize;
        let exponent = math::log_base_two(region_size as u32);

        if exponent < 8 {
            // Region sizes must be 256 Bytes or larger in order to support subregions
            region_size = 256;
        } else if exponent > 32 {
            // Region sizes must be 4GB or smaller
            return None;
        }

        // The region should start as close as possible to the start of the unallocated memory.
        let mut region_start = unallocated_memory_start as usize;

        // If the start and length don't align, move region up until it does
        if region_start % region_size != 0 {
            region_start += region_size - (region_start % region_size);
        }

        // We allocate an MPU region exactly over the process memory block, and we disable
        // subregions at the end of this region to disallow access to the memory past the app
        // break. As the app break later increases, we will be able to linearly grow
        // the logical region covering app-owned memory by enabling more and more subregions.
        // The Cortex-M MPU supports 8 subregions, so the size of this logical region is always a
        // multiple of an eighth of the MPU region length.

        // Determine the number of subregions to enable.
        let mut num_subregions_used = {
            if initial_kernel_memory_size == 0 {
                8
            } else {
                initial_app_memory_size * 8 / region_size + 1
            }
        };

        let subregion_size = region_size / 8;

        // Calculates the end address of the enabled subregions and the initial kernel memory break.
        let subregions_end = region_start + num_subregions_used * subregion_size;
        let kernel_memory_break = region_start + region_size - initial_kernel_memory_size;

        // If the last subregion covering app-owned memory overlaps the start of kernel-owned
        // memory, we make the entire process memory block twice as big so there is plenty of space
        // between app-owned and kernel-owned memory.
        if subregions_end > kernel_memory_break {
            region_size *= 2;

            if region_start % region_size != 0 {
                region_start += region_size - (region_start % region_size);
            }

            num_subregions_used = {
                if initial_kernel_memory_size == 0 {
                    8
                } else {
                    initial_app_memory_size * 8 / region_size + 1
                }
            };
        }

        // Make sure the region fits in the unallocated memory.
        if region_start + region_size
            > (unallocated_memory_start as usize) + unallocated_memory_size
        {
            return None;
        }

        let region = CortexMRegion::new(
            region_start as *const u8,
            region_size,
            region_start as *const u8,
            region_size,
            APP_MEMORY_REGION_NUM,
            Some((0, num_subregions_used - 1)),
            permissions,
        );

        config.regions[APP_MEMORY_REGION_NUM] = region;
        config.is_dirty.set(true);

        Some((region_start as *const u8, region_size))
    }

    fn update_app_memory_region(
        &self,
        app_memory_break: *const u8,
        kernel_memory_break: *const u8,
        permissions: mpu::Permissions,
        config: &mut Self::MpuConfig,
    ) -> Result<(), ()> {
        let (region_start, region_size) = match config.regions[APP_MEMORY_REGION_NUM].location() {
            Some((start, size)) => (start as usize, size),
            None => {
                // Error: Process tried to update app memory MPU region before it was created.
                return Err(());
            }
        };

        let app_memory_break = app_memory_break as usize;
        let kernel_memory_break = kernel_memory_break as usize;

        // Out of memory
        if app_memory_break > kernel_memory_break {
            return Err(());
        }

        let app_memory_size = app_memory_break - region_start;
        let kernel_memory_size = region_start + region_size - kernel_memory_break;

        // Determine the number of subregions to enable.
        let num_subregions_used = {
            if kernel_memory_size == 0 {
                8
            } else {
                app_memory_size * 8 / region_size + 1
            }
        };

        let subregion_size = region_size / 8;
        let subregions_end = region_start + subregion_size * num_subregions_used;

        // If we can no longer cover app memory with an MPU region without overlapping kernel
        // memory, we fail.
        if subregions_end > kernel_memory_break {
            return Err(());
        }

        let region = CortexMRegion::new(
            region_start as *const u8,
            region_size,
            region_start as *const u8,
            region_size,
            APP_MEMORY_REGION_NUM,
            Some((0, num_subregions_used - 1)),
            permissions,
        );

        config.regions[APP_MEMORY_REGION_NUM] = region;
        config.is_dirty.set(true);

        Ok(())
    }

    fn configure_mpu(&self, config: &Self::MpuConfig, app_id: &AppId) {
        // If the hardware is already configured for this app and the app's MPU
        // configuration has not changed, then skip the hardware update.
        if !self.hardware_is_configured_for.contains(app_id) || config.is_dirty.get() {
            // Set MPU regions
            for region in config.regions.iter() {
                self.registers.rbar.write(region.base_address());
                self.registers.rasr.write(region.attributes());
            }
            self.hardware_is_configured_for.set(*app_id);
            config.is_dirty.set(false);
        }
    }
}
