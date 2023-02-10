#![forbid(unsafe_code)]
#![no_std]

pub mod test;

#[macro_use]
pub mod stream;

pub mod adc;
pub mod alarm;
pub mod button;
pub mod console;
pub mod driver;
pub mod gpio;
pub mod i2c_master;
pub mod i2c_master_slave_driver;
pub mod led;
pub mod low_level_debug;
pub mod process_console;
pub mod rng;
pub mod spi_controller;
pub mod spi_peripheral;
pub mod virtual_adc;
pub mod virtual_aes_ccm;
pub mod virtual_alarm;
pub mod virtual_digest;
pub mod virtual_flash;
pub mod virtual_hmac;
pub mod virtual_i2c;
pub mod virtual_pwm;
pub mod virtual_rng;
pub mod virtual_sha;
pub mod virtual_spi;
pub mod virtual_timer;
pub mod virtual_uart;
