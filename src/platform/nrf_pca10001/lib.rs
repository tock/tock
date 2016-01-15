#![crate_name = "platform"]
#![crate_type = "rlib"]
#![no_std]

extern crate drivers;
extern crate hil;
extern crate nrf51822;

pub struct Firestorm {
    gpio: &'static drivers::gpio::GPIO<[&'static hil::gpio::GPIOPin; 32]>,
}

impl Firestorm {
    pub unsafe fn service_pending_interrupts(&mut self) {
    }

    pub unsafe fn has_pending_interrupts(&mut self) -> bool {
        // FIXME: The wfi call from main() blocks forever if no interrupts are generated. For now,
        // pretend we have interrupts to avoid blocking.
        true
    }

    #[inline(never)]
    pub fn with_driver<F, R>(&mut self, driver_num: usize, f: F) -> R where
            F: FnOnce(Option<&hil::Driver>) -> R {
        match driver_num {
            1 => f(Some(self.gpio)),
            _ => f(None)
        }
    }
}

pub unsafe fn init<'a>() -> &'a mut Firestorm {
    use core::mem;
    use nrf51822::gpio::PA;

    static mut FIRESTORM_BUF : [u8; 1024] = [0; 1024];
    static mut GPIO_BUF : [u8; 1024] = [0; 1024];

    let gpio : &mut drivers::gpio::GPIO<[&'static hil::gpio::GPIOPin; 32]> = mem::transmute(&mut GPIO_BUF);

    *gpio = drivers::gpio::GPIO::new([
        &mut PA[0], &mut PA[1], &mut PA[2], &mut PA[3],
        &mut PA[4], &mut PA[5], &mut PA[6], &mut PA[7],
        &mut PA[8], &mut PA[9], &mut PA[10], &mut PA[11],
        &mut PA[12], &mut PA[13], &mut PA[14], &mut PA[15],
        &mut PA[16], &mut PA[17], &mut PA[18], &mut PA[19],
        &mut PA[20], &mut PA[21], &mut PA[22], &mut PA[23],
        &mut PA[24], &mut PA[25], &mut PA[26], &mut PA[27],
        &mut PA[28], &mut PA[29], &mut PA[30], &mut PA[31],
    ]);

    let firestorm : &'static mut Firestorm = mem::transmute(&mut FIRESTORM_BUF);
    *firestorm = Firestorm {
        gpio: gpio,
    };

    firestorm
}
