#![no_std]

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate core as std;

mod cauterize;
pub use cauterize::*;

mod error;
pub use error::Error;

mod vector;
pub use vector::Vector;

mod range;
pub use range::Range;

mod stream;

mod array;
