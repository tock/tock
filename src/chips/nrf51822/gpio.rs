use core::mem;
use core::ops::{Index, IndexMut};
use common::VolatileCell;
use hil;

#[allow(non_snake_case)]
struct GPIO {
    _pad0: [u8; 1284],
    pub OUT: VolatileCell<u32>,
    pub OUTSET: VolatileCell<u32>,
    pub OUTCLR: VolatileCell<u32>,
    pub IN: VolatileCell<u32>,
    pub DIR: VolatileCell<u32>,
    pub DIRSET: VolatileCell<u32>,
    pub DIRCLR: VolatileCell<u32>,
    _pad1: [u8; 480],
    pub PIN_CNF: [VolatileCell<u32>; 32],
}

#[allow(non_snake_case)]
fn GPIO() -> &'static GPIO {
    unsafe { mem::transmute(0x50000000 as usize) }
}

pub struct GPIOPin {
    pin: u8,
}

impl GPIOPin {
    const fn new(pin: u8) -> GPIOPin {
        GPIOPin { pin: pin }
    }
}

impl hil::gpio::GPIOPin for GPIOPin {
    fn enable_output(&self) {
        GPIO().PIN_CNF[self.pin as usize].set((1 << 0) | (1 << 1) | (0 << 2) | (0 << 8) | (0 << 16));
    }

    fn enable_input(&self, _mode: hil::gpio::InputMode) {
        unimplemented!();
    }

    fn disable(&self) {
        unimplemented!();
    }

    fn set(&self) {
        GPIO().OUTSET.set(1 << self.pin);
    }

    fn clear(&self) {
        GPIO().OUTCLR.set(1 << self.pin);
    }

    fn toggle(&self) {
        unimplemented!();
    }

    fn read(&self) -> bool {
        unimplemented!();
    }

    fn enable_interrupt(&self, _identifier: usize, _mode: hil::gpio::InterruptMode) {
        unimplemented!();
    }
}

pub struct Port {
    pins: [GPIOPin; 32]
}

impl Index<usize> for Port {
    type Output = GPIOPin;

    fn index(&self, index: usize) -> &GPIOPin {
        &self.pins[index]
    }
}

impl IndexMut<usize> for Port {
    fn index_mut(&mut self, index: usize) -> &mut GPIOPin {
        &mut self.pins[index]
    }
}

pub static mut PA : Port = Port {
    pins: [
        GPIOPin::new(0), GPIOPin::new(1), GPIOPin::new(2), GPIOPin::new(3),
        GPIOPin::new(4), GPIOPin::new(5), GPIOPin::new(6), GPIOPin::new(7),
        GPIOPin::new(8), GPIOPin::new(9), GPIOPin::new(10), GPIOPin::new(11),
        GPIOPin::new(12), GPIOPin::new(13), GPIOPin::new(14), GPIOPin::new(15),
        GPIOPin::new(16), GPIOPin::new(17), GPIOPin::new(18), GPIOPin::new(19),
        GPIOPin::new(20), GPIOPin::new(21), GPIOPin::new(22), GPIOPin::new(23),
        GPIOPin::new(24), GPIOPin::new(25), GPIOPin::new(26), GPIOPin::new(27),
        GPIOPin::new(28), GPIOPin::new(29), GPIOPin::new(30), GPIOPin::new(31),
    ],
};
