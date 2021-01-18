//! Tock Binary Format (TBF) header parsing library.

// Parsing the headers does not require any unsafe operations.
#![forbid(unsafe_code)]
#![no_std]

pub mod parse;
pub mod types;
