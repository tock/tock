#[macro_export]
macro_rules! bitmasks {
    {
        $(#[$outer:meta])*
        $valtype:ty, $reg_desc:ident, [
            $( $(#[$inner:meta])* $field:ident $a:tt ),+
        ]
    } => {
        $(#[$outer])*
        $( bitmasks!($valtype, $reg_desc, $(#[$inner])* $field, $a, []); )*
    };


    {
        $(#[$outer:meta])*
        $valtype:ty, $reg_desc:ident, [
            $( $(#[$inner:meta])* $field:ident $a:tt $b:tt ),+
        ]
    } => {
        $(#[$outer])*
        $( bitmasks!($valtype, $reg_desc, $(#[$inner])* $field, $a, $b); )*
    };



    {
        $(#[$outer:meta])*
        $valtype:ty, $reg_desc:ident, $field:ident,
                    ($shift:expr, Mask($mask:expr)),
                    [$( $(#[$inner:meta])* $valname:ident = $value:expr ),*]
    } => {
        #[allow(non_upper_case_globals)]
        #[allow(unused)]
        pub const $field: Field<$valtype, $reg_desc> = Field::<$valtype, $reg_desc>::new($mask, $shift);

        #[allow(non_snake_case)]
        #[allow(unused)]
        pub mod $field {
            #[allow(unused_imports)]
            use $crate::regs::FieldValue;
            use super::super::$reg_desc;

            $(
            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            $(#[$inner])*
            pub const $valname: FieldValue<$valtype, $reg_desc> = FieldValue::<$valtype, $reg_desc>::new($mask, $shift, $value);
            )*

            #[allow(dead_code)]
            #[allow(non_camel_case_types)]
            pub enum Value {
                $(
                    $(#[$inner])*
                    $valname = $value,
                )*
            }
        }
    };

    {
        $(#[$outer:meta])* $valtype:ty, $reg_desc:ident, $field:ident, $bit:expr,
        [$( $(#[$inner:meta])* $valname:ident = $value:expr),* ]
    } => {
        #[allow(non_upper_case_globals)]
        #[allow(unused)]
        $(#[$outer:meta])*
        pub const $field: Field<$valtype, $reg_desc> = Field::<$valtype, $reg_desc>::new(1, $bit);

        #[allow(non_snake_case)]
        #[allow(unused)]
        pub mod $field {
            #[allow(unused_imports)]
            use $crate::regs::FieldValue;
            use super::super::$reg_desc;

            $(
            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            $(#[$inner])*
            pub const $valname: FieldValue<$valtype, $reg_desc> = FieldValue::<$valtype, $reg_desc>::new(1, $bit, $value);
            )*

            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            pub const SET: FieldValue<$valtype, $reg_desc> = FieldValue::<$valtype, $reg_desc>::new(1, $bit, 1);

            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            pub const CLEAR: FieldValue<$valtype, $reg_desc> = FieldValue::<$valtype, $reg_desc>::new(1, $bit, 0);

            #[allow(dead_code)]
            #[allow(non_camel_case_types)]
            pub enum Value {
                $(
                    $(#[$inner])*
                    $valname = $value,
                )*
            }
        }
    };
}

#[macro_export]
macro_rules! bitfields {
    {
        $valtype:ty, $( $(#[$inner:meta])* $reg:ident $reg_desc:ident $fields:tt ),*
    } => {
        $(
            $(#[$inner])*
            pub struct $reg_desc;
            impl $crate::regs::RegisterLongName for $reg_desc {}

            #[allow(non_snake_case)]
            pub mod $reg {
                use $crate::regs::Field;
                use super::$reg_desc;


                bitmasks!( $valtype, $reg_desc, $fields );
            }
        )*
    }
}
