// Non-Volatile Memory Controller
// Used in order read and write to internal flash
// minimal implementation to support activation of the reset button

use peripheral_registers;

pub struct NVMC {
    regs: *const peripheral_registers::NVMC,
}

impl NVMC {
    pub const fn new() -> NVMC {
        NVMC {
            regs: peripheral_registers::NVMC_BASE as *mut peripheral_registers::NVMC,
        }
    }

    pub fn configure_writeable(&self) {
        let regs = unsafe { &*self.regs };
        regs.config.set(1);
    }

    pub fn is_ready(&self) -> bool {
        let regs = unsafe { &*self.regs };
        regs.ready.get() == 1
    }
}
