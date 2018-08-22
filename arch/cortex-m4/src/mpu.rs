//! Implementation of the ARM memory protection unit.

use kernel;
use kernel::common::math;
use kernel::common::registers::{FieldValue, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::mpu::Permissions;

/// MPU Registers for the Cortex-M4 family
///
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

/// Constructor field is private to limit who can create a new MPU
pub struct MPU(StaticRef<MpuRegisters>);

impl MPU {
    pub const unsafe fn new() -> MPU {
        MPU(MPU_BASE_ADDRESS)
    }
}

#[derive(Copy, Clone)]
pub struct CortexMConfig {
    memory_info: Option<ProcessMemoryInfo>,
    regions: [Region; 8],
    next_region_num: usize,
}

const APP_MEMORY_REGION_NUM: usize = 0;

impl Default for CortexMConfig {
    fn default() -> CortexMConfig {
        CortexMConfig {
            memory_info: None,
            regions: [
                Region::empty(0),
                Region::empty(1),
                Region::empty(2),
                Region::empty(3),
                Region::empty(4),
                Region::empty(5),
                Region::empty(6),
                Region::empty(7),
            ],
            next_region_num: 1, // Region 0 is reserved for app memory
        }
    }
}

#[derive(Copy, Clone)]
pub struct ProcessMemoryInfo {
    memory_start: *const u8,
    memory_size: usize,
    permissions: Permissions,
}

#[derive(Copy, Clone)]
pub struct Region {
    base_address: FieldValue<u32, RegionBaseAddress::Register>,
    attributes: FieldValue<u32, RegionAttributes::Register>,
}

impl Region {
    fn new(
        address: u32,
        size: u32,
        region_num: u32,
        subregion_mask: Option<u32>,
        permissions: Permissions,
    ) -> Region {
        // Determine access and execute permissions
        let (access, execute) = match permissions {
            Permissions::ReadWriteExecute => (
                RegionAttributes::AP::ReadWrite,
                RegionAttributes::XN::Enable,
            ),
            Permissions::ReadWriteOnly => (
                RegionAttributes::AP::ReadWrite,
                RegionAttributes::XN::Disable,
            ),
            Permissions::ReadExecuteOnly => {
                (RegionAttributes::AP::ReadOnly, RegionAttributes::XN::Enable)
            }
            Permissions::ReadOnly => (
                RegionAttributes::AP::ReadOnly,
                RegionAttributes::XN::Disable,
            ),
            Permissions::ExecuteOnly => {
                (RegionAttributes::AP::NoAccess, RegionAttributes::XN::Enable)
            }
        };

        // Base address register
        let base_address = RegionBaseAddress::ADDR.val(address >> 5)
            + RegionBaseAddress::VALID::UseRBAR
            + RegionBaseAddress::REGION.val(region_num);

        let size = math::log_base_two(size) - 1;

        // Attributes register
        let mut attributes =
            RegionAttributes::ENABLE::SET + RegionAttributes::SIZE.val(size) + access + execute;

        // If using subregions, add the mask
        if let Some(mask) = subregion_mask {
            attributes += RegionAttributes::SRD.val(mask);
        }

        Region {
            base_address,
            attributes,
        }
    }

    fn empty(region_num: u32) -> Region {
        Region {
            base_address: RegionBaseAddress::VALID::UseRBAR
                + RegionBaseAddress::REGION.val(region_num),
            attributes: RegionAttributes::ENABLE::CLEAR,
        }
    }
}

impl kernel::mpu::MPU for MPU {
    type MpuConfig = CortexMConfig;

    fn enable_mpu(&self) {
        let regs = &*self.0;

        // Enable the MPU, disable it during HardFault/NMI handlers, and allow
        // privileged code access to all unprotected memory.
        regs.ctrl
            .write(Control::ENABLE::SET + Control::HFNMIENA::CLEAR + Control::PRIVDEFENA::SET);
    }

    fn disable_mpu(&self) {
        let regs = &*self.0;
        regs.ctrl.write(Control::ENABLE::CLEAR);
    }

    fn number_total_regions(&self) -> usize {
        let regs = &*self.0;
        regs.mpu_type.read(Type::DREGION) as usize
    }

    fn allocate_region(
        &self,
        parent_start: *const u8,
        parent_size: usize,
        min_region_size: usize,
        permissions: Permissions,
        config: &mut Self::MpuConfig,
    ) -> Option<(*const u8, usize)> {
        let region_num = config.next_region_num;

        // Cortex-M only supports 8 regions
        if region_num >= 8 {
            return None;
        }

        // Logical region
        let mut start = parent_start as usize;
        let mut size = min_region_size;

        // Physical MPU region
        let mut region_start = start;
        let mut region_size = size;
        let mut subregion_mask = None;

        // Region start always has to align to 32 bytes
        if start % 32 != 0 {
            start += 32 - (start % 32);
        }

        // Regions must be at least 32 bytes
        if size < 32 {
            size = 32;
        }

        // We can only create an MPU region if the size is a power of two and it divides
        // the start address. If this is not the case, the first thing we try to do to
        // cover the memory region is to use a larger MPU region and expose certain subregions.
        if size.count_ones() > 1 || start % size != 0 {
            // Which (power-of-two) subregion size would align with the base
            // address?
            //
            // We find this by taking smallest binary substring of the base
            // address with exactly one bit:
            //
            //      1 << (start.trailing_zeros())
            let subregion_size = {
                let tz = start.trailing_zeros();
                if tz < 32 {
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

                // Turn the min/max subregion into a bitfield where all bits are `1`
                // except for the bits whose index lie within
                // [min_subregion, max_subregion]
                //
                // Note: Rust ranges are minimum inclusive, maximum exclusive, hence
                // max_subregion + 1.
                let mask =
                    (min_subregion..(max_subregion + 1)).fold(!0, |res, i| res & !(1 << i)) & 0xff;

                region_start = underlying_region_start;
                region_size = underlying_region_size;
                subregion_mask = Some(mask);
            } else {
                // In this case, we can't use subregions to solve the alignment
                // problem. Instead, we will round up `size` to a power of two and
                // shift `start` up in memory to make it align with `size`.
                size = math::closest_power_of_two(size as u32) as usize;
                start += size - (start % size);

                region_start = start;
                region_size = size;
            }
        }

        // Regions can't be greater than 4 GB.
        if math::log_base_two(size as u32) >= 32 {
            return None;
        }

        // Check that our region fits in memory.
        if start + size > (parent_start as usize) + parent_size {
            return None;
        }

        let region = Region::new(
            region_start as u32,
            region_size as u32,
            region_num as u32,
            subregion_mask,
            permissions,
        );

        config.regions[region_num] = region;
        config.next_region_num += 1;

        Some((start as *const u8, size))
    }

    fn allocate_app_memory_region(
        &self,
        parent_start: *const u8,
        parent_size: usize,
        min_memory_size: usize,
        initial_app_memory_size: usize,
        initial_kernel_memory_size: usize,
        permissions: Permissions,
        config: &mut Self::MpuConfig,
    ) -> Option<(*const u8, usize)> {
        // Make sure there is enough memory for app memory and kernel memory
        let memory_size = {
            if min_memory_size < initial_app_memory_size + initial_kernel_memory_size {
                initial_app_memory_size + initial_kernel_memory_size
            } else {
                min_memory_size
            }
        };

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

        // Ideally, the region will start at the start of the parent memory block
        let mut region_start = parent_start as usize;

        // If the start and length don't align, move region up until it does
        if region_start % region_size != 0 {
            region_start += region_size - (region_start % region_size);
        }

        // The memory initially allocated for app memory will be aligned to an eigth of the total region length.
        // This allows Cortex-M subregions to cover incrementally growing app memory in linear way.
        // The Cortex-M has a total of 8 subregions per region, which is why we can have precision in
        // eights of total region lengths.
        //
        // For example: subregions_used = (3500 * 8)/8192 + 1 = 4;
        let mut subregions_used = (initial_app_memory_size * 8) / region_size + 1;

        let kernel_memory_break = region_start + region_size - initial_kernel_memory_size;
        let subregion_size = region_size / 8;
        let subregions_end = region_start + subregions_used * subregion_size;

        // If the last subregion for app memory overlaps the start of kernel
        // memory, we can fix this by doubling the region size.
        if subregions_end > kernel_memory_break {
            region_size *= 2;
            if region_start % region_size != 0 {
                region_start += region_size - (region_start % region_size);
            }
            subregions_used = (initial_app_memory_size * 8) / region_size + 1;
        }

        // Make sure the region fits in the parent memory block
        if region_start + region_size > (parent_start as usize) + parent_size {
            return None;
        }

        // For example: 11111111 & 11111110 = 11111110 --> Use only the first subregion (0 = enable)
        let subregion_mask = (0..subregions_used).fold(!0, |res, i| res & !(1 << i)) & 0xff;

        let memory_info = ProcessMemoryInfo {
            memory_start: region_start as *const u8,
            memory_size: region_size,
            permissions: permissions,
        };

        let region = Region::new(
            region_start as u32,
            region_size as u32,
            APP_MEMORY_REGION_NUM as u32,
            Some(subregion_mask),
            permissions,
        );

        config.memory_info = Some(memory_info);
        config.regions[APP_MEMORY_REGION_NUM] = region;

        Some((region_start as *const u8, region_size))
    }

    fn update_app_memory_region(
        &self,
        app_memory_break: *const u8,
        kernel_memory_break: *const u8,
        config: &mut Self::MpuConfig,
    ) -> Result<(), ()> {
        let (region_start, region_size, permissions) = match config.memory_info {
            Some(memory_info) => (
                memory_info.memory_start as u32,
                memory_info.memory_size as u32,
                memory_info.permissions,
            ),
            None => panic!(
                "Error: Process tried to update app memory MPU region before it was created."
            ),
        };

        let app_memory_break = app_memory_break as u32;
        let kernel_memory_break = kernel_memory_break as u32;

        // Out of memory
        if app_memory_break > kernel_memory_break {
            return Err(());
        }

        let app_memory_size = app_memory_break - region_start;

        let num_subregions_used = (app_memory_size * 8) / region_size + 1;

        // We can no longer cover app memory with an MPU region without overlapping kernel memory
        let subregion_size = region_size / 8;
        let subregions_end = region_start + subregion_size * num_subregions_used;
        if subregions_end > kernel_memory_break {
            return Err(());
        }

        let subregion_mask = (0..num_subregions_used).fold(!0, |res, i| res & !(1 << i)) & 0xff;

        let region = Region::new(
            region_start,
            region_size,
            APP_MEMORY_REGION_NUM as u32,
            Some(subregion_mask),
            permissions,
        );

        config.regions[APP_MEMORY_REGION_NUM] = region;

        Ok(())
    }

    fn configure_mpu(&self, config: &Self::MpuConfig) {
        let regs = &*self.0;

        // Set MPU regions
        for region in config.regions.iter() {
            regs.rbar.write(region.base_address);
            regs.rasr.write(region.attributes);
        }
    }
}
