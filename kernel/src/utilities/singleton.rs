// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

/// Internal helper function for [`only_once!()`](crate::only_once).
///
/// This must be public to work within the macro but should never be used
/// directly.
///
/// This is a `#[inline(never)]` function that panics internally if the passed
/// reference is `true`. This function is intended for use within the
/// [`only_once!()`](crate::only_once) macro to detect multiple uses of the
/// macro on the same name.
///
/// This function is implemented separately without inlining to removes the size
/// bloat of track_caller saving the location of every single call to
/// [`only_once!()`](crate::only_once).
#[inline(never)]
pub unsafe fn only_once_check_used(used: &mut bool) {
    // Check if this bool has already been declared and initialized. If it
    // has, then this is a call which is an error.
    if *used {
        // panic, this buf has already been declared and initialized.
        // NOTE: To save 144 bytes of code size, use loop {} instead of this
        // panic.
        panic!("Error! only_once!() called twice.");
    } else {
        // Otherwise, mark our uninitialized buffer as used.
        *used = true;
    }
}

/// Helper macro to ensure an object is only created once.
///
/// Panics if the same object is created twice.
///
/// ```ignore
///
/// /// This DMA-enabled peripheral manager MUST only be created once.
/// struct PeripheralManagerWithDma;
///
/// impl PeripheralManagerWithDma {
///     fn new() -> Self {
///     	// Ensure this will panic if constructed twice.
///     	kernel::only_once!(PERIPHERAL_MANAGER_WITH_DMA);
///     	Self {}
///     }
/// }
/// ```
#[macro_export]
macro_rules! only_once {
    ($N:ident $(,)?) => {{
        // Statically allocate a read-write buffer for the value without
        // actually writing anything, as well as a flag to track if
        // this memory has been initialized yet.
        static mut $N: bool = false;

        // To minimize the amount of code duplicated across every invocation
        // of this macro, all of the logic for checking if the buffer has been
        // used is contained within the static_buf_check_used function,
        // which panics if the passed boolean has been used and sets the
        // boolean to true otherwise.
        unsafe { $crate::utilities::singleton::only_once_check_used(&mut $N) };
    }};
}
