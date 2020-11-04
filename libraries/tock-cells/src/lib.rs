//! Tock Cell types.

#![feature(const_fn)]
// Feature used to opt-in the new `core::Option::contains()` API.
//
// This feature can be removed if needed by manually reimplementing the
// `contains` logic for `Option`.
//
// Tock expects this feature to stabilize in the near future.
// Tracking: https://github.com/rust-lang/rust/issues/62358
#![feature(option_result_contains)]
// Feature required with newer versions of rustc (at least 2020-10-25).
#![feature(const_mut_refs)]
#![no_std]

pub mod map_cell;
pub mod numeric_cell_ext;
pub mod optional_cell;
pub mod take_cell;
pub mod volatile_cell;
