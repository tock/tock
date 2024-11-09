// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::parsing::ErrorAccumulator;
use crate::parsing::{
    MULTIPLE_SAME_OP, NOT_AN_OFFSET, NOT_A_DATA_TYPE, NOT_A_NAME, UNKNOWN_ATTRIBUTE, UNKNOWN_OP,
};
use crate::Safety::{Safe, Unsafe};
use crate::{Field, FieldContents, Register};
use syn::parse::{Parse, ParseStream};
use syn::{braced, Attribute, Error, Ident, Meta, Result, Token};

/// A field specification that has been completely parsed, but which may have
/// errors. Note that there are two types of parsing errors:
/// 1. Errors that prevent further parsing. These errors will make
///    `<ParsedField as syn::Parse>::parse` fail.
/// 2. Errors that do not prevent further parsing. These errors result in
///    `<ParsedField as syn::Parse>::parse` returning a `ParsedField(Err(...))`.
/// This distinction allows the proc macro to output as many errors as possible
/// on each invocation, rather than only reporting one error at a time.
#[cfg_attr(test, derive(Debug))]
pub struct ParsedField(pub Result<Field>);

// A field may either be padding or contain a register.
//
// Padding fields look like:
// ```ignore
// 0x1 => _,
// AAA    B
// ```
// Components:
// A. The padding's start offset. May be specified as _ to infer the offset.
// B. An underscore (rather than a name as a register has).
//
// Register fields look like:
// ```ignore
// 0x1 => ctrl: u32 { Read, Write },
// AAA    BBBB  CCC DDDDDDDDDDDDDDD
// ```
// Components:
// A. Start offset. Like padding, this may be _ to infer the offset.
// B. Register name.
// C. Register data type.
// D. Operation list (required, but may be empty).
impl Parse for ParsedField {
    fn parse(input: ParseStream) -> Result<ParsedField> {
        let mut errors = ErrorAccumulator::default();
        let mut cfgs = vec![];
        let mut comments = vec![];
        for attr in input.call(Attribute::parse_outer)? {
            match attr.meta {
                Meta::List(ref list) if list.path.is_ident("cfg") => cfgs.push(attr),
                Meta::NameValue(ref name_value) if name_value.path.is_ident("doc") => {
                    comments.push(attr)
                }
                _ => errors.push(Error::new_spanned(attr, UNKNOWN_ATTRIBUTE)),
            }
        }
        let offset = match input.parse::<Option<Token![_]>>()? {
            None => Some(match input.parse() {
                Err(_) => return Err(errors.push_take(input.error(NOT_AN_OFFSET))),
                Ok(offset) => offset,
            }),
            Some(_) => None,
        };
        input
            .parse::<Token![=>]>()
            .map_err(|e| errors.push_take(e))?;
        if let Some(underscore) = input.parse()? {
            // This is a padding field.
            return Ok(ParsedField(match errors.into() {
                None => Ok(Field {
                    cfgs,
                    comments,
                    contents: FieldContents::Padding(underscore),
                    offset,
                }),
                Some(err) => Err(err),
            }));
        }
        let name = input
            .parse()
            .map_err(|_| errors.push_take(input.error(NOT_A_NAME)))?;
        input
            .parse::<Token![:]>()
            .map_err(|e| errors.push_take(e))?;
        let data_type = input
            .parse()
            .map_err(|_| errors.push_take(input.error(NOT_A_DATA_TYPE)))?;
        let ops = (|| {
            let ops;
            braced!(ops in input);
            Ok(ops)
        })()
        .map_err(|e| errors.push_take(e))?
        .parse_terminated(Ident::parse, Token![,])
        .map_err(|e| errors.push_take(e))?;
        let mut read = None;
        let mut write = None;
        for op in ops {
            let (var, safety) = match op {
                op if op == "Read" => (&mut read, Safe(op)),
                op if op == "UnsafeRead" => (&mut read, Unsafe(op)),
                op if op == "Write" => (&mut write, Safe(op)),
                op if op == "UnsafeWrite" => (&mut write, Unsafe(op)),
                op => {
                    errors.push(Error::new(op.span(), UNKNOWN_OP));
                    continue;
                }
            };
            if var.is_some() {
                errors.push(Error::new(safety.span(), MULTIPLE_SAME_OP));
            }
            *var = Some(safety);
        }
        Ok(ParsedField(match errors.into() {
            None => Ok(Field {
                cfgs,
                comments,
                contents: FieldContents::Register(Register {
                    data_type,
                    name,
                    read,
                    write,
                }),
                offset,
            }),
            Some(error) => Err(error),
        }))
    }
}
