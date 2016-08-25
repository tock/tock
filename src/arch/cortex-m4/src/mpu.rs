use common::volatile_cell::VolatileCell;
use main;

/// Indicates whether the MPU is present and, if so, how many regions it
/// supports.
#[repr(C,packed)]
pub struct MpuType {
    /// Indicates whether the processor support unified (0) or separate
    /// (1) instruction and data regions. Always reads 0 on the
    /// Cortex-M4.
    pub is_separate: VolatileCell<u8>,

    /// The number of data regions supported. Always reads 8.
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
    pub control: VolatileCell<u32>,

    /// Selects the region number (zero-indexed) referenced by the region base
    /// address and region attribute and size registers.
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

impl main::MPU for MPU {
    fn enable_mpu(&self) {
        let regs = unsafe { &*self.0 };
        regs.control.set(0b101);
    }

    fn set_mpu(&self, region_num: u32, start_addr: u32, len: u32, execute: bool, ap: u32) {
        let regs = unsafe { &*self.0 };
        regs.region_base_address.set(region_num | 1 << 4 | start_addr);
        let xn = if execute { 0 } else { 1 };
        regs.region_attributes_and_size.set(1 | len << 1 | ap << 24 | xn << 28);
    }
}
