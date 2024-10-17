// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::parsing::field::ParsedField;
use crate::parsing::{ErrorAccumulator, UNKNOWN_ATTRIBUTE};
use crate::Input;
use quote::format_ident;
use std::collections::HashSet;
use syn::parse::{self, Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{braced, parse2, Attribute, Error, Meta, Token};

// tock_registers; #[]* pub Foo { ... }
impl Parse for Input {
    fn parse(input: ParseStream) -> parse::Result<Input> {
        let mut errors = ErrorAccumulator::default();
        let tock_registers = input.parse()?;
        input.parse::<Token![;]>()?;
        let mut allow_bus_adapter = false;
        let mut cfgs = vec![];
        let mut comments = vec![];
        let mut real_name = None;
        // Process attributes on the peripheral definition, setting the
        // corresponding variables (allow_bus_adapter, cfgs, comments, and
        // real_name).
        for attr in input.call(Attribute::parse_outer)? {
            match attr.meta {
                Meta::Path(ref path) if path.is_ident("allow_bus_adapter") => {
                    if allow_bus_adapter {
                        errors.push(Error::new(path.span(), DUPLICATE_BUS_ADAPTER));
                    }
                    allow_bus_adapter = true;
                }
                Meta::List(ref list) if list.path.is_ident("cfg") => cfgs.push(attr),
                Meta::NameValue(ref name_value) if name_value.path.is_ident("doc") => {
                    comments.push(attr)
                }
                Meta::List(list) if list.path.is_ident("real") => {
                    if real_name.is_some() {
                        errors.push(Error::new_spanned(list, DUPLICATE_REAL_ATTR));
                        continue;
                    }
                    real_name = parse2(list.tokens).map_err(|e| errors.push(e)).ok();
                }
                _ => errors.push(Error::new_spanned(attr.meta, UNKNOWN_ATTRIBUTE)),
            }
        }
        let visibility = input.parse().map_err(|e| errors.push_take(e))?;
        let name = input.parse().map_err(|e| errors.push_take(e))?;
        let real_name = real_name.unwrap_or_else(|| format_ident!("Real{}", name));
        // Parse the fields, then combine them into a vector of all parsed
        // fields. Append any errors found during field parsing to errors.
        let fields: Vec<_> = (|| {
            let fields;
            braced!(fields in input);
            Ok(fields)
        })()
        .map_err(|e| errors.push_take(e))?
        .parse_terminated(ParsedField::parse, Token![,])
        .map_err(|e| errors.push_take(e))?
        .into_iter()
        .filter_map(|parsed| match parsed {
            ParsedField(Err(e)) => {
                errors.push(e);
                None
            }
            ParsedField(Ok(field)) => Some(field),
        })
        .collect();
        // Check for errors that involve multiple fields: duplicate names, and
        // padding with an undeterminable size.
        let mut names = HashSet::with_capacity(fields.len());
        let mut padding: Option<&Token![_]> = None;
        for field in &fields {
            if let (None, Some(underscore)) = (&field.offset, padding) {
                errors.push(Error::new(underscore.span(), UNSIZED_PADDING));
            }
            if let Some(register) = field.contents.register() {
                if !names.insert(&register.name) {
                    errors.push(Error::new(register.name.span(), DUPLICATE_NAME));
                }
            }
            padding = field.contents.padding();
        }
        if let Some(underscore) = padding {
            errors.push(Error::new(underscore.span(), UNSIZED_PADDING));
        }
        if let Some(error) = errors.into() {
            return Err(error);
        }
        Ok(Input {
            allow_bus_adapter,
            cfgs,
            comments,
            fields,
            name,
            real_name,
            tock_registers,
            visibility,
        })
    }
}

// Error messages.
const DUPLICATE_BUS_ADAPTER: &str = "duplicate #[allow_bus_adapter] attribute";
const DUPLICATE_NAME: &str = "duplicate register name";
const DUPLICATE_REAL_ATTR: &str = "duplicate #[real()] attribute";
const UNSIZED_PADDING: &str = "padding must be followed by a field with a specified offset";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::{assert_next_contains, NOT_A_DATA_TYPE};
    use crate::{Field, FieldContents, Register};
    use pretty_assertions::assert_eq;
    use quote::quote;
    use syn::{parse2, parse_quote};

    #[test]
    fn attr_and_name_errors() {
        let iter = &mut parse2::<Input>(quote![tock_registers;
            #[real(Real1)]
            #[allow_bus_adapter]
            #[real(Real2)]
            #[allow_bus_adapter]
            #[unknown_attr = "foo"]
            #[unknown_attr]
            123 {}
        ])
        .expect_err("parsing should have failed")
        .into_iter();
        assert_next_contains(iter, DUPLICATE_REAL_ATTR);
        assert_next_contains(iter, DUPLICATE_BUS_ADAPTER);
        assert_next_contains(iter, UNKNOWN_ATTRIBUTE);
        assert_next_contains(iter, UNKNOWN_ATTRIBUTE);
        assert_next_contains(iter, "expected identifier");
        assert!(iter.next().is_none());
    }

    #[test]
    fn complex() {
        assert_eq!(
            parse2::<Input>(quote![tock_registers;
                /// Doc comment
                #[allow_bus_adapter]
                #[cfg(feature = "a")]
                #[cfg(not(feature = "b"))]
                #[real(Bar)]
                pub Foo {
                    _   => _,
                    0x1 => ctrl: u16 {},
                }
            ])
            .expect("parsing failed"),
            Input {
                allow_bus_adapter: true,
                cfgs: vec![
                    parse_quote![#[cfg(feature = "a")]],
                    parse_quote![#[cfg(not(feature = "b"))]]
                ],
                comments: vec![parse_quote![#[doc = r" Doc comment"]]],
                fields: vec![
                    Field {
                        cfgs: vec![],
                        comments: vec![],
                        contents: FieldContents::Padding(parse_quote![_]),
                        offset: None,
                    },
                    Field {
                        cfgs: vec![],
                        comments: vec![],
                        contents: FieldContents::Register(Register {
                            data_type: parse_quote![u16],
                            name: parse_quote![ctrl],
                            read: None,
                            write: None,
                        }),
                        offset: Some(parse_quote![0x1]),
                    }
                ],
                name: parse_quote![Foo],
                real_name: parse_quote![Bar],
                tock_registers: parse_quote![tock_registers],
                visibility: parse_quote![pub],
            }
        );
    }

    #[test]
    fn duplicate_name_unsized_padding() {
        let iter = &mut parse2::<Input>(quote![tock_registers;
            Foo { _ => _, _ => abc: u8 {}, _ => abc: u8 {}, _ => _ }])
        .expect_err("parsing should have failed")
        .into_iter();
        assert_next_contains(iter, UNSIZED_PADDING);
        assert_next_contains(iter, DUPLICATE_NAME);
        assert_next_contains(iter, UNSIZED_PADDING);
        assert!(iter.next().is_none());
    }

    #[test]
    fn field_parse_unrecoverable_error() {
        // Include an unknown attribute to confirm that prior errors are
        // returned as well.
        let iter = &mut parse2::<Input>(quote![tock_registers;
            #[unknown_attr] Foo { _ => a: 123 }
        ])
        .expect_err("parsing should have failed")
        .into_iter();
        assert_next_contains(iter, UNKNOWN_ATTRIBUTE);
        assert_next_contains(iter, NOT_A_DATA_TYPE);
        assert!(iter.next().is_none());
    }

    #[test]
    fn field_parse_recoverable_errors() {
        // Include an unknown attribute on the peripheral to confirm that prior
        // errors are returned as well.
        let iter = &mut parse2::<Input>(quote![tock_registers;
            #[unknown_attr] Foo {
                #[unknown_attr] _ => _,
                _ => ctrl: u32 {},
                #[unknown_attr] _ => fifo: u16 {},
            }
        ])
        .expect_err("parsing should have failed")
        .into_iter();
        assert_next_contains(iter, UNKNOWN_ATTRIBUTE);
        assert_next_contains(iter, UNKNOWN_ATTRIBUTE);
        assert_next_contains(iter, UNKNOWN_ATTRIBUTE);
        assert!(iter.next().is_none());
    }

    #[test]
    fn simple() {
        assert_eq!(
            parse2::<Input>(quote![tock_registers; Foo {}]).expect("parsing failed"),
            Input {
                allow_bus_adapter: false,
                cfgs: vec![],
                comments: vec![],
                fields: vec![],
                name: parse_quote![Foo],
                real_name: parse_quote![RealFoo],
                tock_registers: parse_quote![tock_registers],
                visibility: parse_quote![],
            }
        );
    }
}
