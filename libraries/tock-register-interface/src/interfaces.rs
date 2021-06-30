//! Interfaces (traits) to register types
//!
//! This module contains traits which reflect standardized interfaces
//! to different types of registers. Examples of registers
//! implementing these interfaces are [`ReadWrite`](crate::registers::ReadWrite) or
//! [`InMemoryRegister`](crate::registers::InMemoryRegister).
//!
//! Each trait has two associated type parameters, namely:
//!
//! - `T`: [`UIntLike`](crate::UIntLike), indicating the underlying
//!   integer type used to represent the register's raw contents.
//!
//! - `R`: [`RegisterLongName`](crate::RegisterLongName), functioning
//!   as a type to identify this register's descriptive name and
//!   semantic meaning. It is further used to impose type constraints
//!   on values passed through the API, such as
//!   [`FieldValue`](crate::fields::FieldValue).
//!
//! Registers can have different access levels, which are mapped to
//! different traits respectively:
//!
//! - [`Readable`]: indicates that the current value of this register
//!   can be read. Implementations will need to provide the
//!   [`get`](crate::interfaces::Readable::get) method.
//!
//! - [`Writeable`]: indicates that the value of this register can be
//!   set. Implementations will need to provide the
//!   [`set`](crate::interfaces::Writeable::set) method.
//!
//! - [`ReadWriteable`]: indicates that this register can be
//!   _modified_. It is not sufficient for registers to be both read-
//!   and writable, they must also have the same semantic meaning when
//!   read from and written to. This is not true in general, for
//!   example a memory-mapped UART register might transmit when
//!   writing and receive when reading.
//!
//!   If a type implements both [`Readable`] and [`Writeable`], and
//!   the associated [`RegisterLongName`](crate::RegisterLongName)
//!   type parameters are identical, it will automatically implement
//!   [`ReadWriteable`]. In particular, for
//!   [`Aliased`](crate::registers::Aliased) this is -- in general --
//!   not the case, so
//!
//!   ```rust
//!   # use tock_registers::interfaces::{Readable, Writeable, ReadWriteable};
//!   # use tock_registers::registers::ReadWrite;
//!   # use tock_registers::register_bitfields;
//!   register_bitfields![u8,
//!       A [
//!           DUMMY OFFSET(0) NUMBITS(1) [],
//!       ],
//!   ];
//!   let read_write_reg: &ReadWrite<u8, A::Register> = unsafe {
//!       core::mem::transmute(Box::leak(Box::new(0_u8)))
//!   };
//!   ReadWriteable::modify(read_write_reg, A::DUMMY::SET);
//!   ```
//!
//!   works, but not
//!
//!   ```compile_fail
//!   # use tock_registers::interfaces::{Readable, Writeable, ReadWriteable};
//!   # use tock_registers::registers::Aliased;
//!   # use tock_registers::register_bitfields;
//!   register_bitfields![u8,
//!       A [
//!           DUMMY OFFSET(0) NUMBITS(1) [],
//!       ],
//!       B [
//!           DUMMY OFFSET(0) NUMBITS(1) [],
//!       ],
//!   ];
//!   let aliased_reg: &Aliased<u8, A::Register, B::Register> = unsafe {
//!       core::mem::transmute(Box::leak(Box::new(0_u8)))
//!   };
//!   ReadWriteable::modify(aliased_reg, A::DUMMY::SET);
//!   ```
//!
//! ## Example: implementing a custom register type
//!
//! These traits can be used to implement custom register types, which
//! are compatible to the ones shipped in this crate. For example, to
//! define a register which sets a `u8` value using a Cell reference,
//! always reads the bitwise-negated vale and prints every written
//! value to the console:
//!
//! ```rust
//! # use core::cell::Cell;
//! # use core::marker::PhantomData;
//! #
//! # use tock_registers::interfaces::{Readable, Writeable, ReadWriteable};
//! # use tock_registers::RegisterLongName;
//! # use tock_registers::register_bitfields;
//! #
//! struct DummyRegister<'a, R: RegisterLongName> {
//!     cell_ref: &'a Cell<u8>,
//!     _register_long_name: PhantomData<R>,
//! }
//!
//! impl<'a, R: RegisterLongName> Readable for DummyRegister<'a, R> {
//!     type T = u8;
//!     type R = R;
//!
//!     fn get(&self) -> u8 {
//!         // Return the bitwise-inverse of the current value
//!         !self.cell_ref.get()
//!     }
//! }
//!
//! impl<'a, R: RegisterLongName> Writeable for DummyRegister<'a, R> {
//!     type T = u8;
//!     type R = R;
//!
//!     fn set(&self, value: u8) {
//!         println!("Setting Cell to {:02x?}!", value);
//!         self.cell_ref.set(value);
//!     }
//! }
//!
//! register_bitfields![u8,
//!     DummyReg [
//!         HIGH OFFSET(4) NUMBITS(4) [
//!             A = 0b0001,
//!             B = 0b0010,
//!             C = 0b0100,
//!             D = 0b1000,
//!         ],
//!         LOW OFFSET(0) NUMBITS(4) [],
//!     ],
//! ];
//!
//! // Create a new DummyRegister over some Cell<u8>
//! let cell = Cell::new(0);
//! let dummy = DummyRegister {
//!     cell_ref: &cell,
//!     _register_long_name: PhantomData,
//! };
//!
//! // Set a value and read it back. This demonstrates the raw getters
//! // and setters of Writeable and Readable
//! dummy.set(0xFA);
//! assert!(dummy.get() == 0x05);
//!
//! // Use some of the automatically derived APIs, such as
//! // ReadWriteable::modify and Readable::read
//! dummy.modify(DummyReg::HIGH::C);
//! assert!(dummy.read(DummyReg::HIGH) == 0xb);
//! ```

use crate::fields::{Field, FieldValue, TryFromValue};
use crate::{LocalRegisterCopy, RegisterLongName, UIntLike};

/// Readable register
///
/// Register which at least supports reading the current value. Only
/// [`Readable::get`] must be implemented, as for other methods a
/// default implementation is provided.
///
/// A register that is both [`Readable`] and [`Writeable`] will also
/// automatically be [`ReadWriteable`], if the [`RegisterLongName`] of
/// [`Readable`] is the same as that of [`Writeable`] (i.e. not for
/// [`Aliased`](crate::registers::Aliased) registers).
pub trait Readable {
    type T: UIntLike;
    type R: RegisterLongName;

    /// Get the raw register value
    fn get(&self) -> Self::T;

    #[inline]
    /// Read the value of the given field
    fn read(&self, field: Field<Self::T, Self::R>) -> Self::T {
        field.read(self.get())
    }

    #[inline]
    /// Set the raw register value
    fn read_as_enum<E: TryFromValue<Self::T, EnumType = E>>(
        &self,
        field: Field<Self::T, Self::R>,
    ) -> Option<E> {
        field.read_as_enum(self.get())
    }

    #[inline]
    /// Make a local copy of the register
    fn extract(&self) -> LocalRegisterCopy<Self::T, Self::R> {
        LocalRegisterCopy::new(self.get())
    }

    #[inline]
    /// Check if one or more bits in a field are set
    fn is_set(&self, field: Field<Self::T, Self::R>) -> bool {
        field.is_set(self.get())
    }

    #[inline]
    /// Check if any specified parts of a field match
    fn matches_any(&self, field: FieldValue<Self::T, Self::R>) -> bool {
        field.matches_any(self.get())
    }

    #[inline]
    /// Check if all specified parts of a field match
    fn matches_all(&self, field: FieldValue<Self::T, Self::R>) -> bool {
        field.matches_all(self.get())
    }
}

/// Writeable register
///
/// Register which at least supports setting a value. Only
/// [`Writeable::set`] must be implemented, as for other methods a
/// default implementation is provided.
///
/// A register that is both [`Readable`] and [`Writeable`] will also
/// automatically be [`ReadWriteable`], if the [`RegisterLongName`] of
/// [`Readable`] is the same as that of [`Writeable`] (i.e. not for
/// [`Aliased`](crate::registers::Aliased) registers).
pub trait Writeable {
    type T: UIntLike;
    type R: RegisterLongName;

    /// Set the raw register value
    fn set(&self, value: Self::T);

    #[inline]
    /// Write the value of one or more fields, overwriting the other fields with zero
    fn write(&self, field: FieldValue<Self::T, Self::R>) {
        self.set(field.value);
    }

    #[inline]
    /// Write the value of one or more fields, maintaining the value of unchanged fields via a
    /// provided original value, rather than a register read.
    fn modify_no_read(
        &self,
        original: LocalRegisterCopy<Self::T, Self::R>,
        field: FieldValue<Self::T, Self::R>,
    ) {
        self.set(field.modify(original.get()));
    }
}

/// [`Readable`] and [`Writeable`] register, over the same
/// [`RegisterLongName`]
///
/// Register which supports both reading and setting a value.
///
/// **This trait does not have to be implemented manually!** It is
/// automatically implemented for every type that is both [`Readable`]
/// and [`Writeable`], as long as [`Readable::R`] == [`Writeable::R`]
/// (i.e. not for [`Aliased`](crate::registers::Aliased) registers).
pub trait ReadWriteable {
    type T: UIntLike;
    type R: RegisterLongName;

    /// Write the value of one or more fields, leaving the other fields unchanged
    fn modify(&self, field: FieldValue<Self::T, Self::R>);
}

impl<T: UIntLike, R: RegisterLongName, S> ReadWriteable for S
where
    S: Readable<T = T, R = R> + Writeable<T = T, R = R>,
{
    type T = T;
    type R = R;

    #[inline]
    fn modify(&self, field: FieldValue<Self::T, Self::R>) {
        self.set(field.modify(self.get()));
    }
}
