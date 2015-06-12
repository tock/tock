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

use hil::Controller;

pub static mut CHIP : Option<sam4l::Sam4l> = None;

pub static mut FIRESTORM : Option<Firestorm> = None;

pub struct Firestorm {
    chip: &'static mut sam4l::Sam4l,
    console: drivers::console::Console<sam4l::usart::USART>,
    gpio: drivers::gpio::GPIO<[&'static mut hil::gpio::GPIOPin; 14]>
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
            _ => None
        })
    }

}

pub unsafe fn init() -> &'static mut Firestorm {
    CHIP = Some(sam4l::Sam4l::new());
    let chip = CHIP.as_mut().unwrap();

    let usart3 = &mut chip.usarts[3];
    chip.pb09.configure(Some(sam4l::gpio::PeripheralFunction::A));
    chip.pb10.configure(Some(sam4l::gpio::PeripheralFunction::A));

    FIRESTORM = Some(Firestorm {
        chip: chip,
        console: drivers::console::Console::new(usart3),
        gpio: drivers::gpio::GPIO::new(
            [ &mut chip.pc10, &mut chip.pc19, &mut chip.pc13
            , &mut chip.pa09, &mut chip.pa17, &mut chip.pc20
            , &mut chip.pa19, &mut chip.pa14, &mut chip.pa16
            , &mut chip.pa13, &mut chip.pa11, &mut chip.pa10
            , &mut chip.pa12, &mut chip.pc09])
    });

    let firestorm = FIRESTORM.as_mut().unwrap();

    usart3.configure(sam4l::usart::USARTParams {
        client: &mut firestorm.console,
        baud_rate: 115200,
        data_bits: 8,
        parity: hil::uart::Parity::None
    });


    firestorm.console.initialize();

    firestorm
}

