// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::parsing::NOT_A_DATA_TYPE;
use syn::parse::{Parse, ParseStream};
use syn::token::Paren;
use syn::{parenthesized, Ident, Type};

/// Specification of an operation, such as `UnsafeRead`, `Read`, or
/// `Read(Ctrl)`.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct OpSpec {
    pub name: Ident,
    pub long_name: Option<Type>,
}

impl Parse for OpSpec {
    fn parse(input: ParseStream) -> syn::Result<OpSpec> {
        Ok(OpSpec {
            name: input.parse()?,
            long_name: maybe_long_name(input)?,
        })
    }
}

/// Parses a long name specification. A long name specification may be empty (in
/// which case this returns `None`) or may consist of a type in parenthesis
/// (e.g. `(Ctrl)`).
pub fn maybe_long_name(input: ParseStream) -> syn::Result<Option<Type>> {
    if !input.peek(Paren) {
        return Ok(None);
    }
    let long_name;
    parenthesized!(long_name in input);
    match long_name.parse() {
        Err(_) => Err(long_name.error(NOT_A_DATA_TYPE)),
        Ok(long_name) => Ok(Some(long_name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::assert_next_contains;
    use quote::quote;
    use syn::{parse2, parse_quote};

    #[test]
    fn invalid_long_name() {
        let iter = &mut parse2::<OpSpec>(quote![Read(1)])
            .expect_err("parsing a bad long name did not fail")
            .into_iter();
        assert_next_contains(iter, NOT_A_DATA_TYPE);
        assert!(iter.next().is_none());
    }

    #[test]
    fn invalid_op_name() {
        let iter = &mut parse2::<OpSpec>(quote![3])
            .expect_err("parsing a bad op name did not fail")
            .into_iter();
        assert_next_contains(iter, "expected identifier");
        assert!(iter.next().is_none());
    }

    #[test]
    fn no_long_name() {
        assert_eq!(
            parse2::<OpSpec>(quote![Read]).expect("parsing a valid OpSpec failed"),
            OpSpec {
                name: parse_quote![Read],
                long_name: None,
            }
        );
    }

    #[test]
    fn with_long_name() {
        assert_eq!(
            parse2::<OpSpec>(quote![Read(Ctrl)]).expect("parsing a valid OpSpec failed"),
            OpSpec {
                name: parse_quote![Read],
                long_name: Some(parse_quote![Ctrl]),
            }
        );
    }
}
