// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Register debug information
//!
//! This module provide types that is used when implementing a register description.
//!
//! These types allow to querying type information about a register that is later used for implementing
//! [`fmt::Debug`] for the wrapper value [`RegisterDebugValue`].

use core::fmt;

use crate::{
    fields::{Field, TryFromValue},
    RegisterLongName, UIntLike,
};

/// `EnumDebug` is a debug helper trait that is implemented for Enums used in [`Fields`], and tuples of Enums.
///
/// This trait contain reference to the int type used on the enum type associated with a type
///
/// This is auto used by [`register_bitfields!`], and don't need to be implemented manually.
///
/// [`register_bitfields!`]: crate::register_bitfields
/// [`Fields`]: crate::fields::Field
pub trait EnumDebug<T: UIntLike> {
    fn debug_field(data: &mut impl FnMut() -> T, f: &mut impl FnMut(&dyn fmt::Debug));
}

// hack: using `tuple` instead of directly on `E1` as it would cause conflicting implementation
impl<T, E1> EnumDebug<T> for (E1,)
where
    T: UIntLike,
    E1: TryFromValue<T, EnumType = E1> + fmt::Debug,
{
    fn debug_field(data: &mut impl FnMut() -> T, f: &mut impl FnMut(&dyn fmt::Debug)) {
        let data = data();
        match E1::try_from_value(data) {
            Some(v) => f(&v),
            None => f(&data),
        }
    }
}

// implement for 2 enums, the rest is recursive
impl<T, E1, E2> EnumDebug<T> for (E1, E2)
where
    T: UIntLike,
    E1: EnumDebug<T>,
    E2: EnumDebug<T>,
{
    fn debug_field(data: &mut impl FnMut() -> T, f: &mut impl FnMut(&dyn fmt::Debug)) {
        E1::debug_field(data, f);
        E2::debug_field(data, f);
    }
}

/// `RegisterDebugInfo` is a trait for types that can provide debug information for the `Register`.
///
/// It provide:
/// - The name of the Register since we don't store that anywhere else.
/// - The names of the fields in the register.
/// - The fields themselves, these are of type [`Field`].
pub trait RegisterDebugInfo<T: UIntLike>: RegisterLongName {
    /// A type containing a tuple of all the enum types used in the register in order
    type EnumTypes: EnumDebug<T>;

    /// The name of the register.
    fn name() -> &'static str;
    /// The names of the fields in the register.
    fn fields_names() -> &'static [&'static str];
    /// The fields themselves, these are of type [`Field`],
    /// these are returned as a tuple of fields.
    fn fields() -> &'static [Field<T, Self>]
    where
        Self: Sized;
}

/// `RegisterDebugValue` is a container for the debug information and the value of the register
/// that we will read from and output the results.
///
/// The data is read once into this register and used for all the fields printing to avoid multiple reads
/// to hardware.
pub struct RegisterDebugValue<T, E>
where
    T: UIntLike,
    E: RegisterDebugInfo<T>,
{
    pub(crate) data: T,
    pub(crate) _reg: core::marker::PhantomData<E>,
}

impl<T, E> fmt::Debug for RegisterDebugValue<T, E>
where
    T: UIntLike + 'static,
    E: RegisterDebugInfo<T>,
    E: 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct(E::name());

        let mut names = E::fields_names().iter();
        let mut fields = E::fields().iter();

        let mut data = || fields.next().unwrap().read(self.data);
        let mut debug_field = |f: &dyn fmt::Debug| {
            debug_struct.field(names.next().unwrap(), f);
        };

        E::EnumTypes::debug_field(&mut data, &mut debug_field);

        debug_struct.finish()
    }
}
