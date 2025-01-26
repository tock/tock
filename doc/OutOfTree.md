Out of Tree Tock
================

This guide covers best practices for maintaining subsytems not in the main Tock
repository.

_It is a work in progress. Comments and pull requests are appreciated!_

<!-- npm i -g markdown-toc; markdown-toc -i OutOfTree.md -->

<!-- toc -->

- [Overview](#overview)
- [Structure](#structure)
- [Boards](#boards)
- [Everything Else](#everything-else)
- [Examples](#examples)

<!-- tocstop -->

Overview
--------

Tock aims to maintain a stable syscall ABI, but does not guarantee
stability of kernel interfaces. There are two primary channels to stay
abreast of Tock development:

- [tock-dev mailing list](https://lists.tockos.org/postorius/lists/): Any major
  Tock changes will be announced via this list. The list also supports general
  Tock development, however, it is relatively low traffic (<1 email/day on
  average).
- [Tock GitHub](https://github.com/tock/tock/): All Tock changes go through Pull
  Requests. Non-trivial changes will generally wait at least one week to merge
  to allow for feedback.

Finally, please don't hesitate to
[ask for help](https://github.com/tock/tock/#keep-up-to-date).


Structure
---------

We suggest generally mirroring the Tock directory structure within your
repository. This looks something like:

    $ tree .
    .
    ├── boards
    │  └── my_board
    │     ├── Cargo.toml
    │     ├── Makefile
    │     ├── layout.ld
    │     └── src
    │         └── main.rs
    ├── capsules
    │  ├── Cargo.toml
    │  └── src
    │     ├── my_radio.rs
    │     └── my_sensor.rs

Your code is then in individual Cargo crates. This is important because you can
then use Cargo to include dependencies (including the upstream Tock kernel
crates).


Boards
------

Boards are likely to start with a copy of an existing Tock board.

You can build a Tock board using Cargo commands (i.e., `cargo build --release`).
If you prefer to have the option to use Make you will need to copy the
`Makefile.common` makefile from the Tock repository and then include a Makefile
in your board's directory.

Your board's Makefile will need to include the primary Tock Makefile. We also
strongly suggest defining `program` and `flash` targets that specify how the
kernel is loaded onto your board.

```make
# Include Tock build rules
include ../Makefile.common

# Rules for loading via bootloader or other simple, direct connection
program:
  ...

# Rules for loading via JTAG or other external programmer
flash:
  ...
```

Your board's Cargo.toml will need to express how to find all the components that
your board uses. Most of these will likely be references to elements of Tock.

```toml
[package]
name = "my_board"
version = "0.1.0"
authors = ["Example Developer <developer@example.com>"]
build = "build.rs"

[profile.dev]
panic = "abort"
lto = true
opt-level = "z"
debug = true

[profile.release]
panic = "abort"
lto = true
opt-level = "z"
debug = true
codegen-units = 1

[dependencies]
cortexm4 = { git = "https://github.com/tock/tock", rev = "0c1b63b49" }
capsules = { git = "https://github.com/tock/tock", rev = "0c1b63b49" }
sam4l = { git = "https://github.com/tock/tock", rev = "0c1b63b49" }
kernel = { git = "https://github.com/tock/tock", rev = "0c1b63b49" }
my_drivers = { path = "../../my_drivers" }

[build-dependencies]
tock_build_scripts = { git = "https://github.com/tock/tock", rev = "0c1b63b49" }
```

You will need to create a `build.rs` file that simply calls into the generic
build script provided in the Tock repository:

```rust
// build.rs
//
fn main() {
    tock_build_scripts::default_linker_script();
}
```

You can use the default Tock linker script by creating a `layout.ld` file,
specifying the memory map, and then including the linker script
`tock_kernel_layout.ld`.

```ld
/* layout.ld */

...

INCLUDE tock_kernel_layout.ld
```

Everything Else
---------------

Custom chips, drivers, or other components should only require a Cargo.toml.

```toml
[package]
name = "my_drivers"
version = "0.1.0"
authors = ["Example Developer <developer@example.com>"]

[dependencies]
kernel = { git = "https://github.com/tock/tock", rev = "0c1b63b49" }
```


Examples
--------

- Several of the Tock core developers also work on the [Signpost
  project](https://github.com/lab11/signpost-software). The project includes
  [seven boards](https://github.com/lab11/signpost-software/tree/master/signpost/kernel/boards)
  that run Tock.
- New chips and boards often begin life out of tree. A current effort is [the
  STM32 port](https://github.com/tock/tock-stm32).
