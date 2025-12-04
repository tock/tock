// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use x86::InterruptPoller;

use super::pic;

/// Handler for external interrupts.
///
/// This function is called by the [`x86`] crate to handle interrupts from external devices.
/// It calls [`InterruptPoller::set_pending`] to mark the interrupt as pending, masks the specific
/// interrupt, then issues an EOI message to the system interrupt controller so that subsequent
/// interrupts can be delivered.
///
/// ## Safety
///
/// This function must only be called when handling an interrupt. It should _never_ be called by
/// other Rust code.
// `allow(unsupported_calling_conventions)`: cdecl is not valid when testing
// this code on an x86_64 machine. This avoids a warning until a more permanent
// fix is decided. See: https://github.com/tock/tock/pull/4662
#[allow(unsupported_calling_conventions)]
#[no_mangle]
unsafe extern "cdecl" fn handle_external_interrupt(num: u32) {
    unsafe {
        InterruptPoller::set_pending(num);
        pic::mask(num);
        pic::eoi(num);
    }
}
