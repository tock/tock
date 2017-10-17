#[macro_export]
macro_rules! bitmasks {
    {
        $valtype:ty, $reg_desc:ident, [
            $( $field:ident $a:tt ),+
        ]
    } => {
        $( bitmasks!($valtype, $reg_desc, $field, $a, []); )*
    };

    {
        $valtype:ty, $reg_desc:ident, [
            $( $field:ident $a:tt $b:tt ),+
        ]
    } => {
        $( bitmasks!($valtype, $reg_desc, $field, $a, $b); )*
    };

    {
        $valtype:ty, $reg_desc:ident, $field:ident,
                    ($shift:expr, Mask($mask:expr)),
                    [$( $valname:ident = $value:expr ),*]
    } => {
        #[allow(non_upper_case_globals)]
        #[allow(unused)]
        pub const $field: Field<$valtype, $reg_desc> =
            Field::<$valtype, $reg_desc>::new($mask, $shift);

        #[allow(non_snake_case)]
        #[allow(unused)]
        pub mod $field {
            #[allow(unused_imports)]
            use $crate::common::regs::FieldValue;
            use super::super::$reg_desc;

            $(
            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            pub const $valname: FieldValue<$valtype, $reg_desc> =
                FieldValue::<$valtype, $reg_desc>::new($mask, $shift, $value);
            )*

            #[allow(dead_code)]
            #[allow(non_camel_case_types)]
            pub enum Value {
                $(
                    $valname = $value,
                )*
            }
        }
    };

    {
        $valtype:ty, $reg_desc:ident, $field:ident, $bit:expr,
        [$( $valname:ident = $value:expr),* ]
    } => {
        #[allow(non_upper_case_globals)]
        #[allow(unused)]
        pub const $field: Field<$valtype, $reg_desc> =
            Field::<$valtype, $reg_desc>::new(1, $bit);

        #[allow(non_snake_case)]
        #[allow(unused)]
        pub mod $field {
            #[allow(unused_imports)]
            use $crate::common::regs::FieldValue;
            use super::super::$reg_desc;

            $(
            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            pub const $valname: FieldValue<$valtype, $reg_desc> =
                FieldValue::<$valtype, $reg_desc>::new(1, $bit, $value);
            )*

            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            pub const SET: FieldValue<$valtype, $reg_desc> =
                FieldValue::<$valtype, $reg_desc>::new(1, $bit, 1);

            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            pub const CLEAR: FieldValue<$valtype, $reg_desc> =
                FieldValue::<$valtype, $reg_desc>::new(1, $bit, 0);

            #[allow(dead_code)]
            #[allow(non_camel_case_types)]
            pub enum Value {
                $(
                    $valname = $value,
                )*
            }
        }
    };
}

#[macro_export]
macro_rules! bitfields {
    {
        $valtype:ty, $( $reg:ident $reg_desc:ident $fields:tt ),*
    } => {
        $(
            pub struct $reg_desc;
            impl $crate::common::regs::RegisterLongName for $reg_desc {}

            #[allow(non_snake_case)]
            pub mod $reg {
                use $crate::common::regs::Field;
                use super::$reg_desc;

                bitmasks!( $valtype, $reg_desc, $fields );
            }
        )*
    }
}
