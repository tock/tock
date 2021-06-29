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

use core::cell::UnsafeCell;
use core::marker::PhantomData;

use crate::interfaces::{Readable, Writeable};
use crate::{IntLike, RegisterLongName};

/// Read/Write registers.
///
/// For accessing and manipulating the register contents, the
/// [`Readable`], [`Writeable`] and
/// [`ReadWriteable`](crate::interfaces::ReadWriteable) traits are
/// implemented.
// To successfully alias this structure onto hardware registers in memory, this
// struct must be exactly the size of the `T`.
#[repr(transparent)]
pub struct ReadWrite<T: IntLike, R: RegisterLongName = ()> {
    value: UnsafeCell<T>,
    associated_register: PhantomData<R>,
}
impl<T: IntLike, R: RegisterLongName> Readable for ReadWrite<T, R> {
    type T = T;
    type R = R;

    #[inline]
    fn get(&self) -> Self::T {
        unsafe { ::core::ptr::read_volatile(self.value.get()) }
    }
}
impl<T: IntLike, R: RegisterLongName> Writeable for ReadWrite<T, R> {
    type T = T;
    type R = R;

    #[inline]
    fn set(&self, value: T) {
        unsafe { ::core::ptr::write_volatile(self.value.get(), value) }
    }
}

/// Read-only registers.
///
/// For accessing the register contents the [`Readable`] trait is
/// implemented.
// To successfully alias this structure onto hardware registers in memory, this
// struct must be exactly the size of the `T`.
#[repr(transparent)]
pub struct ReadOnly<T: IntLike, R: RegisterLongName = ()> {
    value: T,
    associated_register: PhantomData<R>,
}
impl<T: IntLike, R: RegisterLongName> Readable for ReadOnly<T, R> {
    type T = T;
    type R = R;

    #[inline]
    fn get(&self) -> T {
        unsafe { ::core::ptr::read_volatile(&self.value) }
    }
}

/// Write-only registers.
///
/// For setting the register contents the [`Writeable`] trait is
/// implemented.
// To successfully alias this structure onto hardware registers in memory, this
// struct must be exactly the size of the `T`.
#[repr(transparent)]
pub struct WriteOnly<T: IntLike, R: RegisterLongName = ()> {
    value: UnsafeCell<T>,
    associated_register: PhantomData<R>,
}
impl<T: IntLike, R: RegisterLongName> Writeable for WriteOnly<T, R> {
    type T = T;
    type R = R;

    #[inline]
    fn set(&self, value: T) {
        unsafe { ::core::ptr::write_volatile(self.value.get(), value) }
    }
}

/// Read-only and write-only registers aliased to the same address.
///
/// Unlike the [`ReadWrite`] register, this represents a register
/// which has different meanings based on if it is written or read.
/// This might be found on a device where control and status registers
/// are accessed via the same memory address via writes and reads,
/// respectively.
///
/// This register implements [`Readable`] and [`Writeable`], but in
/// general does not implement
/// [`ReadWriteable`](crate::interfaces::ReadWriteable) (only if the
/// type parameters `R` and `W` are identical, in which case a
/// [`ReadWrite`] register might be a better choice).
// To successfully alias this structure onto hardware registers in memory, this
// struct must be exactly the size of the `T`.
#[repr(transparent)]
pub struct Aliased<T: IntLike, R: RegisterLongName = (), W: RegisterLongName = ()> {
    value: UnsafeCell<T>,
    associated_register: PhantomData<(R, W)>,
}
impl<T: IntLike, R: RegisterLongName, W: RegisterLongName> Readable for Aliased<T, R, W> {
    type T = T;
    type R = R;

    #[inline]
    fn get(&self) -> Self::T {
        unsafe { ::core::ptr::read_volatile(self.value.get()) }
    }
}
impl<T: IntLike, R: RegisterLongName, W: RegisterLongName> Writeable for Aliased<T, R, W> {
    type T = T;
    type R = W;

    #[inline]
    fn set(&self, value: Self::T) {
        unsafe { ::core::ptr::write_volatile(self.value.get(), value) }
    }
}

/// In memory volatile register.
///
/// Like [`ReadWrite`], but can be safely constructed using the
/// [`InMemoryRegister::new`] method. It will always be initialized to
/// the passed in, well-defined initial value.
///
/// For accessing and manipulating the register contents, the
/// [`Readable`], [`Writeable`] and
/// [`ReadWriteable`](crate::interfaces::ReadWriteable) traits are
/// implemented.
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
}
impl<T: IntLike, R: RegisterLongName> Readable for InMemoryRegister<T, R> {
    type T = T;
    type R = R;

    #[inline]
    fn get(&self) -> Self::T {
        unsafe { ::core::ptr::read_volatile(self.value.get()) }
    }
}
impl<T: IntLike, R: RegisterLongName> Writeable for InMemoryRegister<T, R> {
    type T = T;
    type R = R;

    #[inline]
    fn set(&self, value: T) {
        unsafe { ::core::ptr::write_volatile(self.value.get(), value) }
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

    impl crate::fields::TryFromValue<u16> for Foo {
        type EnumType = Foo;

        fn try_from(v: u16) -> Option<Self::EnumType> {
            Self::try_from(v as u32)
        }
    }
    impl crate::fields::TryFromValue<u32> for Foo {
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
        use super::Foo;
        use crate::fields::{Field, TryFromValue};

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
            let field128 = Field::<u128, ()>::new(0x12345678_9abcdef0_0fedcba9_87654321, 1);
            assert_eq!(field128.mask, 0x12345678_9abcdef0_0fedcba9_87654321_u128);
            assert_eq!(field128.shift, 1);
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
        use crate::fields::Field;

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
