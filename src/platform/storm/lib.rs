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

pub static mut BLINK : Option<drivers::blink::Blink> = None;
pub static mut CONSOLE :
    Option<drivers::console::Console<sam4l::usart::USART>> = None;

pub unsafe fn init() -> &'static mut sam4l::Sam4l {
    CHIP = Some(sam4l::Sam4l::new());
    let chip = CHIP.as_mut().unwrap();
    chip.led.configure(None);

    let led = &mut chip.led;
    let ast = &mut chip.ast;
    let usart3 = &mut chip.usarts[3];
    chip.pb09.configure(Some(sam4l::gpio::PeripheralFunction::A));
    chip.pb10.configure(Some(sam4l::gpio::PeripheralFunction::A));

    BLINK = Some(drivers::blink::Blink::new(
                ast,
                led));
    let blink = BLINK.as_mut().unwrap();

    CONSOLE = Some(drivers::console::Console::new(usart3));
    let console = CONSOLE.as_mut().unwrap();

    ast.configure(blink);
    led.configure(None);
    usart3.configure(sam4l::usart::USARTParams {
        client: console,
        baud_rate: 115200,
        data_bits: 8,
        parity: hil::uart::Parity::None
    });


    blink.initialize();
    console.initialize();
    chip
}

