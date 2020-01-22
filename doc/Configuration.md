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
  `Config` struct is defined in `kernel/src/config.rs`. To change the
  configuration, modify the values in the static `const` object defined in this
  file. To use the configuration, simply read the values. For example, to use a
  boolean configuration, just use an if statement: the fact that the
  configuration is `const` should allow the compiler to optimize away dead code
  (so that this configuration has zero cost), while still checking syntax and
  types.
