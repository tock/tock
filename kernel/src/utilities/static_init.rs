// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Support for statically initializing objects in memory.

/// Allocates a statically-sized global array of memory and initializes the
/// memory for a particular data structure.
///
/// This macro creates the static buffer, ensures it is initialized to the
/// proper type, and then returns a `&'static mut` reference to it.
///
/// Note: Because this instantiates a static object, you generally cannot pass
/// a type with generic parameters. github.com/tock/tock/issues/2995 for detail.
///
/// # Safety
///
/// As this macro will write directly to a global area without acquiring a lock
/// or similar, calling this macro is inherently unsafe. The caller should take
/// care to never call the code that initializes this buffer twice, as doing so
/// will overwrite the value from first allocation without running its
/// destructor.
#[macro_export]
macro_rules! static_init {
    ($T:ty, $e:expr $(,)?) => {{
        let mut buf = $crate::static_buf!($T);
        buf.write($e)
    }};
}

/// An `#[inline(never)]` function that panics internally if the passed reference
/// is `true`. This function is intended for use within
/// the `static_buf!()` macro, which removes the size bloat of track_caller
/// saving the location of every single call to `static_init!()`.
/// If you hit this panic, you are either calling `static_buf!()` in
/// a loop or calling a function multiple times which internally
/// contains a call to `static_buf!()`. Typically, calls to
/// `static_buf!()` are hidden within calls to `static_init!()` or
/// component helper macros, so start your search there.
#[inline(never)]
pub fn static_buf_check_used(used: &mut bool) {
    // Check if this `BUF` has already been declared and initialized. If it
    // has, then this is a repeated `static_buf!()` call which is an error
    // as it will alias the same `BUF`.
    if *used {
        // panic, this buf has already been declared and initialized.
        // NOTE: To save 144 bytes of code size, use loop {} instead of this
        // panic.
        panic!("Error! Single static_buf!() called twice.");
    } else {
        // Otherwise, mark our uninitialized buffer as used.
        *used = true;
    }
}

/// Allocates a statically-sized global region of memory for data structures but
/// does not initialize the memory. Checks that the buffer is not aliased and is
/// only used once.
///
/// This macro creates the static buffer, and returns a
/// `StaticUninitializedBuffer` wrapper containing the buffer. The memory is
/// allocated, but it is guaranteed to be uninitialized inside of the wrapper.
///
/// Before the static buffer can be used it must be initialized. For example:
///
/// ```ignore
/// let mut static_buffer = static_buf!(T);
/// let static_reference: &'static mut T = static_buffer.initialize(T::new());
/// ```
///
/// Separating the creation of the static buffer into its own macro is not
/// strictly necessary, but it allows for more flexibility in Rust when boards
/// are initialized and the static structures are being created. Since creating
/// and initializing static buffers requires knowing the particular types (and
/// their sizes), writing shared initialization code (in components for example)
/// where the types are unknown since they vary across boards is difficult. By
/// splitting buffer creating from initialization, creating shared components is
/// possible.
#[macro_export]
macro_rules! static_buf {
    ($T:ty $(,)?) => {{
        // Statically allocate a read-write buffer for the value without
        // actually writing anything, as well as a flag to track if
        // this memory has been initialized yet.
        static mut BUF: (core::mem::MaybeUninit<$T>, bool) =
            (core::mem::MaybeUninit::uninit(), false);

        // To minimize the amount of code duplicated across every invocation
        // of this macro, all of the logic for checking if the buffer has been
        // used is contained within the static_buf_check_used function,
        // which panics if the passed boolean has been used and sets the
        // boolean to true otherwise.
        $crate::utilities::static_init::static_buf_check_used(&mut BUF.1);

        // If we get to this point we can wrap our buffer to be eventually
        // initialized.
        &mut BUF.0
    }};
}
