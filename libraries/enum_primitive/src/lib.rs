// copied from https://github.com/andersk/enum_primitive-rs which did not with with nostd out of box
#![no_std]
pub mod cast;

/// Helper macro for internal use by `enum_from_primitive!`.
#[macro_export]
macro_rules! enum_from_primitive_impl_ty {
    ($meth:ident, $ty:ty, $name:ident, $( $variant:ident )*) => {
        #[allow(non_upper_case_globals, unused)]
        fn $meth(n: $ty) -> Option<Self> {
            $( if n == $name::$variant as $ty {
                Some($name::$variant)
            } else )* {
                None
            }
        }
    };
}

/// Helper macro for internal use by `enum_from_primitive!`.
#[macro_export]
macro_rules! enum_from_primitive_impl {
    ($name:ident, $( $variant:ident )*) => {
        impl FromPrimitive for $name {
            enum_from_primitive_impl_ty! { from_i64, i64, $name, $( $variant )* }
            enum_from_primitive_impl_ty! { from_u64, u64, $name, $( $variant )* }

        }
    };
}

/// Wrap this macro around an `enum` declaration to get an
/// automatically generated implementation of `num::FromPrimitive`.
#[macro_export]
macro_rules! enum_from_primitive {
    (
        $( #[$enum_attr:meta] )*
        enum $name:ident {
            $( $( #[$variant_attr:meta] )* $variant:ident ),+ $( = $discriminator:expr, $( $( #[$variant_two_attr:meta] )* $variant_two:ident ),+ )*
        }
    ) => {
        $( #[$enum_attr] )*
        enum $name {
            $( $( #[$variant_attr] )* $variant ),+ $( = $discriminator, $( $( #[$variant_two_attr] )* $variant_two ),+ )*
        }
        enum_from_primitive_impl! { $name, $( $variant )+ $( $( $variant_two )+ )* }
    };

    (
        $( #[$enum_attr:meta] )*
        enum $name:ident {
            $( $( $( #[$variant_attr:meta] )* $variant:ident ),+ = $discriminator:expr ),*
        }
    ) => {
        $( #[$enum_attr] )*
        enum $name {
            $( $( $( #[$variant_attr] )* $variant ),+ = $discriminator ),*
        }
        enum_from_primitive_impl! { $name, $( $( $variant )+ )* }
    };

    (
        $( #[$enum_attr:meta] )*
        enum $name:ident {
            $( $( #[$variant_attr:meta] )* $variant:ident ),+ $( = $discriminator:expr, $( $( #[$variant_two_attr:meta] )* $variant_two:ident ),+ )*,
        }
    ) => {
        $( #[$enum_attr] )*
        enum $name {
            $( $( #[$variant_attr] )* $variant ),+ $( = $discriminator, $( $( #[$variant_two_attr] )* $variant_two ),+ )*,
        }
        enum_from_primitive_impl! { $name, $( $variant )+ $( $( $variant_two )+ )* }
    };

    (
        $( #[$enum_attr:meta] )*
        enum $name:ident {
            $( $( $( #[$variant_attr:meta] )* $variant:ident ),+ = $discriminator:expr ),+,
        }
    ) => {
        $( #[$enum_attr] )*
        enum $name {
            $( $( $( #[$variant_attr] )* $variant ),+ = $discriminator ),+,
        }
        enum_from_primitive_impl! { $name, $( $( $variant )+ )+ }
    };

    (
        $( #[$enum_attr:meta] )*
        pub enum $name:ident {
            $( $( #[$variant_attr:meta] )* $variant:ident ),+ $( = $discriminator:expr, $( $( #[$variant_two_attr:meta] )* $variant_two:ident ),+ )*
        }
    ) => {
        $( #[$enum_attr] )*
        pub enum $name {
            $( $( #[$variant_attr] )* $variant ),+ $( = $discriminator, $( $( #[$variant_two_attr] )* $variant_two ),+ )*
        }
        enum_from_primitive_impl! { $name, $( $variant )+ $( $( $variant_two )+ )* }
    };

    (
        $( #[$enum_attr:meta] )*
        pub enum $name:ident {
            $( $( $( #[$variant_attr:meta] )* $variant:ident ),+ = $discriminator:expr ),*
        }
    ) => {
        $( #[$enum_attr] )*
        pub enum $name {
            $( $( $( #[$variant_attr] )* $variant ),+ = $discriminator ),*
        }
        enum_from_primitive_impl! { $name, $( $( $variant )+ )* }
    };

    (
        $( #[$enum_attr:meta] )*
        pub enum $name:ident {
            $( $( #[$variant_attr:meta] )* $variant:ident ),+ $( = $discriminator:expr, $( $( #[$variant_two_attr:meta] )* $variant_two:ident ),+ )*,
        }
    ) => {
        $( #[$enum_attr] )*
        pub enum $name {
            $( $( #[$variant_attr] )* $variant ),+ $( = $discriminator, $( $( #[$variant_two_attr] )* $variant_two ),+ )*,
        }
        enum_from_primitive_impl! { $name, $( $variant )+ $( $( $variant_two )+ )* }
    };

    (
        $( #[$enum_attr:meta] )*
        pub enum $name:ident {
            $( $( $( #[$variant_attr:meta] )* $variant:ident ),+ = $discriminator:expr ),+,
        }
    ) => {
        $( #[$enum_attr] )*
        pub enum $name {
            $( $( $( #[$variant_attr] )* $variant ),+ = $discriminator ),+,
        }
        enum_from_primitive_impl! { $name, $( $( $variant )+ )+ }
    };
}
