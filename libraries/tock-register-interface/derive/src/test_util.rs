// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use pretty_assertions::StrComparison;
use prettyplease::unparse;
use proc_macro2::TokenStream;
use syn::parse2;

/// Asserts that two token streams are identical. If they differ, panics with a
/// pretty-printed diff.
#[track_caller]
pub fn assert_tokens_eq(left: TokenStream, right: TokenStream) {
    if let Err((left, right)) = assert_tokens_eq_impl(left, right) {
        panic!("left != right:\n{}", StrComparison::new(&left, &right));
    }
}

/// Testable implementation of `assert_tokens_eq`. Returns `Ok` if
/// `assert_tokens_eq` should succeed and
/// `Err((left string to diff, right string to diff))` if it should fail.
fn assert_tokens_eq_impl(left: TokenStream, right: TokenStream) -> Result<(), (String, String)> {
    let (left_string, right_string) = (left.to_string(), right.to_string());
    if left_string == right_string {
        return Ok(());
    }
    // Attempt to parse and format both inputs. If parsing either input fails,
    // diff the unformatted strings.
    let (left, right) = match (parse2(left), parse2(right)) {
        (Ok(left), Ok(right)) => (unparse(&left), unparse(&right)),
        _ => return Err((left_string, right_string)),
    };
    // Formatting the inputs can result in identical strings, in which case we
    // should diff the unformatted strings.
    Err(match left == right {
        true => (left_string, right_string),
        false => (left, right),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::File;

    #[test]
    fn assert_tokens_eq() {
        // Two different TokenStreams that don't parse as a syn::File.
        let no_parse1 = quote! { let x = 1; };
        let no_parse2 = quote! { let x = 2; };
        // Just confirm these don't parse.
        assert!(parse2::<File>(no_parse1.clone()).is_err());
        assert!(parse2::<File>(no_parse2.clone()).is_err());
        // Two different TokenStreams that format to the same string.
        let eq_formatted1 = quote! { struct A { var: (), } };
        let eq_formatted2 = quote! { struct A { var: () } };
        let eq_formatted_string = unparse(&parse2(eq_formatted1.clone()).unwrap());
        // Confirm they do format to the same string.
        assert_eq!(
            eq_formatted_string,
            unparse(&parse2(eq_formatted2.clone()).unwrap())
        );
        // A TokenStream that parses and differs from eq_formatted even after
        // formatting.
        let different = quote! { enum Foo {} };
        let different_string = unparse(&parse2(different.clone()).unwrap());

        // Two inputs that match (both unparsable and parsable).
        assert_eq!(
            assert_tokens_eq_impl(no_parse1.clone(), no_parse1.clone()),
            Ok(())
        );
        assert_eq!(
            assert_tokens_eq_impl(eq_formatted1.clone(), eq_formatted1.clone()),
            Ok(())
        );

        // One input that parses, one that does not.
        assert_eq!(
            assert_tokens_eq_impl(no_parse1.clone(), eq_formatted1.clone()),
            Err((no_parse1.to_string(), eq_formatted1.to_string()))
        );
        assert_eq!(
            assert_tokens_eq_impl(eq_formatted1.clone(), no_parse1.clone()),
            Err((eq_formatted1.to_string(), no_parse1.to_string()))
        );

        // Two different inputs that don't parse.
        assert_eq!(
            assert_tokens_eq_impl(no_parse1.clone(), no_parse2.clone()),
            Err((no_parse1.to_string(), no_parse2.to_string()))
        );

        // Two inputs that parse, but are distinct.
        assert_eq!(
            assert_tokens_eq_impl(eq_formatted1.clone(), different),
            Err((eq_formatted_string, different_string))
        );

        // Two distinct inputs that format to the same string.
        assert_eq!(
            assert_tokens_eq_impl(eq_formatted1.clone(), eq_formatted2.clone()),
            Err((eq_formatted1.to_string(), eq_formatted2.to_string()))
        );
    }
}
