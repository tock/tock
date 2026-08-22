// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Typed proofs for the current context code is executing in.
//!
//! This module provides zero-sized marker tokens that signify the current
//! execution context. These are designed to provide compile-time assertions
//! for code which is sensitive to execution context. E.g., when manipulating
//! state that is shared across top-half and bottom-half interrupt handlers, it
//! is important to ensure that interrupts are disabled.
//!
//! Enforcing context with comments is fragile and subject to user error. The
//! `unsafe` keyword is overloaded here, as context-sensitive operations are
//! not necessarily soundness concerns (though, being able to assert an
//! execution context is often necessary to justify `unsafe` operations).
//!
//! Contexts are generally orthogonal. A core which is executing an interrupt
//! service routine (in handler context) may or may not automatically be in an
//! interrupt-disabled context---the details depend on the architecture. Code
//! using contexts should be careful to specify exactly which context(s) they
//! need and why.

use core::marker::PhantomData;

/// Proof that interrupts are currently disabled on the current core.
///
/// Like [`Chip::with_interrupts_disabled`](crate::platform::chip::Chip::with_interrupts_disabled),
/// this type makes no guarantees about memory consistency on a multi-core
/// system -- it only proves that interrupts are disabled on the current
/// core.
///
/// This type is neither [`Send`] nor [`Sync`]: a token minted on one core
/// must not be used to vouch for the interrupt state of another core.
pub struct InterruptsDisabledContext {
    _not_send_sync: PhantomData<*const ()>,
}

impl InterruptsDisabledContext {
    /// Mint a new [`InterruptsDisabledContext`] token.
    ///
    /// Prefer [`kernel::mint_interrupts_disabled_context!`](crate::mint_interrupts_disabled_context),
    /// which wraps this in the required `unsafe` block, over calling this
    /// directly.
    ///
    /// This is only intended to be called from the small set of places that
    /// actually establish that interrupts are disabled on this core: the
    /// architecture-level `with_interrupts_disabled` primitives, and trap or
    /// interrupt handlers on architectures where hardware disables
    /// interrupts automatically on entry (e.g. RISC-V clearing
    /// `mstatus.MIE`, or x86 executing through an IDT interrupt gate).
    ///
    /// # Safety
    ///
    /// The caller must guarantee that interrupts are actually disabled on
    /// this core for as long as any reference to the returned token is used.
    #[doc(hidden)]
    pub unsafe fn new_trusted() -> Self {
        InterruptsDisabledContext {
            _not_send_sync: PhantomData,
        }
    }
}
