// Copyright OxidOS Automotive 2024.

use proc_macro::TokenStream;
use quote::quote;

use syn::braced;
use syn::parse::{Parse, ParseStream};
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::token::{Brace, Comma};
use syn::{Generics, Ident, Token, Variant};

/// The input parsed for the `configuration_fields!` macro,
/// consisting of the enum ident definitions and the the list of fields.
#[derive(Debug)]
pub(crate) struct EnumsDefInput {
    group1: IdentAssoc,
    _tk1: Token![,],
    group2: FieldAssocList,
}

impl Parse for EnumsDefInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(EnumsDefInput {
            group1: input.parse()?,
            _tk1: input.parse()?,
            group2: input.parse()?,
        })
    }
}

/// The list of fields for the key enums and the value enums.
#[derive(Debug)]
pub(crate) struct FieldAssocList {
    _brace: Brace,
    fields: Punctuated<FieldAssoc, Comma>,
}

impl Parse for FieldAssocList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(FieldAssocList {
            _brace: braced!(content in input),
            fields: content.parse_terminated(FieldAssoc::parse, Token![,])?,
        })
    }
}

#[derive(Debug)]
pub(crate) struct IdentAssoc {
    ident1: Ident,
    _tk1: Token![=>],
    ident2: Ident,
    generics2: Generics,
}

impl Parse for IdentAssoc {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident1: input.parse()?,
            _tk1: input.parse()?,
            ident2: input.parse()?,
            generics2: input.parse()?,
        })
    }
}

#[derive(Debug)]
pub(crate) struct FieldAssoc {
    var1: Ident,
    _tk2: Token![=>],
    var2: Variant,
}

impl Parse for FieldAssoc {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            var1: input.parse()?,
            _tk2: input.parse()?,
            var2: input.parse()?,
        })
    }
}

pub(crate) fn gen_key_val(item: TokenStream) -> TokenStream {
    // Parse the input token stream.
    let input = parse_macro_input!(item as EnumsDefInput);

    // Retrieve the identifiers of the enums.
    let (ident1, ident2, generics2) = (
        input.group1.ident1,
        input.group1.ident2,
        input.group1.generics2,
    );

    // Iterate from inputs and separate the one from the first enum to the one in second enum.
    let (idents1, idents2): (Vec<_>, Vec<_>) = input
        .group2
        .fields
        .into_iter()
        .map(|field| (field.var1, field.var2))
        .unzip();

    quote!(
        #[derive(Debug, serde::Serialize, serde::Deserialize, Hash, PartialEq, Eq)]
        #[allow(non_camel_case_types)]
        #[non_exhaustive]
        pub enum #ident1 {
            #(#idents1),*
        }

        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(untagged)]
        #[non_exhaustive]
        #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        pub enum #ident2 #generics2 {
            #(#idents2),*
        }
    )
    .into()
}
