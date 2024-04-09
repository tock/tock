// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::parsing::{maybe_long_name, ErrorAccumulator, OpSpec};
use crate::parsing::{
    MULTIPLE_SAME_OP, NOT_AN_OFFSET, NOT_A_DATA_TYPE, NOT_A_NAME, OP_LONG_NAME_SINGLE_OP,
    SHARED_AND_OP_LONG_NAME, UNKNOWN_ATTRIBUTE, UNKNOWN_OP,
};
use crate::Safety::{Safe, Unsafe};
use crate::{Aliased, Field, FieldContents, LongNames, Register};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{braced, parse_quote, Attribute, Error, Meta, Result, Token};

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
// 0x1 => ctrl(Ctrl): u32 { Read, Write },
// AAA    BBBBCCCCCC  DDD EEEEEEEEEEEEEEE
// ```
// Components:
// A. Start offset. Like padding, this may be _ to infer the offset.
// B. Register name.
// C. Register long name (optional).
// D. Register data type.
// E. Operation list (required, but may be empty).
//
// Note that long names may be specified on the operation list instead of the
// register type, which is necessary if the long name differs between operation
// types:
// ```ignore
// 0x1 => fifo: u32 { Read(RxByte), Write(TxByte) },
// ```
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
        let shared_long_name = maybe_long_name(input).map_err(|e| errors.push_take(e))?;
        let op_specs = (|| {
            let op_specs;
            braced!(op_specs in input);
            Ok(op_specs)
        })()
        .map_err(|e| errors.push_take(e))?
        .parse_terminated(OpSpec::parse, Token![,])
        .map_err(|e| errors.push_take(e))?;
        // (Safety, long_name: Option<Type>)
        let mut read = None;
        let mut write = None;
        for op_spec in op_specs {
            let (var, safety) = match op_spec.name {
                name if name == "Read" => (&mut read, Safe(name)),
                name if name == "UnsafeRead" => (&mut read, Unsafe(name)),
                name if name == "Write" => (&mut write, Safe(name)),
                name if name == "UnsafeWrite" => (&mut write, Unsafe(name)),
                name => {
                    errors.push(Error::new(name.span(), UNKNOWN_OP));
                    continue;
                }
            };
            if var.is_some() {
                errors.push(Error::new(safety.span(), MULTIPLE_SAME_OP));
            }
            *var = Some((safety, op_spec.long_name));
        }
        let (read_safety, read_long_name) = read.unzip();
        let (write_safety, write_long_name) = write.unzip();
        // To determine the correct LongNames to use, we first look only at
        // the operations list. If LongName(s) were specified in the
        // operations list, this sets op_long_names to Some(...). If not,
        // this sets op_long_names to None.
        #[deny(clippy::match_overlapping_arm)]
        let op_long_names = match (read_long_name, write_long_name) {
            // Cases where no long name is specified.
            (None | Some(None), None | Some(None)) => None,
            // Cases where a single op was specified, and it has a long
            // name.
            (None, Some(Some(long_name))) | (Some(Some(long_name)), None) => {
                errors.push(Error::new(long_name.span(), OP_LONG_NAME_SINGLE_OP));
                None
            }
            // Cases where both ops were specified, and only one has a long
            // name.
            (Some(Some(read)), Some(None)) => Some(LongNames::Aliased(Aliased {
                read,
                write: parse_quote![()],
            })),
            (Some(None), Some(Some(write))) => Some(LongNames::Aliased(Aliased {
                read: parse_quote![()],
                write,
            })),
            // Case where both ops have long names.
            (Some(Some(read)), Some(Some(write))) => {
                Some(LongNames::Aliased(Aliased { read, write }))
            }
        };
        // Second, combined the LongName specified on the data type with
        // the LongName(s) specified in the operations list to compute the
        // register's actual LongNames.
        let long_names = match (shared_long_name, op_long_names) {
            (None, None) => LongNames::Single(parse_quote![()]),
            (None, Some(name)) => name,
            (Some(name), None) => LongNames::Single(name),
            (Some(name), Some(_)) => {
                errors.push(Error::new(name.span(), SHARED_AND_OP_LONG_NAME));
                LongNames::Single(name)
            }
        };
        Ok(ParsedField(match errors.into() {
            None => Ok(Field {
                cfgs,
                comments,
                contents: FieldContents::Register(Register {
                    data_type,
                    long_names,
                    name,
                    read: read_safety,
                    write: write_safety,
                }),
                offset,
            }),
            Some(error) => Err(error),
        }))
    }
}
