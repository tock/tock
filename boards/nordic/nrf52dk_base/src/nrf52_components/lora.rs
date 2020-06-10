use capsules::virtual_spi::VirtualSpiMasterDevice;
use kernel::component::Component;
use kernel::static_init;

use capsules::lora::driver::RadioDriver;
use capsules::lora::radio::Radio;

pub struct LoraComponent {
    radio: &'static Radio<'static, VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>>,
}

impl LoraComponent {
    pub fn new(
        radio: &'static Radio<'static, VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>>,
    ) -> LoraComponent {
        LoraComponent {
            radio: radio,
        }
    }
}

impl Component for LoraComponent {
    type StaticInput = ();
    type Output =
        (&'static RadioDriver<'static, VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>>,);

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let radio_driver = static_init!(
            RadioDriver<'static, VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>>,
            RadioDriver::new(self.radio,)
        );

        (radio_driver,)
    }
}
