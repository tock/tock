Porting Tock
============

This guide covers how to port Tock to a new platform.

_It is a work in progress. Comments and pull requests are appreciated!_

Overview
--------

At a high level, to port Tock to a new microcontroller, you need to write a new
"chip" crate and a new "board" crate (porting to a new board with an already
supported microcontroller just needs a new "board" crate). The chip crate
implements the traits found in `kernel/src/hil` for controllers (e.g.  the
UART, GPIO, alarms, etc) and the board crate stitches capsules together with
the chip crates (e.g. assigning pins, baud rates, etc).


`arch` Crate
------------

Tock currently supports the ARM Cortex-M0, Cortex-M3, and Cortex M4, and the
riscv32imac architectures. There is not much
architecture-specific code in Tock, the list is pretty much:

 - Syscall entry/exit
 - Interrupt configuration
 - Top-half interrupt handlers
 - MPU configuration (if appropriate)
 - Power management configuration (if appropriate)

It would likely be fairly easy to port Tock to another ARM Cortex M
(specifically the M0+, M23, M4F, or M7) or another riscv32 variant. It will
probably be more work to port Tock to other architectures. While we aim to be
architecture agnostic, this has only been tested on a small number of
architectures.

If you are interested in porting Tock to a new architecture, it's likely best
to reach out to us via email or Slack before digging in too deep.


`chip` Crate
------------

The `chip` crate is specific to a particular microcontroller, but should attempt
to be general towards a family of microcontrollers. For example, support for the
`nRF58240` and `nRF58230` microcontrollers is shared in the `chips/nrf52` and
`chips/nrf5x` crates. This helps reduce duplicated code and simplifies adding
new specific microcontrollers.

The `chip` crate contains microcontroller-specific implementations of the
interfaces defined in `kernel/src/hil`.

Chips have a lot of features and Tock supports a large number of interfaces to
express them. Build up the implementation of a new chip incrementally. Get
reset and initialization code working. Set it up to run on the chip's default
clock and add a GPIO interface. That's a good point to put together a minimal
board that uses the chip and validate with an end-to-end userland application
that uses GPIOs.

Once you have something small like GPIOs working, it's a great time to open a
pull request to Tock. This lets others know about your efforts with this chip
and can hopefully attract additional support. It also is a chance to get some
feedback from the Tock core team before you have written too much code.

Moving forward, chips tend to break down into reasonable units of work.
Implement something like `kernel::hil::UART` for your chip, then submit a pull
request. Pick a new peripheral and repeat!


`board` Crate
-------------

The `board` crate, in `boards/src`, is specific to a physical hardware platform.
The board file essentially configures the kernel to support the specific
hardware setup. This includes instantiating drivers for sensors, mapping
communication buses to those sensors, configuring GPIO pins, etc.

Initializing a board is a bit obtuse and verbose right now, we have some plans
for how to improve the process, but in the short term the best bet is to start
from an existing board's `main.rs` file and adapt it. Initially, you will
likely want to delete most of the capsules and add them slowly as you get
things working.

### Board Support

In addition to kernel code, boards also require some support files. These
specify metadata such as the board name, how to load code onto the board, and
anything special that userland applications may need for this board.

#### `panic!`s (aka `io.rs`)

Each board must author a custom routine to handle `panic!`s. Most `panic!`
machinery is handled by the Tock kernel, but the board author must provide
some minimalist access to hardware interfaces, specifically LEDs and/or UART.

As a first step, it is simplest to just get LED-based `panic!` working. Have
your `panic!` handler set up a prominent LED and then call
[kernel::debug::panic_blink_forever](https://docs.tockos.org/kernel/debug/fn.panic_blink_forever.html).

If UART is available, the kernel is capable of printing a lot of very helpful
additional debugging information. However, as we are in a `panic!` situation,
it's important to strip this down to a minimalist implementation. In particular,
the supplied UART must be synchronous (note that this in contrast to the rest of
the kernel UART interfaces, which are all asynchronous). Usually implementing a
very simple `Writer` that simply writes one byte at a time directly to the UART
is easiest/best. It is not important that `panic!` UART writer be efficient.
You can then replace the call to
[kernel::debug::panic_blink_forever](https://docs.tockos.org/kernel/debug/fn.panic_blink_forever.html)
with a call to
[kernel::debug::panic](https://docs.tockos.org/kernel/debug/fn.panic.html).

For largely historical reasons, panic implementations for all boards live in
a file named `io.rs` adjacent to the board's `main.rs` file.

#### Board Cargo.toml, build.rs

Every board crate must author a top-level manifest, `Cargo.toml`. In general,
you can probably simply copy this from another board, modifying the board name
and author(s) as appropriate. Note that Tock also includes a build script,
`build.rs`, that you should also copy. The build script simply adds a
dependency on the kernel layout.

#### Board Makefile

There is a Makefile in the root of every board crate, at a minimum, the board
Makefile must include:

```make
# Makefile for building the tock kernel for the Hail platform

TARGET=thumbv7em-none-eabi      # Target triple
PLATFORM=hail                   # Board name here

include ../Makefile.common      # ../ assumes board lives in $(TOCK)/boards/<board>
```

Tock provides `boards/Makefile.common` that drives most of the build system.
In general, you should not need to
dig into this Makefile -- if something doesn't seem to be working, hop on slack
and ask.

##### Getting the built kernel onto a board

In addition to building the kernel, the board Makefile should include rules
for getting code onto the board. This will naturally be fairly board-specific,
but Tock does have two targets normally supplied:

  - _program_: For "plug-'n-plug" loading. Usually these are boards with a
    bootloader or some other support IC. The expectation is that during normal
    operation, a user could simply plug in a board and type `make program` to
    load code.
  - _flash_: For "more direct" loading. Usually this means that a JTAG or some
    equivalent interface is being used. Often it implies that external
    hardware is required, though some of the development kit boards have an
    integrated JTAG on-board, so external hardware is not a hard and fast
    rule.

If you don't support _program_ or _flash_, you should define an empty rule
that explains how to program the board:

```make
.PHONY: program
        echo "To program, run SPEICAL_COMMAND"
        exit 1
```

### Loading Apps

Ideally, [Tockloader](https://github.com/tock/tockloader) will support loading
apps on to your board (perhaps with some flags set to specific values). If that
is not the case, please create an issue on the Tockloader repo so we can update
the tool to support loading code onto your board.

### Common Pitfalls

- Make sure you are careful when setting up the board `main.rs` file. In
  particular, it is important to ensure that all of the required `set_client`
  functions for capsules are called so that callbacks are not lost. Forgetting
  these often results in the platform looking like it doesn't do anything.
