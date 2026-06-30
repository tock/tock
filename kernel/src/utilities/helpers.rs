// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Helper functions and macros.
//!
//! These are various utility functions and macros that are useful throughout
//! the Tock kernel and are provided here for convenience.
//!
//! The macros are exported through the top level of the `kernel` crate.

/// Create an object with the given capabilities.
///
/// ```
/// # use kernel::capabilities::{ProcessManagementCapability, MemoryAllocationCapability};
/// # use kernel::create_capability;
/// let process_mgmt_cap = create_capability!(ProcessManagementCapability);
/// let unified_cap = create_capability!(ProcessManagementCapability, MemoryAllocationCapability);
/// ```
///
/// This helper macro cannot be called from `#![forbid(unsafe_code)]` crates,
/// and is used by trusted code to generate a capability that it can either use
/// or pass to another module.
///
/// # Safety
///
/// This macro can only be used in a context that is allowed to use
/// `unsafe`. Specifically, an internal `allow(unsafe_code)` directive
/// will conflict with any `forbid(unsafe_code)` at the crate or block
/// level.
///
/// ```compile_fail
/// # use kernel::capabilities::ProcessManagementCapability;
/// # use kernel::create_capability;
/// #[forbid(unsafe_code)]
/// fn untrusted_fn() {
///     let process_mgmt_cap = create_capability!(ProcessManagementCapability);
/// }
/// ```
#[macro_export]
macro_rules! create_capability {
    ($($T:ty),+) => {{
        #[allow(unsafe_code)]
        struct Cap(());
        $(
            unsafe impl $T for Cap {}
        )*
        Cap(())
    }};
}

/// Declare a named struct that implements the given capability traits.
///
/// Unlike [`create_capability!`], this macro creates a named type that can be
/// used as a generic parameter (e.g., in component static macros). Use this
/// when you need to name the capability type, such as when passing it to a
/// component's static buffer macro.
///
/// ```
/// # use kernel::capabilities::ProcessManagementCapability;
/// # use kernel::declare_capability;
/// declare_capability!(MyCapability: ProcessManagementCapability);
/// let cap = MyCapability;
/// ```
///
/// This helper macro cannot be called from `#![forbid(unsafe_code)]` crates,
/// and is used by trusted code to generate a capability type.
///
/// # Safety
///
/// This macro can only be used in a context that is allowed to use
/// `unsafe`. Specifically, an internal `allow(unsafe_code)` directive
/// will conflict with any `forbid(unsafe_code)` at the crate or block
/// level.
#[macro_export]
macro_rules! declare_capability {
    (pub $name:ident: $($T:ty),+) => {
        pub struct $name;
        $(
            #[allow(unsafe_code)]
            unsafe impl $T for $name {}
        )*
    };
    ($name:ident: $($T:ty),+) => {
        struct $name;
        $(
            #[allow(unsafe_code)]
            unsafe impl $T for $name {}
        )*
    };
}

/// Count the number of passed expressions.
///
/// Useful for constructing variable sized arrays in other macros.
/// Taken from the Little Book of Rust Macros.
///
/// ```ignore
/// use kernel:count_expressions;
///
/// let count: usize = count_expressions!(1+2, 3+4);
/// ```
#[macro_export]
macro_rules! count_expressions {
    () => (0usize);
    ($head:expr $(,)?) => (1usize);
    ($head:expr, $($tail:expr),* $(,)?) => (1usize + count_expressions!($($tail),*));
}

/// Executables must specify their stack size by using the `stack_size!` macro.
///
/// It takes a single argument, the desired stack size in bytes. Example:
/// ```
/// kernel::stack_size!{0x1000}
/// ```
// stack_size works by putting a symbol equal to the size of the stack in the
// .stack_buffer section. The linker script uses the .stack_buffer section to
// size the stack.
#[macro_export]
macro_rules! stack_size {
    {$size:expr} => {
        /// Size to allocate for the stack.
        ///
        /// This creates a static buffer inserted into the `.stack_buffer`
        /// section that the linker script picks up and places at the correct
        /// location in RAM.
        ///
        /// When compiling for a macOS host, this section attribute is elided as
        /// it is incompatible with Mach-O objects and yields the following
        /// error: `mach-o section specifier requires a segment and section
        /// separated by a comma`.
        #[cfg_attr(not(target_os = "macos"), unsafe(link_section = ".stack_buffer"))]
        #[unsafe(no_mangle)]
        static mut STACK_MEMORY: [u8; $size] = [0; $size];
    }
}

/// Initialize all fields of a `MaybeUninit<T>` struct.
///
/// Use this macro to guarantee that all fields in `T` are initialized.
///
/// Instead of the normal code, which would look like this:
///
/// ```rust,ignore
/// let process_uninit: &mut MaybeUninit<ProcessStandard<C, D>> =
///     unsafe { &mut *process_struct_memory_location };
///
/// let process_uptr = process_uninit.as_mut_ptr();
///
/// unsafe {
///     (&raw mut (*process_uptr).kernel).write(kernel);
///     (&raw mut (*process_uptr).chip).write(chip);
///     ...
/// }
/// ```
///
/// which has the limitation that if not every field is set, then this code is
/// unsafe. With this macro, the code looks like this:
///
/// ```rust,ignore
/// let process_uninit: &mut MaybeUninit<ProcessStandard<C, D>> =
///     unsafe { &mut *process_struct_memory_location };
///
/// unsafe {
///     init_uninit_struct!(process_uninit => ProcessStandard<C, D> {
///         kernel: kernel,
///         chip: chip,
///         ...
///     )};
/// }
/// ```
///
/// If not every field is set then there will be a compiler error.
///
/// # Implementation
///
/// This macro creates a fake implementation of the struct `T` and then
/// populates all of the provided fields. This allows the normal Rust compiler
/// to check that all fields are actually set.
///
/// The generated code looks something like this:
///
/// ```rust,ignore
/// #[allow(unreachable_code)]
/// if false {
///     let _: ProcessStandard<C, D> = ProcessStandard {
///         kernel: ::core::panicking::panic("not yet implemented"),
///         chip: ::core::panicking::panic("not yet implemented"),
///         ...
///     };
/// }
/// ```
///
/// Using `todo!()` avoids any issues with the borrow checker. However, using
/// `todo!()` causes the `diverging_sub_expression` clippy lint to trigger.
/// Since we are doing this intentionally, we manually ignore the
/// `diverging_sub_expression` lint.
///
/// # Safety
///
/// The struct to be initialized needs to be correctly allocated and all fields
/// need to be correctly aligned.
#[macro_export]
macro_rules! init_uninit_struct {
    (@field $field:ident : $value:expr) => {
        $value
    };

    (@field $field:ident) => {
        $field
    };

    ( $s: expr => $t: ident < $($gen:tt),* > { $( $field:ident : $value:expr ),* $(,)? } ) => {
        #[allow(unreachable_code)]
        #[allow(clippy::diverging_sub_expression)]
        if false {
            let _: $t<$($gen),*> = $t {
                $( $field: todo!() ),*
            };
        }

        let s = $s.as_mut_ptr();
        $(
            (&raw mut (*s).$field).write(init_uninit_struct!(@field $field : $value));
        )*
    };
}

/// Compute a POSIX-style CRC32 checksum of a slice.
///
/// Online calculator: <https://crccalc.com/>
pub fn crc32_posix(b: &[u8]) -> u32 {
    let mut crc: u32 = 0;

    for c in b {
        crc ^= (*c as u32) << 24;

        for _i in 0..8 {
            if crc & (0b1 << 31) > 0 {
                crc = (crc << 1) ^ 0x04c11db7;
            } else {
                crc <<= 1;
            }
        }
    }
    !crc
}
