//! Tock Register Interface
//!
//! Provides efficient mechanisms to express and use type-checked
//! memory mapped registers and bitfields.
//!
//! ```rust
//! # fn main() {}
//!
//! use tock_registers::registers::{ReadOnly, ReadWrite};
//! use tock_registers::register_bitfields;
//!
//! // Register maps are specified like this:
//! #[repr(C)]
//! struct Registers {
//!     // Control register: read-write
//!     cr: ReadWrite<u32, Control::Register>,
//!     // Status register: read-only
//!     s: ReadOnly<u32, Status::Register>,
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

#![feature(const_fn_trait_bound)]
#![no_std]
// If we don't build any actual register types, we don't need unsafe
// code in this crate
#![cfg_attr(not(feature = "register_types"), forbid(unsafe_code))]

pub mod fields;
pub mod interfaces;
pub mod macros;

#[cfg(feature = "register_types")]
pub mod registers;

mod local_register;
pub use local_register::LocalRegisterCopy;

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

/// Descriptive name for each register.
pub trait RegisterLongName {}

// Useful implementation for when no RegisterLongName is required
// (e.g. no fields need to be accessed, just the raw register values)
impl RegisterLongName for () {}
