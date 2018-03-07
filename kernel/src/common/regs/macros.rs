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
        $( register_bitmasks!($valtype, $reg_desc, $(#[$inner])* $field, $offset, 1, []); )*
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
        $( register_bitmasks!($valtype, $reg_desc, $(#[$inner])* $field, $offset, 1, []); )*
    };

    {
        // BITFIELD_NAME OFFSET(x) NUMBITS(y)
        $(#[$outer:meta])*
        $valtype:ty, $reg_desc:ident, [
            $( $(#[$inner:meta])* $field:ident OFFSET($offset:expr) NUMBITS($numbits:expr) ),+
        ]
    } => {
        $(#[$outer])*
        $( register_bitmasks!($valtype, $reg_desc, $(#[$inner])* $field, $offset, $numbits, []); )*
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
        $( register_bitmasks!($valtype, $reg_desc, $(#[$inner])* $field, $offset, $numbits,
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
            use $crate::common::regs::FieldValue;
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
                impl $crate::common::regs::RegisterLongName for Register {}

                use $crate::common::regs::Field;

                register_bitmasks!( $valtype, Register, $fields );
            }
        )*
    }
}
