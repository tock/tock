// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use std::collections::BTreeMap;

use proc_macro2::{Literal, TokenStream};
use syn::{parenthesized, punctuated::Punctuated, Token};

mod sections {
    syn::custom_keyword!(commands);
}

mod allow_sections {
    syn::custom_keyword!(allow_ro);
    syn::custom_keyword!(allow_rw);
}

#[derive(Clone, Debug)]
enum Section {
    Commands { commands: Vec<CommandMapper> },
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct Sig {
    ident: syn::Ident,
    inputs: Vec<syn::Ident>,
}

/// The specific [`kernel::syscall::CommandReturn`] success constructor a
/// command number is declared to use. A command may still fail with any
/// [`ErrorCode`](kernel::ErrorCode)-carrying variant, but every success
/// path it takes must produce this exact variant: TRD104 requires a given
/// command number to always encode its success payload the same way, since
/// userspace decodes it according to a fixed, per-command schema.
#[derive(Clone, Copy, Debug)]
enum ReturnVariant {
    Success,
    SuccessU32,
    SuccessU32U32,
    SuccessU32U32U32,
    SuccessU64,
    SuccessU32U64,
}

impl ReturnVariant {
    /// The `CommandReturn::is_*` predicate that checks for this variant.
    fn is_variant_method(self) -> syn::Ident {
        let name = match self {
            ReturnVariant::Success => "is_success",
            ReturnVariant::SuccessU32 => "is_success_u32",
            ReturnVariant::SuccessU32U32 => "is_success_2_u32",
            ReturnVariant::SuccessU32U32U32 => "is_success_3_u32",
            ReturnVariant::SuccessU64 => "is_success_u64",
            ReturnVariant::SuccessU32U64 => "is_success_u32_u64",
        };
        quote::format_ident!("{}", name)
    }

    /// The DSL spelling of this variant, as used in `-> <variant>` and in
    /// the generated documentation.
    fn label(self) -> &'static str {
        match self {
            ReturnVariant::Success => "success",
            ReturnVariant::SuccessU32 => "success_u32",
            ReturnVariant::SuccessU32U32 => "success_u32_u32",
            ReturnVariant::SuccessU32U32U32 => "success_u32_u32_u32",
            ReturnVariant::SuccessU64 => "success_u64",
            ReturnVariant::SuccessU32U64 => "success_u32_u64",
        }
    }
}

impl syn::parse::Parse for ReturnVariant {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        match ident.to_string().as_str() {
            "success" => Ok(ReturnVariant::Success),
            "success_u32" => Ok(ReturnVariant::SuccessU32),
            "success_u32_u32" => Ok(ReturnVariant::SuccessU32U32),
            "success_u32_u32_u32" => Ok(ReturnVariant::SuccessU32U32U32),
            "success_u64" => Ok(ReturnVariant::SuccessU64),
            "success_u32_u64" => Ok(ReturnVariant::SuccessU32U64),
            other => Err(syn::Error::new(
                ident.span(),
                format!(
                    "unknown success return variant `{other}`; expected one of: \
                     success, success_u32, success_u32_u32, success_u32_u32_u32, \
                     success_u64, success_u32_u64"
                ),
            )),
        }
    }
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
    return_variant: ReturnVariant,
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
        let is_variant = self.return_variant.is_variant_method();
        let assert_message = format!(
            "command {} (`{}`) must always return CommandReturn::{}(..) on success",
            self.num,
            self.signature.ident,
            self.return_variant.label(),
        );
        quote::quote! {
            #num => {
            #(#inputs),*
            let __auto_command_return: kernel::syscall::CommandReturn = #block;
            debug_assert!(
                __auto_command_return.is_failure()
                    || __auto_command_return.is_failure_u32()
                    || __auto_command_return.is_failure_2_u32()
                    || __auto_command_return.is_failure_u64()
                    || __auto_command_return.#is_variant(),
                #assert_message
            );
            __auto_command_return
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
        input.parse::<Token![->]>()?;
        let return_variant = input.parse()?;
        input.parse::<Token![=>]>()?;
        let block = input.parse()?;

        Ok(CommandMapper {
            comment,
            num,
            signature,
            return_variant,
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
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Clone, Debug)]
enum AllowSection {
    AllowRO {},
    AllowRW {},
}

impl syn::parse::Parse for AllowSection {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(allow_sections::allow_ro) {
            input.parse::<allow_sections::allow_ro>()?;
            let _content;
            syn::braced!(_content in input);
            Ok(AllowSection::AllowRO {})
        } else if lookahead.peek(allow_sections::allow_rw) {
            input.parse::<allow_sections::allow_rw>()?;
            let _content;
            syn::braced!(_content in input);
            Ok(AllowSection::AllowRW {})
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug)]
struct DriverDef {
    struct_name: syn::Ident,
    #[allow(dead_code)]
    generics: syn::Generics,
    commands: BTreeMap<usize, CommandMapper>,
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
            .map(|Section::Commands { commands }| commands)
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

        Ok(DriverDef {
            struct_name,
            generics,
            commands,
        })
    }
}

#[proc_macro]
pub fn syscall_driver(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let driver_def = syn::parse_macro_input!(item as DriverDef);
    let struct_name = &driver_def.struct_name;
    let commands = driver_def.commands.values();

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
        writeln!(
            extractor,
            "  - Success return: {}",
            command.return_variant.label()
        )
        .unwrap();
    }

    let tokens = quote::quote! {
        #[doc = #extractor]
        fn command(&self, command_num: usize, arg0: usize, arg1: usize, processid: kernel::ProcessId)
               -> kernel::syscall::CommandReturn {
            match command_num {
                #(#commands),*
                // default
                _ => kernel::syscall::CommandReturn::failure(ErrorCode::NOSUPPORT),
            }
        }
    };
    tokens.into()
}

#[derive(Debug)]
struct SubscribesDef {
    struct_name: syn::Ident,
    #[allow(dead_code)]
    generics: syn::Generics,
    subscribes: BTreeMap<usize, SubscribeMapper>,
}

impl syn::parse::Parse for SubscribesDef {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let struct_name: syn::Ident = input.parse()?;
        let generics: syn::Generics = input.parse()?;
        let content: syn::parse::ParseBuffer;
        syn::braced!(content in input);
        let entries: Punctuated<SubscribeMapper, Token![,]> =
            Punctuated::parse_terminated(&content)?;

        let mut subscribes = BTreeMap::new();
        for subscribe_mapper in entries {
            subscribes.insert(subscribe_mapper.num, subscribe_mapper);
        }

        Ok(SubscribesDef {
            struct_name,
            generics,
            subscribes,
        })
    }
}

/// Generates the grant type alias and `UPCALL_*` constants for a
/// driver's subscribe numbers. Kept separate from `syscall_driver!` so
/// the upcall count and grant type (needed by the struct definition and
/// by code outside the `SyscallDriver` impl, such as callback methods)
/// don't depend on where the `command` dispatch is generated.
#[proc_macro]
pub fn subscribes(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let subscribes_def = syn::parse_macro_input!(item as SubscribesDef);
    let struct_name = &subscribes_def.struct_name;
    let grant_name = quote::format_ident!("{}Grant", struct_name);
    let upcalls = subscribes_def.subscribes.values().map(|s| Upcall(s.clone()));

    let num_subscribes = subscribes_def
        .subscribes
        .keys()
        .max()
        .map_or(0, |x| (x + 1) as u8);
    let num_allow_ro = 0u8;
    let num_allow_rw = 0u8;

    let tokens = quote::quote! {
    type #grant_name<A> =
        kernel::grant::Grant<
        A,
        kernel::grant::UpcallCount<#num_subscribes>,

        kernel::grant::AllowRoCount<#num_allow_ro>,
        AllowRwCount<#num_allow_rw>>;

    #(#upcalls)*
        };
    tokens.into()
}

#[derive(Debug)]
#[allow(dead_code)]
struct AllowDef {
    struct_name: syn::Ident,
    generics: syn::Generics,
}

impl syn::parse::Parse for AllowDef {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let struct_name: syn::Ident = input.parse()?;
        let generics: syn::Generics = input.parse()?;
        let content: syn::parse::ParseBuffer;
        syn::braced!(content in input);
        let _sections: Punctuated<AllowSection, Token![,]> = Punctuated::parse_terminated(&content)?;

        Ok(AllowDef {
            struct_name,
            generics,
        })
    }
}

/// Parses a driver's `allow_ro`/`allow_rw` sections. Neither section
/// carries any buffer entries yet (they're still `{}` placeholders), so
/// this currently just validates the syntax and generates nothing. Once
/// allow buffers are supported, this is where their `ALLOW_RO_*`/
/// `ALLOW_RW_*` constants and counts would be generated.
#[proc_macro]
pub fn allow(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _allow_def = syn::parse_macro_input!(item as AllowDef);
    quote::quote! {}.into()
}
