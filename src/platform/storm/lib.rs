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
pub static mut BLINK2 : Option<drivers::blink::Blink> = None;

pub unsafe fn init() -> &'static mut sam4l::Sam4l {
    CHIP = Some(sam4l::Sam4l::new());
    let chip = CHIP.as_mut().unwrap();
    chip.led.configure(None);

    let led = &mut chip.led;
    let ast = &mut chip.ast;

    BLINK = Some(drivers::blink::Blink::new(
                ast,
                led));
    let blink = BLINK.as_mut().unwrap();

    ast.configure(blink);
    led.configure(None);

    blink.initialize();
    chip
}

