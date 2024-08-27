// Copyright OxidOS Automotive 2024.

//! Not fully supported yet.

use parse::constants::PERIPHERALS;
use parse::peripheral;

#[derive(Debug)]
#[peripheral(serde, ident = "twi")]
pub struct Twi {}

impl parse::I2c for Twi {}
impl parse::Component for Twi {}

impl std::fmt::Display for Twi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "twi")
    }
}
