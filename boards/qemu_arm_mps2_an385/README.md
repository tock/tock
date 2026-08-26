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
  simulated LEDs. There's no display on this `-nographic` machine to show
  them; observe their state by reading the register directly through the
  QEMU monitor (`C-a c` to switch from the serial console, then `xp/1xw
  0x40028000`).
- One PL022 SPI controller (the "Shield0" instance), run in hardware
  loopback mode — see the note below.
- The CMSDK APB Watchdog, backing the kernel's `WatchDog` resource.

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
first place.

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
kernel's own static allocations).

Running an application
-----------------------

- **`run-app`**: Start Tock with one or more apps loaded at
  `APP_ADDRESS` (0x00040000):

  ```
  $ make run-app APP=$PATH_TO_APP.tbf
  ```

  To load more than one app at once, concatenate their `.tbf` files (e.g.
  `cat app1.tbf app2.tbf > apps.bin`) largest-first: `elf2tab` pads each
  `.tbf` to a power-of-two size for MPU alignment, and the loader assumes
  that ordering.
