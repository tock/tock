// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! A `Cell` whose writes require proof that interrupts are disabled.

use core::cell::Cell;

use crate::context_tokens::InterruptsDisabledContext;

/// A `Cell<T>` that can be freely read, but can only be written while
/// holding proof that interrupts are disabled on this core.
///
/// This is intended for state shared between top half (handler context)
/// code and normal kernel main loop code, e.g., a software-maintained
/// "saved interrupts" bitmap.
///
/// Reads are unguarded because a stale read here is not a consistency
/// problem -- interrupts can occur any time generally. However, when
/// writing, it's important that the writer understand the implications
/// of writing to state shared between handler and non-handler context.
///
/// This type is deliberately not [`Sync`]: `Sync` would promise safe access
/// from any hart with no further checking, which would not hold on genuine
/// multi-hart hardware (the same reason [`InterruptsDisabledContext`] itself is
/// `!Send + !Sync`). Consequently, a struct embedding this type still needs
/// to live behind a `static mut` (or equivalent) at the top level, just as
/// it would with a plain `Cell`.
pub struct InterruptsDisabledCell<T> {
    value: Cell<T>,
}

impl<T> InterruptsDisabledCell<T> {
    /// Create a new `InterruptsDisabledCell` containing `val`.
    pub const fn new(val: T) -> Self {
        Self {
            value: Cell::new(val),
        }
    }

    /// Overwrite the contained value.
    ///
    /// Requires proof that interrupts are disabled on this core, so this
    /// cannot race a concurrent read-modify-write from the other half of a
    /// top/bottom-half handler pair.
    pub fn set(&self, val: T, _interrupts_disabled: &InterruptsDisabledContext) {
        self.value.set(val);
    }
}

impl<T: Copy> InterruptsDisabledCell<T> {
    /// Read the contained value.
    ///
    /// Unguarded: a torn or stale read is not a soundness problem for this
    /// type's intended use.
    pub fn get(&self) -> T {
        self.value.get()
    }

    /// Read the contained value, apply `f`, and store the result.
    pub fn update(
        &self,
        f: impl FnOnce(T) -> T,
        interrupts_disabled: &InterruptsDisabledContext,
    ) -> T {
        let new = f(self.get());
        self.set(new, interrupts_disabled);
        new
    }
}
