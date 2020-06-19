use core::ops::{Index, IndexMut};

use kernel::common::StaticRef;
pub use sifive::gpio::GpioPin;
use sifive::gpio::{pins, GpioRegisters};

const GPIO0_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x2000_2000 as *const GpioRegisters) };

pub struct Port<'a> {
    pins: [GpioPin<'a>; 16],
}

impl<'a> Index<usize> for Port<'a> {
    type Output = GpioPin<'a>;

    fn index(&self, index: usize) -> &GpioPin<'a> {
        &self.pins[index]
    }
}

impl<'a> IndexMut<usize> for Port<'a> {
    fn index_mut(&mut self, index: usize) -> &mut GpioPin<'a> {
        &mut self.pins[index]
    }
}

pub static mut PORT: Port = Port {
    pins: [
        GpioPin::new(GPIO0_BASE, pins::pin0, pins::pin0::SET, pins::pin0::CLEAR),
        GpioPin::new(GPIO0_BASE, pins::pin1, pins::pin1::SET, pins::pin1::CLEAR),
        GpioPin::new(GPIO0_BASE, pins::pin2, pins::pin2::SET, pins::pin2::CLEAR),
        GpioPin::new(GPIO0_BASE, pins::pin3, pins::pin3::SET, pins::pin3::CLEAR),
        GpioPin::new(GPIO0_BASE, pins::pin4, pins::pin4::SET, pins::pin4::CLEAR),
        GpioPin::new(GPIO0_BASE, pins::pin5, pins::pin5::SET, pins::pin5::CLEAR),
        GpioPin::new(GPIO0_BASE, pins::pin6, pins::pin6::SET, pins::pin6::CLEAR),
        GpioPin::new(GPIO0_BASE, pins::pin7, pins::pin7::SET, pins::pin7::CLEAR),
        GpioPin::new(GPIO0_BASE, pins::pin8, pins::pin8::SET, pins::pin8::CLEAR),
        GpioPin::new(GPIO0_BASE, pins::pin9, pins::pin9::SET, pins::pin9::CLEAR),
        GpioPin::new(
            GPIO0_BASE,
            pins::pin10,
            pins::pin10::SET,
            pins::pin10::CLEAR,
        ),
        GpioPin::new(
            GPIO0_BASE,
            pins::pin11,
            pins::pin11::SET,
            pins::pin11::CLEAR,
        ),
        GpioPin::new(
            GPIO0_BASE,
            pins::pin12,
            pins::pin12::SET,
            pins::pin12::CLEAR,
        ),
        GpioPin::new(
            GPIO0_BASE,
            pins::pin13,
            pins::pin13::SET,
            pins::pin13::CLEAR,
        ),
        GpioPin::new(
            GPIO0_BASE,
            pins::pin14,
            pins::pin14::SET,
            pins::pin14::CLEAR,
        ),
        GpioPin::new(
            GPIO0_BASE,
            pins::pin15,
            pins::pin15::SET,
            pins::pin15::CLEAR,
        ),
    ],
};
