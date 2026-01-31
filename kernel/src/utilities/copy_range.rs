// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use core::ops::Range;

/// A copyable twin of `core::ops::Range`:
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct CopyRange<Idx> {
    pub start: Idx,
    pub end: Idx,
}

impl<Idx> From<Range<Idx>> for CopyRange<Idx> {
    fn from(r: Range<Idx>) -> Self {
        CopyRange {
            start: r.start,
            end: r.end,
        }
    }
}

impl<Idx> From<CopyRange<Idx>> for Range<Idx> {
    fn from(c: CopyRange<Idx>) -> Self {
        Range {
            start: c.start,
            end: c.end,
        }
    }
}
