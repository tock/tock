// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

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
