//! Implementation of the flash controller

use core::mem;
use kernel::common::VolatileCell;

pub enum Latency {
    NoWaitStates,
    OneWaitState,
    TwoWaitStates,
}

#[repr(C, packed)]
struct Registers {
    acr: VolatileCell<u32>,
    keyr: VolatileCell<u32>,
    optkeyr: VolatileCell<u32>,
    sr: VolatileCell<u32>,
    cr: VolatileCell<u32>,
    ar: VolatileCell<u32>,
    _reserved: VolatileCell<u32>,
    obr: VolatileCell<u32>,
    wpbr: VolatileCell<u32>,
}

const BASE_ADDRESS: usize = 0x40022000;

pub static mut FLASH: FlashController = FlashController::new(BASE_ADDRESS);

pub struct FlashController {
    registers: *mut Registers,
}

impl FlashController {
    const fn new(base_addr: usize) -> FlashController {
        FlashController { registers: base_addr as *mut Registers }
    }

    pub fn set_latency(&self, latency: Latency) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };

        let bits = match latency {
            Latency::NoWaitStates => 0,
            Latency::OneWaitState => 1,
            Latency::TwoWaitStates => 2,
        };
        regs.acr.set((regs.acr.get() & !0x7) | bits);
    }
}
