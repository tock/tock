//! Components for LoRa
//!
//! This provides the following components:
//!
//! * `LoraSyscallComponent` provides a controller system call interface to LoRa.

// use core::mem::MaybeUninit;

// use capsules::{
//     lmic_spi::LMICSpi,
//     lora_controller::{Lora, MAX_LORA_PACKET_SIZE},
//     virtual_spi::VirtualSpiMasterDevice,
// };
// use kernel::{capabilities, static_init};
// use kernel::{component::Component, create_capability, hil::lmic};

// pub struct LoraSyscallComponent<L: 'static + lmic::LMIC> {
//     board_kernel: &'static kernel::Kernel,
//     lmic: &'static L,
// }

// impl<L: 'static + lmic::LMIC> LoraSyscallComponent<L> {
//     pub fn new(board_kernel: &'static kernel::Kernel, lmic: &'static L) -> Self {
//         LoraSyscallComponent { board_kernel, lmic }
//     }
// }

// TODO: For less initialization in main.rs, could initialize lmic_spi component
// here instead. but to do that would need VirtualLMIC<'static, L>, where L is LMICSpi.
// VirtualLMIC would probably need it's own HIL with its own virtualization traits
// called LMICDevice or something lmao
// impl<L: 'static + lmic::LMIC> Component for LoraSyscallComponent<L> {
// type StaticInput = (&'static mut MaybeUninit<LMICSpi<'static>>);
// type Output = &'static Lora<'static, L>;
// unsafe fn finalize(self, static_memory: Self::StaticInput) -> Self::Output {
//     let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

//     // Initialize VirtualSpiMasterDevice here
//     // Then initialize LMIC_SPI here (but then it'd have to be nrf specific? - use another macro component helper that accepts nrf spi hw type)
//     // Or maybe the above two's static uninitialized buffers of allocated space
//     // will be passed in
//     // Then initialize loraSyscall

//     let syscall_lora = static_init!(
//         Lora<'static, LMICSpi<'static, VirtualSpiMasterDevice<'static, nrf52840::spi::SPIM>>>, // TODO: Somehow also gotta add virtual spi master device here... maybe it is easier to init lmic_spi here
//         Lora::new(self.lmic, self.board_kernel.create_grant(&grant_cap))
//     );

//     let lora_read_buf = static_init!([u8; MAX_LORA_PACKET_SIZE], [0; MAX_LORA_PACKET_SIZE]);
//     let lora_write_buf = static_init!([u8; MAX_LORA_PACKET_SIZE], [0; MAX_LORA_PACKET_SIZE]);

//     syscall_lora.config_buffers(lora_read_buf, lora_write_buf);

//     syscall_lora
// }
// }
