// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use std::collections::BTreeMap;

use proc_macro2::{Literal, TokenStream};
use syn::{parenthesized, punctuated::Punctuated, Token};

mod sections {
    syn::custom_keyword!(commands);
    syn::custom_keyword!(subscribes);
    syn::custom_keyword!(allow_ro);
    syn::custom_keyword!(allow_rw);
}

#[derive(Clone, Debug)]
enum Section {
    Commands { commands: Vec<CommandMapper> },
    Subscribes { subscribes: Vec<SubscribeMapper> },
    AllowRO {},
    AllowRW {},
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct Sig {
    ident: syn::Ident,
    inputs: Vec<syn::Ident>,
}

impl syn::parse::Parse for Sig {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let ident = input.parse()?;
        let content;
        parenthesized!(content in input);
        let inputs: Punctuated<syn::Ident, Token![,]> = Punctuated::parse_terminated(&content)?;
        Ok(Sig {
            ident,
            inputs: inputs.iter().map(Clone::clone).collect(),
        })
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct CommandMapper {
    comment: Option<String>,
    num: usize,
    signature: Sig,
    block: syn::Expr,
}

impl quote::ToTokens for CommandMapper {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let num = Literal::usize_unsuffixed(self.num);
        let block = &self.block;
        let inputs = self.signature.inputs.iter().enumerate().map(|(i, ident)| {
            let arg_ident = quote::format_ident!("arg{}", i);
            quote::quote! {
            let #ident = #arg_ident;
            }
        });
        quote::quote! {
            #num => {
            #(#inputs),*
            #block
            }
        }
        .to_tokens(tokens);
    }
}

impl syn::parse::Parse for CommandMapper {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let comment = if let Some(attr) = attrs.first() {
            let nv = attr.meta.require_name_value()?;
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(ref lstr),
                attrs: _,
            }) = nv.value
            {
                Some(lstr.value())
            } else {
                None
            }
        } else {
            None
        };
        let lit: syn::LitInt = input.parse()?;
        let num = lit.base10_parse::<usize>()?;
        input.parse::<Token![:]>()?;
        let signature = input.parse()?;
        input.parse::<Token![=>]>()?;
        let block = input.parse()?;

        Ok(CommandMapper {
            comment,
            num,
            signature,
            block,
        })
    }
}

impl syn::parse::Parse for SubscribeMapper {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let comment = if let Some(attr) = attrs.first() {
            let nv = attr.meta.require_name_value()?;
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(ref lstr),
                attrs: _,
            }) = nv.value
            {
                Some(lstr.value())
            } else {
                None
            }
        } else {
            None
        };
        let lit: syn::LitInt = input.parse()?;
        let num = lit.base10_parse::<usize>()?;
        input.parse::<Token![:]>()?;
        let signature = input.parse()?;

        Ok(SubscribeMapper {
            comment,
            num,
            signature,
        })
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct SubscribeMapper {
    comment: Option<String>,
    num: usize,
    signature: Sig,
}

struct Upcall(SubscribeMapper);

impl quote::ToTokens for Upcall {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let num = self.0.num;
        let ident = quote::format_ident!(
            "UPCALL_{}",
            self.0.signature.ident.to_string().to_uppercase()
        );
        quote::quote! {
            const #ident: usize = #num;
        }
        .to_tokens(tokens);
    }
}

impl syn::parse::Parse for Section {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(sections::commands) {
            input.parse::<sections::commands>()?;
            let content;
            syn::braced!(content in input);
            let commands: Punctuated<CommandMapper, Token![,]> =
                Punctuated::parse_terminated(&content)?;
            Ok(Section::Commands {
                commands: commands.iter().map(Clone::clone).collect(),
            })
        } else if lookahead.peek(sections::subscribes) {
            input.parse::<sections::subscribes>()?;
            let content;
            syn::braced!(content in input);
            let subscribes: Punctuated<SubscribeMapper, Token![,]> =
                Punctuated::parse_terminated(&content)?;
            Ok(Section::Subscribes {
                subscribes: subscribes.iter().map(Clone::clone).collect(),
            })
        } else if lookahead.peek(sections::allow_ro) {
            input.parse::<sections::allow_ro>()?;
            let _content;
            syn::braced!(_content in input);
            Ok(Section::AllowRO {})
        } else if lookahead.peek(sections::allow_rw) {
            input.parse::<sections::allow_rw>()?;
            let _content;
            syn::braced!(_content in input);
            Ok(Section::AllowRW {})
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug)]
struct DriverDef {
    struct_name: syn::Ident,
    generics: syn::Generics,
    commands: BTreeMap<usize, CommandMapper>,
    subscribes: BTreeMap<usize, SubscribeMapper>,
    allocate_grant: syn::ItemFn,
}

impl syn::parse::Parse for DriverDef {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let struct_name: syn::Ident = input.parse()?;
        let generics: syn::Generics = input.parse()?;
        let content: syn::parse::ParseBuffer;
        syn::braced!(content in input);
        let sections: Punctuated<Section, Token![,]> = Punctuated::parse_terminated(&content)?;

        let mut commands = BTreeMap::new();
        let command_sections: Vec<&Vec<CommandMapper>> = sections
            .iter()
            .filter_map(|s| {
                if let Section::Commands { commands } = s {
                    Some(commands)
                } else {
                    None
                }
            })
            .collect();
        if command_sections.len() > 1 {
            return Err(syn::Error::new(
                content.span(),
                "Only one command section allowed",
            ));
        }
        if let Some(command_section) = command_sections.first() {
            for command_mapper in command_section.iter() {
                commands.insert(command_mapper.num, command_mapper.clone());
            }
        }

        let mut subscribes = BTreeMap::new();
        let subscribe_sections: Vec<&Vec<SubscribeMapper>> = sections
            .iter()
            .filter_map(|s| {
                if let Section::Subscribes { subscribes } = s {
                    Some(subscribes)
                } else {
                    None
                }
            })
            .collect();
        if subscribe_sections.len() > 1 {
            return Err(syn::Error::new(
                content.span(),
                "Only one subscribe section allowed",
            ));
        }
        if let Some(subscribe_section) = subscribe_sections.first() {
            for subscribe_mapper in subscribe_section.iter() {
                subscribes.insert(subscribe_mapper.num, subscribe_mapper.clone());
            }
        }

        let allocate_grant = input.parse()?;
        Ok(DriverDef {
            struct_name,
            generics,
            commands,
            subscribes,
            allocate_grant,
        })
    }
}

#[proc_macro]
pub fn syscall_driver(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let driver_def = syn::parse_macro_input!(item as DriverDef);
    let struct_name = &driver_def.struct_name;
    let allocate_grant = driver_def.allocate_grant;
    let (impl_generics, ty_generics, where_clause) = driver_def.generics.split_for_impl();
    let grant_name = quote::format_ident!("{}Grant", driver_def.struct_name);
    let commands = driver_def.commands.values();
    let upcalls = driver_def.subscribes.values().map(|s| Upcall(s.clone()));

    let num_subscribes = driver_def
        .subscribes
        .keys()
        .max()
        .map_or(0, |x| (x + 1) as u8);
    let num_allow_ro = 0u8;
    let num_allow_rw = 0u8;

    let api_var_name = quote::format_ident!("__{}API", struct_name);
    use std::fmt::Write;
    let mut extractor = String::new();
    writeln!(extractor, "# {}", struct_name).unwrap();
    writeln!(extractor, "## Commands").unwrap();
    for command in driver_def.commands.values() {
        writeln!(extractor, "- {}:", command.num).unwrap();
        writeln!(
            extractor,
            "  - Comment: {}",
            command.comment.clone().unwrap_or_default().trim()
        )
        .unwrap();
        writeln!(extractor, "  - Command name: {}", command.signature.ident).unwrap();
        writeln!(
            extractor,
            "  - Command args: {:?}",
            command
                .signature
                .inputs
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
        )
        .unwrap();
    }

    writeln!(extractor, "## Subscribes").unwrap();
    for subscribe in driver_def.subscribes.values() {
        writeln!(extractor, "- {}:", subscribe.num).unwrap();
        writeln!(
            extractor,
            "  - Comment: {}",
            subscribe.comment.clone().unwrap_or_default().trim()
        )
        .unwrap();
        writeln!(
            extractor,
            "  - Callback name: {}",
            subscribe.signature.ident
        )
        .unwrap();
        writeln!(
            extractor,
            "  - Callback args: {:?}",
            subscribe
                .signature
                .inputs
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
        )
        .unwrap();
    }
    let tokens = quote::quote! {
    type #grant_name<A> =
        kernel::grant::Grant<
        A,
        kernel::grant::UpcallCount<#num_subscribes>,

        kernel::grant::AllowRoCount<#num_allow_ro>,
        AllowRwCount<#num_allow_rw>>;

    #(#upcalls)*

    #[doc = #extractor]
    impl #impl_generics kernel::syscall::SyscallDriver for #struct_name #ty_generics #where_clause {
        fn command(&self, command_num: usize, arg0: usize, arg1: usize, processid: kernel::ProcessId)
               -> kernel::syscall::CommandReturn {
        match command_num {
            #(#commands),*
            // default
            _ => kernel::syscall::CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
        }

        #allocate_grant

    }

    #[allow(non_upper_case_globals)]
    pub const #api_var_name: &str = #extractor;
        };
    tokens.into()
}
