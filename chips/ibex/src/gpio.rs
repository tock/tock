//! GPIO instantiation.

use core::ops::{Index, IndexMut};

use kernel::common::StaticRef;
use lowrisc::gpio::{pins, GpioPin, GpioRegisters};

const GPIO0_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x4001_0000 as *const GpioRegisters) };

pub struct Port {
    pins: [GpioPin; 32],
}

impl Index<usize> for Port {
    type Output = GpioPin;

    fn index(&self, index: usize) -> &GpioPin {
        &self.pins[index]
    }
}

impl IndexMut<usize> for Port {
    fn index_mut(&mut self, index: usize) -> &mut GpioPin {
        &mut self.pins[index]
    }
}

pub static mut PORT: Port = Port {
    pins: [
        GpioPin::new(GPIO0_BASE, pins::pin0),
        GpioPin::new(GPIO0_BASE, pins::pin1),
        GpioPin::new(GPIO0_BASE, pins::pin2),
        GpioPin::new(GPIO0_BASE, pins::pin3),
        GpioPin::new(GPIO0_BASE, pins::pin4),
        GpioPin::new(GPIO0_BASE, pins::pin5),
        GpioPin::new(GPIO0_BASE, pins::pin6),
        GpioPin::new(GPIO0_BASE, pins::pin7),
        GpioPin::new(GPIO0_BASE, pins::pin8),
        GpioPin::new(GPIO0_BASE, pins::pin9),
        GpioPin::new(GPIO0_BASE, pins::pin10),
        GpioPin::new(GPIO0_BASE, pins::pin11),
        GpioPin::new(GPIO0_BASE, pins::pin12),
        GpioPin::new(GPIO0_BASE, pins::pin13),
        GpioPin::new(GPIO0_BASE, pins::pin14),
        GpioPin::new(GPIO0_BASE, pins::pin15),
        GpioPin::new(GPIO0_BASE, pins::pin16),
        GpioPin::new(GPIO0_BASE, pins::pin17),
        GpioPin::new(GPIO0_BASE, pins::pin18),
        GpioPin::new(GPIO0_BASE, pins::pin19),
        GpioPin::new(GPIO0_BASE, pins::pin20),
        GpioPin::new(GPIO0_BASE, pins::pin21),
        GpioPin::new(GPIO0_BASE, pins::pin22),
        GpioPin::new(GPIO0_BASE, pins::pin23),
        GpioPin::new(GPIO0_BASE, pins::pin24),
        GpioPin::new(GPIO0_BASE, pins::pin25),
        GpioPin::new(GPIO0_BASE, pins::pin26),
        GpioPin::new(GPIO0_BASE, pins::pin27),
        GpioPin::new(GPIO0_BASE, pins::pin28),
        GpioPin::new(GPIO0_BASE, pins::pin29),
        GpioPin::new(GPIO0_BASE, pins::pin30),
        GpioPin::new(GPIO0_BASE, pins::pin31),
    ],
};
