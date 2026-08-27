QEMU ARM MPS2 AN386 (Cortex-M4) Platform
=========================================

This board crate targets QEMU's `mps2-an386` machine: an emulation of ARM's own
"MPS2 + AN386" Cortex-M System Design Kit (CMSDK) reference platform, not a
real vendor chip. This is a purely virtual target for exercising the Cortex-M4
arch under QEMU, useful for CI and kernel development without access to real
ARM hardware.

The AN386 does have a FPU available, but this board does not yet exercise that
(i.e., it uses the `cortexm4` architecture crate, not `cortexm4f`).

See `mps2_base`'s README for peripheral details.

Running QEMU
------------

- **`run`**: Start Tock on an emulated QEMU board:

  ```
  $ make run
  [...]
     text	   data	    bss	    dec	    hex	filename
    63532	      0	  15664	  79196	  1355c	target/thumbv7em-none-eabi/release/mps2-an386

  Running QEMU emulator version 10.2.1 with
   - kernel target/thumbv7em-none-eabi/release/mps2-an386.elf
  To exit type C-a x

  QEMU MPS2 AN386 (Cortex-M4) initialization complete.
  Entering main loop.
  tock$
  ```

- **`run-app`**: same as `mps2-an385`'s (`make run-app
  APP=$PATH_TO_APP.tbf`).
