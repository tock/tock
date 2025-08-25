// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use core::cell::Cell;

/// Different deferred calls requested by and delivered to the
/// [`VirtIOGPU`] driver.
#[derive(Copy, Clone)]
#[repr(usize)]
pub enum PendingDeferredCall {
    SetWriteFrame,
}

/// Manager of the deferred calls requested by and delivered to the
/// [`VirtIOGPU`] driver.
pub struct PendingDeferredCallMask(Cell<usize>);

impl PendingDeferredCallMask {
    pub fn new() -> Self {
        PendingDeferredCallMask(Cell::new(0))
    }

    pub fn get_copy_and_clear(&self) -> PendingDeferredCallMask {
        let old = PendingDeferredCallMask(self.0.clone());
        self.0.set(0);
        old
    }

    pub fn set(&self, call: PendingDeferredCall) {
        self.0.set(self.0.get() | (1 << (call as usize)));
    }

    pub fn is_set(&self, call: PendingDeferredCall) -> bool {
        (self.0.get() & (1 << (call as usize))) != 0
    }

    pub fn for_each_call(&self, mut f: impl FnMut(PendingDeferredCall)) {
        let mut check_and_invoke = |call| {
            if self.is_set(call) {
                f(call)
            }
        };

        check_and_invoke(PendingDeferredCall::SetWriteFrame);
    }
}
