Configuration
=============

Because Tock is meant to run on various platforms (spanning multiple
architectures and various available peripherals), and with multiple use cases in
mind (for example, "production" vs. debug build with various levels of debugging
detail), Tock provides various configuration options so that each build can be
adapted to each use case.

In Tock, configuration follows some principles to avoid pitfalls of "ifdef"
conditional code (which can be tricky to test). This is currently done in two
ways.

- **Separation of the code into multiple packages.** Each level of abstraction
  (core kernel, CPU architecture, chip, board) has its own package, so that
  configuring a board is done by depending on the relevant chip and declaring
  the relevant drivers for peripherals avaialble on the board. You can see more
  details on the [compilation page](Compilation.md).

- **Custom kernel configuration.** To facilitate fine-grained configuration of
  the kernel (for example to enable tracing the syscalls to the debug output), a
  `Config` struct is defined in `kernel/src/config.rs`. The `Config` struct defines
  a collection of boolean values which can be imported throughout the kernel
  crate to configure the behavior of the kernel. The values of these booleans
  are determined by cargo features, which allows for individual boards to determine
  which features of the kernel crate are included without users having to manually
  modify the code in the kernel crate. Notably, Tock requires that these features *only*
  modify values in the global `CONFIG` constant, -- in general, features are not
  allowed to be used directly throughout the rest of the crate. Because of how
  feature unification works, all features are off-by-default, so if the Tock kernel
  wants a default value for a config option to be turning something on, the feature
  should be named appropriately -- e.g. the `no_debug_panics` feature is enabled to
  set the `debug_panics` config option to `false`.
  In order to enable any feature, modify the Cargo.toml in your board file as follows:
  ```toml
  [dependencies]
  # Turn off debug_panics, turn on trace_syscalls
  kernel = { path = "../../kernel", features = ["no_debug_panics", "trace_syscalls"]}
  ```
  These features should not be set from any crate other than the top-level board crate.
  If you prefer not to rely on the features, you can still directly modify the boolean
  config value in kernel/src/config.rs if you prefer -- this can be easier when rapidly
  debugging on an upstream board, for example.

  To use the configuration, simply read the values. For example, to use a
  boolean configuration, just use an if statement: the fact that the
  configuration is `const` should allow the compiler to optimize away dead code
  (so that this configuration has zero cost), while still checking syntax and
  types.
