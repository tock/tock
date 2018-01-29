//! Implementation of the SAM4L TRNG.

use core::cell::Cell;
use kernel::common::VolatileCell;
use kernel::hil::rng::{self, Continue};
use pm;

#[repr(C)]
struct Registers {
    control: VolatileCell<u32>,
    _reserved0: [u32; 3],
    interrupt_enable: VolatileCell<u32>,
    interrupt_disable: VolatileCell<u32>,
    interrupt_mask: VolatileCell<u32>,
    interrupt_status: VolatileCell<u32>,
    _reserved1: [u32; 12],
    data: VolatileCell<u32>,
}

const BASE_ADDRESS: *const Registers = 0x40068000 as *const Registers;

pub struct Trng<'a> {
    regs: *const Registers,
    client: Cell<Option<&'a rng::Client>>,
}

pub static mut TRNG: Trng<'static> = Trng::new();
const KEY: u32 = 0x524e4700;

impl<'a> Trng<'a> {
    const fn new() -> Trng<'a> {
        Trng {
            regs: BASE_ADDRESS,
            client: Cell::new(None),
        }
    }

    pub fn handle_interrupt(&self) {
        let regs = unsafe { &*self.regs };

        if regs.interrupt_mask.get() == 0 {
            return;
        }
        regs.interrupt_disable.set(1);

        self.client.get().map(|client| {
            let result = client.randomness_available(&mut TrngIter(self));
            if let Continue::Done = result {
                // disable controller
                regs.control.set(KEY | 0);
                unsafe {
                    pm::disable_clock(pm::Clock::PBA(pm::PBAClock::TRNG));
                }
            } else {
                regs.interrupt_enable.set(1);
            }
        });
    }

    pub fn set_client(&self, client: &'a rng::Client) {
        self.client.set(Some(client));
    }
}

struct TrngIter<'a, 'b: 'a>(&'a Trng<'b>);

impl<'a, 'b> Iterator for TrngIter<'a, 'b> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        let regs = unsafe { &*self.0.regs };
        if regs.interrupt_status.get() != 0 {
            Some(regs.data.get())
        } else {
            None
        }
    }
}

impl<'a> rng::RNG for Trng<'a> {
    fn get(&self) {
        let regs = unsafe { &*self.regs };
        unsafe {
            pm::enable_clock(pm::Clock::PBA(pm::PBAClock::TRNG));
        }

        regs.control.set(KEY | 1);
        regs.interrupt_enable.set(1);
    }
}
