# Tock OS Course Part 1: Getting your environment set up

The goal of this part of the course is to make sure you have a working
development environment for Tock.

During this you will:

...

## 1. Presentation: Tock's goals, architecture and components (10 min)

## 2. Check your understanding (10 min)

1. What kinds of binaries exist on a Tock board? Hint: There are three, and
   only two can be programmed using `tockloader`.

2. What are the differences between capsules and processes? What performance
   and memory overhead does each entail? Why would you choose to write
   something as a process instead of a capsule and vice versa?

## 3. Compile and flash the kernel (10 min)
To build the kernel, type `make` in the root directory, or in `boards/hail/`.

`cd tock`
`make`

or

`cd tock/boards/hail`
`make`

The root Makefile uses the `TOCK_BOARD` environment variable to determine the
board and architecture to build the kernel for. All calls are then routed to
that board's specific Makefile. By default, the root Makefile builds for the
Hail platform. To build for another board, change the `TOCK_BOARD` environment
variable to indicate another board: 

`export TOCK_BOARD=imix`

To flash the kernel, run `make program` in the `boards/hail/` directory.

`cd tock/boards/hail`
`make program`

## 4. Customize, compile and flash the `ble-env-sense` service (10 min)

## 5. (Optional) Familiarize yourself with `tockloader` commands (10 min)
