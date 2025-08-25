// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Interface for keyboard key presses.

use crate::ErrorCode;

/// Receiver for keyboard key presses.
pub trait KeyboardClient {
    /// Called when one or more keys are pressed or un-pressed.
    ///
    /// `keys` is an array of `(key_code, is_pressed)` tuples. `key_code` is the
    /// same as the codes used by Linux to identify different keys.
    /// `is_pressed` is true if the key was pressed, and false if the key was
    /// un-pressed. A list of keycodes can be found here:
    /// https://manpages.ubuntu.com/manpages/focal/man7/virkeycode-linux.7.html
    ///
    /// `result` is `Ok(())` if the keys were received correctly.
    fn keys_pressed(&self, keys: &[(u16, bool)], result: Result<(), ErrorCode>);
}

/// Represents a keyboard that can generate button presses.
pub trait Keyboard<'a> {
    fn set_client(&self, client: &'a dyn KeyboardClient);
}
