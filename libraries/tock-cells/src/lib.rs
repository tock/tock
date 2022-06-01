//! Tock Cell types.

// Feature required with newer versions of rustc (at least 2020-10-25).
#![feature(const_mut_refs)]
#![no_std]

pub mod map_cell;
pub mod numeric_cell_ext;
pub mod optional_cell;
pub mod take_cell;
pub mod volatile_cell;
