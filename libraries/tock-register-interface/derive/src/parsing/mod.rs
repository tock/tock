// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

mod error_accumulator;
mod field;
mod input;

#[cfg(test)]
mod field_tests;

use error_accumulator::ErrorAccumulator;

// Error messages.
const MULTIPLE_SAME_OP: &str = "multiple operations of the same type";
const NOT_A_DATA_TYPE: &str = "expected data type";
const NOT_A_NAME: &str = "expected register name or '_'";
const NOT_AN_OFFSET: &str = "expected register offset (integer literal or '_')";
const UNKNOWN_ATTRIBUTE: &str = "unknown attribute";
const UNKNOWN_OP: &str = "unknown operation";

/// Test utility: pops the next error from the error iterator, and asserts that
/// it contains the provided string.
#[cfg(test)]
#[track_caller]
fn assert_next_contains(iter: &mut <syn::Error as IntoIterator>::IntoIter, message: &str) {
    let error = iter.next().expect("not enough errors").to_string();
    assert!(
        error.contains(message),
        "error '{}' does not contain message '{}'",
        error,
        message
    );
}
