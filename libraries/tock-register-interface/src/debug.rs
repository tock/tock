use core::fmt;

use crate::{
    fields::{Field, TryFromValue},
    RegisterLongName, UIntLike,
};

pub trait EnumTuples<T: UIntLike, E> {
    fn debug_field(&self, data: T, a: &mut impl FnMut(&dyn fmt::Debug));
}

impl<T: UIntLike, R: RegisterLongName, E: TryFromValue<T, EnumType = E> + fmt::Debug>
    EnumTuples<T, E> for Field<T, R>
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
        impl<T: UIntLike, $($enum),* , $($field: EnumTuples<T, $enum>),*> EnumTuples<T, ($($enum),*)> for ($($field),*) {
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

pub trait RegisterDebugInfo<T: UIntLike, E> {
    // methods to query the required debug information
    fn name() -> &'static str;
    fn fields_names() -> &'static [&'static str];
    fn fields_enums() -> impl EnumTuples<T, E>;
}

pub struct RegisterDebug<T: UIntLike, E, R: RegisterDebugInfo<T, E>>(
    core::marker::PhantomData<(T, E, R)>,
);

impl<T: UIntLike, E, R: RegisterDebugInfo<T, E>> RegisterLongName for RegisterDebug<T, E, R> {}

pub struct RegisterDebugValue<T: UIntLike, E, R: RegisterDebugInfo<T, E>> {
    pub(crate) data: T,
    pub(crate) _reg: core::marker::PhantomData<(E, R)>,
}

impl<'a, T: UIntLike, E, R: RegisterDebugInfo<T, E>> fmt::Debug for RegisterDebugValue<T, E, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct(R::name());
        let mut names = R::fields_names().iter();
        R::fields_enums().debug_field(self.data, &mut |v| {
            debug_struct.field(names.next().unwrap(), &v);
        });
        debug_struct.finish()
    }
}
