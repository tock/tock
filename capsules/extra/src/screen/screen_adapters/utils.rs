// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

#[derive(Clone, Copy, Default)]
pub struct Frame {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}
