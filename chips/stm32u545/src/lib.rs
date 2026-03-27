// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

#![no_std]

pub use stm32u5xx::{chip, generic_init};

pub unsafe fn init() {
    generic_init();
}
