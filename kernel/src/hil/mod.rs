//! Public traits for interfaces between Tock components.

pub mod adc;
pub mod ble_advertising;
pub mod crc;
pub mod dac;
pub mod flash;
pub mod gpio;
pub mod gpio_async;
pub mod i2c;
pub mod led;
pub mod nonvolatile_storage;
pub mod radio;
pub mod rng;
pub mod sensors;
pub mod spi;
pub mod symmetric_encryption;
pub mod time;
pub mod uart;
pub mod usb;
pub mod watchdog;
pub mod entropy;

/// Shared interface for configuring components.
pub trait Controller {
    type Config;

    fn configure(&self, Self::Config);
}
