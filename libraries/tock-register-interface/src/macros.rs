//! Macros for cleanly defining peripheral registers.

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

#[cfg(feature = "std_unit_tests")]
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

#[cfg(not(feature = "std_unit_tests"))]
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
