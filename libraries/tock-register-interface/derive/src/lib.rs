extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{AngleBracketedGenericArguments, braced, Ident, LitInt, parse2, Token, TypePath};
use syn::parse::{self, Parse, ParseStream};
use syn::punctuated::Punctuated;

struct Operation {
    op_trait: TypePath,
    generics: Option<AngleBracketedGenericArguments>,
}

impl Parse for Operation {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        Ok(Self {
            op_trait: input.parse()?,
            generics: match input.peek(Token![<]) {
                false => None,
                true => Some(input.parse()?),
            },
        })
    }
}

struct Register {
    offset: LitInt,
    name: Ident,
    reg_type: TypePath,
    operations: Punctuated<Operation, Token![+]>,
}

impl Parse for Register {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let offset = input.parse()?;
        let _: Token![=>] = input.parse()?;
        let name = input.parse()?;
        let _: Token![:] = input.parse()?;
        let reg_type = input.parse()?;
        let operations;
        braced!(operations in input);
        Ok(Self {
            offset,
            name,
            reg_type,
            operations: Punctuated::parse_terminated(&operations)?,
        })
    }
}

struct Peripheral {
    name: Ident,
    registers: Punctuated<Register, Token![,]>,
}

impl Parse for Peripheral {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let name = input.parse()?;
        let registers;
        braced!(registers in input);
        Ok(Self {
            name,
            registers: Punctuated::parse_terminated(&registers)?,
        })
    }
}

fn peripheral_impl(input: TokenStream) -> TokenStream {
    let peripheral: Peripheral = match parse2(input) {
        Err(error) => return error.into_compile_error(),
        Ok(peripheral) => peripheral,
    };
    let peripheral_name = peripheral.name;
    let register_names: Vec<_> = peripheral.registers.iter().map(|r| r.name).collect();
    quote! {
        struct #peripheral_name {
            #(#register_names: ),*
        }
    }
}

#[proc_macro]
pub fn peripheral(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    peripheral_impl(input.into()).into()
}
