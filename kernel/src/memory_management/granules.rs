// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2025.

//! Memory granules.

use crate::utilities::alignment::Alignment;
use crate::utilities::misc::{create_non_zero_usize, divide_non_zero_usize, modulo_non_zero_usize};

use core::num::NonZero;

/// A memory granule.
pub trait Granule: Alignment {
    /// The size of the granule, in bytes.
    const SIZE_U8: NonZero<usize>;

    fn ceil_from_byte_count(byte_count: NonZero<usize>) -> NonZero<usize> {
        let quotient = divide_non_zero_usize(byte_count, Self::SIZE_U8);
        let remainder = modulo_non_zero_usize(byte_count, Self::SIZE_U8);
        let result = if remainder != 0 {
            quotient + 1
        } else {
            quotient
        };

        // PANIC:
        //
        // Case 1: remainder != 0
        //
        // [1] result must be 0 to panic.
        // [2] result = quotient + 1 according to the hypothesis (remainder != 0)
        // [3] quotient + 1 can be 0 <==> quotient == usize::MAX
        // [4] quotient = byte_count / Self::SIZE_U8
        // [5] [3], [4] ==> quotient == usize::MAX <==> byte_count == usize::MAX and
        // Self::SIZE_U8 == 1.
        // [6] [5] ==> remainder == 0 since Self::SIZE_U8 == 1. Contradiction with the hypothesis.
        //
        // Case 2: remainder == 0
        //
        // [1] result must be 0 to panic.
        // [2] result = quotient
        // [3] [1], [2] ==> quotient == 0
        // [4] quotient = byte_count / Self::SIZE_U8
        // [5] [3], [4] ==> byte_count / Self::SIZE_U8 == 0
        // [6] remainder == 0 (hypothesis) => byte_count = k * Self::SIZE_U8, 0 <= k <= usize::MAX.
        // [7] byte_count != 0 (non-zero value)
        // [8] [6], [7] ==> 0 < k <= usize::MAX
        // [9] [5], [6] ==> byte_count / Self::SIZE_U8 == k == 0
        // [10] [8], [9] ==> contradiction
        //
        // Therefore, the unwrap() may never panic.
        NonZero::new(result).unwrap()
    }
}

impl Granule for u8 {
    const SIZE_U8: NonZero<usize> = create_non_zero_usize(core::mem::size_of::<Self>());
}
