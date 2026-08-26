QEMU ARM MPS2 AN385 (Cortex-M3) Platform
=========================================

This board crate targets QEMU's `mps2-an385` machine: an emulation of ARM's
own "MPS2 + AN385" Cortex-M System Design Kit (CMSDK) reference platform,
not a real vendor chip. It is the ARM counterpart to `qemu_rv32_virt` /
`qemu_rv64_virt`: a stable, purely virtual target for exercising the
Cortex-M port under QEMU, useful for CI and kernel development without
access to real ARM hardware.

Currently supported peripherals:

- One CMSDK APB UART (of the five present on the machine), used as the
  console/debug UART.
- One CMSDK APB Timer, backing the kernel's `Alarm`/`Time` HIL.
- The `FPGAIO` block's `LED0` register, exposing the machine's two
  simulated LEDs.
- One PL022 SPI controller (the "Shield0" instance), run in hardware
  loopback mode — see the note below.

Not supported, and not planned for this machine specifically:

- **GPIO.** QEMU emulates all four CMSDK AHB GPIO banks on this machine (and
  on every other MPS2/MPS2-TZ machine, including the Cortex-M33 `an505`/
  `an521` images) as inert stubs: writes are discarded and reads always
  return 0 (see QEMU's `hw/arm/mps2.c`, `create_unimplemented_device(...,
  "cmsdk-ahb-gpio", ...)`). There is no way to observe pin state changes
  under this QEMU machine, so this board does not implement a GPIO capsule.
  LEDs are wired to the separate, genuinely-emulated `FPGAIO` register
  instead (see `chips/qemu_arm_mps2_chip/src/led.rs`).
- I2C and the machine's LAN9118 Ethernet controller: present on the memory
  map but not driven by this chip crate.

**SPI note**: none of the machine's five PL022 instances have an SSI slave
device attached in QEMU, so a non-loopback transfer just reads back
whatever the empty bus's default is, not meaningful data. The driver
therefore always enables `CR1.LBM` (loopback) — see
`chips/qemu_arm_mps2_chip/src/spi.rs`'s module docs. Chip select is a
zero-sized placeholder for the same reason GPIO is unavailable: there's no
functional GPIO pin to toggle for it, and no real device to select in the
first place. Separately, Tock's own `capsules_core::spi_controller` capsule
has never implemented the "set chip select" command on any board (it's a
hard-coded `NOSUPPORT`, unrelated to this board) — see the comment in
`capsules/core/src/spi_controller.rs`.

See also `qemu_arm_mps2_an386`, the Cortex-M4 sibling of this board: same
peripheral map (an385/an386 differ only in CPU core), sharing this same
`qemu_arm_mps2_chip` crate.

Running QEMU
------------

To run the board in QEMU, `qemu-system-arm` must be started with the
`-machine mps2-an385` argument and `-kernel $TOCK_KERNEL.elf`. Unlike the
RISC-V `virt` boards, QEMU loads and executes a Cortex-M ELF directly from
its vector table at address 0; no bootloader or `-bios` indirection is
needed. `-nographic` suppresses QEMU's graphical window (there is no display
device on this machine to show regardless).

- **`run`**: Start Tock on an emulated QEMU board:

  ```
  $ make run
  [...]
     text	   data	    bss	    dec	    hex	filename
    57388	      0	  13356	  70744	  11458	target/thumbv7m-none-eabi/release/qemu_arm_mps2_an385

  Running QEMU emulator version 10.2.1 with
   - kernel target/thumbv7m-none-eabi/release/qemu_arm_mps2_an385.elf
  To exit type C-a x

  QEMU MPS2 AN385 (Cortex-M3) initialization complete.
  Entering main loop.
  tock$
  ```

With the default linker script, this board loads processes from
flash=0x00040000-0x0007FFFF into ram=0x21004000-0x2101FFFF (RAM above the
kernel's own static allocations). Kernel and app flash are both well within
QEMU's hard 4 MiB cap for code at address 0 (`armv7m_load_kernel(..., 0,
0x400000)` in `hw/arm/mps2.c`); RAM is a modest slice of the 16 MiB QEMU
always backs at 0x21000000 regardless of what a board's linker script
claims.

Running an application
-----------------------

- **`run-app`**: Start Tock with one or more apps loaded at
  `APP_ADDRESS` (0x00040000):

  ```
  $ make run-app APP=$PATH_TO_APP.tbf
  ```

  Verified end-to-end with `libtock-c`'s `c_hello` and `blink` examples,
  built for `TOCK_TARGETS=cortex-m3` and loaded **at the same time**:

  ```
  $ cd $LIBTOCK_C/examples/c_hello && TOCK_TARGETS=cortex-m3 make
  $ cd $LIBTOCK_C/examples/blink && TOCK_TARGETS=cortex-m3 make
  $ cat $LIBTOCK_C/examples/blink/build/cortex-m3/cortex-m3.tbf \
        $LIBTOCK_C/examples/c_hello/build/cortex-m3/cortex-m3.tbf \
        > apps.bin
  $ make run-app APP=$PWD/apps.bin
  [...]
  QEMU MPS2 AN385 (Cortex-M3) initialization complete.
  Entering main loop.
  tock$ list
  Hello World!
  list
   PID    ShortID    Name                Quanta  Syscalls  Restarts  Grants  State
   0      Unique     blink               183303         7         0   1/ 2   Running
   1      Unique     c_hello             131473         8         0   0/ 2   Terminated
  tock$
  ```

  `blink`'s LED toggling was confirmed for real by reading the `FPGAIO`
  `LED0` register directly (`0x40028000`) through the QEMU monitor
  (`C-a c` to switch from the serial console, then `xp/1xw 0x40028000`)
  while it ran, observing the value change between `0x00000000` and
  `0x00000002`.

  SPI was verified the same way, alongside `c_hello` and `blink`, with a
  small loopback test app (write a known pattern, read it back over
  `libtock_spi_controller_read_write`, `memcmp` the result, print
  `SPI PASS`/`SPI FAIL`) — not currently part of `libtock-c`, since none of
  its existing SPI examples build against the current
  `libtock/peripherals/spi_controller.h` API. Loading it alongside the
  other two confirmed three concurrently-scheduled processes all get
  correctly serviced through this chip crate's UART, Timer, and SPI
  drivers at once:

  ```
  tock$ list
  Hello World!
  SPI PASS
  list
   PID    ShortID    Name                Quanta  Syscalls  Restarts  Grants  State
   0      Unique     blink               470841       171         0   1/ 3   Running
   1      Unique     spi_loopback_test   272451        18         0   0/ 3   Terminated
   2      Unique     c_hello             238836         8         0   0/ 3   Terminated
  tock$
  ```
