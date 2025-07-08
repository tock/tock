Nested Boards
=============

Some hardware platforms are designed to be extended with additional hardware
(e.g., shields, plug-in sensors) or serve as the basis for other hardware
platforms. This describes how Tock boards can support this use case.

<!-- npm i -g markdown-toc; markdown-toc -i NestedBoards.md -->

<!-- toc -->

- [Overview](#overview)
  * [Rationale](#rationale)
- [Platform Board Setup](#platform-board-setup)

<!-- tocstop -->

Overview
--------

Some boards in Tock can serve as a platform for other boards the be built on top
of. Each board is its own crate, and the platform board becomes a dependency for
the dependent board. The platform board crate is both a library and a binary,
and the binary is implemented almost entirely within the library. The dependent
board also uses the platform board's library.

The dependent board "inherits" all of the platform board's functionality and
then can extend it by instantiating additional capsules.

### Rationale

This design attempts to meet the following goals for nested boards in Tock:

1. A "normal" Tock board can be a platform board with no changes.
2. The implementation is accomplished with only standard Rust/cargo mechanisms
   and without macros.
3. It is easy to reason about the functionality included in the dependent board.

The first goal promotes consistency among boards, easing the learning curve for
new users and reducing the overhead of maintaining the boards. It also means
that any board can become a platform board, as the board does not change. The
second goal helps promote code readability, as there are no custom scripts or
macros generating new code. The third goal reduces the mental complexity of
nested boards, as there are a limited ways that the platform board and dependent
board can interact, meaning not every imaginable configuration is possible.
Further, complete customizability is _not_ a goal, and two hardware platforms
which are related but sufficiently dissimilar should be implemented as
completely separate boards.

Unfortunately, we do not know of a way to implement nested boards while meeting
all of those goals. In particular, the limitations on how Rust binary crates
work mean they cannot be dependencies for other crates. As a result, our
approach for nested boards satisfies goals two and three while compromising
somewhat on goal one. The upside of nested boards is sufficient to offset the
drawbacks.

Platform Board Setup
--------------------

The platform board crate supports both a library and a binary. To do this, its
`Cargo.toml` file looks something like:

```toml
[package]
name = "nrf52840dk"

[lib]
name = "nrf52840dk_lib"

[[bin]]
name = "nrf52840dk"
path = "src/main.rs"
```

Then, the platform board crate includes two files: `main.rs` and `lib.rs`. The
lib.rs file should expose a function with a signature similar to:

```rust
pub unsafe fn start() -> (
    &'static kernel::Kernel,
    Platform,
    &'static nrf52840::chip::NRF52<'static, Nrf52840DefaultPeripherals<'static>>,
    &'static Nrf52DefaultPeripherals<'static>,
);
```

The main.rs file then calls start before loading processes and starting the
kernel loop.
