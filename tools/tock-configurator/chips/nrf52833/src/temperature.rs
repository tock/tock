// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

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
