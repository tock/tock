// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock Register Interface
//!
//! Provides efficient mechanisms to express and use type-checked
//! memory mapped registers and bitfields.
//!
//! ```rust
//! # fn main() {}
//!
//! use tock_registers::{peripheral, register_bitfields};
//!
//! // Register maps are specified like this:
//! peripheral! {
//!     Registers {
//!         0x0 => cr: u32(Control::Register) { Read, Write },
//!         0x4 => s: u32(Status::Register) { Read },
//!     }
//! }
//!
//! // Register fields and definitions look like this:
//! register_bitfields![u32,
//!     // Simpler bitfields are expressed concisely:
//!     Control [
//!         /// Stop the Current Transfer
//!         STOP 8,
//!         /// Software Reset
//!         SWRST 7,
//!         /// Master Disable
//!         MDIS 1,
//!         /// Master Enable
//!         MEN 0
//!     ],
//!
//!     // More complex registers can express subtypes:
//!     Status [
//!         TXCOMPLETE  OFFSET(0) NUMBITS(1) [],
//!         TXINTERRUPT OFFSET(1) NUMBITS(1) [],
//!         RXCOMPLETE  OFFSET(2) NUMBITS(1) [],
//!         RXINTERRUPT OFFSET(3) NUMBITS(1) [],
//!         MODE        OFFSET(4) NUMBITS(3) [
//!             FullDuplex = 0,
//!             HalfDuplex = 1,
//!             Loopback = 2,
//!             Disabled = 3
//!         ],
//!         ERRORCOUNT OFFSET(6) NUMBITS(3) []
//!     ]
//! ];
//! ```
//!
//! Author
//! ------
//! - Shane Leonard <shanel@stanford.edu>

#![no_std]

mod access;
mod bus_adapter;
mod data_type;
mod fake_register;
pub mod fields;
pub mod interfaces;
mod long_names;
pub mod macros;
mod peripheral;
pub mod reexport;
mod register_traits;

#[cfg(feature = "register_types")]
pub mod registers;

pub mod debug;

mod local_register;
pub use local_register::LocalRegisterCopy;

pub use access::{Access, NoAccess, Safe, Unsafe};
pub use bus_adapter::{BusAdapter, DirectBus};
pub use data_type::ArrayDataType;
pub use fake_register::FakeRegister;
pub use long_names::{Aliased, LongNames, RegisterLongName};
pub use register_traits::{Read, Register, UnsafeRead, UnsafeWrite, Write};

use core::fmt::Debug;
use core::ops::{BitAnd, BitOr, BitOrAssign, Not, Shl, Shr};

/// Trait representing the base type of registers.
///
/// UIntLike defines basic properties of types required to
/// read/write/modify a register through its methods and supertrait
/// requirements.
///
/// It features a range of default implementations for common unsigned
/// integer types, such as [`u8`], [`u16`], [`u32`], [`u64`], [`u128`],
/// and [`usize`].
pub trait UIntLike:
    BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitOrAssign
    + Not<Output = Self>
    + Eq
    + Shr<usize, Output = Self>
    + Shl<usize, Output = Self>
    + Copy
    + Clone
    + Debug
{
    /// Return the representation of the value `0` in the implementing
    /// type.
    ///
    /// This can be used to acquire values of the [`UIntLike`] type,
    /// even in generic implementations. For instance, to get the
    /// value `1`, one can use `<T as UIntLike>::zero() + 1`. To get
    /// the largest representable value, use a bitwise negation: `~(<T
    /// as UIntLike>::zero())`.
    fn zero() -> Self;
}

// Helper macro for implementing the UIntLike trait on differrent
// types.
macro_rules! UIntLike_impl_for {
    ($type:ty) => {
        impl UIntLike for $type {
            fn zero() -> Self {
                0
            }
        }
    };
}

UIntLike_impl_for!(u8);
UIntLike_impl_for!(u16);
UIntLike_impl_for!(u32);
UIntLike_impl_for!(u64);
UIntLike_impl_for!(u128);
UIntLike_impl_for!(usize);

/// Error indicating an array index was out of bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OutOfBounds;
