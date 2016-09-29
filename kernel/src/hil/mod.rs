pub mod led;
pub mod alarm;
pub mod gpio;
pub mod i2c;
pub mod spi;
pub mod timer;
pub mod uart;
pub mod adc;
pub mod flash;

pub trait Controller {
    type Config;

    fn configure(&self, Self::Config);
}
