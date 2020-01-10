use crate::interrupt_service::Nrf52840InterruptService;
use nrf52::chip::NRF52;

pub type Chip = NRF52<Nrf52840InterruptService>;

pub unsafe fn new() -> Chip {
    NRF52::new(Nrf52840InterruptService::new())
}
