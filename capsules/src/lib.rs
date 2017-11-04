#![feature(const_fn,const_cell_new)]
#![forbid(unsafe_code)]
#![no_std]

#[allow(unused_imports)]
#[macro_use(debug)]
extern crate kernel;

pub mod alarm;
pub mod ambient_light;
pub mod button;
pub mod console;
pub mod fm25cl;
pub mod gpio;
pub mod isl29035;
pub mod led;
pub mod nrf51822_serialization;
pub mod tmp006;
pub mod sdcard;
pub mod si7021;
pub mod spi;
pub mod virtual_alarm;
pub mod virtual_flash;
pub mod virtual_i2c;
pub mod virtual_spi;
pub mod adc;
pub mod dac;
pub mod i2c_master_slave_driver;
pub mod lps25hb;
pub mod tsl2561;
pub mod fxos8700cq;
pub mod crc;
pub mod rf233;
pub mod rf233_const;
pub mod rng;
pub mod symmetric_encryption;
pub mod ninedof;
pub mod ltc294x;
pub mod mcp23008;
pub mod gpio_async;
pub mod max17205;
pub mod pca9544a;
pub mod nonvolatile_to_pages;
pub mod nonvolatile_storage_driver;
pub mod app_flash_driver;
pub mod usb;
pub mod usb_user;
pub mod usbc_client;
#[macro_use]
pub mod net;
pub mod ieee802154;
pub mod temperature;
pub mod humidity;
//pub mod nrf_internal_temp_sensor;
