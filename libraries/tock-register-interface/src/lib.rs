//! Tock Register Interface
//!
//!

#![feature(const_fn_trait_bound)]
#![no_std]

pub mod fields;
pub mod interfaces;
pub mod macros;
pub mod registers;

mod local_register;
pub use local_register::LocalRegisterCopy;

use core::ops::{BitAnd, BitOr, BitOrAssign, Not, Shl, Shr};

/// Trait representing the base type of registers.
///
/// IntLike defines basic properties of types required to
/// read/write/modify a register through its methods and supertrait
/// requirements.
///
/// It features a range of default implementations for common integer
/// types, such as [`u8`], [`u16`], [`u32`], [`u64`], [`u128`] and
/// [`usize`].
pub trait IntLike:
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
    /// This can be used to acquire values of the [`IntLike`] type,
    /// even in generic implementations. For instance, to get the
    /// value `1`, one can use `<T as IntLike>::zero() + 1`. To get
    /// the largest representable value, use a bitwise negation: `~(<T
    /// as IntLike>::zero())`.
    fn zero() -> Self;
}

// Helper macro for implementing the IntLike trait on differrent
// types.
macro_rules! IntLike_impl_for {
    ($type:ty) => {
        impl IntLike for $type {
            fn zero() -> Self {
                0
            }
        }
    };
}

IntLike_impl_for!(u8);
IntLike_impl_for!(u16);
IntLike_impl_for!(u32);
IntLike_impl_for!(u64);
IntLike_impl_for!(u128);
IntLike_impl_for!(usize);

/// Descriptive name for each register.
pub trait RegisterLongName {}

// Useful implementation for when no RegisterLongName is required
// (e.g. no fields need to be accessed, just the raw register values)
impl RegisterLongName for () {}
