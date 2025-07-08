// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Data structure for storing compile-time configuration options in the kernel.
//!
//! The rationale for configuration based on a `const` object is twofold.
//!
//! - In theory, Cargo features could be used for boolean-based configuration.
//!   However, these features are generally error-prone for non-trivial use
//!   cases. First, they are globally enabled as long as a dependency
//!   relationship requires a feature (even for other dependency relationships
//!   that do not want the feature). Second, code gated by a non-enabled feature
//!   isn't even type-checked by the compiler, and therefore we can end up with
//!   broken features due to refactoring code (if these features aren't tested
//!   during the refactoring), or to incompatible feature combinations.
//!
//! - Cargo features can only contain bits. On the other hand, a constant value
//!   can contain arbitrary types, which allow configuration based on integers,
//!   strings, or even more complex values.
//!
//! With a typed `const` configuration, all code paths are type-checked by the
//! compiler - even those that end up disabled - which greatly reduces the risks
//! of breaking a feature or combination of features because they are disabled
//! in tests.
//!
//! In the meantime, after type-checking, the compiler can optimize away dead
//! code by folding constants throughout the code, so for example a boolean
//! condition used in an `if` block will in principle have a zero cost on the
//! resulting binary - as if a Cargo feature was used instead. Some simple
//! experiments on generated Tock code have confirmed this zero cost in
//! practice.

/// Data structure holding compile-time configuration options.
///
/// To change the configuration, modify the relevant values in the `CONFIG`
/// constant object defined at the end of this file.
pub(crate) struct Config {
    /// Whether the kernel should trace syscalls to the debug output.
    ///
    /// If enabled, the kernel will print a message in the debug output for each
    /// system call and upcall, with details including the application ID, and
    /// system call or upcall parameters.
    pub(crate) trace_syscalls: bool,

    /// Whether the kernel should show debugging output when loading processes.
    ///
    /// If enabled, the kernel will show from which addresses processes are
    /// loaded in flash and into which SRAM addresses. This can be useful to
    /// debug whether the kernel could successfully load processes, and whether
    /// the allocated SRAM is as expected.
    pub(crate) debug_load_processes: bool,

    /// Whether the kernel should output additional debug information on panics.
    ///
    /// If enabled, the kernel will include implementations of
    /// `Process::print_full_process()` and `Process::print_memory_map()` that
    /// display the process's state in a human-readable form.
    // This config option is intended to allow for smaller kernel builds (in
    // terms of code size) where printing code is removed from the kernel
    // binary. Ideally, the compiler would automatically remove
    // printing/debugging functions if they are never called, but due to
    // limitations in Rust (as of Sep 2021) that does not happen if the
    // functions are part of a trait (see
    // https://github.com/tock/tock/issues/2594).
    //
    // Attempts to separate the printing/debugging code from the Process trait
    // have only been moderately successful (see
    // https://github.com/tock/tock/pull/2826 and
    // https://github.com/tock/tock/pull/2759). Until a more complete solution
    // is identified, using configuration constants is the most effective
    // option.
    pub(crate) debug_panics: bool,

    /// Whether the kernbel should output debug information when it is checking
    /// the cryptographic credentials of a userspace process. If enabled, the
    /// kernel will show which footers were found and why processes were started
    /// or not.
    // This config option is intended to provide some visibility into process
    // credentials checking, e.g., whether elf2tab and tockloader are generating
    // properly formatted footers.
    pub(crate) debug_process_credentials: bool,
}

/// A unique instance of `Config` where compile-time configuration options are
/// defined. These options are available in the kernel crate to be used for
/// relevant configuration. Notably, this is the only location in the Tock
/// kernel where we permit `#[cfg(x)]` to be used to configure code based on
/// Cargo features.
pub(crate) const CONFIG: Config = Config {
    trace_syscalls: cfg!(feature = "trace_syscalls"),
    debug_load_processes: cfg!(feature = "debug_load_processes"),
    debug_panics: !cfg!(feature = "no_debug_panics"),
    debug_process_credentials: cfg!(feature = "debug_process_credentials"),
};
