// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Interface for keyboard key presses.

use crate::ErrorCode;

pub trait Client {
    fn keys_pressed(&self, keys: &[u16], result: Result<(), ErrorCode>);
}

pub trait Keyboard<'a> {
    fn set_client(&self, client: &'a dyn Client);
}
