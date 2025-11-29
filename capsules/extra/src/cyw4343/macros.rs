// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

/// Macro for parsing into/from bytes for structs that represent protocol packet headers.
/// This generates an `impl` block with `into_bytes` and `from_bytes` const methods and an
/// associated `SIZE` constant for the struct size in bytes.
///
/// ## Example
///
/// ```rust
/// // Usage
///
/// parse!{
///     #[derive(Debug)]
///     struct Header {
///         data: u32,
///         crc: u8,
///         another_field: [u8; 10]
///     }
/// };
///
/// // This generates:
///
/// #[derive(Debug)]
/// struct Header {
///     data: u32,
///     crc: u8,
///     another_field: [u8; 10]
/// }
///
/// impl Header {
///     pub const SIZE: usize = core::mem::size_of::<Self>();
///     pub const fn into_bytes(self) -> [u8; Self::SIZE] {
///         // ...
///     }
///     pub const fn from_bytes(__bytes: &[u8]) -> Self {
///         // ...
///     }
/// }
/// ```
macro_rules! parse {
    (
        $(#[$attr_struct:meta])* $vis_struct:vis struct $name:ident { $($(#[$attr_field:meta])* $vis_field:vis $field:ident : $field_ty:tt),* $(,)? }
        ) => {
        $(#[$attr_struct])*
        $vis_struct struct $name {
            $($(#[$attr_field])* $vis_field $field : $field_ty),*,
        }
        impl $name {
            #![allow(unused)]
            pub const SIZE: usize = core::mem::size_of::<Self>();
            pub const fn into_bytes(self) -> [u8; Self::SIZE] {
                let mut __bytes = [0u8; Self::SIZE];
                let mut __len = 0;
                $(
                    parse!(@f __len, __bytes, self.$field, $field_ty);
                )*
                __bytes
            }
            pub const fn from_bytes(__bytes: &[u8]) -> Self {
                let mut __len = 0;
                $(
                    parse!(@from_f __len, __bytes, $field, $field_ty);
                )*
                Self {
                    $($field),*
                }
            }
        }
    };

    // Inner macros for copying the bytes from the buffer into a field.
    (@from_f $len: ident, $bytes:ident, $field:ident, u8) => {
        let $field = $bytes[$len];
        $len += 1;
    };
    (@from_f $len: ident, $bytes:ident, $field:ident, u16) => {
        let $field = u16::from_le_bytes([$bytes[$len], $bytes[$len + 1]]);
        $len += 2;
    };
    (@from_f $len: ident, $bytes:ident, $field:ident, u32) => {
        let $field = u32::from_le_bytes([$bytes[$len], $bytes[$len + 1], $bytes[$len + 2], $bytes[$len + 3]]);
        $len += 4;
    };
    (@from_f $len: ident, $bytes:ident, $field:ident, i32) => {
        let $field = i32::from_le_bytes([$bytes[$len], $bytes[$len + 1], $bytes[$len + 2], $bytes[$len + 3]]);
        $len += 4;
    };
    (@from_f $len: ident, $bytes:ident, $field:ident, [u8; $N:literal]) => {
        let mut $field = [0u8; $N];
        let mut __idx = 0;
        while __idx < $N {
            $field[__idx] = $bytes[$len];
            __idx += 1;
            $len += 1;
        }
    };

    // Inner macros for copying the field value to the bytes buffer.
    (@f $len:ident, $bytes:ident, $field:expr, u8) => {
        $bytes[$len] = $field;
        $len += 1;
    };
    (@f $len:ident, $bytes: ident, $field: expr, u16) => {
        let __field_le_bytes = $field.to_le_bytes();
        $bytes[$len] = __field_le_bytes[0];
        $bytes[$len + 1] = __field_le_bytes[1];
        $len += 2;
    };
    (@f $len:ident, $bytes: ident, $field: expr, i32) => {
        let __field_le_bytes = $field.to_le_bytes();
        $bytes[$len] = __field_le_bytes[0];
        $bytes[$len + 1] = __field_le_bytes[1];
        $bytes[$len + 2] = __field_le_bytes[2];
        $bytes[$len + 3] = __field_le_bytes[3];
        $len += 4;
    };
    (@f $len:ident, $bytes: ident, $field: expr, u32) => {
        let __field_le_bytes = $field.to_le_bytes();
        $bytes[$len] = __field_le_bytes[0];
        $bytes[$len + 1] = __field_le_bytes[1];
        $bytes[$len + 2] = __field_le_bytes[2];
        $bytes[$len + 3] = __field_le_bytes[3];
        $len += 4;
    };
    (@f $len:ident, $bytes:ident, $field:expr, [u8; $N:literal]) => {
        let mut __idx = 0;
        while __idx < $N {
            $bytes[$len] = $field[__idx];
            $len += 1;
            __idx += 1;
        }
    };
}

macro_rules! backplane_window_bits {
    ($addr:expr) => {
        ($addr & !$crate::cyw4343::constants::BACKPLANE_ADDRESS_MASK) >> 8
    };
}

macro_rules! reset_and_restore_bufs {
    ($self: ident, $($buf:ident),*) => {{
        $($buf.reset();)*
        $($self.$buf.set($buf);)*
    }}
}

pub(crate) use backplane_window_bits;
pub(crate) use parse;
pub(crate) use reset_and_restore_bufs;
