Cargo Configuration Files
=========================

This folder contains [cargo configuration
files](https://doc.rust-lang.org/cargo/reference/config.html) that Tock uses for
building the kernel for different boards. As different platforms use different
flags, each board can individually include these configuration files as needed.


Using a Cargo Configuration File
--------------------------------

To use one of these configurations in a board build file, the board's
`.cargo/config.toml` file must use the `include` key.

Example:

```toml
include = [
  "../../cargo/tock_flags.toml",
  "../../cargo/unstable_flags.toml",
]
```


Custom Target Specifications
-----------------------------

This folder also contains custom target specification JSON files for RISC-V
targets (`riscv32imc-unknown-none-elf.json`, `riscv32imac-unknown-none-elf.json`,
`riscv64imac-unknown-none-elf.json`). These are derived from the built-in rustc
target specs (obtainable via `rustc -Z unstable-options --print target-spec-json
--target <triple>`) with two modifications:

- `+relax` is appended to `features` to enable LLVM to emit relaxable
  relocations, allowing the linker to perform RISC-V linker relaxation
  (a significant code size optimization).
- The `metadata` block is stripped as it is informational only and not
  used by the build.

The reason for using custom specs rather than passing `-Ctarget-feature=+relax`
is that `+relax` is not yet stabilized in rustc's target feature API, so
specifying it via `-Ctarget-feature` emits an "unstable feature" warning that
cannot be suppressed. Embedding it in the base target spec avoids the warning.
Upstream stabilization is tracked at https://github.com/rust-lang/rust/issues/150257.

When updating the Rust toolchain (`rust-toolchain.toml`), run
`tools/repo-maintenance/update_rust_version.sh`, which will automatically
check these files against the new toolchain's target specs and update them
if needed.

Boards reference these specs by setting `target` to a relative path in their
`.cargo/config.toml`:

```toml
[build]
target = "../../cargo/riscv32imc-unknown-none-elf.json"
```
