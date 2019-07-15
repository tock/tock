//! Tock Cell types.

#![feature(maybe_uninit)]
#![feature(const_fn, untagged_unions)]
#![no_std]

pub mod map_cell;
pub mod numeric_cell_ext;
pub mod optional_cell;
pub mod take_cell;
pub mod volatile_cell;
