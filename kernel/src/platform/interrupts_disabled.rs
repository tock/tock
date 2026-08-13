// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! A typed proof that interrupts are currently disabled on this core.

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
pub struct InterruptsDisabled {
    _not_send_sync: PhantomData<*const ()>,
}

impl InterruptsDisabled {
    /// Mint a new [`InterruptsDisabled`] token.
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
        InterruptsDisabled {
            _not_send_sync: PhantomData,
        }
    }
}
