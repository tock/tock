use crate::peripheral_interrupts;
use nrf52::chip::InterruptServiceTrait;

pub struct InterruptService {
    nrf52: nrf52::chip::InterruptService,
}

impl InterruptService {
    pub unsafe fn new(gpio_port: &'static nrf52::gpio::Port) -> InterruptService {
        InterruptService {
            nrf52: nrf52::chip::InterruptService::new(gpio_port),
        }
    }
}

impl InterruptServiceTrait for InterruptService {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            peripheral_interrupts::USBD => nrf52::usbd::USBD.handle_interrupt(),
            _ => return self.nrf52.service_interrupt(interrupt),
        }
        true
    }
}
