// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

mod definition;
mod deref_impl;
mod parsing;

#[cfg(test)]
mod test_util;

use definition::definition;
use deref_impl::deref_impl;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Attribute, Ident, LitInt, Path, Token, Type, Visibility};

#[proc_macro]
pub fn peripheral(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Input);
    let definition = definition(&input);
    let deref_impl = deref_impl(&input);
    quote! {
        #definition
        #deref_impl
    }
    .into()
}

#[cfg_attr(test, derive(Debug, PartialEq))]
struct Input {
    #[allow(dead_code)] // TODO: Remove
    pub allow_bus_adapter: bool,
    pub cfgs: Vec<Attribute>,
    pub comments: Vec<Attribute>,
    pub fields: Vec<Field>,
    pub name: Ident,
    #[allow(dead_code)] // TODO: Remove
    pub real_name: Ident,
    pub tock_registers: Path,
    pub visibility: Visibility,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
struct Field {
    pub cfgs: Vec<Attribute>,
    pub comments: Vec<Attribute>,
    pub contents: FieldContents,
    #[allow(dead_code)] // TODO: Remove
    pub offset: Option<LitInt>,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
enum FieldContents {
    Padding(Token![_]),
    Register(Register),
}

impl FieldContents {
    pub fn padding(&self) -> Option<&Token![_]> {
        match self {
            FieldContents::Padding(underscore) => Some(underscore),
            FieldContents::Register(_) => None,
        }
    }

    pub fn register(&self) -> Option<&Register> {
        match self {
            FieldContents::Padding(_) => None,
            FieldContents::Register(register) => Some(register),
        }
    }
}

#[cfg_attr(test, derive(Debug, PartialEq))]
struct Register {
    pub data_type: Type,
    pub long_names: LongNames,
    pub name: Ident,
    pub read: Option<Safety>,
    pub write: Option<Safety>,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
enum LongNames {
    Single(Type),
    Aliased(Aliased),
}

impl LongNames {
    pub fn read(&self) -> &Type {
        match self {
            LongNames::Single(name) => name,
            LongNames::Aliased(Aliased { read, .. }) => read,
        }
    }

    pub fn write(&self) -> &Type {
        match self {
            LongNames::Single(name) => name,
            LongNames::Aliased(Aliased { write, .. }) => write,
        }
    }
}

#[cfg_attr(test, derive(Debug, PartialEq))]
struct Aliased {
    pub read: Type,
    pub write: Type,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
enum Safety {
    Safe(Ident),
    Unsafe(Ident),
}

impl Safety {
    pub fn span(&self) -> Span {
        match self {
            Safety::Safe(ident) => ident.span(),
            Safety::Unsafe(ident) => ident.span(),
        }
    }
}
