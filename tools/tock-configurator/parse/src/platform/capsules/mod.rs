// Copyright OxidOS Automotive 2024.

pub mod alarm;
pub use alarm::*;

pub mod console;
pub use console::*;

pub mod led;
pub use led::*;

pub mod spi_controller;
pub use spi_controller::*;

pub mod ble_radio;
pub use ble_radio::*;

pub mod lsm303agr;
pub use lsm303agr::*;

pub mod temperature;
pub use temperature::Temperature as TemperatureCapsule;

pub mod rng_capsule;
pub use rng_capsule::*;
