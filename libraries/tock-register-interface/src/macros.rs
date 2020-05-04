//! Macros for cleanly defining peripheral registers.

/// Helper macro for defining register fields.
#[macro_export]
macro_rules! register_bitmasks {
    {
        // BITFIELD_NAME OFFSET(x)
        $(#[$outer:meta])*
        $valtype:ty, $reg_desc:ident, [
            $( $(#[$inner:meta])* $field:ident OFFSET($offset:expr)),+
        ]
    } => {
        $(#[$outer])*
        $( $crate::register_bitmasks!($valtype, $reg_desc, $(#[$inner])* $field, $offset, 1, []); )*
    };
    {
        // BITFIELD_NAME OFFSET
        // All fields are 1 bit
        $(#[$outer:meta])*
        $valtype:ty, $reg_desc:ident, [
            $( $(#[$inner:meta])* $field:ident $offset:expr ),+
        ]
    } => {
        $(#[$outer])*
        $( $crate::register_bitmasks!($valtype, $reg_desc, $(#[$inner])* $field, $offset, 1, []); )*
    };

    {
        // BITFIELD_NAME OFFSET(x) NUMBITS(y)
        $(#[$outer:meta])*
        $valtype:ty, $reg_desc:ident, [
            $( $(#[$inner:meta])* $field:ident OFFSET($offset:expr) NUMBITS($numbits:expr) ),+
        ]
    } => {
        $(#[$outer])*
        $( $crate::register_bitmasks!($valtype, $reg_desc, $(#[$inner])* $field, $offset, $numbits, []); )*
    };

    {
        // BITFIELD_NAME OFFSET(x) NUMBITS(y) []
        $(#[$outer:meta])*
        $valtype:ty, $reg_desc:ident, [
            $( $(#[$inner:meta])* $field:ident OFFSET($offset:expr) NUMBITS($numbits:expr)
               $values:tt ),+
        ]
    } => {
        $(#[$outer])*
        $( $crate::register_bitmasks!($valtype, $reg_desc, $(#[$inner])* $field, $offset, $numbits,
                              $values); )*
    };
    {
        $valtype:ty, $reg_desc:ident, $(#[$outer:meta])* $field:ident,
                    $offset:expr, $numbits:expr,
                    [$( $(#[$inner:meta])* $valname:ident = $value:expr ),+]
    } => {
        #[allow(non_upper_case_globals)]
        #[allow(unused)]
        pub const $field: Field<$valtype, $reg_desc> =
            Field::<$valtype, $reg_desc>::new((1<<($numbits-1))+((1<<($numbits-1))-1), $offset);

        #[allow(non_snake_case)]
        #[allow(unused)]
        $(#[$outer])*
        pub mod $field {
            #[allow(unused_imports)]
            use $crate::registers::{FieldValue, TryFromValue};
            use super::$reg_desc;

            $(
            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            $(#[$inner])*
            pub const $valname: FieldValue<$valtype, $reg_desc> =
                FieldValue::<$valtype, $reg_desc>::new((1<<($numbits-1))+((1<<($numbits-1))-1),
                    $offset, $value);
            )*

            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            pub const SET: FieldValue<$valtype, $reg_desc> =
                FieldValue::<$valtype, $reg_desc>::new((1<<($numbits-1))+((1<<($numbits-1))-1),
                    $offset, (1<<($numbits-1))+((1<<($numbits-1))-1));

            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            pub const CLEAR: FieldValue<$valtype, $reg_desc> =
                FieldValue::<$valtype, $reg_desc>::new((1<<($numbits-1))+((1<<($numbits-1))-1),
                    $offset, 0);

            #[allow(dead_code)]
            #[allow(non_camel_case_types)]
            #[repr(usize)] //use repr(usize) so that unsigned literals larger than max isize can be stored
            $(#[$outer])*
            pub enum Value {
                $(
                    $(#[$inner])*
                    $valname = $value,
                )*
            }

            impl TryFromValue<$valtype> for Value {
                type EnumType = Value;

                fn try_from(v: $valtype) -> Option<Self::EnumType> {
                    match v {
                        $(
                            $(#[$inner])*
                            x if x == Value::$valname as $valtype => Some(Value::$valname),
                        )*

                        _ => Option::None
                    }
                }
            }
        }
    };
    {
        $valtype:ty, $reg_desc:ident, $(#[$outer:meta])* $field:ident,
                    $offset:expr, $numbits:expr,
                    [$( $(#[$inner:meta])* $valname:ident = $value:expr ),*]
    } => { //same pattern as previous match arm, except allows for 0 elements in array. Required because [repr(usize)] cannot be used for zero-variant enums.
        #[allow(non_upper_case_globals)]
        #[allow(unused)]
        pub const $field: Field<$valtype, $reg_desc> =
            Field::<$valtype, $reg_desc>::new((1<<($numbits-1))+((1<<($numbits-1))-1), $offset);

        #[allow(non_snake_case)]
        #[allow(unused)]
        $(#[$outer])*
        pub mod $field {
            #[allow(unused_imports)]
            use $crate::registers::{FieldValue, TryFromValue};
            use super::$reg_desc;

            $(
            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            $(#[$inner])*
            pub const $valname: FieldValue<$valtype, $reg_desc> =
                FieldValue::<$valtype, $reg_desc>::new((1<<($numbits-1))+((1<<($numbits-1))-1),
                    $offset, $value);
            )*

            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            pub const SET: FieldValue<$valtype, $reg_desc> =
                FieldValue::<$valtype, $reg_desc>::new((1<<($numbits-1))+((1<<($numbits-1))-1),
                    $offset, (1<<($numbits-1))+((1<<($numbits-1))-1));

            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            pub const CLEAR: FieldValue<$valtype, $reg_desc> =
                FieldValue::<$valtype, $reg_desc>::new((1<<($numbits-1))+((1<<($numbits-1))-1),
                    $offset, 0);

            #[allow(dead_code)]
            #[allow(non_camel_case_types)]
            $(#[$outer])*
            pub enum Value {
                $(
                    $(#[$inner])*
                    $valname = $value,
                )*
            }

            impl TryFromValue<$valtype> for Value {
                type EnumType = Value;

                fn try_from(v: $valtype) -> Option<Self::EnumType> {
                    match v {
                        $(
                            $(#[$inner])*
                            x if x == Value::$valname as $valtype => Some(Value::$valname),
                        )*

                        _ => Option::None
                    }
                }
            }
        }
    };
}

/// Define register types and fields.
#[macro_export]
macro_rules! register_bitfields {
    {
        $valtype:ty, $( $(#[$inner:meta])* $vis:vis $reg:ident $fields:tt ),*
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
                impl $crate::registers::RegisterLongName for Register {}

                use $crate::registers::Field;

                $crate::register_bitmasks!( $valtype, Register, $fields );
            }
        )*
    }
}

#[macro_export]
macro_rules! register_fields {
    // Macro entry point.
    (@root $(#[$attr_struct:meta])* $vis_struct:vis $name:ident $(<$life:lifetime>)? { $($input:tt)* } ) => {
        $crate::register_fields!(
            @munch (
                $($input)*
            ) -> {
                $vis_struct struct $(#[$attr_struct])* $name $(<$life>)?
            }
        );
    };

    // Print the struct once all fields have been munched.
    (@munch
        (
            $(#[$attr_end:meta])*
            ($offset:expr => @END),
        )
        -> {$vis_struct:vis struct $(#[$attr_struct:meta])* $name:ident $(<$life:lifetime>)? $(
                $(#[$attr:meta])*
                ($vis:vis $id:ident: $ty:ty)
            )*}
    ) => {
        $(#[$attr_struct])*
        #[repr(C)]
        $vis_struct struct $name $(<$life>)? {
            $(
                $(#[$attr])*
                $vis $id: $ty
            ),*
        }
    };

    // Munch field.
    (@munch
        (
            $(#[$attr:meta])*
            ($offset_start:expr => $vis:vis $field:ident: $ty:ty),
            $($after:tt)*
        )
        -> {$($output:tt)*}
    ) => {
        $crate::register_fields!(
            @munch (
                $($after)*
            ) -> {
                $($output)*
                $(#[$attr])*
                ($vis $field: $ty)
            }
        );
    };

    // Munch padding.
    (@munch
        (
            $(#[$attr:meta])*
            ($offset_start:expr => $padding:ident),
            $(#[$attr_next:meta])*
            ($offset_end:expr => $($next:tt)*),
            $($after:tt)*
        )
        -> {$($output:tt)*}
    ) => {
        $crate::register_fields!(
            @munch (
                $(#[$attr_next])*
                ($offset_end => $($next)*),
                $($after)*
            ) -> {
                $($output)*
                $(#[$attr])*
                ($padding: [u8; $offset_end - $offset_start])
            }
        );
    };
}

#[macro_export]
macro_rules! test_fields {
    // Macro entry point.
    (@root $struct:ident $(<$life:lifetime>)? { $($input:tt)* } ) => {
        $crate::test_fields!(@munch $struct $(<$life>)? sum ($($input)*) -> {});
    };

    // Print the tests once all fields have been munched.
    // We wrap the tests in a "detail" function that potentially takes a lifetime parameter, so that
    // the lifetime is declared inside it - therefore all types using the lifetime are well-defined.
    (@munch $struct:ident $(<$life:lifetime>)? $sum:ident
        (
            $(#[$attr_end:meta])*
            ($size:expr => @END),
        )
        -> {$($stmts:block)*}
    ) => {
        {
        fn detail $(<$life>)? ()
        {
            let mut $sum: usize = 0;
            $($stmts)*
            let size = core::mem::size_of::<$struct $(<$life>)?>();
            assert!(
                size == $size,
                "Invalid size for struct {} (expected {:#X} but was {:#X})",
                stringify!($struct),
                $size,
                size
            );
        }

        detail();
        }
    };

    // Munch field.
    (@munch $struct:ident $(<$life:lifetime>)? $sum:ident
        (
            $(#[$attr:meta])*
            ($offset_start:expr => $vis:vis $field:ident: $ty:ty),
            $(#[$attr_next:meta])*
            ($offset_end:expr => $($next:tt)*),
            $($after:tt)*
        )
        -> {$($output:block)*}
    ) => {
        $crate::test_fields!(
            @munch $struct $(<$life>)? $sum (
                $(#[$attr_next])*
                ($offset_end => $($next)*),
                $($after)*
            ) -> {
                $($output)*
                {
                    assert!(
                        $sum == $offset_start,
                        "Invalid start offset for field {} (expected {:#X} but was {:#X})",
                        stringify!($field),
                        $offset_start,
                        $sum
                    );
                    let align = core::mem::align_of::<$ty>();
                    assert!(
                        $sum & (align - 1) == 0,
                        "Invalid alignment for field {} (expected alignment of {:#X} but offset was {:#X})",
                        stringify!($field),
                        align,
                        $sum
                    );
                    $sum += core::mem::size_of::<$ty>();
                    assert!(
                        $sum == $offset_end,
                        "Invalid end offset for field {} (expected {:#X} but was {:#X})",
                        stringify!($field),
                        $offset_end,
                        $sum
                    );
                }
            }
        );
    };

    // Munch padding.
    (@munch $struct:ident $(<$life:lifetime>)? $sum:ident
        (
            $(#[$attr:meta])*
            ($offset_start:expr => $padding:ident),
            $(#[$attr_next:meta])*
            ($offset_end:expr => $($next:tt)*),
            $($after:tt)*
        )
        -> {$($output:block)*}
    ) => {
        $crate::test_fields!(
            @munch $struct $(<$life>)? $sum (
                $(#[$attr_next])*
                ($offset_end => $($next)*),
                $($after)*
            ) -> {
                $($output)*
                {
                    assert!(
                        $sum == $offset_start,
                        "Invalid start offset for padding {} (expected {:#X} but was {:#X})",
                        stringify!($padding),
                        $offset_start,
                        $sum
                    );
                    $sum = $offset_end;
                }
            }
        );
    };
}

#[cfg(not(feature = "no_std_unit_tests"))]
#[macro_export]
macro_rules! register_structs {
    {
        $(
            $(#[$attr:meta])*
            $vis_struct:vis $name:ident $(<$life:lifetime>)? {
                $( $fields:tt )*
            }
        ),*
    } => {
        $( $crate::register_fields!(@root $(#[$attr])* $vis_struct $name $(<$life>)? { $($fields)* } ); )*

        #[cfg(test)]
        mod test_register_structs {
        $(
            #[allow(non_snake_case)]
            mod $name {
                use super::super::*;
                #[test]
                fn test_offsets() {
                    $crate::test_fields!(@root $name $(<$life>)? { $($fields)* } )
                }
            }
        )*
        }
    };
}

#[cfg(feature = "no_std_unit_tests")]
#[macro_export]
macro_rules! register_structs {
    {
        $(
            $(#[$attr:meta])*
            $vis_struct:vis $name:ident $(<$life:lifetime>)? {
                $( $fields:tt )*
            }
        ),*
    } => {
        $( $crate::register_fields!(@root $(#[$attr])* $vis_struct $name $(<$life>)? { $($fields)* } ); )*
    };
}
