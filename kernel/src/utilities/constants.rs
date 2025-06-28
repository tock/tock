// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024.

//! Various constants useful throughout the kernel and architecture, chip and
//! board implementations.

use super::misc::{create_non_zero_usize, divide_non_zero_usize};

use core::num::NonZeroUsize;

/// KiB constant
#[allow(non_upper_case_globals)]
// PANIC: 2 ^ 10 != 0
pub const KiB: NonZeroUsize = create_non_zero_usize(1 << 10);

/// 4KiB constant
#[allow(non_upper_case_globals)]
// PANIC: 2 ^ 12 != 0
pub const FourKiB: NonZeroUsize = create_non_zero_usize(1 << 12);

/// 1MiB constant
#[allow(non_upper_case_globals)]
// PANIC: 2 ^ 20 != 0
pub const MiB: NonZeroUsize = create_non_zero_usize(1 << 20);

/// 2MiB constant
#[allow(non_upper_case_globals)]
// PANIC: 2 ^ 21 != 0
pub const TwoMiB: NonZeroUsize = create_non_zero_usize(1 << 21);

/// 1GiB constant
#[allow(non_upper_case_globals)]
// PANIC: 2 ^ 30 != 0
pub const GiB: NonZeroUsize = create_non_zero_usize(1 << 30);

/// The constant to divide by to convert seconds to micro seconds.
// PANIC: 1_000_000 != 0
pub const SECONDS_TO_MICRO_SECONDS: NonZeroUsize = create_non_zero_usize(1_000_000);

/// Number of bits in a u128.
// CAST: u128 fits in a usize on both 32-bit architectures and 64-bit architectures
// PANIC: 128 != 0
pub const BITS_PER_U128: NonZeroUsize = create_non_zero_usize(u128::BITS as usize);

/// Number of bits in a u64.
// CAST: u64 fits in a usize on both 32-bit architectures and 64-bit architectures
// PANIC: 64 != 0
pub const BITS_PER_U64: NonZeroUsize = create_non_zero_usize(u64::BITS as usize);

/// Number of bits in a u32.
// CAST: u32 fits in a usize on both 32-bit architectures and 64-bit architectures
// PANIC: 32 != 0
pub const BITS_PER_U32: NonZeroUsize = create_non_zero_usize(u32::BITS as usize);

/// Number of bits in a u16.
// CAST: u32 fits in a usize on both 32-bit architectures and 64-bit architectures
// PANIC: 16 != 0
pub const BITS_PER_U16: NonZeroUsize = create_non_zero_usize(u16::BITS as usize);

/// Number of bits in a u8.
// CAST: u32 fits in a usize on both 32-bit architectures and 64-bit architectures
// PANIC: 8 != 0
pub const BITS_PER_U8: NonZeroUsize = create_non_zero_usize(u8::BITS as usize);

/// Number of bytes in a u128.
pub const BYTES_PER_U128: NonZeroUsize =
    create_non_zero_usize(divide_non_zero_usize(BITS_PER_U128, BITS_PER_U8));

/// Number of bytes in a u64.
pub const BYTES_PER_U64: NonZeroUsize =
    create_non_zero_usize(divide_non_zero_usize(BITS_PER_U64, BITS_PER_U8));

/// Number of bytes in a u32.
pub const BYTES_PER_U32: NonZeroUsize =
    create_non_zero_usize(divide_non_zero_usize(BITS_PER_U32, BITS_PER_U8));

/// Number of bytes in a u16.
pub const BYTES_PER_U16: NonZeroUsize =
    create_non_zero_usize(divide_non_zero_usize(BITS_PER_U16, BITS_PER_U8));
