// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

// This is inspired and adapted for Tock from the [x86](https://github.com/gz/rust-x86) crate.

//! Tock x86.

pub mod bits32;
pub mod controlregs;
pub mod dtables;
pub mod io;
pub mod irq;
pub mod ring;
pub mod segmentation;
pub mod task;
pub mod tlb;
