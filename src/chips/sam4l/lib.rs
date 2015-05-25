#![crate_name = "sam4l"]
#![crate_type = "rlib"]
#![feature(asm,core,concat_idents,no_std)]
#![no_std]

extern crate core;
extern crate common;
extern crate hil;

pub fn volatile_load<T>(item: &T) -> T {
    unsafe {
        core::intrinsics::volatile_load(item)
    }
}

pub fn volatile_store<T>(item: &mut T, val: T) {
    unsafe {
        core::intrinsics::volatile_store(item, val)
    }
}

macro_rules! volatile {
    ($item:expr) => ({
        ::volatile_load(&$item)
    });

    ($item:ident = $value:expr) => ({
        ::volatile_store(&mut $item, $value)
    });

    ($item:ident |= $value:expr) => ({
        ::volatile_store(&mut $item, ::volatile_load(&$item) | $value)
    });

    ($item:ident &= $value:expr) => ({
        ::volatile_store(&mut $item, ::volatile_load(&$item) & $value)
    });
}

pub mod ast;
pub mod nvic;
pub mod pm;
pub mod gpio;

pub struct Sam4l {
    pub ast: ast::Ast,
    pub led: gpio::GPIOPin
}

impl Sam4l {
    pub fn new() -> Sam4l {
        Sam4l {
            ast: ast::Ast::new(),
            led: gpio::GPIOPin::new(gpio::Pin::PC10),
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

