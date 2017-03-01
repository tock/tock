//! TRNG driver for the nrf51dk

use chip;
use core::cell::Cell;
use core::mem;
use kernel::hil::rng::{self, Continue};
use nvic;
use peripheral_interrupts::NvicIdx;
use peripheral_registers::{RNG_BASE, RNG_REGS};

pub static mut DMY: [u8; 4] = [0; 4];

pub struct Trng<'a> {
    regs: *mut RNG_REGS,
    client: Cell<Option<&'a rng::Client>>,
    done: Cell<u8>,
}

pub static mut TRNG: Trng<'static> = Trng::new();

impl<'a> Trng<'a> {
    const fn new() -> Trng<'a> {
        Trng {
            regs: RNG_BASE as *mut RNG_REGS,
            client: Cell::new(None),
            done: Cell::new(0),
        }
    }
    
    // ONLY VALRDY CAN TRIGGER THIS INTERRUPT
    pub fn handle_interrupt(&self) {
        let regs: &mut RNG_REGS = unsafe { mem::transmute(self.regs) };

        // disable interrupts
        self.disable_interrupts();
        self.disable_nvic();
        regs.STOP.set(1);
        nvic::clear_pending(NvicIdx::RNG);

        match self.done.get() {
            e @ 0...3 => {
                unsafe {
                    DMY[e as usize] = regs.VALUE.get() as u8;
                }
                self.done.set(e + 1);
                self.start_rng()
            }
            4 => {
                self.client.get().map(|client| {
                    let result = client.randomness_available(&mut TrngIter(self));
                    if Continue::Done != result {
                        self.start_rng();
                    }
                });
            }
            _ => panic!("invalid length of data\r\n"),
        }
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
        regs.INTENCLR.set(1);
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

        // clear registers
        regs.VALRDY.set(0);

        // enable interrupts
        self.enable_nvic();
        self.enable_interrupts();

        // start rng
        regs.START.set(1);
    }
}

struct TrngIter<'a, 'b: 'a>(&'a Trng<'b>);

impl<'a, 'b> Iterator for TrngIter<'a, 'b> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.0.done.get() == 4 {
            let b = unsafe { mem::transmute::<[u8; 4], u32>(DMY) };
            // indicate 4 bytes of randomness taken by the capsule
            self.0.done.set(0);
            Some(b)
        } else {
            None
        }
    }
}

impl<'a> rng::RNG for Trng<'a> {
    fn get(&self) {
        self.start_rng()
    }
}

#[inline(never)]
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn RNG_Handler() {
    use kernel::common::Queue;
    nvic::disable(NvicIdx::RNG);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(NvicIdx::RNG);
}
