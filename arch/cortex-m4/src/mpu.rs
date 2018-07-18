//! Implementation of the ARM memory protection unit.

use kernel;
use kernel::common::math::PowerOfTwo;
use kernel::common::registers::{ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::mpu::{Permission, Region};
use kernel::ReturnCode;

#[repr(C)]
/// MPU Registers for the Cortex-M4 family
///
/// Described in section 4.5 of
/// <http://infocenter.arm.com/help/topic/com.arm.doc.dui0553a/DUI0553A_cortex_m4_dgug.pdf>
pub struct MpuRegisters {
    pub mpu_type: ReadOnly<u32, Type::Register>,
    pub ctrl: ReadWrite<u32, Control::Register>,
    pub rnr: ReadWrite<u32, RegionNumber::Register>,
    pub rbar: ReadWrite<u32, RegionBaseAddress::Register>,
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

    fn allocate_region(&self, region: &Region, region_num: usize) -> ReturnCode {
        let regs = &*self.0;

        if region_num >= 8 {
            // There are only 8 (0-indexed) regions available
            return ReturnCode::FAIL;
        }

        let start = region.get_start();
        let len = region.get_len();
        let read = region.get_read_permission();
        let write = region.get_write_permission();
        let execute = region.get_execute_permission();

        let region_value = (region_num & 0xf) as u32;

        // Empty region
        if len == 0 {
            regs.rbar.write(
                RegionBaseAddress::VALID::UseRBAR + RegionBaseAddress::REGION.val(region_value),
            );
            regs.rasr.set(0);
            return ReturnCode::SUCCESS;
        }

        // Convert execute permission to a bitfield
        let execute_value = match execute {
            Permission::NoAccess => RegionAttributes::XN::Disable,
            Permission::Full => RegionAttributes::XN::Enable,
            _ => {
                return ReturnCode::FAIL;
            } // Not supported
        };

        // Convert read & write permissions to bitfields
        let access_value = match read {
            Permission::NoAccess => RegionAttributes::AP::NoAccess,
            Permission::PrivilegedOnly => {
                match write {
                    Permission::NoAccess => RegionAttributes::AP::PrivilegedOnlyReadOnly,
                    Permission::PrivilegedOnly => RegionAttributes::AP::PrivilegedOnly,
                    _ => {
                        return ReturnCode::FAIL;
                    } // Not supported
                }
            }
            Permission::Full => match write {
                Permission::NoAccess => RegionAttributes::AP::ReadOnly,
                Permission::PrivilegedOnly => RegionAttributes::AP::UnprivilegedReadOnly,
                Permission::Full => RegionAttributes::AP::ReadWrite,
            },
        };

        // There are two possibilities we support:
        //
        // 1. The base address is aligned exactly to the size of the region,
        //    which uses an MPU region with the exact base address and size of
        //    the memory region.
        //
        // 2. Otherwise, we can use a larger MPU region and expose only MPU
        //    subregions, as long as the memory region's base address is aligned
        //    to 1/8th of a larger region size.

        // Possibility 1
        if start % len == 0 {
            // Memory base aligned to memory size - straight forward case
            let region_len = PowerOfTwo::floor(len as u32);

            // exponent = log2(region_len)
            let exponent = region_len.exp::<u32>();

            if exponent < 5 {
                // Region sizes must be 32 Bytes or larger
                return ReturnCode::FAIL;
            } else if exponent > 32 {
                // Region sizes must be 4GB or smaller
                return ReturnCode::FAIL;
            }

            let address_value = (start >> 5) as u32;
            let region_len_value = exponent - 1;

            regs.rbar.write(
                RegionBaseAddress::ADDR.val(address_value)
                    + RegionBaseAddress::VALID::UseRBAR
                    + RegionBaseAddress::REGION.val(region_value),
            );

            regs.rasr.write(
                RegionAttributes::ENABLE::SET
                    + RegionAttributes::SIZE.val(region_len_value)
                    + access_value
                    + execute_value,
            );
        }
        // Possibility 2
        else {
            // Memory base not aligned to memory size

            // Which (power-of-two) subregion size would align with the base
            // address?
            //
            // We find this by taking smallest binary substring of the base
            // address with exactly one bit:
            //
            //      1 << (start.trailing_zeros())
            let subregion_size = {
                let tz = start.trailing_zeros();
                // `start` should never be 0 because of that's taken care of by
                // the previous branch, but in case it is, do the right thing
                // anyway.
                if tz < 32 {
                    (1 as usize) << tz
                } else {
                    0
                }
            };

            // Once we have a subregion size, we get a region size by
            // multiplying it by the number of subregions per region.
            let region_size = subregion_size * 8;
            // Finally, we calculate the region base by finding the nearest
            // address below `start` that aligns with the region size.
            let region_start = start - (start % region_size);

            if region_size + region_start - start < len {
                // Sanity check that the amount left over space in the region
                // after `start` is at least as large as the memory region we
                // want to reference.
                return ReturnCode::FAIL;
            }
            if len % subregion_size != 0 {
                // Sanity check that there is some integer X such that
                // subregion_size * X == len so none of `len` is left over when
                // we take the max_subregion.
                return ReturnCode::FAIL;
            }

            // The index of the first subregion to activate is the number of
            // regions between `region_start` (MPU) and `start` (memory).
            let min_subregion = (start - region_start) / subregion_size;
            // The index of the last subregion to activate is the number of
            // regions that fit in `len`, plus the `min_subregion`, minus one
            // (because subregions are zero-indexed).
            let max_subregion = min_subregion + len / subregion_size - 1;

            let region_len = PowerOfTwo::floor(region_size as u32);
            // exponent = log2(region_len)
            let exponent = region_len.exp::<u32>();
            if exponent < 7 {
                // Subregions only supported for regions sizes 128 bytes and up.
                return ReturnCode::FAIL;
            } else if exponent > 32 {
                // Region sizes must be 4GB or smaller
                return ReturnCode::FAIL;
            }

            // Turn the min/max subregion into a bitfield where all bits are `1`
            // except for the bits whose index lie within
            // [min_subregion, max_subregion]
            //
            // Note: Rust ranges are minimum inclusive, maximum exclusive, hence
            // max_subregion + 1.
            let subregion_mask =
                (min_subregion..(max_subregion + 1)).fold(!0, |res, i| res & !(1 << i)) & 0xff;

            let address_value = (region_start >> 5) as u32;
            let region_len_value = exponent - 1;

            regs.rbar.write(
                RegionBaseAddress::ADDR.val(address_value)
                    + RegionBaseAddress::VALID::UseRBAR
                    + RegionBaseAddress::REGION.val(region_value),
            );

            regs.rasr.write(
                RegionAttributes::ENABLE::SET
                    + RegionAttributes::SRD.val(subregion_mask)
                    + RegionAttributes::SIZE.val(region_len_value)
                    + access_value
                    + execute_value,
            );
        }
        ReturnCode::SUCCESS
    }
}

impl kernel::mpu::MPU for MPU {
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

    fn num_supported_regions(&self) -> u32 {
        let regs = &*self.0;
        regs.mpu_type.read(Type::DREGION)
    }

    fn allocate_regions(&self, regions: &[Region]) -> Result<(), usize> {
        for (index, region) in regions.iter().enumerate() {
            if let ReturnCode::FAIL = self.allocate_region(region, index) {
                return Err(index);
            }
        }
        Ok(())
    }
}
