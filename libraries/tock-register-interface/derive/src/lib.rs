extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::collections::HashSet;
use syn::{AngleBracketedGenericArguments, braced, GenericArgument, Ident, LitInt, parse2, Token, TypePath};
use syn::parse::{self, Parse, ParseStream};
use syn::punctuated::Punctuated;

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

struct Peripheral {
    register_crate: Ident,
    name: Ident,
    registers: Punctuated<Register, Token![,]>,
}

struct Register {
    offset: LitInt,
    name: Ident,
    reg_type: TypePath,
    operations: Punctuated<Operation, Token![+]>,
}

struct Operation {
    op_trait: Ident,
    args: Punctuated<GenericArgument, Token![,]>,
}

fn peripheral_impl(input: TokenStream) -> TokenStream {
    let peripheral: Peripheral = match parse2(input) {
        Err(error) => return error.into_compile_error(),
        Ok(peripheral) => peripheral,
    };

    let mut accessor_where = vec![];
    let mut fields = vec![];
    let name = &peripheral.name;
    let mut ops = HashSet::new();
    let register_crate = &peripheral.register_crate;
    let mut struct_impls = vec![];
    for register in peripheral.registers {
        let name = register.name;
        let offset = register.offset;
        let reg_type = register.reg_type;
        accessor_where.push(quote! {
            #register_crate::ValueAt<#offset, Value = #reg_type>
        });
        fields.push(quote! { #name: #register_crate::Register<#offset, Self, A> });
        struct_impls.push(quote! {
            impl<A> #register_crate::ValueAt<#offset> for Registers<A> {
                type Value = #reg_type;
            }
        });
        for operation in register.operations {
            let op_trait = operation.op_trait;
            let args = operation.args;
            accessor_where.push(quote! {
                #op_trait::At<#offset, #args>
            });
            struct_impls.push(quote! {
                impl<A> #op_trait::Has<#offset, #args> for Registers<A> {}
            });
            ops.insert(op_trait);
        }
    }
    if fields.is_empty() {
        fields.push(quote!{ _accessor: A });
    }
    let mut ops: Vec<_> = ops.iter().map(|op| (op, op.to_token_stream().to_string())).collect();
    ops.sort_unstable_by(|(_, lhs), (_, rhs)| lhs.cmp(rhs));
    let ops: Vec<_> = ops.iter().map(|(op, _)| op).collect();
    quote! {
        mod #name {
            use super::{#(#ops),*};

            trait Accessor: #(#accessor_where)+* {}
            impl<A: #(#accessor_where)+*> Accessor for A {}

            struct Registers<A> {
                #(#fields),*
            }

            #(#struct_impls)*
        }
    }
}

#[proc_macro]
pub fn peripheral(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    peripheral_impl(input.into()).into()
}
