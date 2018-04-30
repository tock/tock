use peripheral_registers;

pub struct PPIRegisters {
    regs: *const peripheral_registers::PPI,
}

pub static mut PPI: PPIRegisters = PPIRegisters::new();

impl PPIRegisters {
    pub const fn new() -> PPIRegisters {
        PPIRegisters {
            regs: peripheral_registers::PPI_BASE as *const peripheral_registers::PPI,
        }
    }

    pub fn enable(&self, pins: u32) {
        let regs = unsafe { &*self.regs };
        regs.chenset.set(pins);
    }

    pub fn disable(&self, pins: u32) {
        let regs = unsafe { &*self.regs };
        regs.chenclr.set(pins);
    }
}
