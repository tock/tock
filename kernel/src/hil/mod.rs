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
pub mod temperature;
pub mod crc;
pub mod symmetric_encryption;

pub trait Controller {
    type Config;

    fn configure(&self, Self::Config);
}
