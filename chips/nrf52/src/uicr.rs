// User information configuration registers
// minimal implementation to support activation of the reset button

use peripheral_registers;

pub struct UICR {
    regs: *const peripheral_registers::UICR,
}

impl UICR {
    pub const fn new() -> UICR {
        UICR {
            regs: peripheral_registers::UICR_BASE as *mut peripheral_registers::UICR,
        }
    }

    pub fn set_psel0_reset_pin(&self, pin: usize) {
        let regs = unsafe { &*self.regs };
        regs.pselreset0.set(pin as u32);
    }
    pub fn set_psel1_reset_pin(&self, pin: usize) {
        let regs = unsafe { &*self.regs };
        regs.pselreset1.set(pin as u32);
    }
}
