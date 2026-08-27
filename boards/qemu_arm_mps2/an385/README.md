QEMU ARM MPS2 AN385 (Cortex-M3) Platform
=========================================

This board crate targets QEMU's `mps2-an385` machine: an emulation of ARM's own
"MPS2 + AN385" Cortex-M System Design Kit (CMSDK) reference platform, not a
real vendor chip. This is a purely virtual target for exercising the Cortex-M3
arch under QEMU, useful for CI and kernel development without access to real
ARM hardware.

See `mps2_base`'s README for peripheral details.

Running QEMU
------------

To run the board in QEMU, `qemu-system-arm` must be started with the
`-machine mps2-an385` argument and `-kernel $TOCK_KERNEL.elf`.

QEMU loads and executes a Cortex-M ELF directly from its vector table at
address 0; no bootloader or `-bios` indirection is needed.

`-nographic` suppresses QEMU's graphical window (there is no display device).

- **`run`**: Start Tock on an emulated QEMU board:

  ```
  $ make run
  [...]
     text	   data	    bss	    dec	    hex	filename
    61484	      0	  15664	  77148	  12d5c	target/thumbv7m-none-eabi/release/mps2-an385

  Running QEMU emulator version 10.2.1 with
   - kernel target/thumbv7m-none-eabi/release/mps2-an385.elf
  To exit type C-a x

  QEMU MPS2 AN385 (Cortex-M3) initialization complete.
  Entering main loop.
  tock$
  ```

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
