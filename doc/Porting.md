Porting Tock
============

_This guide covers how to port Tock to a new platform. It is a work in progress. Comments and pull requests are appreciated!_

Overview
--------

At a high level, to port Tock to a new microcontroller, you need to write a new "chip" crate
and a new "board" crate (porting to a new board with an already supported microcontroller
just needs a new "board" crate). At a high level, the chip crate implements the traits found
in `kernel/src/hil` for controllers (e.g. the UART, GPIO, alarms, etc) and the board crate
stitches capsules together with the chip crates (e.g. assigning pins, baud rates, etc).


`chip` Crate
------------

The `chip` crate is specific to a specific microcontroller.


`board` Crate
-------------

The `board` crate, in `boards/src`, is specific to a physical hardware platform.
The board file essentially configures the kernel to support the specific hardware
setup. This includes instantiating drivers for sensors, mapping communication busses
to those sensors, configuring GPIO pins, etc.


### Loading Apps

You can create a custom [Makefile-app](https://github.com/helena-project/tock/blob/master/boards/imix/Makefile-app)
and include the commands needed to program an app and kernels on your board.

Common Pitfalls
---------------

- Make sure you are careful when setting up the board `main.rs` file. In particular,
it is important to ensure that all of the required `set_client` functions for capsules
are called so that callbacks are not lost. Forgetting these often results in the platform
looking like it doesn't do anything.

