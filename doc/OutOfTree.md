Out of Tree Tock
================

This guide covers best practices for maintaining subsytems not in the
Tock master repository.

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

  - [tock-dev mailing list](https://groups.google.com/forum/#!forum/tock-dev):
    Any major Tock changes will be announced via this list. The list
    also support general Tock development, however it is relatively low
    traffic (<1 email/day on average).
  - [Tock GitHub](https://github.com/tock/tock/): All Tock
    changes go through Pull Requests. Non-trivial changes will generally
    wait at least one week to merge to allow for feedback.

Finally, please don't hesitate to
[ask for help](https://kiwiirc.com/client/irc.freenode.net/tock).


Structure
---------

Usually it is easiest to keep a
[submodule](https://git-scm.com/docs/git-submodule) of Tock in your
project.

We then suggest generally mirroring the Tock directory structure,
something like:

    $ tree .
    .
    ├── boards
    │   └── my_board
    │       ├── Cargo.toml
    │       ├── Makefile
    │       └── src
    │           └── main.rs
    ├── my_drivers
    │   ├── Cargo.toml
    │   └── src
    │       ├── my_radio.rs
    │       └── my_sensor.rs
    └── tock                   # Where this is a git submodule
    │   ├── ...


Boards
------

Your board's Makefile will need to set a `PLATFORM` variable, specifying
the name of this platform, and include the primary Tock Makefile. We
also strongly suggest defining `program` and `flash` targets that
specify how the kernel is loaded onto your board.

  ```make
  PLATFORM = my_board

  # Include Tock build rules
  include ../../tock/boards/Makefile.common

  # Rules for loading via bootloader or other simple, direct conneciton
  program:
    ...

  # Rules for loading via JTAG or other external programmer
  flash:
    ...
  ```

Your board's Cargo.toml will need to express how to find all the
components that your board uses. Most of these will likely be references
to elements of Tock.

  ```toml
  [package]
  name = "my_board"
  version = "0.1.0"
  authors = ["Example Developer <developer@example.com>"]

  [profile.dev]
  panic = "abort"
  lto = true
  opt-level = 0
  debug = true

  [profile.release]
  panic = "abort"
  lto = true

  [dependencies]
  cortexm4 = { path = "../../tock/arch/cortex-m4" }
  capsules = { path = "../../tock/capsules" }
  sam4l = { path = "../../tock/chips/sam4l" }
  kernel = { path = "../../tock/kernel" }
  my_drivers = { path = "../../my_drivers" }
  ```



Everything Else
---------------

Custom chips, drivers, or other components should only require a
Cargo.toml.

  ```toml
  [package]
  name = "my_drivers"
  version = "0.1.0"
  authors = ["Example Developer <developer@example.com>"]

  [dependencies]
  kernel = { path = "../tock/kernel" }
  ```



Examples
--------

  - Several of the Tock core developers also work on the
    [Signpost project](https://github.com/lab11/signpost-software).
    The project includes
    [seven boards (and growing!)](https://github.com/lab11/signpost-software/tree/master/signpost/kernel/boards)
    that run Tock.
  - New chips and boards often begin life out of tree. A current effort
    is [the STM32 port](https://github.com/tock/tock-stm32).
