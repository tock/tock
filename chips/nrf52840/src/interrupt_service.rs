use crate::peripheral_interrupts;
use nrf52::interrupt_service::InterruptService;

pub struct Nrf52840InterruptService {
    nrf52: nrf52::interrupt_service::Nrf52InterruptService,
}

impl Nrf52840InterruptService {
    pub unsafe fn new(gpio_port: &'static nrf52::gpio::Port) -> Nrf52840InterruptService {
        Nrf52840InterruptService {
            nrf52: nrf52::interrupt_service::Nrf52InterruptService::new(gpio_port),
        }
    }
}

impl InterruptService for Nrf52840InterruptService {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            peripheral_interrupts::USBD => nrf52::usbd::USBD.handle_interrupt(),
            _ => return self.nrf52.service_interrupt(interrupt),
        }
        true
    }
}
