// Copyright OxidOS Automotive 2024.

//! Not fully supported yet.

use parse::constants::PERIPHERALS;
use parse::peripheral;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum FlashType {
    Flash0,
}

#[derive(Debug)]
#[peripheral(serde, ident = "flash")]
pub struct Flash(FlashType);

impl parse::Component for Flash {}
impl parse::Flash for Flash {}

impl std::fmt::Display for Flash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "flash")
    }
}
