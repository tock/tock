QEMU ARM MPS2 AN386 (Cortex-M4) Platform
=========================================

This board crate targets QEMU's `mps2-an386` machine: the Cortex-M4 image of
ARM's "MPS2" Cortex-M System Design Kit (CMSDK) reference platform. It is
identical to `qemu_arm_mps2_an385` in every respect except the CPU core
(an385/an386 share the same `hw/arm/mps2.c` machine init and peripheral
map in QEMU, and this board shares the same `qemu_arm_mps2_chip` crate) —
see that board's README for the full peripheral list and the GPIO
limitation shared by both.

This board uses the soft-float `cortexm4` architecture crate (not
`cortexm4f`), matching the convention of every other Tock Cortex-M4 board
in-tree, even on FPU-capable silicon.

Running QEMU
------------

- **`run`**: Start Tock on an emulated QEMU board:

  ```
  $ make run
  [...]
     text	   data	    bss	    dec	    hex	filename
    57388	      0	  13356	  70744	  11458	target/thumbv7em-none-eabi/release/qemu_arm_mps2_an386

  Running QEMU emulator version 10.2.1 with
   - kernel target/thumbv7em-none-eabi/release/qemu_arm_mps2_an386.elf
  To exit type C-a x

  QEMU MPS2 AN386 (Cortex-M4) initialization complete.
  Entering main loop.
  tock$
  ```

See `qemu_arm_mps2_an385/README.md` for the memory layout (identical
addresses on both machines) and app-loading details.
