//! Data structure for storing compile-time configuration options in the kernel.
//!
//! The rationale for configuration based on a `const` object is twofold.
//!
//! - In theory, Cargo features could be used for boolean-based configuration. However, these
//!   features are generally error-prone for non-trivial use cases. First, they are globally enabled
//!   as long as a dependency relationship requires a feature (even for other dependency
//!   relationships that do not want the feature). Second, code gated by a non-enabled feature
//!   isn't even type-checked by the compiler, and therefore we can end up with broken features due
//!   to refactoring code (if these features aren't tested during the refactoring), or to
//!   incompatible feature combinations.
//!
//! - Cargo features can only contain bits. On the other hand, a constant value can contain
//!   arbitrary types, which allow configuration based on integers, strings, or even more complex
//!   values.
//!
//! With a typed `const` configuration, all code paths are type-checked by the compiler - even
//! those that end up disabled - which greatly reduces the risks of breaking a feature or
//! combination of features because they are disabled in tests.
//!
//! In the meantime, after type-checking, the compiler can optimize away dead code by folding
//! constants throughout the code, so for example a boolean condition used in an `if` block will in
//! principle have a zero cost on the resulting binary - as if a Cargo feature was used instead.
//! Some simple experiments on generated Tock code have confirmed this zero cost in practice.

/// Data structure holding compile-time configuration options.
///
/// To change the configuration, modify the relevant values in the `CONFIG` constant object defined
/// at the end of this file.
crate struct Config {
    /// Whether the kernel should trace syscalls to the debug output.
    ///
    /// If enabled, the kernel will print a message in the debug output for each system call and
    /// callback, with details including the application ID, and system call or callback parameters.
    crate trace_syscalls: bool,
}

/// A unique instance of `Config` where compile-time configuration options are defined. These
/// options are available in the kernel crate to be used for relevant configuration.
crate const CONFIG: Config = Config {
    trace_syscalls: false,
};
