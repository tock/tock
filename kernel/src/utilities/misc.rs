// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024.

//! Miscellaneous functions.

use super::constants::BITS_PER_U8;

use core::num::NonZero;

pub const fn fill_u16_with_byte(byte: u8) -> u16 {
    // CAST: size_of(u16) > size_of(u8)
    let casted_byte = byte as u16;
    // CAST: BITS_PER_U8 == 8 which fits in usize
    const CASTED_RAW_BITS_PER_U8: u16 = BITS_PER_U8.get() as u16;
    casted_byte | (casted_byte << CASTED_RAW_BITS_PER_U8)
}

pub const fn fill_u32_with_byte(byte: u8) -> u32 {
    // CAST: size_of(u32) > size_of(u16)
    let casted_byte = byte as u32;
    casted_byte * 0x1010101
}

pub const fn concatenate_two_u32(lo: u32, hi: u32) -> u64 {
    lo as u64 | ((hi as u64) << u32::BITS)
}

macro_rules! create_non_zero {
    ($name:ident, $zeroable:ty) => {
        /// Create a non-zero type from the given value.
        ///
        /// # Parameters
        ///
        /// + `value`: the inner representation of the non-zero type.
        ///
        /// # Return value
        ///
        /// A new instance of the non-zero type.
        ///
        /// # Panic
        ///
        /// The function panics if value is null.
        #[track_caller]
        pub const fn $name(value: $zeroable) -> NonZero<$zeroable> {
            match NonZero::new(value) {
                None => panic!("Invalid value"),
                Some(non_zero_value) => non_zero_value,
            }
        }
    };
}

create_non_zero!(create_non_zero_usize, usize);
create_non_zero!(create_non_zero_u64, u64);
create_non_zero!(create_non_zero_u32, u32);
create_non_zero!(create_non_zero_u16, u16);

macro_rules! impl_divide_zeroable {
    (
        $divide_name:ident,
        $modulo_name:ident,
        $ceil_name:ident,
        $align_down_name:ident,
        $align_up_name:ident,
        $zeroable:ty,
        $(,)?
    ) => {
        /// Divide a zeroable type by its non-zero equivalent.
        ///
        /// # Parameters
        ///
        /// + `dividend`: the dividend of the division
        /// + `divisor`: the divisor of the division
        ///
        /// # Return value
        ///
        /// The result of the division.
        pub const fn $divide_name(dividend: $zeroable, divisor: NonZero<$zeroable>) -> $zeroable {
            // DIVISION: The type of the divisor guarantees that it can't be null.
            dividend / divisor.get()
        }

        /// Return the remainder of the division of a zeroable type by its non-zero equivalent.
        ///
        /// # Parameters
        ///
        /// + `dividend`: the dividend of the division
        /// + `divisor`: the divisor of the division
        ///
        /// # Return value
        ///
        /// The remainder of the division.
        pub const fn $modulo_name(dividend: $zeroable, divisor: NonZero<$zeroable>) -> $zeroable {
            // DIVISION: The type of the divisor guarantees that it can't be null.
            dividend % divisor.get()
        }

        /// Return the ceil of the division of a zeroable type by its non-zero equivalent.
        ///
        /// # Parameters
        ///
        /// + `dividend`: the dividend of the ceil
        /// + `divisor`: the divisor of the ceil
        ///
        /// # Return value
        ///
        /// The ceil of the division.
        pub const fn $ceil_name(dividend: $zeroable, divisor: NonZero<$zeroable>) -> $zeroable {
            let raw_divisor = divisor.get();
            let actual_dividend = dividend + raw_divisor - 1;
            // DIVISION: The type of the divisor guarantees that it can't be null.
            actual_dividend / raw_divisor
        }

        pub const fn $align_down_name(
            value: $zeroable,
            alignment: NonZero<$zeroable>,
        ) -> $zeroable {
            let modulo = $modulo_name(value, alignment);
            value - modulo
        }

        pub const fn $align_up_name(value: $zeroable, alignment: NonZero<$zeroable>) -> $zeroable {
            let modulo = $modulo_name(value, alignment);
            value + alignment.get() - modulo
        }
    };
}

impl_divide_zeroable!(
    divide_usize,
    modulo_usize,
    ceil_usize,
    align_down_usize,
    align_up_usize,
    usize,
);
impl_divide_zeroable!(
    divide_u64,
    modulo_u64,
    ceil_u64,
    align_down_u64,
    align_up_u64,
    u64,
);
impl_divide_zeroable!(
    divide_u32,
    modulo_u32,
    ceil_u32,
    align_down_u32,
    align_up_u32,
    u32,
);

macro_rules! impl_divide_exact_zeroable {
    ($name:ident, $zeroable:ty) => {
        /// Divide a non-zeroable value by another. Panics if the dividend is not a multiple of the
        /// divisor.
        ///
        /// # Parameters
        ///
        /// + `dividend`: the dividend of the division
        /// + `divisor`: the divisor of the division
        ///
        /// # Return value
        ///
        /// The result of the division
        #[track_caller]
        pub const fn $name(dividend: $zeroable, divisor: NonZero<$zeroable>) -> $zeroable {
            let raw_divisor = divisor.get();
            if dividend % raw_divisor != 0 {
                panic!("Inexact division")
            } else {
                // DIVISION: The type of the divisor guarantees that it can't be null.
                dividend / raw_divisor
            }
        }
    };
}

impl_divide_exact_zeroable!(divide_exact_usize, usize);
impl_divide_exact_zeroable!(divide_exact_u32, u32);

macro_rules! impl_divide_modulo_non_zeroable {
    ($divide_name:ident, $modulo_name:ident, $ceil_name: ident, $zeroable:ty, $(,)?) => {
        /// Divide a non-zeroable value by another
        ///
        /// # Parameters
        ///
        /// + `dividend`: the dividend of the division
        /// + `divisor`: the divisor of the division
        ///
        /// # Return value
        ///
        /// The result of the division.
        pub const fn $divide_name(
            dividend: NonZero<$zeroable>,
            divisor: NonZero<$zeroable>,
        ) -> $zeroable {
            // DIVISION: The type of the divisor guarantees that it can't be null.
            dividend.get() / divisor.get()
        }

        /// Return the remainder of the division of a non-zeroable value by another
        ///
        /// # Parameters
        ///
        /// + `dividend`: the dividend of the division
        /// + `divisor`: the divisor of the division
        ///
        /// # Return value
        ///
        /// The remainder of the division.
        pub const fn $modulo_name(
            dividend: NonZero<$zeroable>,
            divisor: NonZero<$zeroable>,
        ) -> $zeroable {
            // MODULO: The type of the divisor guarantees that it can't be null.
            dividend.get() % divisor.get()
        }

        pub const fn $ceil_name(
            dividend: NonZero<$zeroable>,
            divisor: NonZero<$zeroable>,
        ) -> NonZero<$zeroable> {
            let raw_divisor = divisor.get();
            let actual_dividend = dividend.get() + raw_divisor - 1;
            // DIVISION: the type of the divisor guarantees that it can't be null.
            let divide_result = actual_dividend / raw_divisor;
            // SAFETY: ceil(x / y) = 0 <==> x == 0, but since x is of type NonZero, ceil(x / y)
            // cannot be 0.
            unsafe { NonZero::new_unchecked(divide_result) }
        }
    };
}

impl_divide_modulo_non_zeroable!(
    divide_non_zero_usize,
    modulo_non_zero_usize,
    ceil_non_zero_usize,
    usize,
);
impl_divide_modulo_non_zeroable!(
    divide_non_zero_u32,
    modulo_non_zero_u32,
    ceil_non_zero_u32,
    u32,
);
impl_divide_modulo_non_zeroable!(
    divide_non_zero_u16,
    modulo_non_zero_u16,
    ceil_non_zero_u16,
    u16,
);

macro_rules! impl_divide_exact_non_zeroable {
    ($name:ident, $zeroable:ty) => {
        /// Divide a non-zeroable value by another. Panics if the dividend is not a multiple of the
        /// divisor.
        ///
        /// # Parameters
        ///
        /// + `dividend`: the dividend of the division
        /// + `divisor`: the divisor of the division
        ///
        /// # Return value
        ///
        /// The result of the division
        #[track_caller]
        pub const fn $name(
            dividend: NonZero<$zeroable>,
            divisor: NonZero<$zeroable>,
        ) -> NonZero<$zeroable> {
            let raw_dividend = dividend.get();
            let raw_divisor = divisor.get();
            if raw_dividend % raw_divisor != 0 {
                panic!("Inexact division")
            } else {
                // SAFETY:
                //
                // 1. raw_dividend = k * raw_divisor (multiple)
                // 2. raw_dividend != 0 (coming from NonZero)
                // 3. raw_divisor != 0 (coming from NonZero)
                //
                // [1], [2], [3] => k != 0
                unsafe { NonZero::new_unchecked(raw_dividend / raw_divisor) }
            }
        }
    };
}

impl_divide_exact_non_zeroable!(divide_exact_non_zero_usize, usize);
impl_divide_exact_non_zeroable!(divide_exact_non_zero_u32, u32);

#[track_caller]
pub const fn cast_non_zero_u16_to_non_zero_u32(non_zero_u16: NonZero<u16>) -> NonZero<u32> {
    // CAST: size_of::<u32>() > size_of::<u16>()
    let raw_value = non_zero_u16.get() as u32;
    // PANIC: `NonZero<u16>` guarantees `raw_value` is non-zero
    create_non_zero_u32(raw_value)
}

#[track_caller]
pub const fn cast_non_zero_u16_to_non_zero_usize(non_zero_u16: NonZero<u16>) -> NonZero<usize> {
    // CAST: size_of::<usize>() > size_of::<u16>()
    let raw_value = non_zero_u16.get() as usize;
    // PANIC: `NonZero<u16>` guarantees `raw_value` is non-zero
    create_non_zero_usize(raw_value)
}

#[track_caller]
pub const fn cast_non_zero_u32_to_non_zero_usize(non_zero_u32: NonZero<u32>) -> NonZero<usize> {
    // CAST: Tock runs on 32-bit and 64-bit architectures, which means that
    // size_of::<usize>() >= size_of::<u32>()
    let raw_value = non_zero_u32.get() as usize;
    // PANIC: `NonZero<u32>` guarantees `raw_value` is non-zero
    create_non_zero_usize(raw_value)
}

#[track_caller]
pub const fn cast_non_zero_u32_to_non_zero_u64(non_zero_u32: NonZero<u32>) -> NonZero<u64> {
    // CAST: size_of::<u64>() >= size_of::<u32>()
    let raw_value = non_zero_u32.get() as u64;
    // PANIC: `NonZero<u32>` guarantees `raw_value` is non-zero
    create_non_zero_u64(raw_value)
}

#[track_caller]
pub const fn cast_non_zero_usize_to_non_zero_u64(non_zero_usize: NonZero<usize>) -> NonZero<u64> {
    // CAST: size_of::<u64>() >= size_of::<usize>() on 32 and 64-bit platforms supported by Rust.
    let raw_value = non_zero_usize.get() as u64;
    // PANIC: `NonZero<usize>` guarantees that `raw_value` is non-zero
    create_non_zero_u64(raw_value)
}

#[track_caller]
pub const fn cast_non_zero_usize_to_non_zero_u32(non_zero_usize: NonZero<usize>) -> NonZero<u32> {
    let raw_value = non_zero_usize.get();
    // CAST: On 32-bit and 64-bit platforms Tock runs on, size_of::<usize>() >= size_of::<u32>()
    if raw_value > u32::MAX as usize {
        panic!("Truncation of NonZero<usize> when casting to NonZero<u32>")
    } else {
        // CAST: because of the if condition, raw_value <= u32::MAX
        // PANIC: `NonZero<usize>` guarantees `raw_value` is non-zero
        create_non_zero_u32(raw_value as u32)
    }
}

#[track_caller]
pub const fn cast_non_zero_usize_to_non_zero_u16(non_zero_usize: NonZero<usize>) -> NonZero<u16> {
    let raw_value = non_zero_usize.get();
    // CAST: On 32-bit and 64-bit platforms Tock runs on, size_of::<usize>() >= size_of::<u32>()
    if raw_value > u16::MAX as usize {
        panic!("Truncation of NonZero<usize> when casting to NonZero<u16>")
    } else {
        // CAST: because of the if condition, raw_value <= u16::MAX
        // PANIC: `NonZero<usize>` guarantees `raw_value` is non-zero
        create_non_zero_u16(raw_value as u16)
    }
}

#[track_caller]
pub const fn cast_non_zero_u64_to_non_zero_u32(non_zero_u64: NonZero<u64>) -> NonZero<u32> {
    let raw_value = non_zero_u64.get();
    // CAST: size_of::<u64>() > size_of::<u32>()
    if raw_value > u32::MAX as u64 {
        panic!("Truncation of NonZero<u64> when casting to NonZero<u32>")
    } else {
        // CAST: because of the if condition, raw_value <= u32::MAX
        // PANIC: `NonZero<usize>` guarantees `raw_value` is non-zero
        create_non_zero_u32(raw_value as u32)
    }
}
