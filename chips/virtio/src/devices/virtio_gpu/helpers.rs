// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use kernel::ErrorCode;

/// Const version of `std::cmp::max` for an array of `usize`s.
///
/// Used to determine the maximum VirtIO GPU request / response size,
/// to determine the VirtQueue buffer size required.
///
/// This is ... not great. `const fn`s are pretty restrictive still,
/// and most subslicing or iterator operations can't be used. So this
/// looks like its the best we can do, at least for now.
pub const fn max(elems: &[usize]) -> usize {
    const fn max_inner(elems: &[usize], idx: usize) -> usize {
        match elems.len() - idx {
            0 => usize::MIN,
            1 => elems[idx],
            _ => {
                let max_tail = max_inner(elems, idx + 1);
                if max_tail > elems[idx] {
                    max_tail
                } else {
                    elems[idx]
                }
            }
        }
    }
    max_inner(elems, 0)
}

/// Helper to copy a buffer into an iterator, returning
/// `Err(ErrorCode::SIZE)` if it doesn't fit.
#[inline]
pub fn copy_to_iter<'a, T: 'a>(
    dst: &mut impl Iterator<Item = &'a mut T>,
    src: impl Iterator<Item = T>,
) -> Result<(), ErrorCode> {
    for e in src {
        *dst.next().ok_or(ErrorCode::SIZE)? = e;
    }
    Ok(())
}

/// Create a byte-array from an iterator of `u8`s, returning
/// `Err(ErrorCode::SIZE)` if the iterator does not yield a sufficient
/// number of elements.
#[inline]
pub fn bytes_from_iter<const N: usize>(
    src: &mut impl Iterator<Item = u8>,
) -> Result<[u8; N], ErrorCode> {
    let mut dst: [u8; N] = [0; N];

    for d in dst.iter_mut() {
        *d = src.next().ok_or(ErrorCode::SIZE)?;
    }

    Ok(dst)
}
