//! TRNG driver for nrf51dk
//!
//! The TRNG generates 1 byte randomness at the time value in the interval
//! 0 <= r <= 255
//!
//! The capsule requires 4 bytes of randomness
//!
//! The counter "done" ensures that 4 bytes of randomness have been generated
//! before returning to the capsule.
//!
//! A temporary array "randomness" is used to store the randomness until it is
//! returned to the capsule
//!
//! In the current implementation if done > 4 for some strange reason the
//! random generation will be restarted
//!
//! Author: Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! Author: Fredrik Nilsson <frednils@student.chalmers.se>
//! Date: March 01, 2017

use chip;
use core::cell::Cell;
use core::mem;
use kernel::hil::rng::{self, Continue};
use nvic;
use peripheral_interrupts::NvicIdx;
use peripheral_registers::{RNG_BASE, RNG_REGS};

pub struct Trng<'a> {
    regs: *const RNG_REGS,
    client: Cell<Option<&'a rng::Client>>,
    done: Cell<usize>,
    randomness: Cell<[u8; 4]>,
}

pub static mut TRNG: Trng<'static> = Trng::new();

impl<'a> Trng<'a> {
    const fn new() -> Trng<'a> {
        Trng {
            regs: RNG_BASE as *mut RNG_REGS,
            client: Cell::new(None),
            done: Cell::new(0),
            randomness: Cell::new([0; 4]),
        }
    }

    // only VALRDY register can trigger the interrupt
    pub fn handle_interrupt(&self) {
        let regs = unsafe { &*self.regs };
        // disable interrupts
        self.disable_interrupts();
        self.disable_nvic();
        nvic::clear_pending(NvicIdx::RNG);

        match self.done.get() {
            // fetch more data need 4 bytes because the capsule requires that
            e @ 0...3 => {
                // 3 lines below to change data in Cell, perhaps it can be done more nicely
                let mut arr = self.randomness.get();
                arr[e] = regs.VALUE.get() as u8;
                self.randomness.set(arr);

                self.done.set(e + 1);
                self.start_rng()
            }
            // fetched 4 bytes of data send to the capsule
            4 => {
                self.client.get().map(|client| {
                    let result = client.randomness_available(&mut TrngIter(self));
                    if Continue::Done != result {
                        // need more randomness i.e generate more randomness
                        self.start_rng();
                    }
                });
            }
            // This should never happend if the logic is correct
            // Restart randomness generation if the conditon occurs
            _ => {
                self.done.set(0);
                self.randomness.set([0, 0, 0, 0]);
            }
        }
    }

    pub fn set_client(&self, client: &'a rng::Client) {
        self.client.set(Some(client));
    }

    fn enable_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.INTEN.set(1);
        regs.INTENSET.set(1);
    }

    fn disable_interrupts(&self) {
        let regs = unsafe { &*self.regs };
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
        let regs = unsafe { &*self.regs };

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
            // convert [u8; 4] to u32 and return to rng capsule
            let b = unsafe { mem::transmute::<[u8; 4], u32>(self.0.randomness.get()) };
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
