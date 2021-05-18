//! boards/nordic/nrf52840dk/src/test_lmic_spi.rs
/// Test sending LMIC commands over SPI
use capsules::{lmic_spi, virtual_spi};
use kernel::hil::{gpio, spi};

pub static mut A5: [u8; 16] = [0xA5; 16];

// #[allow(unused_variables, dead_code)]
pub unsafe fn lmic_spi_send_test(lmic_spi: &lmic_spi::LMICSpi) {
    let _ = lmic_spi.spi.read_write_bytes(&mut A5, None, A5.len());
}
