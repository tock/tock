// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Optional in-kernel hardware tests for the NXP S32G3 SAIL board.
//!
//! These tests require real hardware. Enable by uncommenting the
//! corresponding line in `lib.rs::start()`.

#[cfg(feature = "test-harness")]
pub(crate) mod uart_suite;
