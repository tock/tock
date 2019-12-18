use crate::gpio;
use crate::interrupt_service::Nrf52840InterruptService;
use kernel::static_init;
use nrf52::chip::NRF52;

pub type Chip = NRF52<Nrf52840InterruptService>;

pub unsafe fn new() -> &'static Chip {
    let interrupt_service = static_init!(
        Nrf52840InterruptService,
        Nrf52840InterruptService::new(&gpio::PORT)
    );
    let chip = static_init!(Chip, NRF52::new(interrupt_service));
    chip
}
