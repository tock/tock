// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Basic implementation of a thread ID provider for x86 chips.

use kernel::platform::chip::ThreadIdProvider;

/// Implement the [`ThreadIdProvider`] trait for x86 platforms.
pub enum X86ThreadIdProvider {}

// # Safety
//
// By implementing [`ThreadIdProvider`] we are guaranteeing that we correctly
// return the thread ID. THIS IMPLEMENTATION IS BROKEN. We need to implement
// this correctly.
unsafe impl ThreadIdProvider for X86ThreadIdProvider {
    fn running_thread_id() -> usize {
        // TODO: IMPLEMENT!
        0
    }
}
