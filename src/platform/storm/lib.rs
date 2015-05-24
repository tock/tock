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
use common::shared::Shared;

pub static mut CHIP : Option<Shared<sam4l::Sam4l>> = None;

pub static mut BLINK : Option<Shared<drivers::blink::Blink>> = None;

pub unsafe fn init() -> &'static mut sam4l::Sam4l {
    CHIP = Some(Shared::new(sam4l::Sam4l::new()));
    let chip = CHIP.as_ref().unwrap().borrow_mut();
    chip.led.configure(None);

    let led = &mut chip.led;
    let ast = &mut chip.ast;

    BLINK = Some(Shared::new(
            drivers::blink::Blink::new(
                ast,
                led
                )
            ));
    let blink = BLINK.as_ref().unwrap().borrow_mut();

    ast.configure(blink);
    led.configure(None);

    blink.initialize();
    chip
}

