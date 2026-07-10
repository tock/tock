// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

/// Internal helper function for [`only_once!()`](crate::only_once).
///
/// This must be public to work within the macro but should never be used
/// directly.
///
/// This is a `#[inline(never)]` function that panics if the atomic flag has
/// already been set. This function is intended for use within the
/// [`only_once!()`](crate::only_once) macro to detect multiple uses of the
/// macro on the same name.
///
/// This function is implemented separately without inlining to remove the size
/// bloat of track_caller saving the location of every single call to
/// [`only_once!()`](crate::only_once).
#[cfg(target_has_atomic = "ptr")]
#[inline(never)]
pub fn only_once_check_used(used: &core::sync::atomic::AtomicUsize) {
    // swap(1) returns the old value; if it was already 1, this is a second call.
    if used.swap(1, core::sync::atomic::Ordering::Relaxed) != 0 {
        panic!("Error! only_once!() called twice.");
    }
}

/// Helper macro to ensure an object is only created once.
///
/// Panics if the same object is created twice.
///
/// This only exists for targets with atomic pointer-sized operations.
/// Therefore, it can only be used in chip and board crates that can guarantee
/// the atomic support exists.
///
/// ```ignore
///
/// /// This DMA-enabled peripheral manager MUST only be created once.
/// struct PeripheralManagerWithDma;
///
/// impl PeripheralManagerWithDma {
///     fn new() -> Self {
///         // Ensure this will panic if constructed twice.
///         kernel::only_once!(PERIPHERAL_MANAGER_WITH_DMA);
///         Self {}
///     }
/// }
/// ```
#[cfg(target_has_atomic = "ptr")]
#[macro_export]
macro_rules! only_once {
    ($N:ident $(,)?) => {{
        static $N: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);
        $crate::utilities::singleton::only_once_check_used(&$N);
    }};
}
