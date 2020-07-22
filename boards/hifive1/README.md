SiFive HiFive1 Rev B RISC-V Board
==================================

- https://www.sifive.com/boards/hifive1-rev-b

Arduino-compatible dev board for RISC-V. This is the second release of this
board ("Rev B").

Programming
-----------

Running `make flash` should load the kernel onto the board. You will need a
relatively new (i.e. from git) version of OpenOCD.

The kernel also assumes there is the default HiFive1 software bootloader running
on the chip.

Running in QEMU
---------------

The HiFive1 application can be run in the QEMU emulation platform, allowing quick and easy testing.

QEMU can be started with Tock using the following arguments (in Tock's top-level directory):

```bash
$ qemu-system-riscv32 -M sifive_e -kernel $TOCK_ROOT/target/riscv32imac-unknown-none-elf/release/hifive1.elf  -nographic
```

Or with the `qemu` make target:

```bash
$ make qemu
```

QEMU can be started with Tock and a userspace app using the following arguments (in Tock's top-level directory):

```
qemu-system-riscv32 -M sifive_e -kernel $TOCK_ROOT/target/riscv32imac-unknown-none-elf/release/hifive1.elf -device loader,file=./examples/hello.tbf,addr=0x20430000 -nographic
```
Or with the `qemu-app` make target:

```bash
$ make APP=/path/to/app.tbf qemu-app
```

The TBF must be compiled for the HiFive board which is, at the time of writing,
supported for Rust userland apps using libtock-rs. For example, you can build
the Hello World exmple app from the libtock-rs repository by running:

```
$ cd [LIBTOCK-RS-DIR]
$ make flash-hifive1
$ tar xf target/riscv32imac-unknown-none-elf/tab/hifive1/hello_world.tab
$ cd [TOCK_ROOT]/boards/hifive
$ make APP=[LIBTOCK-RS-DIR]/rv32imac.tbf qemu-app
```

HiFive1 Revision A
------------------

Tock has dropped support for the older ("Rev A") version of this board. Since
that version of the hardware is no longer being produced, Tock has decided to no
longer maintain the older board file. If would like to run Tock on the rev A
version, you should use the [version 1.5
release](https://github.com/tock/tock/releases/tag/release-1.5) of Tock.
