pub mod led;
pub mod time;
pub mod gpio;
pub mod i2c;
pub mod spi;
pub mod uart;
pub mod adc;

pub trait Controller {
    type Config;

    fn configure(&self, Self::Config);
}
