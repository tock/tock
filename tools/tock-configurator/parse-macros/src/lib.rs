// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

use proc_macro::TokenStream;
mod component;
mod config_fields;

#[cfg(test)]
mod test;

#[proc_macro_attribute]
pub fn component(attrs: TokenStream, item: TokenStream) -> TokenStream {
    component::define_component(attrs, item)
}

#[proc_macro_attribute]
pub fn peripheral(attrs: TokenStream, item: TokenStream) -> TokenStream {
    component::define_peripheral(attrs, item)
}

#[proc_macro]
pub fn capsules_config(item: TokenStream) -> TokenStream {
    config_fields::gen_key_val(item)
}
