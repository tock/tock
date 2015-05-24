#![crate_name = "sam4l"]
#![crate_type = "rlib"]
#![feature(asm,core,concat_idents,no_std)]
#![no_std]

extern crate core;
extern crate common;
extern crate hil;

macro_rules! volatile {
    ($item:expr) => ({
        use core::intrinsics::volatile_load;
        unsafe { volatile_load(&$item) }
    });

    ($item:ident = $value:expr) => ({
        use core::intrinsics::volatile_store;
        unsafe { volatile_store(&mut $item, $value); }
    });

    ($item:ident |= $value:expr) => ({
        use core::intrinsics::volatile_load;
        use core::intrinsics::volatile_store;
        unsafe { volatile_store(&mut $item, volatile_load(&$item) | $value); }
    });

    ($item:ident &= $value:expr) => ({
        use core::intrinsics::volatile_load;
        use core::intrinsics::volatile_store;
        unsafe { volatile_store(&mut $item, volatile_load(&$item) & $value); }
    });
}

pub mod ast;
pub mod nvic;
pub mod pm;
pub mod gpio;

use core::prelude::*;

pub struct Sam4l {
    pub ast: ast::Ast,
    pub led: gpio::GPIOPin
}

impl Sam4l {
    pub fn new() -> Sam4l {
        use hil::Controller;
        Sam4l {
            ast: Controller::new(()),
            led: Controller::new(gpio::Location::GPIOPin74),
        }
    }

    pub unsafe fn service_pending_interrupts(&mut self) {
        use core::intrinsics::atomic_xchg;

        if atomic_xchg(&mut ast::INTERRUPT, false) {
            self.ast.handle_interrupt();
            nvic::enable(nvic::NvicIdx::ASTALARM);
        }
    }
}

