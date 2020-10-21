//! Tock Cell types.

#![feature(const_fn)]
#![feature(const_mut_refs)]
// Feature used to opt-in the new `core::Option::contains()` API.
//
// This feature can be removed if needed by manually reimplementing the
// `contains` logic for `Option`.
//
// Tock expects this feature to stabilize in the near future.
// Tracking: https://github.com/rust-lang/rust/issues/62358
#![feature(option_result_contains)]
#![no_std]

pub mod map_cell;
pub mod numeric_cell_ext;
pub mod optional_cell;
pub mod take_cell;
pub mod volatile_cell;
