SiFive HiFive1 RISC-V Board
=================

- https://www.sifive.com/boards/hifive1

Arduino-compatible dev board for RISC-V. This is the first release of this
board ("Rev A01").

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

