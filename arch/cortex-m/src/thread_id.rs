// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Basic implementation of a thread ID provider for Cortex-M.

use kernel::platform::chip::ThreadIdProvider;

/// Implement the [`ThreadIdProvider`] trait for Cortex-M platforms.
///
/// We assign thread IDs this way:
///
/// - 0: Main thread
/// - 1: Any interrupt service routine
pub enum CortexMThreadIdProvider {}

// # Safety
//
// By implementing [`ThreadIdProvider`] we are guaranteeing that we correctly
// return the thread ID. On single-core platforms the thread ID only depends on
// whether execution is in an interrupt service routine or not, which is what
// this implementation checks for.
unsafe impl ThreadIdProvider for CortexMThreadIdProvider {
    fn running_thread_id() -> usize {
        crate::support::is_interrupt_context() as usize
    }
}
