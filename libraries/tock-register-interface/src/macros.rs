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
                    [$( $(#[$inner:meta])* $valname:ident = $value:expr ),*]
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
        $valtype:ty, $( $(#[$inner:meta])* $reg:ident $fields:tt ),*
    } => {
        $(
            #[allow(non_snake_case)]
            $(#[$inner])*
            pub mod $reg {
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
    (@root $name:ident { $($input:tt)* } ) => {
        $crate::register_fields!(@munch ($($input)*) -> {struct $name});
    };

    // Print the struct once all fields have been munched.
    (@munch
        (
            $(#[$attr_end:meta])*
            ($offset:expr => @END),
        )
        -> {struct $name:ident $(
                $(#[$attr:meta])*
                ($id:ident: $ty:ty)
            )*}
    ) => {
        #[repr(C)]
        struct $name {
            $(
                $(#[$attr])*
                $id: $ty
            ),*
        }
    };

    // Munch field.
    (@munch
        (
            $(#[$attr:meta])*
            ($offset_start:expr => $field:ident: $ty:ty),
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
                ($field: $ty)
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
    (@root $struct:ident $($input:tt)* ) => {
        $crate::test_fields!(@munch $struct sum ($($input)*) -> {});
    };

    // Print the tests once all fields have been munched.
    (@munch $struct:ident $sum:ident
        (
            $(#[$attr_end:meta])*
            ($size:expr => @END),
        )
        -> {$($stmts:block)*}
    ) => {
        {
        let mut $sum: usize = 0;
        $($stmts)*
        let size = core::mem::size_of::<$struct>();
        assert!(
            size == $size,
            "Invalid size for struct {} (expected {:#X} but was {:#X})",
            stringify!($struct),
            $size,
            size
        );
        }
    };

    // Munch field.
    (@munch $struct:ident $sum:ident
        (
            $(#[$attr:meta])*
            ($offset_start:expr => $field:ident: $ty:ty),
            $(#[$attr_next:meta])*
            ($offset_end:expr => $($next:tt)*),
            $($after:tt)*
        )
        -> {$($output:block)*}
    ) => {
        $crate::test_fields!(
            @munch $struct $sum (
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
    (@munch $struct:ident $sum:ident
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
            @munch $struct $sum (
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

#[macro_export]
macro_rules! register_structs {
    {
        $( $name:ident {
            $( $fields:tt )*
        } ),*
    } => {
        $( $crate::register_fields!(@root $name { $($fields)* } ); )*

        #[cfg(test)]
        mod test_register_structs {
        $(
            #[allow(non_snake_case)]
            mod $name {
                use super::super::*;
                #[test]
                fn test_offsets() {
                    $crate::test_fields!(@root $name $($fields)* )
                }
            }
        )*
        }
    };
}
