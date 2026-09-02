// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Typed proofs for the current context code is executing in.
//!
//! This module provides zero-sized marker tokens that signify the current
//! execution context. These are designed to provide compile-time assertions
//! for code which is sensitive to execution context.
//!
//! Enforcing context with comments is fragile and subject to user error. The
//! `unsafe` keyword is overloaded here, as context-sensitive operations are
//! not necessarily soundness concerns (though, being able to assert an
//! execution context is often necessary to justify `unsafe` operations).

use core::marker::PhantomData;
use core::panic::PanicInfo;

/// Proof that the current code is executing while the kernel is panicking.
///
/// `PanicInfo` has no public constructor: the language only ever creates one
/// to hand to the `#[panic_handler]`. Holding a `&PanicInfo` is therefore
/// unforgeable proof that a panic is genuinely underway, and this token
/// exists so that proof can be threaded through ordinary function
/// signatures instead of re-derived (or merely asserted in a doc comment)
/// at every layer of the panic-printing path.
///
/// This type is neither [`Send`] nor [`Sync`]: it should not be squirreled
/// away and used to vouch for a panic beyond the dynamic extent in which it
/// was minted.
pub struct PanicContext {
    _not_send_sync: PhantomData<*const ()>,
}

impl PanicContext {
    /// Mint a new [`PanicContext`] token.
    ///
    /// Prefer [`kernel::mint_panic_context!`](crate::mint_panic_context),
    /// which wraps this in the required `unsafe` block, over calling this
    /// directly.
    ///
    /// # Safety
    ///
    /// The caller must actually hold a `&PanicInfo`, i.e. must be executing
    /// as part of handling a genuine language panic.
    #[doc(hidden)]
    pub unsafe fn new_trusted(_panic_info: &PanicInfo) -> Self {
        PanicContext {
            _not_send_sync: PhantomData,
        }
    }
}
