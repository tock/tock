//! Implementation of included register types.
//!
//! This module provides a standard set of register types, which can
//! describe different access levels:
//!
//! - [`ReadWrite`] for registers which can be read and written to
//! - [`ReadOnly`] for registers which can only be read
//! - [`WriteOnly`] for registers which can only be written to
//! - [`Aliased`] for registers which can be both read and written,
//!   but represent different registers depending on the operation
//! - [`InMemoryRegister`] provide a register-type in RAM using
//!   volatile operations
//!
//! These types can be disabled by removing the `register_types` crate
//! feature (part of the default features). This is useful if this
//! crate should be used only as an interface library, or if all
//! unsafe code should be disabled.

use core::cell::UnsafeCell;
use core::marker::PhantomData;

use crate::interfaces::{Readable, Writeable};
use crate::{RegisterLongName, UIntLike};

/// Read/Write registers.
///
/// For accessing and manipulating the register contents, the
/// [`Readable`], [`Writeable`] and
/// [`ReadWriteable`](crate::interfaces::ReadWriteable) traits are
/// implemented.
// To successfully alias this structure onto hardware registers in memory, this
// struct must be exactly the size of the `T`.
#[repr(transparent)]
pub struct ReadWrite<T: UIntLike, R: RegisterLongName = ()> {
    value: UnsafeCell<T>,
    associated_register: PhantomData<R>,
}
impl<T: UIntLike, R: RegisterLongName> Readable for ReadWrite<T, R> {
    type T = T;
    type R = R;

    #[inline]
    fn get(&self) -> Self::T {
        unsafe { ::core::ptr::read_volatile(self.value.get()) }
    }
}
impl<T: UIntLike, R: RegisterLongName> Writeable for ReadWrite<T, R> {
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
pub struct ReadOnly<T: UIntLike, R: RegisterLongName = ()> {
    value: T,
    associated_register: PhantomData<R>,
}
impl<T: UIntLike, R: RegisterLongName> Readable for ReadOnly<T, R> {
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
pub struct WriteOnly<T: UIntLike, R: RegisterLongName = ()> {
    value: UnsafeCell<T>,
    associated_register: PhantomData<R>,
}
impl<T: UIntLike, R: RegisterLongName> Writeable for WriteOnly<T, R> {
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
pub struct Aliased<T: UIntLike, R: RegisterLongName = (), W: RegisterLongName = ()> {
    value: UnsafeCell<T>,
    associated_register: PhantomData<(R, W)>,
}
impl<T: UIntLike, R: RegisterLongName, W: RegisterLongName> Readable for Aliased<T, R, W> {
    type T = T;
    type R = R;

    #[inline]
    fn get(&self) -> Self::T {
        unsafe { ::core::ptr::read_volatile(self.value.get()) }
    }
}
impl<T: UIntLike, R: RegisterLongName, W: RegisterLongName> Writeable for Aliased<T, R, W> {
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
pub struct InMemoryRegister<T: UIntLike, R: RegisterLongName = ()> {
    value: UnsafeCell<T>,
    associated_register: PhantomData<R>,
}

impl<T: UIntLike, R: RegisterLongName> InMemoryRegister<T, R> {
    pub const fn new(value: T) -> Self {
        InMemoryRegister {
            value: UnsafeCell::new(value),
            associated_register: PhantomData,
        }
    }
}
impl<T: UIntLike, R: RegisterLongName> Readable for InMemoryRegister<T, R> {
    type T = T;
    type R = R;

    #[inline]
    fn get(&self) -> Self::T {
        unsafe { ::core::ptr::read_volatile(self.value.get()) }
    }
}
impl<T: UIntLike, R: RegisterLongName> Writeable for InMemoryRegister<T, R> {
    type T = T;
    type R = R;

    #[inline]
    fn set(&self, value: T) {
        unsafe { ::core::ptr::write_volatile(self.value.get(), value) }
    }
}
