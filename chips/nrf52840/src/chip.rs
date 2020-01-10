use crate::interrupt_service::Nrf52840InterruptService;
use kernel::static_init;
use nrf52::chip::NRF52;

pub unsafe fn new() -> &'static NRF52<Nrf52840InterruptService> {
    let interrupt_service = static_init!(Nrf52840InterruptService, Nrf52840InterruptService::new());
    let chip = static_init!(
        NRF52<Nrf52840InterruptService>,
        NRF52::new(interrupt_service)
    );
    chip
}
