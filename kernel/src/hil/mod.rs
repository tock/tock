//! Public traits for interfaces between Tock components.

pub mod led;
pub mod time;
pub mod gpio;
pub mod i2c;
pub mod spi;
pub mod uart;
pub mod rng;
pub mod adc;
pub mod flash;
pub mod watchdog;
pub mod radio;
pub mod sensors;
pub mod crc;
pub mod symmetric_encryption;
pub mod gpio_async;
pub mod dac;
pub mod nonvolatile_storage;
pub mod usb;

/// Shared interface for configuring components.
pub trait Controller {
    type Config;

    fn configure(&self, Self::Config);
}
