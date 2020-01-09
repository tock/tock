use crate::gpio;
use crate::interrupt_service::Nrf52832InterruptService;
use kernel::static_init;
use nrf52::chip::NRF52;

pub unsafe fn new() -> &'static NRF52<Nrf52832InterruptService> {
    let interrupt_service = static_init!(
        Nrf52832InterruptService,
        Nrf52832InterruptService::new(&gpio::PORT)
    );
    let chip = static_init!(
        NRF52<Nrf52832InterruptService>,
        NRF52::new(interrupt_service)
    );
    chip
}
