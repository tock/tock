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

/// `FieldDebug` is a debug helper trait that is implemented for Fields, and tuples of Fields.
///
/// This trait contain reference to the int type used on the field and the enum type its associated with.
///
/// This is auto used by [`register_bitfields!`], and don't need to be implemented manually.
///
/// [`register_bitfields!`]: crate::register_bitfields
pub trait FieldDebug<T: UIntLike, E> {
    /// Handles [`fmt::Debug`] value generation for a field.
    /// It will try to convert the data to the associated enum type, and if it fails, it will
    /// print the raw value as a number.
    fn debug_field(&self, data: T, a: &mut impl FnMut(&dyn fmt::Debug));
}

impl<T, R, E> FieldDebug<T, E> for Field<T, R>
where
    T: UIntLike,
    R: RegisterLongName,
    E: TryFromValue<T, EnumType = E> + fmt::Debug,
{
    fn debug_field(&self, data: T, f: &mut impl FnMut(&dyn fmt::Debug)) {
        let v = self.read(data);
        match E::try_from_value(v) {
            Some(v) => f(&v),
            None => f(&v),
        }
    }
}

// implement for 2 value tuple, this will be recursive, see [`impl_register_debug!`] for how it will look like
impl<T, E1, E2, F1, F2> FieldDebug<T, (E1, E2)> for (F1, F2)
where
    T: UIntLike,
    F1: FieldDebug<T, E1>,
    F2: FieldDebug<T, E2>,
{
    fn debug_field(&self, data: T, f: &mut impl FnMut(&dyn fmt::Debug)) {
        self.0.debug_field(data, f);
        self.1.debug_field(data, f);
    }
}

/// `RegisterDebugInfo` is a trait for types that can provide debug information for the `Register`.
///
/// It provide:
/// - The name of the Register since we don't store that anywhere else.
/// - The names of the fields in the register.
/// - The fields themselves, these are of type [`Field`].
pub trait RegisterDebugInfo<T: UIntLike> {
    /// A type containing a tuple of all the enum types used in the register in order
    type EnumTypes;

    /// The name of the register.
    fn name() -> &'static str;
    /// The names of the fields in the register.
    fn fields_names() -> &'static [&'static str];
    /// The fields themselves, these are of type [`Field`],
    /// these are returned as a tuple of fields.
    fn fields() -> impl FieldDebug<T, Self::EnumTypes>;
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
    T: UIntLike,
    E: RegisterDebugInfo<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct(E::name());
        let mut names = E::fields_names().iter();
        E::fields().debug_field(self.data, &mut |v| {
            debug_struct.field(names.next().unwrap(), &v);
        });
        debug_struct.finish()
    }
}
