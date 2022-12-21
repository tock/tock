QEMU RISC-V 32 bit `virt` Platform
==================================

This board crate targets the QEMU RISC-V 32 bit `virt` platform. While this
platform should be generally stable, the board [`Makefile`](./Makefile)
indicates a specific version of QEMU which this board has been tested against.

While this target does not feature many peripherals for now, it represents a
stable QEMU target for using Tock in a virtualized RISC-V environment. This can
be useful for CI and other purposes. In the future, this target can be extended
to support VirtIO peripherals.

Known issues: Application Support
---------------------------------

Upstream QEMU currently contains a bug which makes it impossible to run
userspace applications due to issues with the enforcement of memory protection
(PMP). Tock issue [#3316](https://github.com/tock/tock/issues/3316) tracks these
developments.

Running QEMU
------------

To run the board in QEMU, `qemu-system-riscv32` must be started with the
`-machine virt` argument. The Tock kernel expects to be loaded as the BIOS by
passing `-bios $TOCK_KERNEL.bin`, such that it runs in RISC-V machine mode and
has full control over the virtual board. `-nographic` can be used to suppress
QEMU's graphical interface.

The [`Makefile`] further contains two targets for running QEMU with a standalone
kernel, or with a single app. These can be executed as

```
tock/boards/qemu_rv32_virt $ make run
    Finished release [optimized + debuginfo] target(s) in 0.05s
   text    data     bss     dec     hex filename
  64880      12   11248   76140   1296c tock/target/riscv32imac-unknown-none-elf/release/qemu_rv32_virt
a9de4df9486d724e6bf6a3423af669903dfd2bd1fd65c1dd867ddf9d7bcbec9b  tock/target/riscv32imac-unknown-none-elf/release/qemu_rv32_virt.bin

Running QEMU emulator version 7.0.0 (tested: 7.0.0) with
  - kernel tock/target/riscv32imac-unknown-none-elf/release/qemu_rv32_virt.bin
To exit type C-a x

qemu-system-riscv32 \
  -machine virt \
  -bios tock/target/riscv32imac-unknown-none-elf/release/qemu_rv32_virt.bin \
  -global virtio-mmio.force-legacy=false \
  -device virtio-rng-device \
  -nographic
QEMU RISC-V 32-bit "virt" machine, initialization complete.
Entering main loop.
```

and

```
tock/boards/qemu_rv32_virt $ make run-app APP=$PATH_TO_APP.tbf
```

respectively.

