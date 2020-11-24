//! Implementation of registers and bitfields.
//!
//! Provides efficient mechanisms to express and use type-checked memory mapped
//! registers and bitfields.
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

// The register interface uses `+` in a way that is fine for bitfields, but
// looks unusual (and perhaps problematic) to a linter. We just ignore those
// lints for this file.
#![allow(clippy::suspicious_op_assign_impl)]
#![allow(clippy::suspicious_arithmetic_impl)]

use core::cell::UnsafeCell;
use core::fmt;
use core::marker::PhantomData;
use core::ops::{Add, AddAssign, BitAnd, BitOr, BitOrAssign, Not, Shl, Shr};

/// IntLike properties needed to read/write/modify a register.
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
    fn zero() -> Self;
}

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

/// Descriptive name for each register.
pub trait RegisterLongName {}

impl RegisterLongName for () {}

/// Conversion of raw register value into enumerated values member.
/// Implemented inside register_bitfields! macro for each bit field.
pub trait TryFromValue<V> {
    type EnumType;

    fn try_from(v: V) -> Option<Self::EnumType>;
}

/// Read/Write registers.
// To successfully alias this structure onto hardware registers in memory, this
// struct must be exactly the size of the `T`.
#[repr(transparent)]
pub struct ReadWrite<T: IntLike, R: RegisterLongName = ()> {
    value: UnsafeCell<T>,
    associated_register: PhantomData<R>,
}

/// Read-only registers.
// To successfully alias this structure onto hardware registers in memory, this
// struct must be exactly the size of the `T`.
#[repr(transparent)]
pub struct ReadOnly<T: IntLike, R: RegisterLongName = ()> {
    value: T,
    associated_register: PhantomData<R>,
}

/// Write-only registers.
// To successfully alias this structure onto hardware registers in memory, this
// struct must be exactly the size of the `T`.
#[repr(transparent)]
pub struct WriteOnly<T: IntLike, R: RegisterLongName = ()> {
    value: UnsafeCell<T>,
    associated_register: PhantomData<R>,
}

/// Read-only and write-only registers aliased to the same address.
///
/// Unlike the `ReadWrite` register, this represents a register which has different meanings based
/// on if it is written or read.  This might be found on a device where control and status
/// registers are accessed via the same memory address via writes and reads, respectively.
// To successfully alias this structure onto hardware registers in memory, this
// struct must be exactly the size of the `T`.
#[repr(transparent)]
pub struct Aliased<T: IntLike, R: RegisterLongName = (), W: RegisterLongName = ()> {
    value: UnsafeCell<T>,
    associated_register: PhantomData<(R, W)>,
}

impl<T: IntLike, R: RegisterLongName> ReadWrite<T, R> {
    #[inline]
    /// Get the raw register value
    pub fn get(&self) -> T {
        unsafe { ::core::ptr::read_volatile(self.value.get()) }
    }

    #[inline]
    /// Set the raw register value
    pub fn set(&self, value: T) {
        unsafe { ::core::ptr::write_volatile(self.value.get(), value) }
    }

    #[inline]
    /// Read the value of the given field
    pub fn read(&self, field: Field<T, R>) -> T {
        field.read(self.get())
    }

    #[inline]
    /// Read value of the given field as an enum member
    pub fn read_as_enum<E: TryFromValue<T, EnumType = E>>(&self, field: Field<T, R>) -> Option<E> {
        field.read_as_enum(self.get())
    }

    #[inline]
    /// Make a local copy of the register
    pub fn extract(&self) -> LocalRegisterCopy<T, R> {
        LocalRegisterCopy::new(self.get())
    }

    #[inline]
    /// Write the value of one or more fields, overwriting the other fields with zero
    pub fn write(&self, field: FieldValue<T, R>) {
        self.set(field.value);
    }

    #[inline]
    /// Write the value of one or more fields, leaving the other fields unchanged
    pub fn modify(&self, field: FieldValue<T, R>) {
        self.set(field.modify(self.get()));
    }

    #[inline]
    /// Write the value of one or more fields, maintaining the value of unchanged fields via a
    /// provided original value, rather than a register read.
    pub fn modify_no_read(&self, original: LocalRegisterCopy<T, R>, field: FieldValue<T, R>) {
        self.set(field.modify(original.get()));
    }

    #[inline]
    /// Check if one or more bits in a field are set
    pub fn is_set(&self, field: Field<T, R>) -> bool {
        field.is_set(self.get())
    }

    #[inline]
    /// Check if any specified parts of a field match
    pub fn matches_any(&self, field: FieldValue<T, R>) -> bool {
        field.matches_any(self.get())
    }

    #[inline]
    /// Check if all specified parts of a field match
    pub fn matches_all(&self, field: FieldValue<T, R>) -> bool {
        field.matches_all(self.get())
    }
}

impl<T: IntLike, R: RegisterLongName> ReadOnly<T, R> {
    #[inline]
    /// Get the raw register value
    pub fn get(&self) -> T {
        unsafe { ::core::ptr::read_volatile(&self.value) }
    }

    #[inline]
    /// Read the value of the given field
    pub fn read(&self, field: Field<T, R>) -> T {
        field.read(self.get())
    }

    #[inline]
    /// Read value of the given field as an enum member
    pub fn read_as_enum<E: TryFromValue<T, EnumType = E>>(&self, field: Field<T, R>) -> Option<E> {
        field.read_as_enum(self.get())
    }

    #[inline]
    /// Make a local copy of the register
    pub fn extract(&self) -> LocalRegisterCopy<T, R> {
        LocalRegisterCopy::new(self.get())
    }

    #[inline]
    /// Check if one or more bits in a field are set
    pub fn is_set(&self, field: Field<T, R>) -> bool {
        field.is_set(self.get())
    }

    #[inline]
    /// Check if any specified parts of a field match
    pub fn matches_any(&self, field: FieldValue<T, R>) -> bool {
        field.matches_any(self.get())
    }

    #[inline]
    /// Check if all specified parts of a field match
    pub fn matches_all(&self, field: FieldValue<T, R>) -> bool {
        field.matches_all(self.get())
    }
}

impl<T: IntLike, R: RegisterLongName> WriteOnly<T, R> {
    #[inline]
    /// Set the raw register value
    pub fn set(&self, value: T) {
        unsafe { ::core::ptr::write_volatile(self.value.get(), value) }
    }

    #[inline]
    /// Write the value of one or more fields, overwriting the other fields with zero
    pub fn write(&self, field: FieldValue<T, R>) {
        self.set(field.value);
    }
}

impl<T: IntLike, R: RegisterLongName, W: RegisterLongName> Aliased<T, R, W> {
    #[inline]
    /// Get the raw register value
    pub fn get(&self) -> T {
        unsafe { ::core::ptr::read_volatile(self.value.get()) }
    }

    #[inline]
    /// Set the raw register value
    pub fn set(&self, value: T) {
        unsafe { ::core::ptr::write_volatile(self.value.get(), value) }
    }

    #[inline]
    /// Read the value of the given field
    pub fn read(&self, field: Field<T, R>) -> T {
        field.read(self.get())
    }

    #[inline]
    /// Read value of the given field as an enum member
    pub fn read_as_enum<E: TryFromValue<T, EnumType = E>>(&self, field: Field<T, R>) -> Option<E> {
        field.read_as_enum(self.get())
    }

    #[inline]
    /// Make a local copy of the register
    pub fn extract(&self) -> LocalRegisterCopy<T, R> {
        LocalRegisterCopy::new(self.get())
    }

    #[inline]
    /// Write the value of one or more fields, overwriting the other fields with zero
    pub fn write(&self, field: FieldValue<T, W>) {
        self.set(field.value);
    }

    #[inline]
    /// Check if one or more bits in a field are set
    pub fn is_set(&self, field: Field<T, R>) -> bool {
        field.is_set(self.get())
    }

    #[inline]
    /// Check if any specified parts of a field match
    pub fn matches_any(&self, field: FieldValue<T, R>) -> bool {
        field.matches_any(self.get())
    }

    #[inline]
    /// Check if all specified parts of a field match
    pub fn matches_all(&self, field: FieldValue<T, R>) -> bool {
        field.matches_all(self.get())
    }
}

/// A read-only copy register contents
///
/// This behaves very similarly to a read-only register, but instead of doing a
/// volatile read to MMIO to get the value for each function call, a copy of the
/// register contents are stored locally in memory. This allows a peripheral
/// to do a single read on a register, and then check which bits are set without
/// having to do a full MMIO read each time. It also allows the value of the
/// register to be "cached" in case the peripheral driver needs to clear the
/// register in hardware yet still be able to check the bits.
#[derive(Copy, Clone)]
pub struct LocalRegisterCopy<T: IntLike, R: RegisterLongName = ()> {
    value: T,
    associated_register: PhantomData<R>,
}

impl<T: IntLike, R: RegisterLongName> LocalRegisterCopy<T, R> {
    pub const fn new(value: T) -> Self {
        LocalRegisterCopy {
            value: value,
            associated_register: PhantomData,
        }
    }

    #[inline]
    pub fn get(&self) -> T {
        self.value
    }

    #[inline]
    pub fn read(&self, field: Field<T, R>) -> T {
        field.read(self.get())
    }

    #[inline]
    pub fn read_as_enum<E: TryFromValue<T, EnumType = E>>(&self, field: Field<T, R>) -> Option<E> {
        field.read_as_enum(self.get())
    }

    #[inline]
    pub fn is_set(&self, field: Field<T, R>) -> bool {
        field.is_set(self.get())
    }

    #[inline]
    pub fn matches_any(&self, field: FieldValue<T, R>) -> bool {
        field.matches_any(self.get())
    }

    #[inline]
    pub fn matches_all(&self, field: FieldValue<T, R>) -> bool {
        field.matches_all(self.get())
    }

    /// Do a bitwise AND operation of the stored value and the passed in value
    /// and return a new LocalRegisterCopy.
    #[inline]
    pub fn bitand(&self, rhs: T) -> LocalRegisterCopy<T, R> {
        LocalRegisterCopy::new(self.value & rhs)
    }
}

impl<T: IntLike + fmt::Debug, R: RegisterLongName> fmt::Debug for LocalRegisterCopy<T, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

macro_rules! From_impl_for {
    ($type:ty) => {
        impl<R: RegisterLongName> From<LocalRegisterCopy<$type, R>> for $type {
            fn from(r: LocalRegisterCopy<$type, R>) -> $type {
                r.value
            }
        }
    };
}

From_impl_for!(u8);
From_impl_for!(u16);
From_impl_for!(u32);
From_impl_for!(u64);
From_impl_for!(u128);

/// In memory volatile register.
// To successfully alias this structure onto hardware registers in memory, this
// struct must be exactly the size of the `T`.
#[repr(transparent)]
pub struct InMemoryRegister<T: IntLike, R: RegisterLongName = ()> {
    value: UnsafeCell<T>,
    associated_register: PhantomData<R>,
}

impl<T: IntLike, R: RegisterLongName> InMemoryRegister<T, R> {
    pub const fn new(value: T) -> Self {
        InMemoryRegister {
            value: UnsafeCell::new(value),
            associated_register: PhantomData,
        }
    }

    #[inline]
    pub fn get(&self) -> T {
        unsafe { ::core::ptr::read_volatile(self.value.get()) }
    }

    #[inline]
    pub fn set(&self, value: T) {
        unsafe { ::core::ptr::write_volatile(self.value.get(), value) }
    }

    #[inline]
    pub fn read(&self, field: Field<T, R>) -> T {
        field.read(self.get())
    }

    #[inline]
    pub fn read_as_enum<E: TryFromValue<T, EnumType = E>>(&self, field: Field<T, R>) -> Option<E> {
        field.read_as_enum(self.get())
    }

    #[inline]
    pub fn extract(&self) -> LocalRegisterCopy<T, R> {
        LocalRegisterCopy::new(self.get())
    }

    #[inline]
    pub fn write(&self, field: FieldValue<T, R>) {
        self.set(field.value);
    }

    #[inline]
    pub fn modify(&self, field: FieldValue<T, R>) {
        self.set(field.modify(self.get()));
    }

    #[inline]
    pub fn modify_no_read(&self, original: LocalRegisterCopy<T, R>, field: FieldValue<T, R>) {
        self.set(field.modify(original.get()));
    }

    #[inline]
    pub fn is_set(&self, field: Field<T, R>) -> bool {
        field.is_set(self.get())
    }

    #[inline]
    pub fn matches_any(&self, field: FieldValue<T, R>) -> bool {
        field.matches_any(self.get())
    }

    #[inline]
    pub fn matches_all(&self, field: FieldValue<T, R>) -> bool {
        field.matches_all(self.get())
    }
}

/// Specific section of a register.
///
/// For the Field, the mask is unshifted, ie. the LSB should always be set.
#[derive(Copy, Clone)]
pub struct Field<T: IntLike, R: RegisterLongName> {
    mask: T,
    pub shift: usize,
    associated_register: PhantomData<R>,
}

impl<T: IntLike, R: RegisterLongName> Field<T, R> {
    pub const fn new(mask: T, shift: usize) -> Field<T, R> {
        Field {
            mask: mask,
            shift: shift,
            associated_register: PhantomData,
        }
    }

    #[inline]
    pub fn read(self, val: T) -> T {
        (val & (self.mask << self.shift)) >> self.shift
    }

    #[inline]
    /// Check if one or more bits in a field are set
    pub fn is_set(self, val: T) -> bool {
        val & (self.mask << self.shift) != T::zero()
    }

    #[inline]
    /// Read value of the field as an enum member
    pub fn read_as_enum<E: TryFromValue<T, EnumType = E>>(self, val: T) -> Option<E> {
        E::try_from(self.read(val))
    }
}

macro_rules! Field_impl_for {
    ($type:ty) => {
        impl<R: RegisterLongName> Field<$type, R> {
            pub fn val(&self, value: $type) -> FieldValue<$type, R> {
                FieldValue::<$type, R>::new(self.mask, self.shift, value)
            }
        }
    };
}

Field_impl_for!(u8);
Field_impl_for!(u16);
Field_impl_for!(u32);
Field_impl_for!(u64);
Field_impl_for!(u128);

/// Values for the specific register fields.
///
/// For the FieldValue, the masks and values are shifted into their actual
/// location in the register.
#[derive(Copy, Clone)]
pub struct FieldValue<T: IntLike, R: RegisterLongName> {
    mask: T,
    pub value: T,
    associated_register: PhantomData<R>,
}

macro_rules! FieldValue_impl_for {
    ($type:ty) => {
        // Necessary to split the implementation of new() out because the bitwise
        // math isn't treated as const when the type is generic.
        // Tracking issue: https://github.com/rust-lang/rfcs/pull/2632
        impl<R: RegisterLongName> FieldValue<$type, R> {
            pub const fn new(mask: $type, shift: usize, value: $type) -> Self {
                FieldValue {
                    mask: mask << shift,
                    value: (value & mask) << shift,
                    associated_register: PhantomData,
                }
            }
        }

        // Necessary to split the implementation of From<> out because of the orphan rule
        // for foreign trait implementation (see [E0210](https://doc.rust-lang.org/error-index.html#E0210)).
        impl<R: RegisterLongName> From<FieldValue<$type, R>> for $type {
            fn from(val: FieldValue<$type, R>) -> $type {
                val.value
            }
        }
    };
}

FieldValue_impl_for!(u8);
FieldValue_impl_for!(u16);
FieldValue_impl_for!(u32);
FieldValue_impl_for!(u64);
FieldValue_impl_for!(u128);

impl<T: IntLike, R: RegisterLongName> FieldValue<T, R> {
    /// Get the raw bitmask represented by this FieldValue.
    #[inline]
    pub fn mask(&self) -> T {
        self.mask as T
    }

    #[inline]
    pub fn read(&self, field: Field<T, R>) -> T {
        field.read(self.value)
    }

    /// Modify fields in a register value
    #[inline]
    pub fn modify(self, val: T) -> T {
        (val & !self.mask) | self.value
    }

    /// Check if any specified parts of a field match
    #[inline]
    pub fn matches_any(&self, val: T) -> bool {
        val & self.mask != T::zero()
    }

    /// Check if all specified parts of a field match
    #[inline]
    pub fn matches_all(&self, val: T) -> bool {
        val & self.mask == self.value
    }
}

// Combine two fields with the addition operator
impl<T: IntLike, R: RegisterLongName> Add for FieldValue<T, R> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        FieldValue {
            mask: self.mask | rhs.mask,
            value: self.value | rhs.value,
            associated_register: PhantomData,
        }
    }
}

// Combine two fields with the += operator
impl<T: IntLike, R: RegisterLongName> AddAssign for FieldValue<T, R> {
    #[inline]
    fn add_assign(&mut self, rhs: FieldValue<T, R>) {
        self.mask |= rhs.mask;
        self.value |= rhs.value;
    }
}

#[cfg(not(feature = "no_std_unit_tests"))]
#[cfg(test)]
mod tests {
    #[derive(Debug, PartialEq, Eq)]
    enum Foo {
        Foo0,
        Foo1,
        Foo2,
        Foo3,
        Foo4,
        Foo5,
        Foo6,
        Foo7,
    }

    impl super::TryFromValue<u16> for Foo {
        type EnumType = Foo;

        fn try_from(v: u16) -> Option<Self::EnumType> {
            Self::try_from(v as u32)
        }
    }
    impl super::TryFromValue<u32> for Foo {
        type EnumType = Foo;

        fn try_from(v: u32) -> Option<Self::EnumType> {
            match v {
                0 => Some(Foo::Foo0),
                1 => Some(Foo::Foo1),
                2 => Some(Foo::Foo2),
                3 => Some(Foo::Foo3),
                4 => Some(Foo::Foo4),
                5 => Some(Foo::Foo5),
                6 => Some(Foo::Foo6),
                7 => Some(Foo::Foo7),
                _ => None,
            }
        }
    }

    mod field {
        use super::super::{Field, TryFromValue};
        use super::Foo;

        #[test]
        fn test_new() {
            let field8 = Field::<u8, ()>::new(0x12, 3);
            assert_eq!(field8.mask, 0x12_u8);
            assert_eq!(field8.shift, 3);
            let field16 = Field::<u16, ()>::new(0x1234, 5);
            assert_eq!(field16.mask, 0x1234_u16);
            assert_eq!(field16.shift, 5);
            let field32 = Field::<u32, ()>::new(0x12345678, 9);
            assert_eq!(field32.mask, 0x12345678_u32);
            assert_eq!(field32.shift, 9);
            let field64 = Field::<u64, ()>::new(0x12345678_9abcdef0, 1);
            assert_eq!(field64.mask, 0x12345678_9abcdef0_u64);
            assert_eq!(field64.shift, 1);
        }

        #[test]
        fn test_read() {
            let field = Field::<u32, ()>::new(0xFF, 4);
            assert_eq!(field.read(0x123), 0x12);
            let field = Field::<u32, ()>::new(0xF0F, 4);
            assert_eq!(field.read(0x1234), 0x103);
        }

        #[test]
        fn test_is_set() {
            let field = Field::<u16, ()>::new(0xFF, 4);
            assert_eq!(field.is_set(0), false);
            assert_eq!(field.is_set(0xFFFF), true);
            assert_eq!(field.is_set(0x0FF0), true);
            assert_eq!(field.is_set(0x1000), false);
            assert_eq!(field.is_set(0x0100), true);
            assert_eq!(field.is_set(0x0010), true);
            assert_eq!(field.is_set(0x0001), false);

            for shift in 0..24 {
                let field = Field::<u32, ()>::new(0xFF, shift);
                for x in 1..=0xFF {
                    assert_eq!(field.is_set(x << shift), true);
                }
                assert_eq!(field.is_set(!(0xFF << shift)), false);
            }
        }

        #[test]
        fn test_read_as_enum() {
            let field = Field::<u16, ()>::new(0x7, 4);
            assert_eq!(field.read_as_enum(0x1234), Some(Foo::Foo3));
            assert_eq!(field.read_as_enum(0x5678), Some(Foo::Foo7));
            assert_eq!(field.read_as_enum(0xFFFF), Some(Foo::Foo7));
            assert_eq!(field.read_as_enum(0x0000), Some(Foo::Foo0));
            assert_eq!(field.read_as_enum(0x0010), Some(Foo::Foo1));
            assert_eq!(field.read_as_enum(0x1204), Some(Foo::Foo0));

            for shift in 0..29 {
                let field = Field::<u32, ()>::new(0x7, shift);
                for x in 0..8 {
                    assert_eq!(field.read_as_enum(x << shift), Foo::try_from(x));
                }
            }
        }
    }

    mod field_value {
        use super::super::Field;

        #[test]
        fn test_from() {
            let field = Field::<u32, ()>::new(0xFF, 4);
            assert_eq!(u32::from(field.val(0)), 0);
            assert_eq!(u32::from(field.val(0xFFFFFFFF)), 0xFF0);
            assert_eq!(u32::from(field.val(0x12)), 0x120);
            assert_eq!(u32::from(field.val(0x123)), 0x230);

            for shift in 0..32 {
                let field = Field::<u32, ()>::new(0xFF, shift);
                for x in 0..=0xFF {
                    assert_eq!(u32::from(field.val(x)), x << shift);
                }
            }
        }

        #[test]
        fn test_read_same_field() {
            let field = Field::<u32, ()>::new(0xFF, 4);
            assert_eq!(field.val(0).read(field), 0);
            assert_eq!(field.val(0xFFFFFFFF).read(field), 0xFF);
            assert_eq!(field.val(0x12).read(field), 0x12);
            assert_eq!(field.val(0x123).read(field), 0x23);

            for shift in 0..24 {
                let field = Field::<u32, ()>::new(0xFF, shift);
                for x in 0..=0xFF {
                    assert_eq!(field.val(x).read(field), x);
                }
            }
        }

        #[test]
        fn test_read_disjoint_fields() {
            for shift in 0..24 {
                let field1 = Field::<u32, ()>::new(0xF0, shift);
                let field2 = Field::<u32, ()>::new(0x0F, shift);
                for x in 0..=0xFF {
                    assert_eq!(field1.val(x).read(field2), 0);
                    assert_eq!(field2.val(x).read(field1), 0);
                }
            }
            for shift in 0..24 {
                let field1 = Field::<u32, ()>::new(0xF, shift);
                let field2 = Field::<u32, ()>::new(0xF, shift + 4);
                for x in 0..=0xFF {
                    assert_eq!(field1.val(x).read(field2), 0);
                    assert_eq!(field2.val(x).read(field1), 0);
                }
            }
        }

        #[test]
        fn test_modify() {
            let field = Field::<u32, ()>::new(0xFF, 4);
            assert_eq!(field.val(0x23).modify(0x0000), 0x0230);
            assert_eq!(field.val(0x23).modify(0xFFFF), 0xF23F);
            assert_eq!(field.val(0x23).modify(0x1234), 0x1234);
            assert_eq!(field.val(0x23).modify(0x5678), 0x5238);
        }

        #[test]
        fn test_matches_any() {
            let field = Field::<u32, ()>::new(0xFF, 4);
            assert_eq!(field.val(0x23).matches_any(0x1234), true);
            assert_eq!(field.val(0x23).matches_any(0x5678), true);
            assert_eq!(field.val(0x23).matches_any(0x5008), false);

            for shift in 0..24 {
                let field = Field::<u32, ()>::new(0xFF, shift);
                for x in 0..=0xFF {
                    let field_value = field.val(x);
                    for y in 1..=0xFF {
                        assert_eq!(field_value.matches_any(y << shift), true);
                    }
                    assert_eq!(field_value.matches_any(0), false);
                    assert_eq!(field_value.matches_any(!(0xFF << shift)), false);
                }
            }
        }

        #[test]
        fn test_matches_all() {
            let field = Field::<u32, ()>::new(0xFF, 4);
            assert_eq!(field.val(0x23).matches_all(0x1234), true);
            assert_eq!(field.val(0x23).matches_all(0x5678), false);

            for shift in 0..24 {
                let field = Field::<u32, ()>::new(0xFF, shift);
                for x in 0..=0xFF {
                    assert_eq!(field.val(x).matches_all(x << shift), true);
                    assert_eq!(field.val(x + 1).matches_all(x << shift), false);
                }
            }
        }

        #[test]
        fn test_add_disjoint_fields() {
            let field1 = Field::<u32, ()>::new(0xFF, 24);
            let field2 = Field::<u32, ()>::new(0xFF, 16);
            let field3 = Field::<u32, ()>::new(0xFF, 8);
            let field4 = Field::<u32, ()>::new(0xFF, 0);
            assert_eq!(
                u32::from(
                    field1.val(0x12) + field2.val(0x34) + field3.val(0x56) + field4.val(0x78)
                ),
                0x12345678
            );

            for shift in 0..24 {
                let field1 = Field::<u32, ()>::new(0xF, shift);
                let field2 = Field::<u32, ()>::new(0xF, shift + 4);
                for x in 0..=0xF {
                    for y in 0..=0xF {
                        assert_eq!(
                            u32::from(field1.val(x) + field2.val(y)),
                            (x | (y << 4)) << shift
                        );
                    }
                }
            }
        }

        #[test]
        fn test_add_assign_disjoint_fields() {
            let field1 = Field::<u32, ()>::new(0xFF, 24);
            let field2 = Field::<u32, ()>::new(0xFF, 16);
            let field3 = Field::<u32, ()>::new(0xFF, 8);
            let field4 = Field::<u32, ()>::new(0xFF, 0);

            let mut value = field1.val(0x12);
            value += field2.val(0x34);
            value += field3.val(0x56);
            value += field4.val(0x78);
            assert_eq!(u32::from(value), 0x12345678);

            for shift in 0..24 {
                let field1 = Field::<u32, ()>::new(0xF, shift);
                let field2 = Field::<u32, ()>::new(0xF, shift + 4);
                for x in 0..=0xF {
                    for y in 0..=0xF {
                        let mut value = field1.val(x);
                        value += field2.val(y);
                        assert_eq!(u32::from(value), (x | (y << 4)) << shift);
                    }
                }
            }
        }
    }

    // TODO: More unit tests here.
}
