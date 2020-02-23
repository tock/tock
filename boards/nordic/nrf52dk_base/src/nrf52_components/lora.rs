
use capsules::virtual_spi::VirtualSpiMasterDevice;
use kernel::component::Component;
use kernel::capabilities;
use kernel::{create_capability, static_init};

use capsules::lora::radio::{Radio};
use capsules::lora::driver::{RadioDriver};

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
    type Output = (
        &'static RadioDriver<'static, VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>>,
    );

    unsafe fn finalize(&mut self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        self.radio.begin(865000000);
        self.radio.beginPacket(true);

        let radio_driver = static_init!(
            RadioDriver<'static, VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>>,
            RadioDriver::new(
                self.radio,
                self.board_kernel.create_grant(&grant_cap),
            )
        );
        
        (radio_driver,)
    }
}
