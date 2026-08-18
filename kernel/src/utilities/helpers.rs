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
            // SAFETY: `$T`'s safety contract only requires that capabilities
            // be created in a context permitted to use `unsafe`. The
            // `#[allow(unsafe_code)]` above discharges that: it conflicts
            // with any `forbid(unsafe_code)` at the crate or block level, so
            // this expansion only compiles where `unsafe` is permitted.
            unsafe impl $T for Cap {}
        )*
        Cap(())
    }};
}

/// Create a struct that implements the given capability traits.
///
/// Unlike [`create_capability!`], this macro creates an instance of a struct
/// with a visibly named type. Use this when you need to name the capability
/// type, such as when passing it to a component's static buffer macro.
///
/// # Usage Example
///
/// ```ignore
/// # use kernel::capabilities::ProcessManagementCapability;
/// # use kernel::create_typed_capability;
///
/// create_typed_capability!(proc_cap, ProcCapForManager: ProcessManagementCapability);
///
/// let manager = components::manager::ManagerComponent::new(
///     board_kernel,
///     mux_alarm,
///     proc_cap,
/// )
/// .finalize(components::manager_component_static!(
///     AlarmHw,
///     ProcCapForManager,
/// ));
/// ```
///
/// # Difference from `create_capability!()`
///
/// `create_typed_capability!()` creates a visibly named type, whereas
/// [`create_capability!`] does not. If possible, use [`create_capability!`].
/// However, for components that need to name the type for static constructors,
/// use `create_typed_capability!()` to create the struct that is used in that
/// macro expansion and in the component `finalize()` method.
///
/// # Supporting Multiple Capabilities
///
/// Some type signatures require multiple capabilities. This macro supports
/// declaring multiple capabilities. For example:
///
/// ```
/// kernel::create_typed_capability!(process_console_cap, ProcessConsoleCap:
///     kernel::capabilities::ProcessManagementCapability,
///     kernel::capabilities::ProcessStartCapability
/// );
/// ```
///
/// # Restrictions
///
/// This helper macro cannot be called from `#![forbid(unsafe_code)]` crates,
/// and is used by trusted code to generate a capability type.
///
/// # Safety
///
/// This macro can only be used in a context that is allowed to use `unsafe`.
/// Specifically, an internal `allow(unsafe_code)` directive will conflict with
/// any `forbid(unsafe_code)` at the crate or block level.
#[macro_export]
macro_rules! create_typed_capability {
    ($var:ident, $type:ident: $($T:path),+ $(,)?) => {
        // In plain macros, there's not an ergonomic way to construct a new
        // identifier for the module name, but we do need unique module name.
        // Modules and structs share the same namespace, so we can't use
        // `mod $type` here, but _variables_ exist in a separate namespace,
        // so we can use the variable name as a module name.
        //
        // The implication of this design is that something in the same scope
        // of this macro _could_ write
        //     unsafe { $type::__new_only_for_use_in_macro() }
        // to mint another capability, but doing so requires both an unsafe
        // block and use of an obviously wrong method name.
        mod $var {
            // Bring the invocation site's surrounding module into scope, so
            // `$T` arguments can be given as bare names brought in by a
            // local `use`, not just fully-qualified paths. This is often
            // unused (most call sites already use fully-qualified paths),
            // hence the `allow`.
            #[allow(unused_imports)]
            use super::*;

            // Prevent creation by anything external.
            pub struct $type(());

            // Implement specified capabilities.
            $(
                // SAFETY: `$T`'s safety contract only requires that
                // capabilities be created in a context permitted to use
                // `unsafe`. The `allow(unsafe_code)` below discharges that:
                // it conflicts with any `forbid(unsafe_code)` at the crate
                // or block level.
                #[allow(unsafe_code)]
                unsafe impl $T for $type {}
            )*

            impl $type {
                // Mark the constructor unsafe to prevent non-obvious use.
                #[doc(hidden)]
                pub unsafe fn __new_only_for_use_in_macro() -> Self {
                    $type(())
                }
            }
        }
        // Import the type name to the current scope
        use $var::$type;

        // Create one instance of the capability.
        //
        // SAFETY: this is the macro's own single, controlled call to the
        // gated constructor above.
        let $var = unsafe { $type::__new_only_for_use_in_macro() };
    };
}

/// Define a struct type that implements the given capability traits.
///
/// Unlike [`create_capability!`] or [`create_typed_capability!`], this macro
/// only defines a visibly named type, it does not create any instances.
/// Use this when you need to name the capability type across scope boundaries.
///
/// Use [`mint_defined_capability!`] to create an instance. Note: You can only
/// mint capabilities in the module that defines this type, or in a descendant
/// of that module.
///
/// This macro is intended to be used at module level scope (i.e. not in a
/// code block), since minting requires naming the type across a scope
/// boundary.
///
/// # Usage Example
///
/// ```
/// # use kernel::capabilities::ProcessManagementCapability;
/// # use kernel::define_capability_type;
///
/// define_capability_type!(ProcCapForManager: ProcessManagementCapability);
/// ```
///
/// # Restrictions
///
/// This helper macro cannot be called from `#![forbid(unsafe_code)]` crates,
/// and is used by trusted code to define a capability type.
///
/// # Safety
///
/// This macro can only be used in a context that is allowed to use `unsafe`.
/// Specifically, an internal `allow(unsafe_code)` directive will conflict with
/// any `forbid(unsafe_code)` at the crate or block level.
#[macro_export]
macro_rules! define_capability_type {
    ($type:ident: $($T:path),+ $(,)?) => {
        /// Publicly nameable type for capabilities.
        pub struct $type {
            // Make it obvious that this should not be directly constructed.
            _not_for_public_construction: (),
        }

        // Implement specified capabilities.
        $(
            // SAFETY: `$T`'s safety contract only requires that capabilities
            // be created in a context permitted to use `unsafe`. The
            // `allow(unsafe_code)` below discharges that: it conflicts with
            // any `forbid(unsafe_code)` at the crate or block level.
            #[allow(unsafe_code)]
            unsafe impl $T for $type {}
        )*

        impl $type {
            // Mark the constructor unsafe to prevent non-obvious use.
            //
            // Must not be `pub` to prevent external users. Note this is not
            // airtight: anything in the same module (or a descendant module)
            // as this type definition _could_ write
            //     unsafe { $type::__macro_only_capability_creator() }
            // to mint another capability, but doing so requires both an
            // unsafe block and use of an obviously-not-public method name.
            #[doc(hidden)]
            unsafe fn __macro_only_capability_creator() -> Self {
                Self { _not_for_public_construction: () }
            }
        }
    };
}

/// Create an instance of a capability type defined by [`define_capability_type!`].
///
/// Use this to create one instance of a capability previously declared via
/// [`define_capability_type`]. Note: You can only mint capabilities in the
/// module that defines this type, or in a descendant of that module.
///
/// # Usage Example
///
/// ```ignore
/// # use kernel::mint_defined_capability;
///
/// let proc_cap = unsafe { mint_defined_capability!(ProcCapForManager) };
/// ```
///
/// # Restrictions
///
/// This macro invokes an `unsafe` function. Callers must wrap use of this
/// macro in `unsafe` to assert they are trusted to mint a capability.
// Note: Ultimately, this is just a wrapper around a function call, and does
// not _need_ to be in a macro per se. However, minting capabilities is a
// highly sensitive operation, and encapsulating this in a macro helps code
// review spot this directly as the creation of a capability, which may be less
// apparent when just constructing an arbitrarily named struct.
#[macro_export]
macro_rules! mint_defined_capability {
    ($type:ident) => {
        $type::__macro_only_capability_creator()
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
        /// This section attribute is only applied when targeting bare-metal
        /// (`target_os = "none"`). Host builds (e.g. tests, clippy, doc) use
        /// object formats (Mach-O, PE, ...) that reject a bare section name
        /// like this, yielding errors such as: `mach-o section specifier
        /// requires a segment and section separated by a comma`.
        #[cfg_attr(target_os = "none", unsafe(link_section = ".stack_buffer"))]
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
///     });
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

/// Reinterpret a `&mut usize` as a `&mut u32` (32-bit targets) or `&mut u64`
/// (64-bit targets).
///
/// This is useful for architectures (e.g. RISC-V) that store syscall/upcall
/// register arguments as `usize` so that a single struct definition can serve
/// both 32-bit and 64-bit targets, but need to pass register-width-specific
/// (`u32`/`u64`) references into TRD-defined encode/decode functions.
///
/// This is a plain function, not a macro, on purpose: the conversion is only
/// sound behind a real function-call boundary.
#[cfg(target_pointer_width = "32")]
pub fn usize_as_native_mut(val: &mut usize) -> &mut u32 {
    let ptr = core::ptr::from_mut::<usize>(val);
    // SAFETY: We are guaranteed to be on a 32-bit platform, so a `usize` is
    // the same size and alignment as a `u32`, and `val` is guaranteed to be
    // initialized to some value.
    unsafe { &mut *ptr.cast::<u32>() }
}

/// Reinterpret a `&mut usize` as a `&mut u32` (32-bit targets) or `&mut u64`
/// (64-bit targets).
///
/// This is a plain function, not a macro, on purpose: the conversion is only
/// sound behind a real function-call boundary.
#[cfg(target_pointer_width = "64")]
pub fn usize_as_native_mut(val: &mut usize) -> &mut u64 {
    let ptr = core::ptr::from_mut::<usize>(val);
    // SAFETY: We are guaranteed to be on a 64-bit platform, so a `usize` is
    // the same size and alignment as a `u64`, and `val` is guaranteed to be
    // initialized to some value.
    unsafe { &mut *ptr.cast::<u64>() }
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
