use capsules::{
    lmic_spi::LMICSpi,
    lora_controller::{Lora, MAX_LORA_PACKET_SIZE},
    virtual_spi::VirtualSpiMasterDevice,
};
use kernel::{capabilities, component::Component, create_capability, static_init, Kernel};

pub struct LMICSpiComponent {
    spi: &'static VirtualSpiMasterDevice<'static, nrf52840::spi::SPIM>,
}

impl LMICSpiComponent {
    pub fn new(
        spi: &'static VirtualSpiMasterDevice<'static, nrf52840::spi::SPIM>,
    ) -> LMICSpiComponent {
        LMICSpiComponent { spi }
    }
}

impl Component for LMICSpiComponent {
    type StaticInput = ();
    type Output = &'static LMICSpi<'static, VirtualSpiMasterDevice<'static, nrf52840::spi::SPIM>>;

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let lmic_spi: &LMICSpi<'static, VirtualSpiMasterDevice<'static, nrf52840::spi::SPIM>> = static_init!(
            LMICSpi<'static, VirtualSpiMasterDevice<'static, nrf52840::spi::SPIM>>,
            LMICSpi::new(self.spi)
        );
        self.spi.set_client(lmic_spi); // TODO: Check!
        lmic_spi
    }
}

pub struct LoraSyscallComponent {
    board_kernel: &'static Kernel,
    lmic: &'static LMICSpi<'static, VirtualSpiMasterDevice<'static, nrf52840::spi::SPIM>>,
}

impl LoraSyscallComponent {
    pub fn new(
        board_kernel: &'static Kernel,
        lmic: &'static LMICSpi<'static, VirtualSpiMasterDevice<'static, nrf52840::spi::SPIM>>,
    ) -> Self {
        LoraSyscallComponent { board_kernel, lmic }
    }
}

impl Component for LoraSyscallComponent {
    type StaticInput = ();
    type Output = &'static Lora<
        'static,
        LMICSpi<'static, VirtualSpiMasterDevice<'static, nrf52840::spi::SPIM>>,
    >;
    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let syscall_lora = static_init!(
            Lora<'static, LMICSpi<'static, VirtualSpiMasterDevice<'static, nrf52840::spi::SPIM>>>, // TODO: Somehow also gotta add virtual spi master device here... maybe it is easier to init lmic_spi here
            Lora::new(self.lmic, self.board_kernel.create_grant(&grant_cap))
        );

        let lora_read_buf = static_init!([u8; MAX_LORA_PACKET_SIZE], [0; MAX_LORA_PACKET_SIZE]);
        let lora_write_buf = static_init!([u8; MAX_LORA_PACKET_SIZE], [0; MAX_LORA_PACKET_SIZE]);

        syscall_lora.config_buffers(lora_read_buf, lora_write_buf);

        syscall_lora
    }
}
