use crate::interrupt_service::Nrf52832InterruptService;
use nrf52::chip::NRF52;

pub type Chip = NRF52<Nrf52832InterruptService>;

pub unsafe fn new() -> Chip {
    NRF52::new(Nrf52832InterruptService::new())
}
