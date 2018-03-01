use kernel::common::VolatileCell;
use kernel::{MMIOInterface, MMIOManager, NoClockControl};

/// The MMIO Structure
#[repr(C)]
#[allow(dead_code)]
pub struct TestRegisters {
    control: VolatileCell<u32>,
    interrupt_mask: VolatileCell<u32>,
}

/// The Tock object that holds all information for this peripheral
#[derive(NoClockControlMMIOHardware)]
pub struct TestHw {
    registers: *mut TestRegisters,
}

/// Teaching the kernel how to create TestRegisters
impl MMIOInterface<NoClockControl> for TestHw {
    type MMIORegisterType = TestRegisters;

    fn get_hardware_address(&self) -> *mut TestRegisters {
        self.registers
    }
}

/// Mapping to actual hardware instance(s)
const TEST_BASE_ADDR: *mut TestRegisters = 0x40001000 as *mut TestRegisters;
pub static mut TEST0: TestHw = TestHw::new(TEST_BASE_ADDR);

/// Methods this peripheral exports to the rest of the kernel
impl TestHw {
    const fn new(base_addr: *mut TestRegisters) -> TestHw {
        TestHw { registers: base_addr as *mut TestRegisters }
    }

    pub fn do_thing(&self) {
        let regs_manager = &MMIOManager::new(self);
        regs_manager.registers.control.get();
    }
}
