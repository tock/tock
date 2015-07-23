#![crate_name = "platform"]
#![crate_type = "rlib"]
#![no_std]
#![feature(core,no_std)]

extern crate core;
extern crate common;
extern crate drivers;
extern crate hil;
extern crate sam4l;

use core::prelude::*;
use hil::adc::AdcInternal;
use hil::Controller;

pub struct Firestorm {
    chip: &'static mut sam4l::chip::Sam4l,
    console: drivers::console::Console<sam4l::usart::USART>,
    gpio: drivers::gpio::GPIO<[&'static mut hil::gpio::GPIOPin; 14]>,
    tmp006: drivers::tmp006::TMP006<sam4l::i2c::I2CDevice>
}

impl Drop for Firestorm {
    fn drop(&mut self) {}
}

impl Firestorm {
    pub unsafe fn service_pending_interrupts(&mut self) {
        self.chip.service_pending_interrupts()
    }

    pub fn has_pending_interrupts(&mut self) -> bool {
        self.chip.has_pending_interrupts()
    }

    pub fn with_driver<F, R>(&mut self, driver_num: usize, mut f: F) -> R where
            F: FnMut(Option<&mut hil::Driver>) -> R {

        f(match driver_num {
            0 => Some(&mut self.console),
            1 => Some(&mut self.gpio),
            2 => Some(&mut self.tmp006),
            _ => None
        })
    }

}

static mut FIRESTORM_BUF : [u8; 140] = [0; 140];
static mut CHIP_BUF : [u8; 924] = [0; 924];

pub unsafe fn init<'a>() -> &'a mut Firestorm {
    use core::mem;

    // Just test that CHIP_BUF is correct size
    // (will throw compiler error if too large or small)
    let _ : sam4l::chip::Sam4l = mem::transmute(CHIP_BUF);

    let chip : &'static mut sam4l::chip::Sam4l = mem::transmute(&mut CHIP_BUF);
    *chip = sam4l::chip::Sam4l::new();
    sam4l::chip::INTERRUPT_QUEUE = Some(&mut chip.queue);

    // Just test that FIRESTORM_BUF is correct size
    // (will throw compiler error if too large or small)
    let _ : Firestorm = mem::transmute(FIRESTORM_BUF);

    let firestorm : &'static mut Firestorm = mem::transmute(&mut FIRESTORM_BUF);
    *firestorm = Firestorm {
        chip: chip,
        console: drivers::console::Console::new(&mut chip.usarts[3]),
        gpio: drivers::gpio::GPIO::new(
            [ &mut chip.pc10, &mut chip.pc19, &mut chip.pc13
            , &mut chip.pa09, &mut chip.pa17, &mut chip.pc20
            , &mut chip.pa19, &mut chip.pa14, &mut chip.pa16
            , &mut chip.pa13, &mut chip.pa11, &mut chip.pa10
            , &mut chip.pa12, &mut chip.pc09]),
        tmp006: drivers::tmp006::TMP006::new(&mut chip.i2c[2]),
    };

    chip.usarts[3].configure(sam4l::usart::USARTParams {
        client: &mut firestorm.console,
        baud_rate: 115200,
        data_bits: 8,
        parity: hil::uart::Parity::None
    });

    chip.pb09.configure(Some(sam4l::gpio::PeripheralFunction::A));
    chip.pb10.configure(Some(sam4l::gpio::PeripheralFunction::A));

    chip.pa21.configure(Some(sam4l::gpio::PeripheralFunction::E));
    chip.pa22.configure(Some(sam4l::gpio::PeripheralFunction::E));

    firestorm.console.initialize();
    firestorm
}

