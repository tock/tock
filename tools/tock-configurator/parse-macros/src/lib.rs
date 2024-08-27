// Copyright OxidOS Automotive 2024.

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
