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
pub mod usart;

pub struct Sam4l {
    pub ast: ast::Ast,
    pub usarts: [usart::USART; 4],
    pub led: gpio::GPIOPin,
    pub pb09: gpio::GPIOPin,
    pub pb10: gpio::GPIOPin
}

impl Sam4l {
    pub fn new() -> Sam4l {

        Sam4l {
            ast: ast::Ast::new(),
            usarts: [
                usart::USART::new(usart::Location::USART0),
                usart::USART::new(usart::Location::USART1),
                usart::USART::new(usart::Location::USART2),
                usart::USART::new(usart::Location::USART3),
            ],
            led: gpio::GPIOPin::new(gpio::Pin::PC10),
            pb09: gpio::GPIOPin::new(gpio::Pin::PB09),
            pb10: gpio::GPIOPin::new(gpio::Pin::PB10),
        }
    }

    pub unsafe fn service_pending_interrupts(&mut self) {
        use core::intrinsics::atomic_xchg;

        if atomic_xchg(&mut ast::INTERRUPT, false) {
            self.ast.handle_interrupt();
            nvic::enable(nvic::NvicIdx::ASTALARM);
        }

        if atomic_xchg(&mut usart::USART3_INTERRUPT, false) {
            self.usarts[3].handle_interrupt();
            nvic::enable(nvic::NvicIdx::USART3);
        }

    }

    pub fn has_pending_interrupts(&mut self) -> bool {
        use core::intrinsics::volatile_load;
        unsafe {
            volatile_load(&usart::USART3_INTERRUPT)
                || volatile_load(&ast::INTERRUPT)
        }
    }
}

