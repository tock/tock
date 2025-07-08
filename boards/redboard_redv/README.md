Redboard Red-V B RISC-V Board
==================================

- https://www.sparkfun.com/products/15594

Arduino-compatible dev board for RISC-V clone of the Hifive1.

Programming
-----------

Running `make flash` should load the kernel onto the board.

The kernel also assumes there is the default HiFive1 software bootloader running
on the chip.

Running in QEMU
---------------

The HiFive1 application can be run in the QEMU emulation platform for RISC-V, allowing quick and easy testing.

QEMU can be started with Tock using the following arguments (in Tock's top-level directory):

```bash
$ qemu-system-riscv32 -M sifive_e,revb=true -kernel $TOCK_ROOT/target/riscv32imac-unknown-none-elf/release/hifive1.elf  -nographic
```

Or with the `qemu` make target:

```bash
$ make qemu
```

QEMU can be started with Tock and a userspace app using the following arguments (in Tock's top-level directory):

```
qemu-system-riscv32 -M sifive_e,revb=true -kernel $TOCK_ROOT/target/riscv32imac-unknown-none-elf/release/hifive1.elf -device loader,file=./examples/hello.tbf,addr=0x20040000 -nographic
```
Or with the `qemu-app` make target:

```bash
$ make APP=/path/to/app.tbf qemu-app
```

The TBF must be compiled for the HiFive board which is, at the time of writing,
supported for Rust userland apps using libtock-rs. For example, you can build
the Hello World example app from the libtock-rs repository by running:

```
$ cd [LIBTOCK-RS-DIR]
$ make EXAMPLE=hello_world flash-hifive1
$ tar xf target/riscv32imac-unknown-none-elf/tab/hifive1/hello_world.tab
$ cd [TOCK_ROOT]/boards/hifive
$ make APP=[LIBTOCK-RS-DIR]/rv32imac.tbf qemu-app
```

Changes between Red-V and Hifive1-RevB
------------------

Hifive1 contains a BT module. The LED layout has changed. The boards seem identical otherwise.