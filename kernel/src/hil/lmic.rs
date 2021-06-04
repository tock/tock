//! Interfaces for LoRa-MAC-in-C commands

use crate::ErrorCode;

pub trait LMIC {
    // Prepare upstream data transmission at the next possible time on LoRa
    // radio. Corresponds to call to LMIC's LMIC_setTxData2() API call.
    // NOTE: According to HIL rules, need to include tx_data buffer in ErrorCode
    fn set_tx_data(&self, tx_data: &'static mut [u8], len: u8) -> Result<(), ErrorCode>;
}
