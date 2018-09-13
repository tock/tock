#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
use core as std;

extern crate byteorder;

mod cauterize;
pub use cauterize::*;

mod error;
pub use error::Error;

mod vector;
pub use vector::Vector;

mod range;
pub use range::Range;

mod array;
