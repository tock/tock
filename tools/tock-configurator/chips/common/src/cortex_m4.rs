// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

use parse::{component, Component};
use quote::quote;

#[derive(Debug)]
#[component(serde, ident = "systick")]
pub struct Systick;

impl Component for Systick {
    fn ty(&self) -> Result<parse::proc_macro2::TokenStream, parse::Error> {
        Ok(quote! {
            cortexm4::systick::SysTick
        })
    }

    fn init_expr(&self) -> Result<parse::proc_macro2::TokenStream, parse::Error> {
        Ok(quote! {
            cortexm4::systick::SysTick::new_with_calibration(64000000)
        })
    }
}
