// Copyright OxidOS Automotive 2024.

//! Not fully supported yet.

use parse::constants::PERIPHERALS;
use parse::peripheral;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum BleType {
    RadioBle,
}

#[derive(Debug)]
#[peripheral(serde, ident = "ble")]
pub struct Ble(BleType);

impl parse::Component for Ble {}
impl parse::BleAdvertisement for Ble {}

impl std::fmt::Display for Ble {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ble_radio")
    }
}
