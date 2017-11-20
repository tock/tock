//! TRNG driver, nRF5X-family
//!
//! The TRNG generates 1 byte randomness at the time value in the interval
//! 0 <= r <= 255.
//!
//! Because that the he capsule requires 4 bytes of randomness at the time.
//! 4 bytes of randomness must be generated before returning  back to capsule.
//!
//! Therefore this module will have to use the TRNG four times.
//! A counter `index` has been introduced to keep track of this.
//! The four bytes of randomness is stored in a `Cell<u32>` which shifted
//! according to append one byte at the time.
//!
//! In the current implementation if done > 4 for some strange reason the
//! random generation will be restarted
//!
//! Authors
//! -------------------
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Fredrik Nilsson <frednils@student.chalmers.se>
//! * Date: March 01, 2017

use core::cell::Cell;
use kernel::hil::rng::{self, Continue};
use peripheral_registers::{RNG_BASE, RNG_REGS};

pub struct Trng<'a> {
    regs: *const RNG_REGS,
    client: Cell<Option<&'a rng::Client>>,
    index: Cell<usize>,
    randomness: Cell<u32>,
}

pub static mut TRNG: Trng<'static> = Trng::new();

impl<'a> Trng<'a> {
    const fn new() -> Trng<'a> {
        Trng {
            regs: RNG_BASE as *const RNG_REGS,
            client: Cell::new(None),
            index: Cell::new(0),
            randomness: Cell::new(0),
        }
    }

    // only VALRDY register can trigger the interrupt
    pub fn handle_interrupt(&self) {
        let regs = unsafe { &*self.regs };
        // disable interrupts
        self.disable_interrupts();

        match self.index.get() {
            // fetch more data need 4 bytes because the capsule requires that
            e @ 0...3 => {
                // 3 lines below to change data in Cell, perhaps it can be done more nicely
                let mut rn = self.randomness.get();
                // 1 byte randomness
                let r = regs.value.get();
                //  e = 0 -> byte 1 LSB
                //  e = 1 -> byte 2
                //  e = 2 -> byte 3
                //  e = 3 -> byte 4 MSB
                rn |= r << 8 * e;
                self.randomness.set(rn);

                self.index.set(e + 1);
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
            // This should never happen if the logic is correct
            // Restart randomness generation if the condition occurs
            _ => {
                self.index.set(0);
                self.randomness.set(0);
            }
        }
    }

    pub fn set_client(&self, client: &'a rng::Client) {
        self.client.set(Some(client));
    }

    fn enable_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.inten.set(1);
        regs.intenset.set(1);
    }

    fn disable_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenclr.set(1);
        regs.inten.set(0);
    }

    fn start_rng(&self) {
        let regs = unsafe { &*self.regs };

        // clear registers
        regs.event_valrdy.set(0);

        // enable interrupts
        self.enable_interrupts();

        // start rng
        regs.task_start.set(1);
    }
}

struct TrngIter<'a, 'b: 'a>(&'a Trng<'b>);

impl<'a, 'b> Iterator for TrngIter<'a, 'b> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.0.index.get() == 4 {
            let rn = self.0.randomness.get();
            // indicate 4 bytes of randomness taken by the capsule
            self.0.index.set(0);
            self.0.randomness.set(0);
            Some(rn)
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
