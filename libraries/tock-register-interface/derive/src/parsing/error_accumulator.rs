// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use syn::Error;

/// Test utility that collects errors into a single `syn::Error`.
#[derive(Default)]
pub struct ErrorAccumulator(Option<Error>);

impl ErrorAccumulator {
    pub fn push(&mut self, new_error: Error) {
        match self {
            ErrorAccumulator(None) => *self = ErrorAccumulator(Some(new_error)),
            ErrorAccumulator(Some(error)) => error.combine(new_error),
        }
    }

    /// Appends the provided Error, then returns a combined Error. Leaves the
    /// ErrorAccumulator in an empty state.
    pub fn push_take(&mut self, new_error: Error) -> Error {
        let ErrorAccumulator(option) = self;
        let Some(mut error) = option.take() else {
            return new_error;
        };
        error.combine(new_error);
        error
    }
}

impl From<ErrorAccumulator> for Option<Error> {
    fn from(ErrorAccumulator(option): ErrorAccumulator) -> Option<Error> {
        option
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;

    #[test]
    fn into_option() {
        let mut errors = ErrorAccumulator::default();
        errors.push(Error::new(Span::call_site(), "AAAA"));
        errors.push(Error::new(Span::call_site(), "BBBB"));
        let option: Option<Error> = errors.into();
        let mut iter = option.expect("into() returned None").into_iter();
        assert_eq!(iter.next().expect("len = 0").to_string(), "AAAA");
        assert_eq!(iter.next().expect("len = 1").to_string(), "BBBB");
        assert!(iter.next().is_none(), "len > 2");
    }

    #[test]
    fn push_take() {
        let mut errors = ErrorAccumulator::default();
        errors.push(Error::new(Span::call_site(), "AAAA"));
        errors.push(Error::new(Span::call_site(), "BBBB"));
        let mut iter = errors
            .push_take(Error::new(Span::call_site(), "CCCC"))
            .into_iter();
        assert_eq!(iter.next().expect("len = 0").to_string(), "AAAA");
        assert_eq!(iter.next().expect("len = 1").to_string(), "BBBB");
        assert_eq!(iter.next().expect("len = 2").to_string(), "CCCC");
        assert!(iter.next().is_none(), "len > 3");
    }
}
