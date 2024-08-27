// Copyright OxidOS Automotive 2024.

use parse::constants::PERIPHERALS;
use parse::peripheral;
use quote::quote;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum TemperatureType {
    Temp,
}

#[derive(Debug)]
#[peripheral(serde, ident = ".nrf52.temp")]
pub struct Temperature(TemperatureType);

impl parse::Component for Temperature {
    fn ty(&self) -> Result<parse::proc_macro2::TokenStream, parse::Error> {
        Ok(quote!(nrf52::temperature::Temp<'static>))
    }
}

impl parse::Temperature for Temperature {}

impl std::fmt::Display for Temperature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "temperature")
    }
}
