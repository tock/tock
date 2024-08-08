// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::Input;
use proc_macro2::TokenStream;
use quote::quote;

/// Generates the blanket implementation of the trait for types that deref to an
/// implementation of the trait.
pub fn deref_impl(input: &Input) -> TokenStream {
    let cfgs = &input.cfgs;
    let tock_registers = &input.tock_registers;
    let input_name = &input.name;
    let registers = input.fields.iter().filter_map(|field| {
        let cfgs = &field.cfgs;
        let register_name = &field.contents.register()?.name;
        Some(quote! {
            #(#cfgs)*
            type #register_name<'s> = <T::Target as #input_name>::#register_name<'s> where Self: 's;
            #(#cfgs)*
            fn #register_name(&self) -> Self::#register_name<'_> {
                self.deref().#register_name()
            }
        })
    });
    quote! {
        #(#cfgs)*
        impl<T: #tock_registers::reexport::core::ops::Deref> #input_name for T where T::Target: #input_name {
            #(#registers)*
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_tokens_eq;
    use syn::parse_quote;

    #[test]
    fn test() {
        assert_tokens_eq(
            deref_impl(&parse_quote! {tock_registers;
                #[cfg(feature = "a")]
                #[cfg(feature = "b")]
                Foo {
                    #[cfg(feature = "c")]
                    #[cfg(feature = "d")]
                    0x0 => a: u8 { Read },

                    #[cfg(feature = "e")]
                    #[cfg(feature = "f")]
                    _ => _,

                    #[cfg(feature = "g")]
                    #[cfg(feature = "h")]
                    0x3 => b: u8 { Write },
                }
            }),
            quote! {
                #[cfg(feature = "a")]
                #[cfg(feature = "b")]
                impl<T: tock_registers::reexport::core::ops::Deref> Foo for T
                where T::Target: Foo {
                    #[cfg(feature = "c")]
                    #[cfg(feature = "d")]
                    type a<'s> = <T::Target as Foo>::a<'s> where Self: 's;
                    #[cfg(feature = "c")]
                    #[cfg(feature = "d")]
                    fn a(&self) -> Self::a<'_> { self.deref().a() }

                    #[cfg(feature = "g")]
                    #[cfg(feature = "h")]
                    type b<'s> = <T::Target as Foo>::b<'s> where Self: 's;
                    #[cfg(feature = "g")]
                    #[cfg(feature = "h")]
                    fn b(&self) -> Self::b<'_> { self.deref().b() }
                }
            },
        );
    }
}
