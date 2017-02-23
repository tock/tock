//! TRNG driver for the nrf51dk

use chip;
use core::cell::Cell;
use kernel::hil::rng::{self, Continue};
use nvic;
use peripheral_interrupts::NvicIdx;
use peripheral_registers::{RNG_BASE, RNG_REGS};
use core::mem;

pub struct Trng<'a> {
    regs: *mut RNG_REGS,
    client: Cell<Option<&'a rng::Client>>,
}

pub static mut TRNG: Trng<'static> = Trng::new();

impl<'a> Trng<'a> {
    const fn new() -> Trng<'a> {
        Trng {
            regs: RNG_BASE as *mut RNG_REGS,
            client: Cell::new(None),
        }
    }

    pub fn handle_interrupt(&self) {
        let regs: &mut RNG_REGS = unsafe { mem::transmute(self.regs) };
        // panic!("random number: {:}\r\n", regs.VALUE.get());
        
        // ONLY VALRDY CAN TRIGGER THIS INTERRUPT
        self.disable_interrupts();
        self.disable_nvic();
        //
        self.client.get().map(|client| {
            let result = client.randomness_available(&mut TrngIter(self));
            if Continue::Done != result {
                self.start_rng();
            }
        });
    }

    pub fn set_client(&self, client: &'a rng::Client) {
        self.client.set(Some(client));
    }

    fn enable_interrupts(&self) {
        let regs: &mut RNG_REGS = unsafe { mem::transmute(self.regs) };
        regs.INTEN.set(1);
        regs.INTENSET.set(1);
    }

    fn disable_interrupts(&self) {
        let regs: &mut RNG_REGS = unsafe { mem::transmute(self.regs) };
        regs.INTEN.set(0);
    }

    fn enable_nvic(&self) {
        nvic::enable(NvicIdx::RNG);
    }

    fn disable_nvic(&self) {
        nvic::disable(NvicIdx::RNG);
    }

    fn start_rng(&self) {
    
        let regs: &mut RNG_REGS = unsafe { mem::transmute(self.regs) };
        regs.VALRDY.set(0);
        regs.START.set(1);
    }

}

struct TrngIter<'a, 'b: 'a>(&'a Trng<'b>);

impl<'a, 'b> Iterator for TrngIter<'a, 'b> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        // let regs: &mut RNG_REGS = unsafe { mem::transmute(self.regs) };
        // if regs.VALRDY.get() != 0 {
        //     Some(regs.VALUE.get())
        // } else {
        //     None
        // }
        Some(12 as u32)
    }
}

impl<'a> rng::RNG for Trng<'a> {
    fn get(&self) {
        self.start_rng();
        self.enable_nvic();
        self.enable_interrupts();
    }
}

#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn RNG_Handler() {
    use kernel::common::Queue;
    nvic::disable(NvicIdx::RNG);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(NvicIdx::RNG);
}
