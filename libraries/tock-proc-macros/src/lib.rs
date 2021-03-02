//! Tock procedural macros

#![feature(proc_macro_diagnostic)]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate proc_macro2;

/// `#[derive(GrantDefault)]` implementation
mod derive_grant_default;

// Wrapper function required for exporting the macro, as it must be in
// the root of the crate and the `#[proc_macro_derive]` attribute also
// only works in the root of the crate.
#[proc_macro_derive(
    GrantDefault,
    attributes(subscribe_num, allow_num, grant_default_propagate)
)]
pub fn derive_grant_default(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_grant_default::derive_grant_default_impl(input)
}
