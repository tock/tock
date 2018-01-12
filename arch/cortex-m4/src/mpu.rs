//! Implementation of the ARM memory protection unit.

use kernel;
use kernel::common::VolatileCell;
use kernel::common::math::PowerOfTwo;

/// Indicates whether the MPU is present and, if so, how many regions it
/// supports.
#[repr(C)]
pub struct MpuType {
    /// Indicates whether the processor support unified (0) or separate
    /// (1) instruction and data regions. Always reads 0 on the
    /// Cortex-M4.
    pub is_separate: VolatileCell<u8>,

    /// The number of data regions supported. If this field reads-as-zero the
    /// processor does not implement an MPU
    pub data_regions: VolatileCell<u8>,

    /// The number of instructions regions supported. Always reads 0.
    pub instruction_regions: VolatileCell<u8>,

    _reserved: u8,
}

#[repr(C)]
/// MPU Registers for the Cortex-M4 family
///
/// Described in section 4.5 of
/// <http://infocenter.arm.com/help/topic/com.arm.doc.dui0553a/DUI0553A_cortex_m4_dgug.pdf>
pub struct Registers {
    pub mpu_type: VolatileCell<MpuType>,

    /// The control register:
    ///   * Enables the MPU (bit 0).
    ///   * Enables MPU in hard-fault, non-maskable interrupt (NMI) and
    ///     FAULTMASK escalated handlers (bit 1).
    ///   * Enables the default memory map background region in privileged mode
    ///     (bit 2).
    ///
    /// Bit   | Name       | Function
    /// ----- | ---------- | -----------------------------
    /// 0     | ENABLE     | Enable the MPU (1=enabled)
    /// 1     | HFNMIENA   | 0=MPU disabled during HardFault, NMI, and FAULTMASK
    ///       |            | regardless of bit 0. 1 leaves enabled.
    /// 2     | PRIVDEFENA | 0=Any memory access not explicitly enabled causes fault
    ///       |            | 1=Privledged mode code can read any memory address
    pub control: VolatileCell<u32>,

    /// Selects the region number (zero-indexed) referenced by the region base
    /// address and region attribute and size registers.
    ///
    /// Bit   | Name     | Function
    /// ----- | -------- | -----------------------------
    /// [7:0] | REGION   | Region for writes to MPU_RBAR or MPU_RASR. Range 0-7.
    pub region_number: VolatileCell<u32>,

    /// Defines the base address of the currently selected MPU region.
    ///
    /// When writing, the first 3 bits select a new region if bit-4 is set.
    ///
    /// The top bits set the base address of the register, with the bottom 32-N
    /// bits masked based on the region size (set in the region attribute and
    /// size register) according to:
    ///
    ///   N = Log2(Region size in bytes)
    ///
    /// Bit       | Name    | Function
    /// --------- | ------- | -----------------------------
    /// [31:N]    | ADDR    | Region base address
    /// [(N-1):5] |         | Reserved
    /// [4]       | VALID   | {RZ} 0=Use region_number reg, 1=Use REGION
    ///           |         |      Update base address for chosen region
    /// [3:0]     | REGION  | {W} (see VALID) ; {R} return region_number reg
    pub region_base_address: VolatileCell<u32>,

    /// Defines the region size and memory attributes of the selected MPU
    /// region. The bits are defined as in 4.5.5 of the Cortex-M4 user guide:
    ///
    /// Bit   | Name   | Function
    /// ----- | ------ | -----------------------------
    /// 0     | ENABLE | Region enable
    /// 5:1   | SIZE   | Region size is 2^(SIZE+1) (minimum 3)
    /// 7:6   |        | Unused
    /// 15:8  | SRD    | Subregion disable bits (0 is enable, 1 is disable)
    /// 16    | B      | Memory access attribute
    /// 17    | C      | Memory access attribute
    /// 18    | S      | Shareable
    /// 21:19 | TEX    | Memory access attribute
    /// 23:22 |        | Unused
    /// 26:24 | AP     | Access permission field
    /// 27    |        | Unused
    /// 28    | XN     | Instruction access disable
    pub region_attributes_and_size: VolatileCell<u32>,
}

const MPU_BASE_ADDRESS: *const Registers = 0xE000ED90 as *const Registers;

/// Constructor field is private to limit who can create a new MPU
pub struct MPU(*const Registers);

impl MPU {
    pub const unsafe fn new() -> MPU {
        MPU(MPU_BASE_ADDRESS)
    }
}

type Region = kernel::mpu::Region;

impl kernel::mpu::MPU for MPU {
    fn enable_mpu(&self) {
        let regs = unsafe { &*self.0 };

        // Enable the MPU, disable it during HardFault/NMI handlers, allow
        // privileged code access to all unprotected memory.
        regs.control.set(0b101);

        let mpu_type = regs.mpu_type.get();
        let regions = mpu_type.data_regions.get();
        if regions != 8 {
            panic!(
                "Tock currently assumes 8 MPU regions. This chip has {}",
                regions
            );
        }
    }

    fn disable_mpu(&self) {
        let regs = unsafe { &*self.0 };
        regs.control.set(0b0);
    }

    fn create_region(
        region_num: usize,
        start: usize,
        len: usize,
        execute: kernel::mpu::ExecutePermission,
        access: kernel::mpu::AccessPermission,
    ) -> Option<Region> {
        if region_num >= 8 {
            // There are only 8 (0-indexed) regions available
            return None;
        }

        // There are two possibilities we support:
        //
        // 1. The base address is aligned exactly to the size of the region,
        //    which uses an MPU region with the exact base address and size of
        //    the memory region.
        //
        // 2. Otherwise, we can use a larger MPU region and expose only MPU
        //    subregions, as long as the memory region's base address is aligned
        //    to 1/8th of a larger region size.

        if start % len == 0 {
            // Memory base aligned to memory size - straight forward case
            let region_len = PowerOfTwo::floor(len as u32);
            if region_len.exp::<u32>() < 5 {
                // Region sizes must be 32 Bytes or larger
                return None;
            } else if region_len.exp::<u32>() > 32 {
                // Region sizes must be 4GB or smaller
                return None;
            }

            let xn = execute as u32;
            let ap = access as u32;
            Some(unsafe {
                Region::new(
                    (start | 1 << 4 | (region_num & 0xf)) as u32,
                    1 | (region_len.exp::<u32>() - 1) << 1 | ap << 24 | xn << 28,
                )
            })
        } else {
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
                return None;
            }
            if len % subregion_size != 0 {
                // Sanity check that there is some integer X such that
                // subregion_size * X == len so none of `len` is left over when
                // we take the max_subregion.
                return None;
            }

            // The index of the first subregion to activate is the number of
            // regions between `region_start` (MPU) and `start` (memory).
            let min_subregion = (start - region_start) / subregion_size;
            // The index of the last subregion to activate is the number of
            // regions that fit in `len`, plus the `min_subregion`, minus one
            // (because subregions are zero-indexed).
            let max_subregion = min_subregion + len / subregion_size - 1;

            let region_len = PowerOfTwo::floor(region_size as u32);
            if region_len.exp::<u32>() < 7 {
                // Subregions only supported for regions sizes 128 bytes and up.
                return None;
            } else if region_len.exp::<u32>() > 32 {
                // Region sizes must be 4GB or smaller
                return None;
            }

            // Turn the min/max subregion into a bitfield where all bits are `1`
            // except for the bits whose index lie within
            // [min_subregion, max_subregion]
            //
            // Note: Rust ranges are minimum inclusive, maximum exclusive, hence
            // max_subregion + 1.
            let subregion_mask =
                (min_subregion..(max_subregion + 1)).fold(!0, |res, i| res & !(1 << i)) & 0xff;

            let xn = execute as u32;
            let ap = access as u32;
            Some(unsafe {
                Region::new(
                    (region_start | 1 << 4 | (region_num & 0xf)) as u32,
                    1 | subregion_mask << 8 | (region_len.exp::<u32>() - 1) << 1 | ap << 24
                        | xn << 28,
                )
            })
        }
    }

    fn set_mpu(&self, region: Region) {
        let regs = unsafe { &*self.0 };

        regs.region_base_address.set(region.base_address());

        regs.region_attributes_and_size.set(region.attributes());
    }
}
