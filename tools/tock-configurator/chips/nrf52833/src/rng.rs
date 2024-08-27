// Copyright OxidOS Automotive 2024.

use parse::constants::PERIPHERALS;
use parse::peripheral;
use quote::quote;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum RngType {
    Rng,
}

#[derive(Debug)]
#[peripheral(serde, ident = ".nrf52.trng")]
pub struct Rng(RngType);

impl parse::Component for Rng {
    fn ty(&self) -> Result<proc_macro2::TokenStream, parse::Error> {
        Ok(quote!(nrf52833::trng::Trng<'static>))
    }
}

impl parse::Rng for Rng {}

impl std::fmt::Display for Rng {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rng")
    }
}
