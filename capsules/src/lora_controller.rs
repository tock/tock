//! Provides userspace applications with the ability to communicate over the
//! LoRa network.

use kernel::hil::lmic::LMIC;
use kernel::Driver;

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Lora as usize;

// Temporary. TODO: remove once I figure out how to read buffers passed to kernel
// by app
static mut BUF: [u8; 16] = [0xA5; 16];

pub struct Lora<'a, L: LMIC> {
    lora_device: &'a L,
}

impl<'a, L: LMIC> Lora<'a, L> {
    pub fn new(lora_device: &'a L) -> Lora<'a, L> {
        Lora { lora_device }
    }

    fn do_set_tx_data(&self) {
        // TODO: somehow will have read from app what app wants to send over
        // LoRa network and then call
        // let _ = self.lora_device.set_tx_data(&mut BUF);
    }
}

impl<'a, L: LMIC> Driver for Lora<'a, L> {}
