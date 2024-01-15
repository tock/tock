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

impl<T: UIntLike, R: RegisterLongName, E: TryFromValue<T, EnumType = E> + fmt::Debug>
    FieldDebug<T, E> for Field<T, R>
{
    fn debug_field(&self, data: T, f: &mut impl FnMut(&dyn fmt::Debug)) {
        let v = self.read(data);
        match E::try_from_value(v) {
            Some(v) => f(&v),
            None => f(&v),
        }
    }
}

macro_rules! impl_tuple {
    ($($enum:ident: $field:ident),*) => {
        impl<T: UIntLike, $($enum),* , $($field: FieldDebug<T, $enum>),*> FieldDebug<T, ($($enum),*)> for ($($field),*) {
            fn debug_field(&self, data: T, f: &mut impl FnMut(&dyn fmt::Debug)) {
                #[allow(non_snake_case)]
                let ($($field),*) = self;
                $(
                    $field.debug_field(data, f);
                )*
            }
        }
    };
}

// Implement FieldDebug for tuples of fields
impl_tuple!(E1: F1, E2: F2);
impl_tuple!(E1: F1, E2: F2, E3: F3);
impl_tuple!(E1: F1, E2: F2, E3: F3, E4: F4);
impl_tuple!(E1: F1, E2: F2, E3: F3, E4: F4, E5: F5);
impl_tuple!(E1: F1, E2: F2, E3: F3, E4: F4, E5: F5, E6: F6);
impl_tuple!(E1: F1, E2: F2, E3: F3, E4: F4, E5: F5, E6: F6, E7: F7);
impl_tuple!(E1: F1, E2: F2, E3: F3, E4: F4, E5: F5, E6: F6, E7: F7, E8: F8);
impl_tuple!(E1: F1, E2: F2, E3: F3, E4: F4, E5: F5, E6: F6, E7: F7, E8: F8, E9: F9);
impl_tuple!(E1: F1, E2: F2, E3: F3, E4: F4, E5: F5, E6: F6, E7: F7, E8: F8, E9: F9, E10: F10);
impl_tuple!(E1: F1, E2: F2, E3: F3, E4: F4, E5: F5, E6: F6, E7: F7, E8: F8, E9: F9, E10: F10, E11: F11);
impl_tuple!(E1: F1, E2: F2, E3: F3, E4: F4, E5: F5, E6: F6, E7: F7, E8: F8, E9: F9, E10: F10, E11: F11, E12: F12);

/// `RegisterDebugInfo` is a trait for types that can provide debug information for the `Register`.
///
/// It provide:
/// - The name of the Register since we don't store that anywhere else.
/// - The names of the fields in the register.
/// - The fields themselves, these are of type [`Field`].
pub trait RegisterDebugInfo<T: UIntLike, E> {
    /// The name of the register.
    fn name() -> &'static str;
    /// The names of the fields in the register.
    fn fields_names() -> &'static [&'static str];
    /// The fields themselves, these are of type [`Field`],
    /// these are returned as a tuple of fields.
    fn fields() -> impl FieldDebug<T, E>;
}

/// `RegisterDebugValue` is a container for the debug information and the value of the register
/// that we will read from and output the results.
///
/// The data is read once into this register and used for all the fields printing to avoid multiple reads
/// to hardware.
pub struct RegisterDebugValue<T: UIntLike, E, R: RegisterDebugInfo<T, E>> {
    pub(crate) data: T,
    pub(crate) _reg: core::marker::PhantomData<(E, R)>,
}

impl<'a, T: UIntLike, E, R: RegisterDebugInfo<T, E>> fmt::Debug for RegisterDebugValue<T, E, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct(R::name());
        let mut names = R::fields_names().iter();
        R::fields().debug_field(self.data, &mut |v| {
            debug_struct.field(names.next().unwrap(), &v);
        });
        debug_struct.finish()
    }
}
