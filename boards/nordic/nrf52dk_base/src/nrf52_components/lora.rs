use capsules::virtual_spi::VirtualSpiMasterDevice;
use kernel::capabilities;
use kernel::component::Component;
use kernel::{create_capability, static_init};

use capsules::lora::driver::RadioDriver;
use capsules::lora::radio::Radio;

pub struct LoraComponent {
    board_kernel: &'static kernel::Kernel,
    radio: &'static Radio<'static, VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>>,
}

impl LoraComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        radio: &'static Radio<'static, VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>>,
    ) -> LoraComponent {
        LoraComponent {
            board_kernel: board_kernel,
            radio: radio,
        }
    }
}

impl Component for LoraComponent {
    type StaticInput = ();
    type Output =
        (&'static RadioDriver<'static, VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>>,);

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let radio_driver = static_init!(
            RadioDriver<'static, VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>>,
            RadioDriver::new(self.radio, self.board_kernel.create_grant(&grant_cap),)
        );

        (radio_driver,)
    }
}
