#![crate_name = "platform"]
#![crate_type = "rlib"]
#![no_std]

extern crate hil;

pub struct Firestorm;

impl Firestorm {
    pub unsafe fn service_pending_interrupts(&mut self) {
    }

    pub unsafe fn has_pending_interrupts(&mut self) -> bool {
        false
    }

    #[inline(never)]
    pub fn with_driver<F, R>(&mut self, driver_num: usize, f: F) -> R where
            F: FnOnce(Option<&hil::Driver>) -> R {
        match driver_num {
            _ => f(None)
        }
    }
}

pub unsafe fn init<'a>() -> &'a mut Firestorm {
    use core::mem;

    static mut FIRESTORM_BUF : [u8; 1024] = [0; 1024];

    let firestorm : &'static mut Firestorm = mem::transmute(&mut FIRESTORM_BUF);

    firestorm
}
