use tock_registers::fields::{Field, FieldValue};
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::registers::ReadWrite;
pub use tock_registers::RegisterLongName;
pub use tock_registers::fields::TryFromValue;

#[flux_rs::opaque]
#[flux_rs::refined_by(mask: bitvec<32>, shift: bitvec<32>)]
pub struct FieldU32<R: RegisterLongName> {
    inner: Field<u32, R>,
}

#[allow(dead_code)]
impl<R: RegisterLongName> FieldU32<R> {
    #[flux_rs::trusted]
    #[flux_rs::sig(fn(mask: u32, shift: usize) -> FieldU32<R>[bv_int_to_bv32(mask), bv_int_to_bv32(shift)])]
    pub fn new(mask: u32, shift: usize) -> FieldU32<R> {
        Self {
            inner: Field::new(mask, shift),
        }
    }

    /*
        mask: mask << shift,
        value: (value & mask) << shift,
    */
    #[flux_rs::trusted]
    #[flux_rs::sig(fn(&FieldU32<R>[@mask, @shift], value: u32) -> FieldValueU32<R>[bv_shl(mask, shift), bv_shl(bv_and(bv_int_to_bv32(value), mask), shift)])]
    pub fn val(&self, value: u32) -> FieldValueU32<R> {
        FieldValueU32 {
            inner: FieldValue::<u32, R>::new(self.inner.mask, self.inner.shift, value),
        }
    }
}

use core::ops::Add;

#[derive(Copy, Clone)]
#[flux_rs::opaque]
#[flux_rs::refined_by(mask: bitvec<32>, value: bitvec<32>)]
pub struct FieldValueU32<R: RegisterLongName> {
    inner: FieldValue<u32, R>,
}

#[allow(dead_code)]
impl<R: RegisterLongName> Add for FieldValueU32<R> {
    type Output = Self;

    #[inline]
    #[flux_rs::trusted]
    #[flux_rs::sig(fn(FieldValueU32<R>[@mask0, @value0], FieldValueU32<R>[@mask1, @value1]) -> FieldValueU32<R>[bv_or(mask0, mask1), bv_or(value0, value1)])]
    fn add(self, rhs: Self) -> Self {
        FieldValueU32 {
            inner: FieldValue::<u32, R>::new(
                self.inner.mask | rhs.inner.mask,
                0,
                self.inner.value | rhs.inner.value,
            ),
        }
    }
}

#[flux_rs::opaque]
#[flux_rs::refined_by(value: bitvec<32>)]
pub struct ReadWriteU32<R: RegisterLongName = ()> {
    inner: ReadWrite<u32, R>,
}

#[allow(dead_code)]
impl<R: RegisterLongName> ReadWriteU32<R> {
    fn new(_addr: usize) -> Self {
        unimplemented!()
    }

    #[flux_rs::trusted]
    #[flux_rs::sig(fn(&ReadWriteU32<R>[@n]) -> u32[bv_bv32_to_int(n)])]
    pub fn get(&self) -> u32 {
        self.inner.get()
    }

    #[flux_rs::trusted]
    #[flux_rs::sig(fn(reg: &strg ReadWriteU32<R>, u32[@n]) ensures reg: ReadWriteU32<R>[bv_int_to_bv32(n)])]
    pub fn set(&mut self, value: u32) {
        self.inner.set(value)
    }

    //(val & (self.mask << self.shift)) >> self.shift
    #[flux_rs::trusted]
    #[flux_rs::sig(fn(&ReadWriteU32<R>[@n], FieldU32<R>[@mask, @shift]) -> u32[ bv_bv32_to_int(bv_lshr(bv_and(n, bv_shl(mask, shift)), shift))])]
    pub fn read(&self, field: FieldU32<R>) -> u32 {
        self.inner.read(field.inner)
    }

    #[flux_rs::trusted]
    #[flux_rs::sig(fn(reg: &strg ReadWriteU32<R>, FieldValueU32<R>[@mask, @value]) ensures reg: ReadWriteU32<R>[value])]
    pub fn write(&mut self, fieldval: FieldValueU32<R>) {
        self.inner.write(fieldval.inner);
    }
}

// Macros for declaring named bitfields

/// Define register types and fields.
#[macro_export]
macro_rules! register_bitfields {
    {
        $valtype:ident, $( $(#[$inner:meta])* $vis:vis $reg:ident $fields:tt ),* $(,)?
    } => {
        $(
            #[allow(non_snake_case)]
            $(#[$inner])*
            $vis mod $reg {
                // Visibility note: This is left always `pub` as it is not
                // meaningful to restrict access to the `Register` element of
                // the register module if the module itself is in scope
                //
                // (if you can access $reg, you can access $reg::Register)
                #[derive(Clone, Copy)]
                pub struct Register;
                impl $crate::RegisterLongName for Register {}

                use $crate::{FieldU32, FieldValueU32};
                use $crate::TryFromValue;

                $crate::register_bitmasks!( $valtype, $reg, Register, $fields );
            }
        )*
    }
}


#[macro_export]
macro_rules! bitmask {
    ($numbits:expr) => {
        (1 << ($numbits - 1)) + ((1 << ($numbits - 1)) - 1)
    };
}

/// Helper macro for defining register fields.
#[macro_export]
macro_rules! register_bitmasks {
    {
        // BITFIELD_NAME OFFSET(x)
        $(#[$outer:meta])*
        $valtype:ident, $reg_mod:ident, $reg_desc:ident, [
            $( $(#[$inner:meta])* $field:ident OFFSET($offset:expr)),+ $(,)?
        ]
    } => {
        $(#[$outer])*
        $( $crate::register_bitmasks!($valtype, $reg_desc, $(#[$inner])* $field, $offset, 1, []); )*
        $crate::register_bitmasks!(@debug $valtype, $reg_mod, $reg_desc, [$($field),*]);
    };

    {
        // BITFIELD_NAME OFFSET
        // All fields are 1 bit
        $(#[$outer:meta])*
        $valtype:ident, $reg_mod:ident, $reg_desc:ident, [
            $( $(#[$inner:meta])* $field:ident $offset:expr ),+ $(,)?
        ]
    } => {
        $(#[$outer])*
        $( $crate::register_bitmasks!($valtype, $reg_desc, $(#[$inner])* $field, $offset, 1, []); )*
        $crate::register_bitmasks!(@debug $valtype, $reg_mod, $reg_desc, [$($field),*]);
    };

    {
        // BITFIELD_NAME OFFSET(x) NUMBITS(y)
        $(#[$outer:meta])*
        $valtype:ident, $reg_mod:ident, $reg_desc:ident, [
            $( $(#[$inner:meta])* $field:ident OFFSET($offset:expr) NUMBITS($numbits:expr) ),+ $(,)?
        ]
    } => {
        $(#[$outer])*
        $( $crate::register_bitmasks!($valtype, $reg_desc, $(#[$inner])* $field, $offset, $numbits, []); )*
        $crate::register_bitmasks!(@debug $valtype, $reg_mod, $reg_desc, [$($field),*]);
    };

    {
        // BITFIELD_NAME OFFSET(x) NUMBITS(y) []
        $(#[$outer:meta])*
        $valtype:ident, $reg_mod:ident, $reg_desc:ident, [
            $( $(#[$inner:meta])* $field:ident OFFSET($offset:expr) NUMBITS($numbits:expr)
               $values:tt ),+ $(,)?
        ]
    } => {
        $(#[$outer])*
        $( $crate::register_bitmasks!($valtype, $reg_desc, $(#[$inner])* $field, $offset, $numbits,
                              $values); )*
        $crate::register_bitmasks!(@debug $valtype, $reg_mod, $reg_desc, [$($field),*]);
    };

    {
        $valtype:ident, $reg_desc:ident, $(#[$outer:meta])* $field:ident,
                    $offset:expr, $numbits:expr,
                    [$( $(#[$inner:meta])* $valname:ident = $value:expr ),+ $(,)?]
    } => { // this match arm is duplicated below with an allowance for 0 elements in the valname -> value array,
        // to seperately support the case of zero-variant enums not supporting non-default
        // representations.
        #[allow(non_upper_case_globals)]
        #[allow(unused)]
        pub const $field: FieldU32<$reg_desc> =
            FieldU32::<$reg_desc>::new($crate::bitmask!($numbits), $offset);

        #[allow(non_snake_case)]
        #[allow(unused)]
        $(#[$outer])*
        pub mod $field {
            #[allow(unused_imports)]
            use $crate::{FieldValueU32, TryFromValue};
            use super::$reg_desc;

            $(
            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            $(#[$inner])*
            pub const $valname: FieldValueU32<$reg_desc> =
                FieldValueU32::<$reg_desc>::new($crate::bitmask!($numbits),
                    $offset, $value);
            )*

            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            pub const SET: FieldValueU32<$reg_desc> =
                FieldValueU32::<$reg_desc>::new($crate::bitmask!($numbits),
                    $offset, $crate::bitmask!($numbits));

            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            pub const CLEAR: FieldValueU32<$reg_desc> =
                FieldValueU32::<$reg_desc>::new($crate::bitmask!($numbits),
                    $offset, 0);

            #[allow(dead_code)]
            #[allow(non_camel_case_types)]
            #[derive(Copy, Clone, Debug, Eq, PartialEq)]
            #[repr($valtype)] // so that values larger than isize::MAX can be stored
            $(#[$outer])*
            pub enum Value {
                $(
                    $(#[$inner])*
                    $valname = $value,
                )*
            }

            impl TryFromValue<$valtype> for Value {
                type EnumType = Value;

                fn try_from_value(v: $valtype) -> Option<Self::EnumType> {
                    match v {
                        $(
                            $(#[$inner])*
                            x if x == Value::$valname as $valtype => Some(Value::$valname),
                        )*

                        _ => Option::None
                    }
                }
            }

            impl From<Value> for FieldValueU32<$reg_desc> {
                fn from(v: Value) -> Self {
                    Self::new($crate::bitmask!($numbits), $offset, v as $valtype)
                }
            }
        }
    };
    {
        $valtype:ident, $reg_desc:ident, $(#[$outer:meta])* $field:ident,
                    $offset:expr, $numbits:expr,
                    []
    } => { //same pattern as previous match arm, for 0 elements in array. Removes code associated with array.
        #[allow(non_upper_case_globals)]
        #[allow(unused)]
        pub const $field: FieldU32<$reg_desc> =
            FieldU32::<$reg_desc>::new($crate::bitmask!($numbits), $offset);

        #[allow(non_snake_case)]
        #[allow(unused)]
        $(#[$outer])*
        pub mod $field {
            #[allow(unused_imports)]
            use $crate::{FieldValueU32, TryFromValue};
            use super::$reg_desc;

            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            pub const SET: FieldValueU32<$reg_desc> =
                FieldValueU32::<$reg_desc>::new($crate::bitmask!($numbits),
                    $offset, $crate::bitmask!($numbits));

            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            pub const CLEAR: FieldValueU32<$reg_desc> =
                FieldValueU32::<$reg_desc>::new($crate::bitmask!($numbits),
                    $offset, 0);

            #[allow(dead_code)]
            #[allow(non_camel_case_types)]
            #[derive(Debug)]
            $(#[$outer])*
            pub enum Value {}

            impl TryFromValue<$valtype> for Value {
                type EnumType = Value;

                fn try_from_value(_v: $valtype) -> Option<Self::EnumType> {
                    Option::None
                }
            }
        }
    };

    // Implement the `RegisterDebugInfo` trait for the register. Refer to its
    // documentation for more information on the individual types and fields.
    (
        // final implementation of the macro
        @debug $valtype:ident, $reg_mod:ident, $reg_desc:ident, [$($field:ident),*]
    ) => {
        impl $crate::debug::RegisterDebugInfo<$valtype> for $reg_desc {
            // Sequence of field value enum types (implementing `TryFromValue`,
            // produced above), generated by recursing over the fields:
            type FieldValueEnumTypes = $crate::register_bitmasks!(
                @fv_enum_type_seq $valtype, $($field::Value),*
            );

            fn name() -> &'static str {
                stringify!($reg_mod)
            }

            fn field_names() -> &'static [&'static str] {
                &[
                    $(
                        stringify!($field)
                    ),*
                ]
            }

            fn fields() -> &'static [FieldU32<Self>] {
                &[
                    $(
                        $field
                    ),*
                ]
            }
        }
    };

    // Build the recursive `FieldValueEnumSeq` type sequence. This will generate
    // a type signature of the form:
    //
    // ```
    // FieldValueEnumCons<u32, Foo,
    //     FieldValueEnumCons<u32, Bar,
    //         FieldValueEnumCons<u32, Baz,
    //             FieldValueEnumNil
    //         >
    //     >
    // >
    // ```
    (
        @fv_enum_type_seq $valtype:ident, $enum_val:path $(, $($rest:path),+)?
    ) => {
        $crate::debug::FieldValueEnumCons<
            $valtype,
            $enum_val,
            $crate::register_bitmasks!(@fv_enum_type_seq $valtype $(, $($rest),*)*)
        >
    };
    (
        @fv_enum_type_seq $valtype:ident $(,)?
    ) => {
        $crate::debug::FieldValueEnumNil
    };
}