Cargo Configuration Files
=========================

This folder contains [cargo configuration
files](https://doc.rust-lang.org/cargo/reference/config.html) that Tock uses for
building the kernel for different boards. As different platforms use different
flags, each board can individually include these configuration files as needed.


Using a Cargo Configuration File
--------------------------------

To use one of these configurations in a board build file, the board's
`.cargo/config.toml` file must use the `include` key. This currently (as of July
2024) requires the `config-include` nightly feature.

Example:

```toml
include = [
  "../../cargo/tock_flags.toml",
  "../../cargo/unstable_flags.toml",
]

[unstable]
config-include = true
```
