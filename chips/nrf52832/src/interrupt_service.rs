use crate::gpio;
use nrf52::interrupt_service::InterruptService;

pub struct Nrf52832InterruptService {
    nrf52: nrf52::interrupt_service::Nrf52InterruptService,
}

impl Nrf52832InterruptService {
    pub unsafe fn new() -> Nrf52832InterruptService {
        Nrf52832InterruptService {
            nrf52: nrf52::interrupt_service::Nrf52InterruptService::new(&gpio::PORT),
        }
    }
}

impl InterruptService for Nrf52832InterruptService {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        // Add 52832-specific interrupts here.
        self.nrf52.service_interrupt(interrupt)
    }
}
