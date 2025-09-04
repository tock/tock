// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Virtualizers for screens.
//!
//! Since screens are user-facing, they cannot be completely virtualized such
//! that two users both have an abstraction that they completely own the
//! screen. There may be multiple ways around this, and hence multiple
//! virtualizers.

pub mod virtual_screen_split;
