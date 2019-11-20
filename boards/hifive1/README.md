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

QEMU can be started with Tock using the following arguments:
```
qemu-system-riscv32 -M sifive_e -kernel boards/hifive1/target/riscv32imac-unknown-none-elf/release/hifive1.elf  -nographic
```

QEMU can be started with Tock and a userspace app using the following arguments:
```
qemu-system-riscv32 -M sifive_e -kernel boards/hifive1/target/riscv32imac-unknown-none-elf/release/hifive1.elf -device loader,file=./examples/hello.tbf,addr=0x20430000 -nographic
```
