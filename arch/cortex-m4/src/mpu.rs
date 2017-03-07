use kernel;
use kernel::common::volatile_cell::VolatileCell;

/// Indicates whether the MPU is present and, if so, how many regions it
/// supports.
#[repr(C,packed)]
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

#[repr(C,packed)]
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

impl kernel::mpu::MPU for MPU {
    fn enable_mpu(&self) {
        let regs = unsafe { &*self.0 };

        // Enable the MPU, disable it during HardFault/NMI handlers, disable it
        // when privileged code runs
        regs.control.set(0b101);

        let mpu_type = regs.mpu_type.get();
        let regions = mpu_type.data_regions.get();
        if regions != 8 {
            panic!("Tock currently assumes 8 MPU regions. This chip has {}",
                   regions);
        }
    }

    fn set_mpu(&self,
               region_num: u32,
               start_addr: u32,
               len: u32,
               execute: kernel::mpu::ExecutePermission,
               access: kernel::mpu::AccessPermission) {
        let regs = unsafe { &*self.0 };

        let region_base_address = region_num | 1 << 4 | start_addr;
        regs.region_base_address.set(region_base_address);

        let xn = execute as u32;
        let ap = access as u32;
        let region_attributes_and_size = 1 | len << 1 | ap << 24 | xn << 28;
        regs.region_attributes_and_size.set(region_attributes_and_size);
    }
}
