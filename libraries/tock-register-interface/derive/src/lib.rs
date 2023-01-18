extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{AngleBracketedGenericArguments, braced, GenericArgument, Ident, LitInt, parse2, Token, TypePath};
use syn::parse::{self, Parse, ParseStream};
use syn::punctuated::Punctuated;

struct Operation {
    op_trait: TypePath,
    args: Punctuated<GenericArgument, Token![,]>,
}

impl Parse for Operation {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        Ok(Self {
            op_trait: input.parse()?,
            args: match input.peek(Token![<]) {
                false => Punctuated::new(),
                true => {
                    let args: AngleBracketedGenericArguments = input.parse()?;
                    args.args
                },
            },
        })
    }
}

impl Operation {
    fn impls(&self, peripheral: &Peripheral, register: &Register) -> TokenStream {
        let args = &self.args;
        let name = &peripheral.name;
        let offset = &register.offset;
        let op_trait = &self.op_trait;
        let register_crate = &peripheral.register_crate;
        quote! {
            impl<Accessor: #op_trait::Access<#offset, #args>> #op_trait::Has<#offset, #args> for #name<Accessor> {
            }
        }
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

impl Register {
    fn struct_field(&self, register_crate: &Ident) -> TokenStream {
        let name = &self.name;
        let offset = &self.offset;
        quote! { #name: #register_crate::Register<#offset, Accessor> }
    }
}

struct Peripheral {
    register_crate: Ident,
    name: Ident,
    registers: Punctuated<Register, Token![,]>,
}

impl Parse for Peripheral {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let register_crate = input.parse()?;
        let name = input.parse()?;
        let registers;
        braced!(registers in input);
        Ok(Self {
            register_crate,
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
    let fields = peripheral.registers.iter().map(|r| r.struct_field(&peripheral.register_crate));
    quote! {
        struct #peripheral_name<Accessor> {
            #(#fields),*
        }
    }
}

#[proc_macro]
pub fn peripheral(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    peripheral_impl(input.into()).into()
}
