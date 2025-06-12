// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2025.

//! Pages supported by Tock.

use super::granules::Granule;

use crate::utilities::misc::create_non_zero_usize;

use core::num::NonZero;

/// A standard 4KiB page.
#[repr(align(4096))]
#[derive(Clone)]
#[allow(dead_code)]
pub struct Page4KiB([u8; Self::SIZE_U8.get()]);

impl Granule for Page4KiB {
    const SIZE_U8: NonZero<usize> = create_non_zero_usize(4096);
}
